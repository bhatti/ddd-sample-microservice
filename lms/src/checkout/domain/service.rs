use std::collections::HashMap;
use chrono::Utc;
use async_trait::async_trait;
use crate::books::domain::Book;
use crate::catalog::domain::CatalogService;
use crate::checkout::domain::CheckoutService;
use crate::checkout::domain::model::CheckoutEntity;
use crate::checkout::dto::CheckoutDto;
use crate::checkout::repository::CheckoutRepository;
use crate::core::domain::{Configuration, Identifiable};
use crate::core::events::DomainEvent;
use crate::core::library::{BookStatus, CheckoutStatus, LibraryError, LibraryResult, PaginatedResult};
use crate::gateway::events::EventPublisher;
use crate::patrons::domain::{Patron, PatronService};

pub(crate) struct CheckoutServiceImpl {
    branch_id: String,
    checkout_repository: Box<dyn CheckoutRepository>,
    patron_service: Box<dyn PatronService>,
    catalog_service: Box<dyn CatalogService>,
    events_publisher: Box<dyn EventPublisher>,
}

impl CheckoutServiceImpl {
    pub(crate) fn new(config: &Configuration, checkout_repository: Box<dyn CheckoutRepository>,
                      patron_service: Box<dyn PatronService>, catalog_service: Box<dyn CatalogService>,
                      events_publisher: Box<dyn EventPublisher>) -> Self {
        Self {
            branch_id: config.branch_id.to_string(),
            checkout_repository,
            patron_service,
            catalog_service,
            events_publisher,
        }
    }
    async fn find_first(&self, patron_id: &str, book_id: &str) -> LibraryResult<CheckoutEntity> {
        let res = self.checkout_repository.query(
            &HashMap::from([("patron_id".to_string(), patron_id.to_string()),
                ("book_id".to_string(), book_id.to_string())]), None, 10).await?;
        let mut iter = res.records.iter();
        if let Some(first) = iter.next() {
            Ok(first.clone())
        } else {
            Err(LibraryError::not_found(format!("checkout with id {} for patron {} not found",
                                                book_id, patron_id).as_str()))
        }
    }
}

#[async_trait]
impl CheckoutService for CheckoutServiceImpl {
    async fn checkout(&self, patron_id: &str, book_id: &str) -> LibraryResult<CheckoutDto> {
        let patron = self.patron_service.find_patron_by_id(patron_id).await?;
        let book = self.catalog_service.find_book_by_id(book_id).await?;
        if book.status() != BookStatus::Available {
            return Err(LibraryError::validation(format!("book is not available {}",
                                                        book.id()).as_str(), Some("400".to_string())));
        }
        if book.is_restricted() && patron.is_regular() {
            return Err(LibraryError::validation(format!("patron {} cannot hold restricted books {}",
                                                        patron.id(), book.id()).as_str(), Some("400".to_string())));
        }
        let checkout = CheckoutDto::from_patron_book(self.branch_id.as_str(), &patron, &book);
        self.checkout_repository.create(&CheckoutEntity::from(&checkout)).await?;
        let _ = self.events_publisher.publish(&DomainEvent::added(
            "book_checkout", "checkout", checkout.checkout_id.as_str(), &HashMap::new(), &checkout.clone())?).await?;
        Ok(checkout)
    }

    async fn returned(&self, patron_id: &str, book_id: &str) -> LibraryResult<CheckoutDto> {
        let _ = self.patron_service.find_patron_by_id(patron_id).await?;
        let _ = self.catalog_service.find_book_by_id(book_id).await?;
        let mut existing = self.find_first(patron_id, book_id).await?;
        existing.checkout_status = CheckoutStatus::Returned;
        existing.returned_at = Some(Utc::now().naive_utc());
        self.checkout_repository.update(&existing).await?;
        let checkout = CheckoutDto::from(&existing);
        let _ = self.events_publisher.publish(&DomainEvent::deleted(
            "book_returned", "checkout", checkout.checkout_id.as_str(), &HashMap::new(), &checkout.clone())?).await?;
        Ok(checkout)
    }

    async fn query_overdue(&self, predicate: &HashMap<String, String>,
                           page: Option<&str>, page_size: usize) -> LibraryResult<PaginatedResult<CheckoutDto>> {
        let res = self.checkout_repository.query_overdue(predicate, page, page_size).await?;
        let records = res.records.iter().map(CheckoutDto::from).collect();
        Ok(PaginatedResult::new(page, page_size, res.next_page, records))
    }
}

