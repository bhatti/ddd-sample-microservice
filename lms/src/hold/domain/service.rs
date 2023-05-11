use std::collections::HashMap;

use async_trait::async_trait;
use chrono::{Duration, Utc};
use uuid::Uuid;

use crate::books::domain::Book;
use crate::catalog::domain::CatalogService;
use crate::core::domain::{Configuration, Identifiable};
use crate::core::events::DomainEvent;
use crate::core::library::{BookStatus, HoldStatus, LibraryError, LibraryResult, PaginatedResult};
use crate::gateway::events::EventPublisher;
use crate::hold::domain::HoldService;
use crate::hold::domain::model::HoldEntity;
use crate::hold::dto::HoldDto;
use crate::hold::repository::HoldRepository;
use crate::patrons::domain::PatronService;
use crate::patrons::Patron;

pub(crate) struct HoldServiceImpl {
    branch_id: String,
    hold_repository: Box<dyn HoldRepository>,
    patron_service: Box<dyn PatronService>,
    catalog_service: Box<dyn CatalogService>,
    events_publisher: Box<dyn EventPublisher>,
}

impl HoldServiceImpl {
    pub(crate) fn new(config: &Configuration, hold_repository: Box<dyn HoldRepository>,
                      patron_service: Box<dyn PatronService>, catalog_service: Box<dyn CatalogService>,
                      events_publisher: Box<dyn EventPublisher>) -> Self {
        Self {
            branch_id: config.branch_id.to_string(),
            hold_repository,
            patron_service,
            catalog_service,
            events_publisher,
        }
    }
}

pub(crate) fn from_patron_book(branch_id: &str, patron: &dyn Patron, book: &dyn Book) -> HoldEntity {
    HoldEntity {
        hold_id: Uuid::new_v4().to_string(),
        version: 0,
        branch_id: branch_id.to_string(),
        book_id: book.id(),
        patron_id: patron.id(),
        hold_status: HoldStatus::OnHold,
        hold_at: Utc::now().naive_utc(),
        expires_at: Utc::now().naive_utc() + Duration::days(15),
        canceled_at: None,
        checked_out_at: None,
        created_at: Utc::now().naive_utc(),
        updated_at: Utc::now().naive_utc(),
    }
}

#[async_trait]
impl HoldService for HoldServiceImpl {
    async fn hold(&self, patron_id: &str, book_id: &str) -> LibraryResult<HoldDto> {
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
        let hold = from_patron_book(self.branch_id.as_str(), &patron, &book);
        self.hold_repository.create(&hold).await?;
        let hold = HoldDto::from(&hold);
        let _ = self.events_publisher.publish(&DomainEvent::added(
            "book_hold", "book_hold", hold.hold_id.as_str(), &HashMap::new(), &hold.clone())?).await?;
        Ok(hold)
    }

    async fn cancel(&self, patron_id: &str, book_id: &str) -> LibraryResult<HoldDto> {
        let patron = self.patron_service.find_patron_by_id(patron_id).await?;
        let book = self.catalog_service.find_book_by_id(book_id).await?;
        let mut res = self.hold_repository.query(
            &HashMap::from([("patron_id".to_string(), patron.id().to_string()),
                ("book_id".to_string(), book.id().to_string())]), None, 10).await?;
        let mut iter = res.records.iter_mut();
        if let Some(first) = iter.next() {
            first.hold_status = HoldStatus::Canceled;
            first.canceled_at = Some(Utc::now().naive_utc());
            self.hold_repository.update(first).await?;
            let hold = HoldDto::from(&first.clone());
            let _ = self.events_publisher.publish(&DomainEvent::deleted(
                "book_hold_cancel", "book_hold_cancel", hold.hold_id.as_str(), &HashMap::new(), &hold.clone())?).await?;
            Ok(hold)
        } else {
            Err(LibraryError::not_found(format!("book with id {} for patron {} not found",
                                                book.id(), patron.id()).as_str()))
        }
    }

    async fn checkout(&self, patron_id: &str, book_id: &str) -> LibraryResult<HoldDto> {
        let patron = self.patron_service.find_patron_by_id(patron_id).await?;
        let book = self.catalog_service.find_book_by_id(book_id).await?;
        let mut res = self.hold_repository.query(
            &HashMap::from([("patron_id".to_string(), patron.id().to_string()),
                ("book_id".to_string(), book.id().to_string())]), None, 10).await?;
        let mut iter = res.records.iter_mut();
        if let Some(first) = iter.next() {
            first.hold_status = HoldStatus::CheckedOut;
            first.checked_out_at = Some(Utc::now().naive_utc());
            self.hold_repository.update(first).await?;
            let hold = HoldDto::from(&first.clone());
            let _ = self.events_publisher.publish(&DomainEvent::deleted(
                "book_hold_checkout", "book_hold_checkout", hold.hold_id.as_str(), &HashMap::new(), &hold.clone())?).await?;
            Ok(hold)
        } else {
            Err(LibraryError::not_found(format!("book with id {} for patron {} not found",
                                                book.id(), patron.id()).as_str()))
        }
    }

