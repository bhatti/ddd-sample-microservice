pub mod ddb_book_repository;

use async_trait::async_trait;
use crate::books::domain::model::BookEntity;
use crate::core::library::{LibraryResult, PaginatedResult};
use crate::core::repository::Repository;


#[async_trait]
pub(crate) trait BookRepository: Repository<BookEntity> {
    async fn find_by_author_id(&self, author_id: &str,
                           page: Option<&str>, page_size: usize) -> LibraryResult<PaginatedResult<BookEntity>>;
}

