pub mod ddb_checkout_repository;

use async_trait::async_trait;
use std::collections::HashMap;
use crate::checkout::domain::model::CheckoutEntity;
use crate::core::library::{LibraryResult, PaginatedResult};
use crate::core::repository::Repository;


#[async_trait]
pub(crate) trait CheckoutRepository : Repository<CheckoutEntity> {
    async fn query_overdue(&self, predicate: &HashMap::<String, String>,
                   page: Option<&str>, page_size: usize) -> LibraryResult<PaginatedResult<CheckoutEntity>>;
}
