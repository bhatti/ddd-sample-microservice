use std::cmp;
use std::collections::HashMap;

use async_trait::async_trait;
use aws_sdk_dynamodb::Client;
use aws_sdk_dynamodb::types::AttributeValue;
use chrono::Utc;

use crate::parties::domain::model::{AddressEntity, PartyEntity};
use crate::core::library::{LibraryError, LibraryResult, PaginatedResult, PartyKind};
use crate::core::repository::Repository;
use crate::parties::repository::PartyRepository;
use crate::utils::ddb::{add_filter_expr, from_ddb, parse_bool_attribute, parse_date_attribute, parse_item, parse_number_attribute, parse_string_attribute, string_date, to_ddb_page};

#[derive(Debug)]
pub(crate) struct DDBPartyRepository {
    client: Client,
    table_name: String,
    index_name: String,
}

impl DDBPartyRepository {
    pub(crate) fn new(client: Client, table_name: &str, index_name: &str) -> Self {
        Self {
            client,
            table_name: table_name.to_string(),
            index_name: index_name.to_string(),
        }
    }
}

#[async_trait]
impl Repository<PartyEntity> for DDBPartyRepository {
    async fn create(&self, entity: &PartyEntity) -> LibraryResult<usize> {
        let table_name: &str = self.table_name.as_ref();
        let val = serde_json::to_value(entity)?;
        self.client
            .put_item()
            .table_name(table_name)
            .condition_expression("attribute_not_exists(party_id)")
            .set_item(Some(parse_item(val)?))
            .send()
            .await.map(|_| 1).map_err(LibraryError::from)
    }

    async fn update(&self, entity: &PartyEntity) -> LibraryResult<usize> {
        let now = Utc::now().naive_utc();
        let table_name: &str = self.table_name.as_ref();

        let address = serde_json::to_string(entity.address.as_ref().unwrap_or(&AddressEntity::default()))?;
        let roles = serde_json::to_string(&entity.group_roles)?;
        self.client
            .update_item()
            .table_name(table_name)
            .key("party_id", AttributeValue::S(entity.party_id.clone()))
            .update_expression("SET version = :version, email = :email, kind = :kind, first_name = :first, last_name = :last, address = :address, group_roles = :group_roles, num_holds = :num_holds, num_overdue = :num_overdue, updated_at = :updated_at")
            .expression_attribute_values(":old_version", AttributeValue::N(entity.version.to_string()))
            .expression_attribute_values(":version", AttributeValue::N((entity.version + 1).to_string()))
            .expression_attribute_values(":email", AttributeValue::S(entity.email.to_string()))
            .expression_attribute_values(":kind", AttributeValue::S(entity.kind.to_string()))
            .expression_attribute_values(":first", AttributeValue::S(entity.first_name.to_string()))
            .expression_attribute_values(":last", AttributeValue::S(entity.last_name.to_string()))
            .expression_attribute_values(":address", AttributeValue::S(address))
            .expression_attribute_values(":group_roles", AttributeValue::S(roles))
            .expression_attribute_values(":num_holds", AttributeValue::N(entity.num_holds.to_string()))
            .expression_attribute_values(":num_overdue", AttributeValue::N(entity.num_overdue.to_string()))
            .expression_attribute_values(":updated_at", string_date(now))
            .condition_expression("attribute_exists(version) AND version = :old_version")
            .send()
            .await.map(|_| 1).map_err(LibraryError::from)
    }

    async fn get(&self, id: &str) -> LibraryResult<PartyEntity> {
        let table_name: &str = self.table_name.as_ref();
        self.client
            .query()
            .table_name(table_name)
            .limit(2)
            .consistent_read(true)
            .key_condition_expression(
                "party_id = :party_id",
            )
            .expression_attribute_values(
                ":party_id",
                AttributeValue::S(id.to_string()),
            )
            .send()
            .await.map_err(LibraryError::from).and_then(|req| {
            if let Some(items) = req.items {
                if items.len() > 1 {
                    return Err(LibraryError::database(format!("too many parties for {}", id).as_str(), None, false));
                } else if !items.is_empty() {
                    if let Some(map) = items.first() {
                        return Ok(PartyEntity::from(map));
                    }
                }
                Err(LibraryError::not_found(format!("party not found for {}", id).as_str()))
            } else {
                Err(LibraryError::not_found(format!("party not found for {}", id).as_str()))
            }
        })
    }

