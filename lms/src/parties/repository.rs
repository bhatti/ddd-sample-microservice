pub(crate) mod ddb_party_repository;
use async_trait::async_trait;
use crate::core::library::LibraryResult;
use crate::core::repository::Repository;
use crate::parties::domain::model::PartyEntity;

#[async_trait]
pub(crate) trait PartyRepository: Repository<PartyEntity> {
    async fn find_by_email(&self, email: &str) -> LibraryResult<Vec<PartyEntity>>;
}