    async fn query_expired(&self, predicate: &HashMap<String, String>,
                           page: Option<&str>, page_size: usize) -> LibraryResult<PaginatedResult<HoldDto>> {
        let res = self.hold_repository.query_expired(predicate, page, page_size).await?;
        let records = res.records.iter().map(HoldDto::from).collect();
        Ok(PaginatedResult::new(page, page_size, res.next_page, records))
    }
}

impl From<&HoldDto> for HoldEntity {
    fn from(other: &HoldDto) -> HoldEntity {
        HoldEntity {
            hold_id: other.hold_id.to_string(),
            version: other.version,
            branch_id: other.branch_id.to_string(),
            book_id: other.book_id.to_string(),
            patron_id: other.patron_id.to_string(),
            hold_status: other.hold_status,
            hold_at: other.hold_at,
            expires_at: other.expires_at,
            canceled_at: other.canceled_at,
            checked_out_at: other.checked_out_at,
            created_at: other.created_at,
            updated_at: other.updated_at,
        }
    }
}

impl From<&HoldEntity> for HoldDto {
    fn from(other: &HoldEntity) -> HoldDto {
        HoldDto {
            hold_id: other.hold_id.to_string(),
            version: other.version,
            branch_id: other.branch_id.to_string(),
            book_id: other.book_id.to_string(),
            patron_id: other.patron_id.to_string(),
            hold_status: other.hold_status,
            hold_at: other.hold_at,
            expires_at: other.expires_at,
            canceled_at: other.canceled_at,
            checked_out_at: other.checked_out_at,
            created_at: other.created_at,
            updated_at: other.updated_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use async_once::AsyncOnce;
    use aws_sdk_dynamodb::Client;
    use lazy_static::lazy_static;

    use crate::books::domain::model::BookEntity;
    use crate::books::factory::create_book_repository;
    use crate::books::repository::BookRepository;
    use crate::core::domain::Configuration;
    use crate::core::library::{BookStatus, PartyKind};
    use crate::core::repository::RepositoryStore;
    use crate::hold::domain::HoldService;
    use crate::hold::factory;
    use crate::parties::domain::model::PartyEntity;
    use crate::parties::factory::create_party_repository;
    use crate::parties::repository::PartyRepository;
    use crate::utils::ddb::{build_db_client, create_table, delete_table};

    lazy_static! {
        static ref CLIENT: AsyncOnce<Client> = AsyncOnce::new(async {
                build_db_client(RepositoryStore::LocalDynamoDB).await
            });
        static ref SUT_SVC: AsyncOnce<Box<dyn HoldService>> = AsyncOnce::new(async {
                let _ = delete_table(&CLIENT.get().await.clone(), "hold").await;
                let _ = create_table(&CLIENT.get().await.clone(), "hold", "hold_id", "hold_status", "patron_id").await;
                factory::create_hold_service(&Configuration::new("test"), RepositoryStore::LocalDynamoDB).await
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
    async fn test_should_hold_and_cancel() {
        let hold_svc = SUT_SVC.get().await.clone();

        let patron = &PartyEntity::new(PartyKind::Patron, "email");
        let _ = PARTY_REPO.get().await.create(&patron).await.expect("should get patron");
        let book = BookEntity::new("isbn", "title", BookStatus::Available);
        let _ = BOOK_REPO.get().await.create(&book).await.expect("should get book");
        let res = hold_svc.cancel(patron.party_id.as_str(), book.book_id.as_str()).await;
        assert!(res.is_err());
        let hold = hold_svc.hold(patron.party_id.as_str(), book.book_id.as_str()).await.expect("should hold");
        assert_eq!(patron.party_id, hold.patron_id);
        assert_eq!(book.book_id, hold.book_id);
        let canceled = hold_svc.cancel(patron.party_id.as_str(), book.book_id.as_str()).await.expect("should canceled");
        assert_eq!(patron.party_id, canceled.patron_id);
        assert_eq!(book.book_id, canceled.book_id);
    }

    #[tokio::test]
    async fn test_should_hold_and_checked_out() {
        let hold_svc = SUT_SVC.get().await.clone();

        let patron = &PartyEntity::new(PartyKind::Patron, "email");
        let _ = PARTY_REPO.get().await.create(&patron).await.expect("should get patron");
        let book = BookEntity::new("isbn", "title", BookStatus::Available);
        let _ = BOOK_REPO.get().await.create(&book).await.expect("should get book");
        let res = hold_svc.checkout(patron.party_id.as_str(), book.book_id.as_str()).await;
        assert!(res.is_err());
        let hold = hold_svc.hold(patron.party_id.as_str(), book.book_id.as_str()).await.expect("should hold");
        assert_eq!(patron.party_id, hold.patron_id);
        assert_eq!(book.book_id, hold.book_id);
        let checked_out = hold_svc.checkout(patron.party_id.as_str(), book.book_id.as_str()).await.expect("should checked out");
        assert_eq!(patron.party_id, checked_out.patron_id);
        assert_eq!(book.book_id, checked_out.book_id);
    }


    #[tokio::test]
    async fn test_should_query_expired() {
        let hold_svc = SUT_SVC.get().await.clone();

        let res = hold_svc.query_expired(&HashMap::new(), None, 50).await.expect("should query");
        assert_eq!(0, res.records.len());
    }
}