    async fn delete(&self, id: &str) -> LibraryResult<usize> {
        let table_name: &str = self.table_name.as_ref();
        self.client.delete_item()
            .table_name(table_name)
            .key("party_id", AttributeValue::S(id.to_string()))
            .send()
            .await.map(|_| 1).map_err(LibraryError::from)
    }

    // Note you cannot use certain reserved words per https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/ReservedWords.html
    async fn query(&self, predicate: &HashMap<String, String>,
                   page: Option<&str>, page_size: usize) -> LibraryResult<PaginatedResult<PartyEntity>> {
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
            .expression_attribute_values(":kind", AttributeValue::S(
                predicate.get("kind").unwrap_or(&PartyKind::Patron.to_string()).to_string()
            ));
        // handle GSI keys first
        let mut key_cond = String::new();
        key_cond.push_str("kind = :kind");

        if let Some(email) = predicate.get("email") {
            key_cond.push_str(" AND email = :email");
            request = request.expression_attribute_values(":email", AttributeValue::S(email.to_string()));
        }
        request = request.key_condition_expression(key_cond);
        let mut filter_expr = String::new();
        // then handle other filters
        for (k, v) in predicate {
            if k != "kind" && k != "email" {
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
                .map(PartyEntity::from).collect();
            from_ddb(page, page_size, req.last_evaluated_key(), records)
        })
    }
}

#[async_trait]
impl PartyRepository for DDBPartyRepository {
    async fn find_by_email(&self, email: &str) -> LibraryResult<Vec<PartyEntity>> {
        let predicate = HashMap::from([
            ("email".to_string(), email.to_string()),
        ]);
        let res = self.query(&predicate, None, 50).await?;
        Ok(res.records)
    }
}


