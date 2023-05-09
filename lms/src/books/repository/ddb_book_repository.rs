use std::cmp;
use std::collections::HashMap;

use async_trait::async_trait;
use aws_sdk_dynamodb::Client;
use aws_sdk_dynamodb::types::AttributeValue;
use chrono::Utc;

use crate::books::domain::model::BookEntity;
use crate::books::repository::BookRepository;
use crate::core::library::{BookStatus, LibraryError, LibraryResult, PaginatedResult};
use crate::core::repository::Repository;
use crate::utils::ddb::{add_filter_expr, from_ddb, parse_bool_attribute, parse_date_attribute, parse_item, parse_number_attribute, parse_string_attribute, string_date, to_ddb_page};

#[derive(Debug)]
pub struct DDBBookRepository {
    client: Client,
    table_name: String,
    index_name: String,
}

impl DDBBookRepository {
    pub(crate) fn new(client: Client, table_name: &str, index_name: &str) -> Self {
        Self {
            client,
            table_name: table_name.to_string(),
            index_name: index_name.to_string(),
        }
    }
    async fn scan(&self, page: Option<&str>, page_size: usize) -> LibraryResult<PaginatedResult<BookEntity>> {
        let table_name: &str = self.table_name.as_ref();
        let exclusive_start_key = to_ddb_page(page, &HashMap::new());
        self.client
            .scan()
            .table_name(table_name)
            .consistent_read(false)
            .set_exclusive_start_key(exclusive_start_key)
            .limit(cmp::min(page_size, 500) as i32)
            .send()
            .await.map_err(LibraryError::from).map(|req| {
            let def_items = vec![];
            let items = req.items.as_ref().unwrap_or(&def_items);
            let records = items.iter().map(map_to_book).collect();
            from_ddb(page, page_size, req.last_evaluated_key(), records)
        })
    }
}

#[async_trait]
impl Repository<BookEntity> for DDBBookRepository {
    async fn create(&self, entity: &BookEntity) -> LibraryResult<usize> {
        let table_name: &str = self.table_name.as_ref();
        let val = serde_json::to_value(entity)?;
        self.client
            .put_item()
            .table_name(table_name)
            .condition_expression("attribute_not_exists(book_id)")
            .set_item(Some(parse_item(val)?))
            .send()
            .await.map(|_| 1).map_err(LibraryError::from)
    }

    async fn update(&self, entity: &BookEntity) -> LibraryResult<usize> {
        let now = Utc::now().naive_utc();
        let table_name: &str = self.table_name.as_ref();

        self.client
            .update_item()
            .table_name(table_name)
            .key("book_id", AttributeValue::S(entity.book_id.clone()))
            .update_expression("SET version = :version, title = :title, book_status = :book_status, dewey_decimal_id = :dewey_decimal_id, restricted = :restricted, updated_at = :updated_at")
            .expression_attribute_values(":old_version", AttributeValue::N(entity.version.to_string()))
            .expression_attribute_values(":version", AttributeValue::N((entity.version + 1).to_string()))
            .expression_attribute_values(":title", AttributeValue::S(entity.title.to_string()))
            .expression_attribute_values(":book_status", AttributeValue::S(entity.book_status.to_string()))
            .expression_attribute_values(":restricted", AttributeValue::Bool(entity.restricted))
            .expression_attribute_values(":dewey_decimal_id", AttributeValue::S(entity.dewey_decimal_id.to_string()))
            .expression_attribute_values(":updated_at", string_date(now))
            .condition_expression("attribute_exists(version) AND version = :old_version")
            .send()
            .await.map(|_| 1).map_err(LibraryError::from)
    }

