use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::core::command::{Command, CommandError};
use crate::hold::domain::HoldService;
use crate::hold::dto::HoldDto;

pub(crate) struct CancelHoldBookCommand {
    hold_service: Box<dyn HoldService>,
}

impl CancelHoldBookCommand {
    pub(crate) fn new(checkout_service: Box<dyn HoldService>) -> Self {
        Self {
            hold_service: checkout_service,
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct CancelHoldBookCommandRequest {
    patron_id: String,
    book_id: String,
}

impl CancelHoldBookCommandRequest {
    pub fn new(patron_id: String, book_id: String) -> Self {
        Self {
            patron_id,
            book_id,
        }
    }
}


#[derive(Debug, Serialize)]
pub(crate) struct CancelHoldBookCommandResponse {
    hold: HoldDto,
}

impl CancelHoldBookCommandResponse {
    pub fn new(checkout: HoldDto) -> Self {
        Self {
            hold: checkout,
        }
    }
}

#[async_trait]
impl Command<CancelHoldBookCommandRequest, CancelHoldBookCommandResponse> for CancelHoldBookCommand {
    async fn execute(&self, req: CancelHoldBookCommandRequest) -> Result<CancelHoldBookCommandResponse, CommandError> {
        self.hold_service.cancel(req.patron_id.as_str(), req.book_id.as_str())
            .await.map_err(CommandError::from).map(CancelHoldBookCommandResponse::new)
    }
}

#[cfg(test)]
mod tests {
    use async_once::AsyncOnce;
    use lazy_static::lazy_static;
    use crate::books::dto::BookDto;
    use crate::catalog::command::add_book_cmd::{AddBookCommand, AddBookCommandRequest};
    use crate::catalog::factory::create_catalog_service;
    use crate::core::command::Command;
    use crate::core::library::BookStatus;
    use crate::core::domain::Configuration;
    use crate::core::repository::RepositoryStore;
    use crate::hold::command::cancel_hold_book_cmd::{CancelHoldBookCommand, CancelHoldBookCommandRequest};
    use crate::hold::command::hold_book_cmd::{HoldBookCommand, HoldBookCommandRequest};
    use crate::hold::factory::create_hold_service;
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
        static ref HOLD_CMD : AsyncOnce<HoldBookCommand> = AsyncOnce::new(async {
                let svc = create_hold_service(&Configuration::new("test"), RepositoryStore::LocalDynamoDB).await;
                HoldBookCommand::new(svc)
            });
        static ref CANCEL_CMD : AsyncOnce<CancelHoldBookCommand> = AsyncOnce::new(async {
                let svc = create_hold_service(&Configuration::new("test"), RepositoryStore::LocalDynamoDB).await;
                CancelHoldBookCommand::new(svc)
            });
    }

    #[tokio::test]
    async fn test_should_run_checkout_book() {
        let patron_cmd: &AddPatronCommand = PATRON_CMD.get().await.clone();
        let book_cmd: &AddBookCommand = BOOK_CMD.get().await.clone();
        let hold_cmd: &HoldBookCommand = HOLD_CMD.get().await.clone();
        let cancel_cmd: &CancelHoldBookCommand = CANCEL_CMD.get().await.clone();

        let patron = PatronDto::new("email");
        let _ = patron_cmd.execute(AddPatronCommandRequest::new(patron.email.as_str())).await.expect("should add patron");

        let book = BookDto::new("isbn", "test book", BookStatus::Available);
        let _ = book_cmd.execute(AddBookCommandRequest::new(book.isbn.as_str(), book.title.as_str()))
            .await.expect("should add book");
        let _ = hold_cmd.execute(HoldBookCommandRequest::new(
            patron.patron_id.to_string(), book.book_id.to_string())).await.expect("should hold book");
        let res = cancel_cmd.execute(CancelHoldBookCommandRequest::new(
            patron.patron_id.to_string(), book.book_id.to_string())).await.expect("should cancel book");
        assert_eq!(patron.patron_id, res.hold.patron_id);
        assert_eq!(book.book_id, res.hold.book_id);
    }
}
