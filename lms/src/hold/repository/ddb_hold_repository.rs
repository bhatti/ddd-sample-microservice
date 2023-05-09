use std::cmp;
use std::collections::HashMap;

use async_trait::async_trait;
use aws_sdk_dynamodb::Client;
use aws_sdk_dynamodb::types::AttributeValue;
use chrono::Utc;

use crate::hold::domain::model::HoldEntity;
use crate::core::library::{HoldStatus, LibraryError, LibraryResult, PaginatedResult};
use crate::core::repository::Repository;
use crate::hold::repository::HoldRepository;
use crate::utils::ddb::{add_filter_expr, from_ddb, opt_string_date, parse_date_attribute, parse_item, parse_number_attribute, parse_string_attribute, string_date, to_ddb_page};

#[derive(Debug)]
pub struct DDBHoldRepository {
    client: Client,
    table_name: String,
    index_name: String,
}

impl DDBHoldRepository {
    pub(crate) fn new(client: Client, table_name: &str, index_name: &str) -> Self {
        Self {
            client,
            table_name: table_name.to_string(),
            index_name: index_name.to_string(),
        }
    }
}

#[async_trait]
impl Repository<HoldEntity> for DDBHoldRepository {
    async fn create(&self, entity: &HoldEntity) -> LibraryResult<usize> {
        let table_name: &str = self.table_name.as_ref();
        let val = serde_json::to_value(entity)?;
        self.client
            .put_item()
            .table_name(table_name)
            .condition_expression("attribute_not_exists(hold_id)")
            .set_item(Some(parse_item(val)?))
            .send()
            .await.map(|_| 1).map_err(LibraryError::from)
    }

    async fn update(&self, entity: &HoldEntity) -> LibraryResult<usize> {
        let now = Utc::now().naive_utc();
        let table_name: &str = self.table_name.as_ref();

        self.client
            .update_item()
            .table_name(table_name)
            .key("hold_id", AttributeValue::S(entity.hold_id.clone()))
            .update_expression("SET version = :version, hold_status = :hold_status, hold_at = :hold_at, expires_at = :expires_at, canceled_at = :canceled_at, checked_out_at = :checked_out_at, updated_at = :updated_at")
            .expression_attribute_values(":old_version", AttributeValue::N(entity.version.to_string()))
            .expression_attribute_values(":version", AttributeValue::N((entity.version + 1).to_string()))
            .expression_attribute_values(":hold_status", AttributeValue::S(entity.hold_status.to_string()))
            .expression_attribute_values(":hold_at", string_date(entity.hold_at))
            .expression_attribute_values(":expires_at", string_date(entity.expires_at))
            .expression_attribute_values(":canceled_at", opt_string_date(entity.canceled_at))
            .expression_attribute_values(":checked_out_at", opt_string_date(entity.checked_out_at))
            .expression_attribute_values(":updated_at", string_date(now))
            .condition_expression("attribute_exists(version) AND version = :old_version")
            .send()
            .await.map(|_| 1).map_err(LibraryError::from)
    }

    async fn get(&self, id: &str) -> LibraryResult<HoldEntity> {
        let table_name: &str = self.table_name.as_ref();
        self.client
            .query()
            .table_name(table_name)
            .limit(2)
            .consistent_read(true)
            .key_condition_expression(
                "hold_id = :hold_id",
            )
            .expression_attribute_values(
                ":hold_id",
                AttributeValue::S(id.to_string()),
            )
            .send()
            .await.map_err(LibraryError::from).and_then(|req| {
            if let Some(items) = req.items {
                if items.len() > 1 {
                    return Err(LibraryError::database(format!("too many hold for {}", id).as_str(), None, false));
                } else if !items.is_empty() {
                    if let Some(map) = items.first() {
                        return Ok(HoldEntity::from(map));
                    }
                }
                Err(LibraryError::not_found(format!("hold not found for {}", id).as_str()))
            } else {
                Err(LibraryError::not_found(format!("hold not found for {}", id).as_str()))
            }
        })
    }

    async fn delete(&self, id: &str) -> LibraryResult<usize> {
        let table_name: &str = self.table_name.as_ref();
        self.client.delete_item()
            .table_name(table_name)
            .key("hold_id", AttributeValue::S(id.to_string()))
            .send()
            .await.map(|_| 1).map_err(LibraryError::from)
    }

