use async_trait::async_trait;
use std::collections::HashMap;
use crate::checkout::dto::CheckoutDto;
use crate::core::library::{LibraryResult, PaginatedResult};

pub mod model;
pub mod service;

#[async_trait]
pub(crate) trait CheckoutService: Sync + Send {
    async fn checkout(&self, patron_id: &str, book_id: &str) -> LibraryResult<CheckoutDto>;
    async fn returned(&self, patron_id: &str, book_id: &str) -> LibraryResult<CheckoutDto>;
    async fn query_overdue(&self, predicate: &HashMap<String, String>,
                           page: Option<&str>, page_size: usize) -> LibraryResult<PaginatedResult<CheckoutDto>>;
}
