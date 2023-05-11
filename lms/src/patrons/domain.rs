pub mod service;

use async_trait::async_trait;
use crate::core::library::LibraryResult;
use crate::patrons::dto::PatronDto;

#[async_trait]
pub(crate) trait PatronService: Sync + Send {
    async fn add_patron(&self, patron: &PatronDto) -> LibraryResult<()>;
    async fn remove_patron(&self, id: &str) -> LibraryResult<()>;
    async fn update_patron(&self, patron: &PatronDto) -> LibraryResult<()>;
    async fn find_patron_by_id(&self, id: &str) -> LibraryResult<PatronDto>;
    async fn find_patron_by_email(&self, email: &str) -> LibraryResult<Vec<PatronDto>>;
}
