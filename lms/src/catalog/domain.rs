pub mod service;

use async_trait::async_trait;
use crate::books::dto::BookDto;
use crate::core::library::LibraryResult;

#[async_trait]
pub(crate) trait CatalogService: Sync + Send {
    async fn add_book(&self, book: &BookDto) -> LibraryResult<BookDto>;
    async fn remove_book(&self, id: &str) -> LibraryResult<()>;
    async fn update_book(&self, book: &BookDto) -> LibraryResult<BookDto>;
    async fn find_book_by_id(&self, id: &str) -> LibraryResult<BookDto>;
    async fn find_book_by_isbn(&self, isbn: &str) -> LibraryResult<Vec<BookDto>>;
}

