use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::checkout::domain::CheckoutService;
use crate::checkout::dto::CheckoutDto;
use crate::core::command::{Command, CommandError};

pub(crate) struct ReturnBookCommand {
    checkout_service: Box<dyn CheckoutService>,
}

impl ReturnBookCommand {
    pub(crate) fn new(checkout_service: Box<dyn CheckoutService>) -> Self {
        Self {
            checkout_service,
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReturnBookCommandRequest {
    patron_id: String,
    book_id: String,
}

impl ReturnBookCommandRequest {
    pub fn new(patron_id: String, book_id: String) -> Self {
        Self {
            patron_id,
            book_id,
        }
    }
}


#[derive(Debug, Serialize)]
pub(crate) struct ReturnBookCommandResponse {
    checkout: CheckoutDto,
}

impl ReturnBookCommandResponse {
    pub fn new(checkout: CheckoutDto) -> Self {
        Self {
            checkout,
        }
    }
}

#[async_trait]
impl Command<ReturnBookCommandRequest, ReturnBookCommandResponse> for ReturnBookCommand {
    async fn execute(&self, req: ReturnBookCommandRequest) -> Result<ReturnBookCommandResponse, CommandError> {
        self.checkout_service.returned(req.patron_id.as_str(), req.book_id.as_str())
            .await.map_err(CommandError::from).map(ReturnBookCommandResponse::new)
    }
}

#[cfg(test)]
mod tests {
    use async_once::AsyncOnce;
    use lazy_static::lazy_static;
    use crate::books::dto::BookDto;
    use crate::catalog::command::add_book_cmd::{AddBookCommand, AddBookCommandRequest};
    use crate::catalog::factory::create_catalog_service;
    use crate::checkout::command::checkout_book_cmd::{CheckoutBookCommand, CheckoutBookCommandRequest};
    use crate::checkout::command::return_book_cmd::{ReturnBookCommand, ReturnBookCommandRequest};
    use crate::checkout::factory::create_checkout_service;
    use crate::core::command::Command;
    use crate::core::library::BookStatus;
    use crate::core::domain::Configuration;
    use crate::core::repository::RepositoryStore;
    use crate::patrons::command::add_patron_cmd::{AddPatronCommand, AddPatronCommandRequest};
    use crate::patrons::dto::PatronDto;
    use crate::patrons::factory::create_patron_service;

    lazy_static! {
        static ref BOOK_CMD : AsyncOnce<AddBookCommand> = AsyncOnce::new(async {
                let svc = create_catalog_service(&Configuration::new("test"), RepositoryStore::LocalDynamoDB).await;
                AddBookCommand::new(svc)
            });
        static ref PATRON_CMD : AsyncOnce<AddPatronCommand> = AsyncOnce::new(async {
                let svc = create_patron_service(&Configuration::new("test"), RepositoryStore::LocalDynamoDB).await;
                AddPatronCommand::new(svc)
            });
        static ref CHECKOUT_CMD : AsyncOnce<CheckoutBookCommand> = AsyncOnce::new(async {
                let svc = create_checkout_service(&Configuration::new("test"), RepositoryStore::LocalDynamoDB).await;
                CheckoutBookCommand::new(svc)
            });
        static ref RETURN_CMD : AsyncOnce<ReturnBookCommand> = AsyncOnce::new(async {
                let svc = create_checkout_service(&Configuration::new("test"), RepositoryStore::LocalDynamoDB).await;
                ReturnBookCommand::new(svc)
            });
    }

    #[tokio::test]
    async fn test_should_run_checkout_book() {
        let patron_cmd: &AddPatronCommand = PATRON_CMD.get().await.clone();
        let book_cmd: &AddBookCommand = BOOK_CMD.get().await.clone();
        let checkout_cmd: &CheckoutBookCommand = CHECKOUT_CMD.get().await.clone();
        let return_cmd: &ReturnBookCommand = RETURN_CMD.get().await.clone();

        let patron = PatronDto::new("email");
        let _ = patron_cmd.execute(AddPatronCommandRequest::new(patron.email.as_str())).await.expect("should add patron");

        let book = BookDto::new("isbn", "test book", BookStatus::Available);
        let _ = book_cmd.execute(AddBookCommandRequest::new(book.isbn.as_str(), book.title.as_str()))
            .await.expect("should add book");
        let _ = checkout_cmd.execute(CheckoutBookCommandRequest::new(
            patron.patron_id.to_string(), book.book_id.to_string())).await.expect("should checkout book");
        let res = return_cmd.execute(ReturnBookCommandRequest::new(
            patron.patron_id.to_string(), book.book_id.to_string())).await.expect("should return book");
        assert_eq!(patron.patron_id, res.checkout.patron_id);
        assert_eq!(book.book_id, res.checkout.book_id);
    }
}