    // Note you cannot use certain reserved words per https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/ReservedWords.html
    async fn query(&self, predicate: &HashMap<String, String>,
                   page: Option<&str>, page_size: usize) -> LibraryResult<PaginatedResult<HoldEntity>> {
        let table_name: &str = self.table_name.as_ref();
        let index_name: &str = self.index_name.as_ref();
        let exclusive_start_key = to_ddb_page(page, predicate);
        let mut request = self.client
            .query()
            .table_name(table_name)
            .index_name(index_name)
            .limit(cmp::min(page_size, 500) as i32)
            .consistent_read(false)
            .set_exclusive_start_key(exclusive_start_key)
            .expression_attribute_values(":hold_status", AttributeValue::S(
                predicate.get("hold_status").unwrap_or(&HoldStatus::OnHold.to_string()).to_string()
            ));
        // handle GSI keys first
        let mut key_cond = String::new();
        key_cond.push_str("hold_status = :hold_status");

        if let Some(patron_id) = predicate.get("patron_id") {
            key_cond.push_str(" AND patron_id = :patron_id");
            request = request.expression_attribute_values(":patron_id", AttributeValue::S(patron_id.to_string()));
        }
        request = request.key_condition_expression(key_cond);
        let mut filter_expr = String::new();
        // then handle other filters
        for (k, v) in predicate {
            if k != "hold_status" && k != "patron_id" {
                let ks = add_filter_expr(k.as_str(), &mut filter_expr);
                request = request.expression_attribute_values(format!(":{}", ks).as_str(), AttributeValue::S(v.to_string()));
            }
        }
        if !filter_expr.is_empty() {
            request = request.filter_expression(filter_expr);
        }
        request
            .send()
            .await.map_err(LibraryError::from).map(|req| {
            let records = req.items.as_ref().unwrap_or(&vec![]).iter()
                .map(HoldEntity::from).collect();
            from_ddb(page, page_size, req.last_evaluated_key(), records)
        })
    }
}

#[async_trait]
impl HoldRepository for DDBHoldRepository {
    async fn query_expired(&self, predicate: &HashMap<String, String>, page: Option<&str>, page_size: usize) -> LibraryResult<PaginatedResult<HoldEntity>> {
        let now = Utc::now().naive_utc();
        let mut new_predicate = HashMap::from([
            ("hold_status".to_string(), HoldStatus::OnHold.to_string()),
            ("expires_at:<=".to_string(), string_date(now).as_s().unwrap_or(&"0".to_string()).to_string()),
        ]);
        for (key, value) in predicate {
            new_predicate.insert(key.to_string(), value.to_string());
        }
        self.query(&new_predicate, page, page_size).await
    }
}

