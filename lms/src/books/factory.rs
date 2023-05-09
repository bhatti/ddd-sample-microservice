use crate::books::repository::BookRepository;
use crate::books::repository::ddb_book_repository::DDBBookRepository;
use crate::core::repository::RepositoryStore;
use crate::utils::ddb::{build_db_client, create_table};

pub(crate) async fn create_book_repository(store: RepositoryStore) -> Box<dyn BookRepository> {
    match store {
        RepositoryStore::DynamoDB => {
            let client = build_db_client(store).await;
            Box::new(DDBBookRepository::new(client, "books", "books_ndx"))
        }
        RepositoryStore::LocalDynamoDB => {
            let client = build_db_client(store).await;
            let _ = create_table(&client, "books", "book_id", "book_status", "isbn").await;
            Box::new(DDBBookRepository::new(client, "books", "books_ndx"))
        }
    }
}
