use async_trait::async_trait;
use chrono::Utc;
use rand::Rng;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::books::dto::BookDto;
use crate::catalog::domain::CatalogService;
use crate::core::command::{Command, CommandError};
use crate::core::library::BookStatus;

pub(crate) struct UpdateBookCommand {
    catalog_service: Box<dyn CatalogService>,
}

impl UpdateBookCommand {
    pub(crate) fn new(catalog_service: Box<dyn CatalogService>) -> Self {
        Self {
            catalog_service,
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct UpdateBookCommandRequest {
    pub book_id: String,
    pub isbn: String,
    pub title: String,
    pub book_status: BookStatus,
    pub restricted: bool,
}

impl UpdateBookCommandRequest {
    pub fn new(book_id: &str, isbn: &str, title: &str, status: BookStatus) -> Self {
        Self {
            book_id: book_id.to_string(),
            isbn: isbn.to_string(),
            title: title.to_string(),
            book_status: status,
            restricted: false,
        }
    }
    pub fn build_book(&self) -> BookDto {
        BookDto {
            dewey_decimal_id: format!("{}", rand::thread_rng().gen_range(0..1000)),
            version: 0,
            book_id: self.book_id.to_string(),
            author_id: Uuid::new_v4().to_string(), // random for testing purpose
            publisher_id: Uuid::new_v4().to_string(), // random for testing purpose
            language: "en".to_string(), // random for testing purpose
            isbn: self.isbn.to_string(),
            title: self.title.to_string(),
            book_status: self.book_status,
            restricted: self.restricted,
            published_at: Utc::now().naive_utc(), // for testing purpose
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
        }
    }
}


#[derive(Debug, Serialize)]
pub(crate) struct UpdateBookCommandResponse {
    pub book: BookDto,
}

impl UpdateBookCommandResponse {
    pub fn new(book: BookDto) -> Self {
        Self {
            book,
        }
    }
}

#[async_trait]
impl Command<UpdateBookCommandRequest, UpdateBookCommandResponse> for UpdateBookCommand {
    async fn execute(&self, req: UpdateBookCommandRequest) -> Result<UpdateBookCommandResponse, CommandError> {
        let book = req.build_book();
        self.catalog_service.update_book(&book).await.map_err(CommandError::from).map(|_| UpdateBookCommandResponse::new(book))
    }
}

#[cfg(test)]
mod tests {
    use async_once::AsyncOnce;
    use lazy_static::lazy_static;
    use crate::books::dto::BookDto;
    use crate::catalog::command::add_book_cmd::{AddBookCommand, AddBookCommandRequest};
    use crate::catalog::command::update_book_cmd::{UpdateBookCommand, UpdateBookCommandRequest};
    use crate::catalog::factory;
    use crate::core::command::Command;
    use crate::core::library::BookStatus;
    use crate::core::domain::Configuration;
    use crate::core::repository::RepositoryStore;

    lazy_static! {
        static ref ADD_CMD : AsyncOnce<AddBookCommand> = AsyncOnce::new(async {
                let svc = factory::create_catalog_service(&Configuration::new("test"), RepositoryStore::LocalDynamoDB).await;
                AddBookCommand::new(svc)
            });
        static ref UPDATE_CMD : AsyncOnce<UpdateBookCommand> = AsyncOnce::new(async {
                let svc = factory::create_catalog_service(&Configuration::new("test"), RepositoryStore::LocalDynamoDB).await;
                UpdateBookCommand::new(svc)
            });
    }

    #[tokio::test]
    async fn test_should_run_update_book() {
        let add_cmd = ADD_CMD.get().await.clone();
        let update_cmd = UPDATE_CMD.get().await.clone();

        let book = BookDto::new("isbn", "test book", BookStatus::Available);
        let _ = add_cmd.execute(AddBookCommandRequest::new(book.isbn.as_str(), book.title.as_str()))
                                    .await.expect("should add book");
        let req = UpdateBookCommandRequest::new(book.book_id.as_str(), book.isbn.as_str(), book.title.as_str(), BookStatus::CheckedOut);
        let _ = update_cmd.execute(req).await.expect("should update book");
    }
}
