pub mod ddb_hold_repository;

use async_trait::async_trait;
use std::collections::HashMap;
use crate::hold::domain::model::HoldEntity;
use crate::core::library::{LibraryResult, PaginatedResult};
use crate::core::repository::Repository;


#[async_trait]
pub(crate) trait HoldRepository: Repository<HoldEntity> {
    async fn query_expired(&self, predicate: &HashMap::<String, String>,
                           page: Option<&str>, page_size: usize) -> LibraryResult<PaginatedResult<HoldEntity>>;
}