impl From<&HashMap<String, AttributeValue>> for HoldEntity {
    fn from(map: &HashMap<String, AttributeValue>) -> Self {
        HoldEntity {
            hold_id: parse_string_attribute("hold_id", map).unwrap_or(String::from("")),
            version: parse_number_attribute("version", map),
            branch_id: parse_string_attribute("branch_id", map).unwrap_or(String::from("")),
            book_id: parse_string_attribute("book_id", map).unwrap_or(String::from("")),
            patron_id: parse_string_attribute("patron_id", map).unwrap_or(String::from("")),
            hold_status: HoldStatus::from(parse_string_attribute("hold_status", map).unwrap_or(HoldStatus::OnHold.to_string())),
            hold_at: parse_date_attribute("hold_at", map).unwrap_or(Utc::now().naive_utc()),
            expires_at: parse_date_attribute("expires_at", map).unwrap_or(Utc::now().naive_utc()),
            canceled_at: parse_date_attribute("canceled_at", map),
            checked_out_at: parse_date_attribute("checked_out_at", map),
            created_at: parse_date_attribute("created_at", map).unwrap_or(Utc::now().naive_utc()),
            updated_at: parse_date_attribute("updated_at", map).unwrap_or(Utc::now().naive_utc()),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use async_once::AsyncOnce;

    use aws_sdk_dynamodb::Client;
    use chrono::NaiveDateTime;
    use lazy_static::lazy_static;
    use crate::core::library::HoldStatus;
    use crate::core::repository::{Repository, RepositoryStore};

    use crate::hold::domain::model::HoldEntity;
    use crate::hold::repository::ddb_hold_repository::DDBHoldRepository;
    use crate::utils::ddb::{build_db_client, create_table, delete_table};
    use crate::utils::date::DATE_FMT;

    lazy_static! {
        static ref CLIENT: AsyncOnce<Client> = AsyncOnce::new(async {
                let client = build_db_client(RepositoryStore::LocalDynamoDB).await;
                let _ = delete_table(&client, "hold").await;
                let _ = create_table(&client, "hold", "hold_id", "hold_status", "patron_id").await;
                client
            });
    }

    #[tokio::test]
    async fn test_should_create_get_hold() {
        let hold_repo = DDBHoldRepository::new(
            CLIENT.get().await.clone(), "hold", "hold_ndx");
        let hold = HoldEntity::new("book1", "patron1");
        let size = hold_repo.create(&hold).await.expect("should create hold");
        assert_eq!(1, size);

        let loaded = hold_repo.get(hold.hold_id.as_str()).await.expect("should return hold");
        assert_eq!(hold.hold_id, loaded.hold_id);
    }

    #[tokio::test]
    async fn test_should_create_update_hold() {
        let hold_repo = DDBHoldRepository::new(
            CLIENT.get().await.clone(), "hold", "hold_ndx");
        let mut hold = HoldEntity::new("book2", "patron2");
        let size = hold_repo.create(&hold).await.expect("should create hold");
        assert_eq!(1, size);

        hold.hold_at = NaiveDateTime::parse_from_str("2023-04-12T12:12:12.0", DATE_FMT).unwrap();
        hold.expires_at = NaiveDateTime::parse_from_str("2023-04-25T22:22:22.0", DATE_FMT).unwrap();
        let size = hold_repo.update(&hold).await.expect("should update hold");
        assert_eq!(1, size);

        let loaded = hold_repo.get(hold.hold_id.as_str()).await.expect("should return hold");
        assert_eq!(hold.hold_at, loaded.hold_at);
        assert_eq!(hold.expires_at, loaded.expires_at);
    }

    #[tokio::test]
    async fn test_should_create_query_hold() {
        let hold_repo = DDBHoldRepository::new(
            CLIENT.get().await.clone(), "hold2", "hold2_ndx");
        add_test_hold(&hold_repo, HoldStatus::Waiting).await;
        let mut next_page = None;
        let mut total = 0;
        for _i in 0..10 {
            let predicate = HashMap::from([("hold_status".to_string(), HoldStatus::Waiting.to_string())]);
            let res = hold_repo.query(&predicate,
                                      next_page.as_deref(), 10).await.expect("should return hold");
            next_page = res.next_page;
            if next_page == None {
                break;
            }
            assert_eq!(10, res.records.len());
            total += res.records.len();
        }
        assert_eq!(50, total);
        let mut predicate = HashMap::from([
            ("hold_status".to_string(), HoldStatus::Waiting.to_string()),
            ("hold_at:>=".to_string(), "2023-04-11T11:11:11".to_string()),
        ]);
        let mut res = hold_repo.query(&predicate,
                                      None, 200).await.expect("should return hold");
        assert_eq!(50, res.records.len());
        predicate.insert("expires_at:>=".to_string(), "2023-07-17T17:17:17".to_string());
        res = hold_repo.query(&predicate,
                              None, 200).await.expect("should return hold");
        assert_eq!(25, res.records.len());
    }

    #[tokio::test]
    async fn test_should_create_delete_hold() {
        let hold_repo = DDBHoldRepository::new(
            CLIENT.get().await.clone(), "hold", "hold_ndx");
        let hold = HoldEntity::new("book1", "patron1");
        let size = hold_repo.create(&hold).await.expect("should create hold");
        assert_eq!(1, size);

        let deleted = hold_repo.delete(hold.hold_id.as_str()).await.expect("should delete hold");
        assert_eq!(1, deleted);

        let loaded = hold_repo.get(hold.hold_id.as_str()).await;
        assert!(loaded.is_err());
    }

    async fn add_test_hold(hold_repo: &DDBHoldRepository, status: HoldStatus) {
        for i in 0..50 {
            let mut hold = HoldEntity::new("book1", "patron1");
            hold.hold_status = status;
            hold.hold_at = NaiveDateTime::parse_from_str("2023-04-11T11:11:11", DATE_FMT).unwrap();
            if i % 2 == 0 {
                hold.expires_at = NaiveDateTime::parse_from_str("2023-07-17T17:17:17", DATE_FMT).unwrap();
            } else {
                hold.expires_at = NaiveDateTime::parse_from_str("2023-07-16T16:16:16", DATE_FMT).unwrap();
            }

            let size = hold_repo.create(&hold).await.expect("should create hold");
            assert_eq!(1, size);
        }
    }
}
