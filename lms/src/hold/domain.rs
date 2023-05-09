use async_trait::async_trait;
use std::collections::HashMap;
use crate::core::library::{LibraryResult, PaginatedResult};
use crate::hold::dto::HoldDto;

pub mod model;
pub mod service;

#[async_trait]
pub(crate) trait HoldService: Sync + Send {
    async fn hold(&self, patron_id: &str, book_id: &str) -> LibraryResult<HoldDto>;
    async fn cancel(&self, patron_id: &str, book_id: &str) -> LibraryResult<HoldDto>;
    async fn checkout(&self, patron_id: &str, book_id: &str) -> LibraryResult<HoldDto>;
    async fn query_expired(&self, predicate: &HashMap<String, String>,
                           page: Option<&str>, page_size: usize) -> LibraryResult<PaginatedResult<HoldDto>>;
}