impl From<&CheckoutEntity> for CheckoutDto {
    fn from(other: &CheckoutEntity) -> CheckoutDto {
        CheckoutDto {
            checkout_id: other.checkout_id.to_string(),
            version: other.version,
            branch_id: other.branch_id.to_string(),
            book_id: other.book_id.to_string(),
            patron_id: other.patron_id.to_string(),
            checkout_status: other.checkout_status,
            checkout_at: other.checkout_at,
            due_at: other.due_at,
            returned_at: other.returned_at,
            created_at: other.created_at,
            updated_at: other.updated_at,
        }
    }
}


impl From<&CheckoutDto> for CheckoutEntity {
    fn from(other: &CheckoutDto) -> CheckoutEntity {
        CheckoutEntity {
            checkout_id: other.checkout_id.to_string(),
            version: other.version,
            branch_id: other.branch_id.to_string(),
            book_id: other.book_id.to_string(),
            patron_id: other.patron_id.to_string(),
            checkout_status: other.checkout_status,
            checkout_at: other.checkout_at,
            due_at: other.due_at,
            returned_at: other.returned_at,
            created_at: other.created_at,
            updated_at: other.updated_at,
        }
    }
}


#[cfg(test)]
mod tests {
    use async_once::AsyncOnce;
    use std::collections::HashMap;
    use lazy_static::lazy_static;
    use aws_sdk_dynamodb::Client;
    use crate::books::domain::model::BookEntity;
    use crate::books::repository::BookRepository;
    use crate::books::factory::create_book_repository;
    use crate::checkout::domain::CheckoutService;
    use crate::checkout::factory;
    use crate::core::domain::Configuration;
    use crate::core::library::{BookStatus, PartyKind};
    use crate::core::repository::RepositoryStore;
    use crate::parties::domain::model::PartyEntity;
    use crate::parties::factory::create_party_repository;
    use crate::parties::repository::PartyRepository;
    use crate::utils::ddb::{build_db_client, create_table, delete_table};

    lazy_static! {
        static ref CLIENT: AsyncOnce<Client> = AsyncOnce::new(async {
                build_db_client(RepositoryStore::LocalDynamoDB).await
            });
        static ref SUT_SVC: AsyncOnce<Box<dyn CheckoutService>> = AsyncOnce::new(async {
                let _ = delete_table(&CLIENT.get().await.clone(), "checkout").await;
                let _ = create_table(&CLIENT.get().await.clone(), "checkout", "checkout_id", "checkout_status", "patron_id").await;
                factory::create_checkout_service(&Configuration::new("test"), RepositoryStore::LocalDynamoDB).await
            });
        static ref BOOK_REPO: AsyncOnce<Box<dyn BookRepository>> = AsyncOnce::new(async {
                let _ = delete_table(&CLIENT.get().await.clone(), "books").await;
                let _ = create_table(&CLIENT.get().await.clone(), "books", "book_id", "book_status", "isbn").await;
                create_book_repository(RepositoryStore::LocalDynamoDB).await
            });
        static ref PARTY_REPO: AsyncOnce<Box<dyn PartyRepository>> = AsyncOnce::new(async {
                let _ = delete_table(&CLIENT.get().await.clone(), "parties").await;
                let _ = create_table(&CLIENT.get().await.clone(), "parties", "party_id", "kind", "email").await;
                create_party_repository(RepositoryStore::LocalDynamoDB).await
            });
    }

    #[tokio::test]
    async fn test_should_checkout_and_returned() {
        let checkout_svc = SUT_SVC.get().await.clone();

        let patron = &PartyEntity::new(PartyKind::Patron, "email");
        let _ = PARTY_REPO.get().await.create(&patron).await.expect("should get patron");
        let book = BookEntity::new("isbn", "title", BookStatus::Available);
        let _ = BOOK_REPO.get().await.create(&book).await.expect("should get book");
        let res = checkout_svc.returned(patron.party_id.as_str(), book.book_id.as_str()).await;
        assert!(res.is_err());
        let checkout = checkout_svc.checkout(patron.party_id.as_str(), book.book_id.as_str()).await.expect("should checkout");
        assert_eq!(patron.party_id, checkout.patron_id);
        assert_eq!(book.book_id, checkout.book_id);
        let returned = checkout_svc.returned(patron.party_id.as_str(), book.book_id.as_str()).await.expect("should returned");
        assert_eq!(patron.party_id, returned.patron_id);
        assert_eq!(book.book_id, returned.book_id);
    }


    #[tokio::test]
    async fn test_should_query_overdue() {
        let checkout_svc = SUT_SVC.get().await.clone();

        let res = checkout_svc.query_overdue(
            &HashMap::new(), None, 50).await.expect("should query");
        assert_eq!(0, res.records.len());
    }
}
