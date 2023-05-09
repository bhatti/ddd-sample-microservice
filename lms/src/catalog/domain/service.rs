use std::collections::HashMap;
use async_trait::async_trait;
use crate::books::domain::model::BookEntity;
use crate::books::dto::BookDto;
use crate::books::repository::BookRepository;
use crate::catalog::domain::CatalogService;
use crate::core::domain::Configuration;
use crate::core::events::DomainEvent;
use crate::core::library::LibraryResult;
use crate::gateway::events::EventPublisher;

pub(crate) struct CatalogServiceImpl {
    book_repository: Box<dyn BookRepository>,
    events_publisher: Box<dyn EventPublisher>,
}

impl CatalogServiceImpl {
    pub(crate) fn new(_config: &Configuration, book_repository: Box<dyn BookRepository>,
                      events_publisher: Box<dyn EventPublisher>) -> Self {
        Self {
            book_repository,
            events_publisher,
        }
    }
}

#[async_trait]
impl CatalogService for CatalogServiceImpl {
    async fn add_book(&self, book: &BookDto) -> LibraryResult<BookDto> {
        let _ = self.book_repository.create(&BookEntity::from(book)).await.map(|_| ())?;
        let _ = self.events_publisher.publish(&DomainEvent::added(
            "books", "books", book.book_id.as_str(), &HashMap::new(), book)?).await?;
        Ok(book.clone())
    }

    async fn remove_book(&self, id: &str) -> LibraryResult<()> {
        let res = self.book_repository.delete(id).await.map(|_| ())?;
        let data = id.to_string();
        let _ = self.events_publisher.publish(&DomainEvent::deleted(
            "books", "books", id, &HashMap::new(), &data)?).await?;
        Ok(res)
    }

    async fn update_book(&self, book: &BookDto) -> LibraryResult<BookDto> {
        let _ = self.book_repository.update(&BookEntity::from(book)).await.map(|_| ())?;
        let _ = self.events_publisher.publish(&DomainEvent::updated(
            "books", "books", book.book_id.as_str(), &HashMap::new(), book)?).await?;
        Ok(book.clone())
    }

    async fn find_book_by_id(&self, id: &str) -> LibraryResult<BookDto> {
        self.book_repository.get(id).await.map(|b| BookDto::from(&b))
    }

    async fn find_book_by_isbn(&self, isbn: &str) -> LibraryResult<Vec<BookDto>> {
        let res = self.book_repository.query(
            &HashMap::from([("isbn".to_string(), isbn.to_string())]), None, 100).await?;
        Ok(res.records.iter().map(BookDto::from).collect())
    }
}

impl From<&BookEntity> for BookDto {
    fn from(other: &BookEntity) -> Self {
        Self {
            dewey_decimal_id: other.dewey_decimal_id.to_string(),
            version: other.version,
            book_id: other.book_id.to_string(),
            author_id: other.author_id.to_string(),
            publisher_id: other.publisher_id.to_string(),
            language: other.language.to_string(),
            isbn: other.isbn.to_string(),
            title: other.title.to_string(),
            book_status: other.book_status,
            restricted: other.restricted,
            published_at: other.published_at,
            created_at: other.created_at,
            updated_at: other.updated_at,
        }
    }
}

impl From<&BookDto> for BookEntity {
    fn from(other: &BookDto) -> Self {
        Self {
            dewey_decimal_id: other.dewey_decimal_id.to_string(),
            version: other.version,
            book_id: other.book_id.to_string(),
            author_id: other.author_id.to_string(),
            publisher_id: other.publisher_id.to_string(),
            language: other.language.to_string(),
            isbn: other.isbn.to_string(),
            title: other.title.to_string(),
            book_status: other.book_status,
            restricted: other.restricted,
            published_at: other.published_at,
            created_at: other.created_at,
            updated_at: other.updated_at,
        }
    }
}


#[cfg(test)]
mod tests {
    use async_once::AsyncOnce;
    use lazy_static::lazy_static;
    use crate::books::dto::BookDto;
    use crate::catalog::domain::CatalogService;
    use crate::catalog::factory;
    use crate::core::library::BookStatus;
    use crate::core::domain::Configuration;
    use crate::core::repository::RepositoryStore;

    lazy_static! {
        static ref SUT_SVC: AsyncOnce<Box<dyn CatalogService>> = AsyncOnce::new(async {
                factory::create_catalog_service(&Configuration::new("test"), RepositoryStore::LocalDynamoDB).await
            });
    }

    #[tokio::test]
    async fn test_should_add_book() {
        let catalog_svc = SUT_SVC.get().await.clone();

        let book = BookDto::new("isbn", "test book", BookStatus::Available);
        let _ = catalog_svc.add_book(&book).await.expect("should add book");

        let loaded = catalog_svc.find_book_by_id(book.book_id.as_str()).await.expect("should return book");
        assert_eq!(book.book_id, loaded.book_id);
    }

    #[tokio::test]
    async fn test_should_update_book() {
        let catalog_svc = SUT_SVC.get().await.clone();

        let mut book = BookDto::new("isbn", "test book", BookStatus::Available);
        let _ = catalog_svc.add_book(&book).await.expect("should add book");

        book.title = "new title".to_string();
        book.book_status = BookStatus::CheckedOut;
        let _ = catalog_svc.update_book(&book).await.expect("should update book");

        let loaded = catalog_svc.find_book_by_id(book.book_id.as_str()).await.expect("should return book");
        assert_eq!(book.title, loaded.title);
        assert_eq!(BookStatus::CheckedOut, book.book_status);
    }


    #[tokio::test]
    async fn test_should_find_by_isbn() {
        let catalog_svc = SUT_SVC.get().await.clone();

        let book = BookDto::new("isbn981", "test book", BookStatus::Available);
        let _ = catalog_svc.add_book(&book).await.expect("should add book");
        let res = catalog_svc.find_book_by_isbn(book.isbn.as_str()).await.expect("should return book");
        assert_eq!(1, res.len());
    }

    #[tokio::test]
    async fn test_should_remove_book() {
        let catalog_svc = SUT_SVC.get().await.clone();

        let book = BookDto::new("isbn123", "test book", BookStatus::Available);
        let _ = catalog_svc.add_book(&book).await.expect("should add book");

        let _ = catalog_svc.remove_book(book.book_id.as_str()).await.expect("should remove book");

        let loaded = catalog_svc.find_book_by_id(book.book_id.as_str()).await;
        assert!(loaded.is_err());
    }
}