    async fn get(&self, id: &str) -> LibraryResult<BookEntity> {
        let table_name: &str = self.table_name.as_ref();
        self.client
            .query()
            .table_name(table_name)
            .limit(2)
            .consistent_read(true)
            .key_condition_expression(
                "book_id = :book_id",
            )
            .expression_attribute_values(
                ":book_id",
                AttributeValue::S(id.to_string()),
            )
            .send()
            .await.map_err(LibraryError::from).and_then(|req| {
            if let Some(items) = req.items {
                if items.len() > 1 {
                    return Err(LibraryError::database(format!("too many books for {}", id).as_str(), None, false));
                } else if !items.is_empty() {
                    if let Some(map) = items.first() {
                        return Ok(map_to_book(map));
                    }
                }
                Err(LibraryError::not_found(format!("book item not found for {}", id).as_str()))
            } else {
                Err(LibraryError::not_found(format!("book not found for {}", id).as_str()))
            }
        })
    }

    async fn delete(&self, id: &str) -> LibraryResult<usize> {
        let table_name: &str = self.table_name.as_ref();
        self.client.delete_item()
            .table_name(table_name)
            .key("book_id", AttributeValue::S(id.to_string()))
            .send()
            .await.map(|_| 1).map_err(LibraryError::from)
    }

    // Note you cannot use certain reserved words per https://docs.aws.amazon.com/amazondynamodb/latest/developerguide/ReservedWords.html
    async fn query(&self, predicate: &HashMap<String, String>,
                   page: Option<&str>, page_size: usize) -> LibraryResult<PaginatedResult<BookEntity>> {
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
            .expression_attribute_values(":status", AttributeValue::S(
                predicate.get("book_status").unwrap_or(&BookStatus::Available.to_string()).to_string()
            ));
        // handle GSI keys first
        let mut key_cond = String::new();
        key_cond.push_str("book_status = :status");

        if let Some(title) = predicate.get("isbn") {
            key_cond.push_str(" AND isbn = :isbn");
            request = request.expression_attribute_values(":isbn", AttributeValue::S(title.to_string()));
        }
        request = request.key_condition_expression(key_cond);
        let mut filter_expr = String::new();
        // then handle other filters
        for (k, v) in predicate {
            if k != "book_status" && k != "isbn" {
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
                .map(map_to_book).collect();
            from_ddb(page, page_size, req.last_evaluated_key(), records)
        })
    }
}

#[async_trait]
impl BookRepository for DDBBookRepository {
    async fn find_by_author_id(&self, author_id: &str, page: Option<&str>, page_size: usize) -> LibraryResult<PaginatedResult<BookEntity>> {
        let predicate = HashMap::from([
            ("author_id".to_string(), author_id.to_string()),
        ]);
        self.query(&predicate, page, page_size).await
    }
}

