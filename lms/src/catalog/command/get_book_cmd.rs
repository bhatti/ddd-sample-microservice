use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::books::dto::BookDto;
use crate::catalog::domain::CatalogService;
use crate::core::command::{Command, CommandError};

pub(crate) struct GetBookCommand {
    catalog_service: Box<dyn CatalogService>,
}

impl GetBookCommand {
    pub(crate) fn new(catalog_service: Box<dyn CatalogService>) -> Self {
        Self {
            catalog_service,
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct GetBookCommandRequest {
    pub(crate) book_id: String,
}

impl GetBookCommandRequest {
    pub fn new(book_id: String) -> Self {
        Self {
            book_id,
        }
    }
}


#[derive(Debug, Serialize)]
pub(crate) struct GetBookCommandResponse {
    book: BookDto,
}

impl GetBookCommandResponse {
    pub fn new(book: BookDto) -> Self {
        Self {
            book,
        }
    }
}

#[async_trait]
impl Command<GetBookCommandRequest, GetBookCommandResponse> for GetBookCommand {
    async fn execute(&self, req: GetBookCommandRequest) -> Result<GetBookCommandResponse, CommandError> {
        self.catalog_service.find_book_by_id(req.book_id.as_str())
            .await.map_err(CommandError::from).map(GetBookCommandResponse::new)
    }
}

#[cfg(test)]
mod tests {
    use async_once::AsyncOnce;
    use lazy_static::lazy_static;
    use crate::books::dto::BookDto;
    use crate::catalog::command::add_book_cmd::{AddBookCommand, AddBookCommandRequest};
    use crate::catalog::command::get_book_cmd::{GetBookCommand, GetBookCommandRequest};
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
        static ref GET_CMD : AsyncOnce<GetBookCommand> = AsyncOnce::new(async {
                let svc = factory::create_catalog_service(&Configuration::new("test"), RepositoryStore::LocalDynamoDB).await;
                GetBookCommand::new(svc)
            });
    }

    #[tokio::test]
    async fn test_should_run_get_book() {
        let add_cmd = ADD_CMD.get().await.clone();
        let get_cmd = GET_CMD.get().await.clone();

        let book = BookDto::new("isbn", "test book", BookStatus::Available);
        let res = add_cmd.execute(AddBookCommandRequest::new(book.isbn.as_str(), book.title.as_str())).await.expect("should add book");
        let loaded = get_cmd.execute(GetBookCommandRequest::new(res.book.book_id.to_string())).await.expect("should get book");
        assert_eq!(book.isbn, loaded.book.isbn);
        assert_eq!(book.title, loaded.book.title);
    }
}
