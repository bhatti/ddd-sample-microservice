use std::collections::HashMap;
use std::time::Duration;
use aws_sdk_dynamodb::Client;
use aws_sdk_dynamodb::config::{Credentials, Region};
use aws_sdk_dynamodb::endpoint::{DefaultResolver, Params};
use aws_sdk_dynamodb::error::SdkError;
use aws_sdk_dynamodb::operation::delete_item::DeleteItemError;
use aws_sdk_dynamodb::operation::execute_statement::ExecuteStatementError;
use aws_sdk_dynamodb::operation::put_item::PutItemError;
use aws_sdk_dynamodb::operation::query::QueryError;
use aws_sdk_dynamodb::operation::scan::ScanError;
use aws_sdk_dynamodb::operation::update_item::UpdateItemError;
use aws_sdk_dynamodb::types::{AttributeDefinition, AttributeValue, GlobalSecondaryIndex, KeySchemaElement, KeyType, Projection, ProjectionType, ProvisionedThroughput, ScalarAttributeType, TableStatus};
use chrono::NaiveDateTime;
use serde_json::Value;
use crate::core::library::{LibraryError, LibraryResult, PaginatedResult};
use crate::core::repository::RepositoryStore;
use crate::utils::date::DATE_FMT;

pub(crate) async fn create_table(client: &Client,
                                 table_name: &str, pk: &str,
                                 gsi_pk: &str, gsi_sk: &str) -> LibraryResult<()> {
    let gsi = GlobalSecondaryIndex::builder()
        .index_name(format!("{}_ndx", table_name))
        .key_schema(KeySchemaElement::builder()
            .attribute_name(gsi_pk)
            .key_type(KeyType::Hash).build())
        .key_schema(KeySchemaElement::builder()
            .attribute_name(gsi_sk)
            .key_type(KeyType::Range).build())
        .projection(Projection::builder().projection_type(ProjectionType::All).build())
        .provisioned_throughput(
            ProvisionedThroughput::builder().read_capacity_units(10).write_capacity_units(10).build())
        .build();

    match client
        .create_table()
        .table_name(table_name)
        .global_secondary_indexes(gsi)
        .key_schema(
            KeySchemaElement::builder()
                .attribute_name(pk)
                .key_type(KeyType::Hash)
                .build(),
        )
        .attribute_definitions(
            AttributeDefinition::builder()
                .attribute_name(pk)
                .attribute_type(ScalarAttributeType::S)
                .build(),
        )
        .attribute_definitions(
            AttributeDefinition::builder()
                .attribute_name(gsi_pk)
                .attribute_type(ScalarAttributeType::S)
                .build(),
        )
        .attribute_definitions(
            AttributeDefinition::builder()
                .attribute_name(gsi_sk)
                .attribute_type(ScalarAttributeType::S)
                .build(),
        )
        .provisioned_throughput(
            ProvisionedThroughput::builder()
                .read_capacity_units(10)
                .write_capacity_units(10)
                .build(),
        )
        .send()
        .await
    {
        Ok(_k) => {
            wait_until_table_status_is_not(client, table_name, TableStatus::Creating).await;
            Ok(())
        }
        Err(err) => {
            Err(LibraryError::database_or_unavailable(format!("failed to create {} table due to {}",
                                                              table_name, err).as_str(), None, false))
        }
    }
}

pub(crate) async fn delete_table(client: &Client, table_name: &str) -> LibraryResult<()> {
    match client.delete_table().table_name(table_name).send().await {
        Ok(_k) => {
            wait_until_table_status_is_not(client, table_name, TableStatus::Deleting).await;
            Ok(())
        }
        Err(err) => {
            Err(LibraryError::database_or_unavailable(format!("failed to delete {} table due to {}",
                                                              table_name, err).as_str(), None, false))
        }
    }
}

