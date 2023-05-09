use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::books::dto::BookDto;
use crate::catalog::domain::CatalogService;
use crate::core::command::{Command, CommandError};
use crate::core::library::BookStatus;

pub(crate) struct AddBookCommand {
    catalog_service: Box<dyn CatalogService>,
}

impl AddBookCommand {
    pub(crate) fn new(catalog_service: Box<dyn CatalogService>) -> Self {
        Self {
            catalog_service,
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct AddBookCommandRequest {
    pub(crate) isbn: String,
    pub(crate) title: String,
}

impl AddBookCommandRequest {
    pub fn new(isbn: &str, title: &str) -> Self {
        Self {
            isbn: isbn.to_string(),
            title: title.to_string(),
        }
    }
    pub fn build_book(&self) -> BookDto {
        BookDto::new(self.isbn.as_str(), self.title.as_str(), BookStatus::Available)
    }
}


#[derive(Debug, Serialize)]
pub(crate) struct AddBookCommandResponse {
    pub book: BookDto,
}

impl AddBookCommandResponse {
    pub fn new(book: BookDto) -> Self {
        Self {
            book,
        }
    }
}

#[async_trait]
impl Command<AddBookCommandRequest, AddBookCommandResponse> for AddBookCommand {
    async fn execute(&self, req: AddBookCommandRequest) -> Result<AddBookCommandResponse, CommandError> {
        let book = req.build_book();
        self.catalog_service.add_book(&book).await.map_err(CommandError::from).map(|_| AddBookCommandResponse::new(book))
    }
}

#[cfg(test)]
mod tests {
    use async_once::AsyncOnce;
    use lazy_static::lazy_static;
    use crate::books::dto::BookDto;
    use crate::catalog::command::add_book_cmd::{AddBookCommand, AddBookCommandRequest};
    use crate::catalog::factory;
    use crate::core::command::Command;
    use crate::core::library::BookStatus;
    use crate::core::domain::Configuration;
    use crate::core::repository::RepositoryStore;

    lazy_static! {
        static ref SUT_CMD : AsyncOnce<AddBookCommand> = AsyncOnce::new(async {
                let svc = factory::create_catalog_service(&Configuration::new("test"), RepositoryStore::LocalDynamoDB).await;
                AddBookCommand::new(svc)
            });
    }

    #[tokio::test]
    async fn test_should_run_add_book() {
        let cmd = SUT_CMD.get().await.clone();

        let book = BookDto::new("isbn", "test book", BookStatus::Available);
        let _ = cmd.execute(AddBookCommandRequest::new(book.isbn.as_str(), book.title.as_str()))
            .await.expect("should add book");
    }
}
