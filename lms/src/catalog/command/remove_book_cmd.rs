use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::catalog::domain::CatalogService;
use crate::core::command::{Command, CommandError};

pub(crate) struct RemoveBookCommand {
    catalog_service: Box<dyn CatalogService>,
}

impl RemoveBookCommand {
    pub(crate) fn new(catalog_service: Box<dyn CatalogService>) -> Self {
        Self {
            catalog_service,
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct RemoveBookCommandRequest {
    pub(crate) book_id: String,
}

impl RemoveBookCommandRequest {
    pub fn new(book_id: String) -> Self {
        Self {
            book_id,
        }
    }
}


#[derive(Debug, Serialize)]
pub(crate) struct RemoveBookCommandResponse {}

impl RemoveBookCommandResponse {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Command<RemoveBookCommandRequest, RemoveBookCommandResponse> for RemoveBookCommand {
    async fn execute(&self, req: RemoveBookCommandRequest) -> Result<RemoveBookCommandResponse, CommandError> {
        self.catalog_service.remove_book(req.book_id.as_str()).await
            .map_err(CommandError::from).map(|_|RemoveBookCommandResponse::new())
    }
}

#[cfg(test)]
mod tests {
    use async_once::AsyncOnce;
    use lazy_static::lazy_static;
    use crate::books::dto::BookDto;
    use crate::catalog::command::add_book_cmd::{AddBookCommand, AddBookCommandRequest};
    use crate::catalog::command::remove_book_cmd::{RemoveBookCommand, RemoveBookCommandRequest};
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
        static ref REMOVE_CMD : AsyncOnce<RemoveBookCommand> = AsyncOnce::new(async {
                let svc = factory::create_catalog_service(&Configuration::new("test"), RepositoryStore::LocalDynamoDB).await;
                RemoveBookCommand::new(svc)
            });
    }

    #[tokio::test]
    async fn test_should_run_remove_book() {
        let add_cmd = ADD_CMD.get().await.clone();
        let remove_cmd = REMOVE_CMD.get().await.clone();

        let book = BookDto::new("isbn", "test book", BookStatus::Available);
        let _ = add_cmd.execute(AddBookCommandRequest::new(book.isbn.as_str(), book.title.as_str()))
            .await.expect("should add book");
        let _ = remove_cmd.execute(RemoveBookCommandRequest::new(book.book_id)).await.expect("should remove book");
    }

}
