use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::core::command::{Command, CommandError};
use crate::patrons::domain::PatronService;

pub(crate) struct RemovePatronCommand {
    patron_service: Box<dyn PatronService>,
}

impl RemovePatronCommand {
    pub(crate) fn new(patron_service: Box<dyn PatronService>) -> Self {
        Self {
            patron_service,
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct RemovePatronCommandRequest {
    pub(crate) patron_id: String,
}

impl RemovePatronCommandRequest {
    pub fn new(patron_id: String) -> Self {
        Self {
            patron_id,
        }
    }
}


#[derive(Debug, Serialize)]
pub(crate) struct RemovePatronCommandResponse {}

impl RemovePatronCommandResponse {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Command<RemovePatronCommandRequest, RemovePatronCommandResponse> for RemovePatronCommand {
    async fn execute(&self, req: RemovePatronCommandRequest) -> Result<RemovePatronCommandResponse, CommandError> {
        self.patron_service.remove_patron(req.patron_id.as_str()).await
            .map_err(CommandError::from).map(|_|RemovePatronCommandResponse::new())
    }
}

#[cfg(test)]
mod tests {
    use async_once::AsyncOnce;
    use lazy_static::lazy_static;
    use crate::patrons::command::add_patron_cmd::{AddPatronCommand, AddPatronCommandRequest};
    use crate::patrons::command::remove_patron_cmd::{RemovePatronCommand, RemovePatronCommandRequest};
    use crate::patrons::factory;
    use crate::core::command::Command;
    use crate::core::domain::Configuration;
    use crate::core::repository::RepositoryStore;

    lazy_static! {
        static ref ADD_CMD : AsyncOnce<AddPatronCommand> = AsyncOnce::new(async {
                let svc = factory::create_patron_service(&Configuration::new("test"), RepositoryStore::LocalDynamoDB).await;
                AddPatronCommand::new(svc)
            });
        static ref REMOVE_CMD : AsyncOnce<RemovePatronCommand> = AsyncOnce::new(async {
                let svc = factory::create_patron_service(&Configuration::new("test"), RepositoryStore::LocalDynamoDB).await;
                RemovePatronCommand::new(svc)
            });
    }

    #[tokio::test]
    async fn test_should_run_remove_patron() {
        let add_cmd = ADD_CMD.get().await.clone();
        let remove_cmd = REMOVE_CMD.get().await.clone();

        let res = add_cmd.execute(AddPatronCommandRequest::new("email")).await.expect("should add patron");
        let _ = remove_cmd.execute(RemovePatronCommandRequest::new(res.patron.patron_id)).await.expect("should remove patron");
    }

}