fn map_to_book(map: &HashMap<String, AttributeValue>) -> BookEntity {
    BookEntity {
        dewey_decimal_id: parse_string_attribute("dewey_decimal_id", map).unwrap_or(String::from("")),
        version: parse_number_attribute("version", map),
        book_id: parse_string_attribute("book_id", map).unwrap_or(String::from("")),
        author_id: parse_string_attribute("author_id", map).unwrap_or(String::from("")),
        publisher_id: parse_string_attribute("publisher_id", map).unwrap_or(String::from("")),
        language: parse_string_attribute("language", map).unwrap_or(String::from("")),
        isbn: parse_string_attribute("isbn", map).unwrap_or(String::from("")),
        title: parse_string_attribute("title", map).unwrap_or(String::from("")),
        book_status: BookStatus::from(parse_string_attribute("book_status", map).unwrap_or(String::from(""))),
        restricted: parse_bool_attribute("restricted", map),
        published_at: parse_date_attribute("published_at", map).unwrap_or(Utc::now().naive_utc()),
        created_at: parse_date_attribute("created_at", map).unwrap_or(Utc::now().naive_utc()),
        updated_at: parse_date_attribute("updated_at", map).unwrap_or(Utc::now().naive_utc()),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use async_once::AsyncOnce;
    use aws_sdk_dynamodb::Client;
    use lazy_static::lazy_static;

    use crate::books::domain::model::BookEntity;
    use crate::books::repository::ddb_book_repository::DDBBookRepository;
    use crate::core::library::BookStatus;
    use crate::core::repository::{Repository, RepositoryStore};
    use crate::utils::ddb::{build_db_client, create_table, delete_table};

    lazy_static! {
        static ref CLIENT: AsyncOnce<Client> = AsyncOnce::new(async {
                let client = build_db_client(RepositoryStore::LocalDynamoDB).await;
                let _ = delete_table(&client, "books").await;
                let _ = create_table(&client, "books", "book_id", "book_status", "isbn").await;
                client
            });
    }

    #[tokio::test]
    async fn test_should_create_get_books() {
        let books_repo = DDBBookRepository::new(CLIENT.get().await.clone(), "books", "books_ndx");
        let book = BookEntity::new("isbn", "test book", BookStatus::Available);
        let size = books_repo.create(&book).await.expect("should create book");
        assert_eq!(1, size);

        let loaded = books_repo.get(book.book_id.as_str()).await.expect("should return book");
        assert_eq!(book.book_id, loaded.book_id);
    }

    #[tokio::test]
    async fn test_should_create_update_books() {
        let books_repo = DDBBookRepository::new(CLIENT.get().await.clone(), "books", "books_ndx");
        let mut book = BookEntity::new("isbn", "test book", BookStatus::Available);
        let size = books_repo.create(&book).await.expect("should create book");
        assert_eq!(1, size);

        book.title = "new title".to_string();
        book.book_status = BookStatus::OnHold;
        let size = books_repo.update(&book).await.expect("should update book");
        assert_eq!(1, size);

        let loaded = books_repo.get(book.book_id.as_str()).await.expect("should return book");
        assert_eq!(book.title, loaded.title);
        assert_eq!(BookStatus::OnHold, book.book_status);
    }

    #[tokio::test]
    async fn test_should_create_scan_books() {
        let books_repo = DDBBookRepository::new(CLIENT.get().await.clone(), "books", "books_ndx");
        add_test_books(&books_repo, BookStatus::OnHold).await;
        let res = books_repo.scan(None, 20).await.expect("should return book");
        assert_eq!(20, res.records.len());
    }

    #[tokio::test]
    async fn test_should_create_query_books() {
        let books_repo = DDBBookRepository::new(CLIENT.get().await.clone(), "books", "books_ndx");
        add_test_books(&books_repo, BookStatus::CheckedOut).await;
        let res = books_repo.query(
            &HashMap::from([("book_status".to_string(), BookStatus::CheckedOut.to_string())]),
            None, 200).await.expect("should return books");
        assert_eq!(50, res.records.len());
        let mut next_page = None;
        let mut total = 0;
        for i in 0..10 {
            let predicate = HashMap::from([("book_status".to_string(), BookStatus::CheckedOut.to_string())]);
            let res = books_repo.query(&predicate,
                                       next_page.as_deref(), 10).await.expect("should return book");
            next_page = res.next_page;
            if i > 10 || next_page == None {
                break;
            }
            assert_eq!(10, res.records.len());
            total += res.records.len();
        }
        assert_eq!(50, total);
        let predicate = HashMap::from([
            ("book_status".to_string(), BookStatus::CheckedOut.to_string()),
            ("isbn".to_string(), "isbn_0".to_string()),
            ("title".to_string(), "title_0".to_string())
        ]);
        let res = books_repo.query(&predicate,
                                   None, 200).await.expect("should return book");
        assert_eq!(10, res.records.len());
    }

    #[tokio::test]
    async fn test_should_create_delete_books() {
        let books_repo = DDBBookRepository::new(CLIENT.get().await.clone(), "books", "books_ndx");
        let book = BookEntity::new("isbn", "test book", BookStatus::Available);
        let size = books_repo.create(&book).await.expect("should create book");
        assert_eq!(1, size);

        let deleted = books_repo.delete(book.book_id.as_str()).await.expect("should delete book");
        assert_eq!(1, deleted);

        let loaded = books_repo.get(book.book_id.as_str()).await;
        assert!(loaded.is_err());
    }

    async fn add_test_books(books_repo: &DDBBookRepository, status: BookStatus) {
        for i in 0..50 {
            let book = BookEntity::new(format!("isbn_{}", i / 10).as_str(),
                                       format!("title_{}", i / 10).as_str(), status);
            let size = books_repo.create(&book).await.expect("should create book");
            assert_eq!(1, size);
        }
    }
}