async fn wait_until_table_status_is_not(client: &Client, table_name: &str, other_status: TableStatus) {
    for _i in 0..30 {
        match describe_table(client, table_name).await {
            Ok(status) => {
                if status != other_status {
                    return;
                }
            }
            Err(_err) => {}
        }
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

async fn describe_table(client: &Client, table_name: &str) -> LibraryResult<TableStatus> {
    match client
        .describe_table()
        .table_name(table_name)
        .send()
        .await
    {
        Ok(out) => {
            if let Some(table) = out.table() {
                if let Some(status) = table.table_status() {
                    return Ok(status.clone());
                }
            }
            Err(LibraryError::runtime(format!("failed to describe {} table",
                                              table_name).as_str(), None))
        }
        Err(err) => {
            Err(LibraryError::database_or_unavailable(format!("failed to describe {} table due to {}",
                                                              table_name, err).as_str(), None, false))
        }
    }
}

pub(crate) fn parse_item(value: Value) -> Result<HashMap<String, AttributeValue>, String> {
    match value_to_item(value) {
        AttributeValue::M(map) => Ok(map),
        other => Err(format!("failed to parse{:?}", other)),
    }
}

pub(crate) fn parse_string_attribute(name: &str, map: &HashMap<String, AttributeValue>) -> Option<String> {
    if let Some(AttributeValue::S(str)) = map.get(name) {
        return Some(str.clone());
    }
    None
}

pub(crate) fn parse_bool_attribute(name: &str, map: &HashMap<String, AttributeValue>) -> bool {
    if let Some(AttributeValue::Bool(b)) = map.get(name) {
        return *b;
    }
    false
}

pub(crate) fn parse_date_attribute(name: &str, map: &HashMap<String, AttributeValue>) -> Option<NaiveDateTime> {
    if let Some(AttributeValue::S(str)) = map.get(name) {
        // e.g. 2022-09-24T04:40:35.726029
        if let Ok(date) = NaiveDateTime::parse_from_str(str, DATE_FMT) {
            return Some(date);
        }
    }
    None
}

pub(crate) fn opt_string_date(opt_date: Option<NaiveDateTime>) -> AttributeValue {
    if let Some(date) = opt_date {
        return string_date(date);
    }
    AttributeValue::S("".to_string())
}

pub(crate) fn string_date(date: NaiveDateTime) -> AttributeValue {
    AttributeValue::S(format!("{}", date.format(DATE_FMT)))
}

pub(crate) fn parse_number_attribute(name: &str, map: &HashMap<String, AttributeValue>) -> i64 {
    if let Some(AttributeValue::N(str)) = map.get(name) {
        if let Ok(n) = str.parse::<i64>() {
            return n;
        }
    }
    0
}

pub(crate) fn add_filter_expr(k: &str, filter_expr: &mut String) -> String {
    let mut op = "=";
    let mut ks = k;
    let parts = k.split(':').collect::<Vec<&str>>();
    if parts.len() > 1 {
        ks = parts[0];
        op = parts[1];
    }
    if filter_expr.is_empty() {
        filter_expr.push_str(format!("{} {} :{}", ks, op, ks).as_str());
    } else {
        filter_expr.push_str(format!(" AND {} {} :{}", ks, op, ks).as_str());
    }
    ks.to_string()
}

pub(crate) fn to_ddb_page(page: Option<&str>,
                          predicate: &HashMap<String, String>) -> Option<HashMap<String, AttributeValue>> {
    if let Some(page) = page {
        if let Ok(str_map) = serde_json::from_str::<HashMap<String, String>>(page) {
            let mut attr_map = HashMap::new();
            for (k, v) in str_map {
                attr_map.insert(k, AttributeValue::S(v));
            }
            for (k, v) in predicate {
                attr_map.insert(k.to_string(), AttributeValue::S(v.to_string()));
            }
            return Some(attr_map);
        }
    }
    None
}

pub(crate) fn from_ddb<T>(page: Option<&str>, page_size: usize,
                          last_evaluated_key: Option<&HashMap<String, AttributeValue>>,
                          records: Vec<T>) -> PaginatedResult<T> {
    let mut next_page: Option<String> = None;
    if let Some(attr_map) = last_evaluated_key {
        let mut str_map = HashMap::new();
        for (k, v) in attr_map {
            if let AttributeValue::S(val) = v {
                str_map.insert(k.clone(), val.to_string());
            }
        }
        if let Ok(j) = serde_json::to_string(&str_map) {
            next_page = Some(j);
        }
    }
    PaginatedResult::new(page, page_size, next_page, records)
}


fn value_to_item(value: Value) -> AttributeValue {
    match value {
        Value::Null => AttributeValue::Null(true),
        Value::Bool(b) => AttributeValue::Bool(b),
        Value::Number(n) => AttributeValue::N(n.to_string()),
        Value::String(s) => AttributeValue::S(s),
        Value::Array(a) => AttributeValue::L(a.into_iter().map(value_to_item).collect()),
        Value::Object(o) => {
            AttributeValue::M(o.into_iter().map(|(k, v)| (k, value_to_item(v))).collect())
        }
    }
}

// helper method to build db-client with tracing enabled
pub(crate) async fn build_db_client(store: RepositoryStore) -> Client {
    match store {
        RepositoryStore::DynamoDB => {
            //Get config from environment.
            let config = aws_config::load_from_env().await;
            //Create the DynamoDB client.
            Client::new(&config)
        }
        RepositoryStore::LocalDynamoDB => {
            // See https://docs.aws.amazon.com/sdk-for-rust/latest/dg/dynamodb-local.html
            let _params = Params::builder()
                .region("local".to_string())
                .use_fips(false)
                .use_dual_stack(false)
                .build()
                .expect("invalid params");
            let resolver = DefaultResolver::new();
            let dynamodb_local_config = aws_sdk_dynamodb::Config::builder()
                .region(Region::new("local"))
                .credentials_provider(
                    Credentials::new("AKIDLOCALSTACK", "localstacksecret", None, None, "faked"))
                .endpoint_resolver(resolver).build();
            Client::from_conf(dynamodb_local_config)
        }
    }
}

// helper method to build ses-client with tracing enabled
pub async fn build_ses_client() -> aws_sdk_sns::Client {
    //Get config from environment.
    let config = aws_config::load_from_env().await;
    //Create the SES client.
    aws_sdk_sns::Client::new(&config)
}

// required to enable CloudWatch error logging by the runtime
pub fn setup_tracing() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // this needs to be set to false, otherwise ANSI color codes will
        // show up in a confusing manner in CloudWatch logs.
        .with_ansi(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .json()
        .init();
}


impl From<SdkError<UpdateItemError>> for LibraryError {
    fn from(err: SdkError<UpdateItemError>) -> Self {
        let (retryable, reason) = retryable_sdk_error(&err);
        LibraryError::database_or_unavailable(format!("{:?}", err).as_str(), reason, retryable)
    }
}

impl From<SdkError<PutItemError>> for LibraryError {
    fn from(err: SdkError<PutItemError>) -> Self {
        let (retryable, reason) = retryable_sdk_error(&err);
        LibraryError::database_or_unavailable(format!("{:?}", err).as_str(), reason, retryable)
    }
}

impl From<SdkError<DeleteItemError>> for LibraryError {
    fn from(err: SdkError<DeleteItemError>) -> Self {
        let (retryable, reason) = retryable_sdk_error(&err);
        LibraryError::database_or_unavailable(format!("{:?}", err).as_str(), reason, retryable)
    }
}

impl From<SdkError<QueryError>> for LibraryError {
    fn from(err: SdkError<QueryError>) -> Self {
        let (retryable, reason) = retryable_sdk_error(&err);
        LibraryError::database_or_unavailable(format!("{:?}", err).as_str(), reason, retryable)
    }
}

impl From<SdkError<ScanError>> for LibraryError {
    fn from(err: SdkError<ScanError>) -> Self {
        let (retryable, reason) = retryable_sdk_error(&err);
        LibraryError::database_or_unavailable(format!("{:?}", err).as_str(), reason, retryable)
    }
}

impl From<SdkError<ExecuteStatementError>> for LibraryError {
    fn from(err: SdkError<ExecuteStatementError>) -> Self {
        let (retryable, reason) = retryable_sdk_error(&err);
        LibraryError::database_or_unavailable(format!("{:?}", err).as_str(), reason, retryable)
    }
}

fn retryable_sdk_error<T>(err: &SdkError<T>) -> (bool, Option<String>) {
    match err {
        SdkError::ConstructionFailure(_) => { (false, Some("ConstructionFailure".to_string())) }
        SdkError::TimeoutError(_) => { (true, Some("TimeoutError".to_string())) }
        SdkError::DispatchFailure(_) => { (true, Some("DispatchFailure".to_string())) }
        SdkError::ResponseError { .. } => { (true, Some("ResponseError".to_string())) }
        SdkError::ServiceError(ctx) => {
            (ctx.raw().http().status().is_server_error() || has_exceeded_limit(ctx.raw().http().body().bytes()), Some(ctx.raw().http().status().to_string()))
        }
        _ => { (true, Some("Unknown".to_string())) }
    }
}

fn has_exceeded_limit(opts: Option<&[u8]>) -> bool {
    if let Some(b) = opts {
        for i in 0..(b.len() - 6) {
            if b[i] == b'c' && b[i + 1] == b'e' && b[i + 2] == b'e' && b[i + 3] == b'd' && b[i + 4] == b'e' && b[i + 5] == b'd' {
                return true; //"ceeded"
            }
        }
    }
    false
}