impl From<&HashMap<String, AttributeValue>> for PartyEntity {
    fn from(map: &HashMap<String, AttributeValue>) -> Self {
        let roles: Vec<String> = serde_json::from_str(
            parse_string_attribute("group_roles", map).unwrap_or(String::from("[]")).as_str()).unwrap_or(vec![]);
        PartyEntity {
            party_id: parse_string_attribute("party_id", map).unwrap_or(String::from("")),
            version: parse_number_attribute("version", map),
            kind: PartyKind::from(parse_string_attribute("kind", map).unwrap_or(PartyKind::Patron.to_string())),
            first_name: parse_string_attribute("first_name", map).unwrap_or(String::from("")),
            last_name: parse_string_attribute("last_name", map).unwrap_or(String::from("")),
            email: parse_string_attribute("email", map).unwrap_or(String::from("")),
            under_13: parse_bool_attribute("under_13", map),
            group_roles: roles,
            num_holds: parse_number_attribute("num_holds", map),
            num_overdue: parse_number_attribute("num_overdue", map),
            home_phone: Some(parse_string_attribute("home_phone", map).unwrap_or(String::from(""))),
            cell_phone: Some(parse_string_attribute("cell_phone", map).unwrap_or(String::from(""))),
            work_phone: Some(parse_string_attribute("work_phone", map).unwrap_or(String::from(""))),
            address: AddressEntity::from_json(parse_string_attribute("address", map).unwrap_or(String::from("{}"))),
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
    use chrono::Utc;
    use lazy_static::lazy_static;
    use crate::core::library::PartyKind;
    use crate::core::repository::{Repository, RepositoryStore};

    use crate::parties::domain::model::{AddressEntity, PartyEntity};
    use crate::parties::repository::ddb_party_repository::DDBPartyRepository;
    use crate::utils::ddb::{build_db_client, create_table, delete_table};

    lazy_static! {
        static ref CLIENT: AsyncOnce<Client> = AsyncOnce::new(async {
                let client = build_db_client(RepositoryStore::LocalDynamoDB).await;
                let _ = delete_table(&client, "parties").await;
                let _ = create_table(&client, "parties", "party_id", "kind", "email").await;
                client
            });
    }

    #[tokio::test]
    async fn test_should_create_get_patrons() {
        let parties_repo = DDBPartyRepository::new(
            CLIENT.get().await.clone(), "parties", "parties_ndx");
        let patron = PartyEntity::new(PartyKind::Patron, "email");
        let size = parties_repo.create(&patron).await.expect("should create patron");
        assert_eq!(1, size);

        let loaded = parties_repo.get(patron.party_id.as_str()).await.expect("should return patron");
        assert_eq!(patron.party_id, loaded.party_id);
    }

    #[tokio::test]
    async fn test_should_create_update_patrons() {
        let parties_repo = DDBPartyRepository::new(
            CLIENT.get().await.clone(), "parties", "parties_ndx");
        let mut patron = PartyEntity::new(PartyKind::Patron, "email");
        let size = parties_repo.create(&patron).await.expect("should create patron");
        assert_eq!(1, size);

        patron.first_name = "first2".to_string();
        patron.last_name = "last2".to_string();
        let size = parties_repo.update(&patron).await.expect("should update patron");
        assert_eq!(1, size);

        let loaded = parties_repo.get(patron.party_id.as_str()).await.expect("should return patron");
        assert_eq!(patron.first_name, loaded.first_name);
        assert_eq!(patron.last_name, loaded.last_name);
    }

    #[tokio::test]
    async fn test_should_create_query_patrons() {
        let parties_repo = DDBPartyRepository::new(
            CLIENT.get().await.clone(), "parties", "parties_ndx");
        add_test_patrons(&parties_repo, PartyKind::Branch).await;
        let mut next_page = None;
        let mut total = 0;
        for _i in 0..10 {
            let predicate = HashMap::from([("kind".to_string(), PartyKind::Branch.to_string())]);
            let res = parties_repo.query(&predicate,
                                         next_page.as_deref(), 10).await.expect("should return patron");
            next_page = res.next_page;
            if next_page == None {
                break;
            }
            assert_eq!(10, res.records.len());
            total += res.records.len();
        }
        assert_eq!(50, total);
        let predicate = HashMap::from([
            ("kind".to_string(), PartyKind::Branch.to_string()),
            ("first_name".to_string(), "first_0".to_string()),
            ("email".to_string(), "email_0".to_string())
        ]);
        next_page = None;
        let res = parties_repo.query(&predicate,
                                     next_page.as_deref(), 200).await.expect("should return patron");
        assert_eq!(10, res.records.len());
    }

    #[tokio::test]
    async fn test_should_create_delete_patrons() {
        let parties_repo = DDBPartyRepository::new(
            CLIENT.get().await.clone(), "parties", "parties_ndx");
        let patron = PartyEntity::new(PartyKind::Patron, "email");
        let size = parties_repo.create(&patron).await.expect("should create patron");
        assert_eq!(1, size);

        let deleted = parties_repo.delete(patron.party_id.as_str()).await.expect("should delete patron");
        assert_eq!(1, deleted);

        let loaded = parties_repo.get(patron.party_id.as_str()).await;
        assert!(loaded.is_err());
    }

    async fn add_test_patrons(parties_repo: &DDBPartyRepository, kind: PartyKind) {
        for i in 0..50 {
            let mut patron = PartyEntity::new(kind, format!("email_{}", i / 10).as_str());
            patron.first_name = format!("first_{}", i / 10);
            patron.last_name = format!("last_{}", i / 10);
            if i % 2 == 0 {
                patron.address = Some(AddressEntity {
                    street_address: "100 main st.".to_string(),
                    city: "Seattle".to_string(),
                    zip_code: "980101".to_string(),
                    state: "WA".to_string(),
                    country: "US".to_string(),
                    created_at: Utc::now().naive_utc(),
                    updated_at: Utc::now().naive_utc(),
                })
            }

            let size = parties_repo.create(&patron).await.expect("should create patron");
            assert_eq!(1, size);
        }
    }
}
