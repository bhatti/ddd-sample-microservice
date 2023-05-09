use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::patrons::dto::PatronDto;
use crate::core::command::{Command, CommandError};
use crate::patrons::domain::PatronService;

pub(crate) struct GetPatronCommand {
    patron_service: Box<dyn PatronService>,
}

impl GetPatronCommand {
    pub(crate) fn new(patron_service: Box<dyn PatronService>) -> Self {
        Self {
            patron_service,
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct GetPatronCommandRequest {
    pub patron_id: String,
}

impl GetPatronCommandRequest {
    pub fn new(patron_id: String) -> Self {
        Self {
            patron_id,
        }
    }
}


#[derive(Debug, Serialize)]
pub(crate) struct GetPatronCommandResponse {
    patron: PatronDto,
}

impl GetPatronCommandResponse {
    pub fn new(patron: PatronDto) -> Self {
        Self {
            patron,
        }
    }
}

#[async_trait]
impl Command<GetPatronCommandRequest, GetPatronCommandResponse> for GetPatronCommand {
    async fn execute(&self, req: GetPatronCommandRequest) -> Result<GetPatronCommandResponse, CommandError> {
        self.patron_service.find_patron_by_id(req.patron_id.as_str())
            .await.map_err(CommandError::from).map(GetPatronCommandResponse::new)
    }
}

#[cfg(test)]
mod tests {
    use async_once::AsyncOnce;
    use lazy_static::lazy_static;
    use crate::patrons::command::add_patron_cmd::{AddPatronCommand, AddPatronCommandRequest};
    use crate::patrons::command::get_patron_cmd::{GetPatronCommand, GetPatronCommandRequest};
    use crate::patrons::factory;
    use crate::core::command::Command;
    use crate::core::domain::Configuration;
    use crate::core::repository::RepositoryStore;

    lazy_static! {
        static ref ADD_CMD : AsyncOnce<AddPatronCommand> = AsyncOnce::new(async {
                let svc = factory::create_patron_service(&Configuration::new("test"), RepositoryStore::LocalDynamoDB).await;
                AddPatronCommand::new(svc)
            });
        static ref GET_CMD : AsyncOnce<GetPatronCommand> = AsyncOnce::new(async {
                let svc = factory::create_patron_service(&Configuration::new("test"), RepositoryStore::LocalDynamoDB).await;
                GetPatronCommand::new(svc)
            });
    }

    #[tokio::test]
    async fn test_should_run_get_patron() {
        let add_cmd = ADD_CMD.get().await.clone();
        let get_cmd = GET_CMD.get().await.clone();

        let add_res = add_cmd.execute(AddPatronCommandRequest::new("email1")).await.expect("should add patron");
        let get_res = get_cmd.execute(GetPatronCommandRequest::new(add_res.patron.patron_id.to_string())).await.expect("should get patron");
        assert_eq!(add_res.patron.patron_id, get_res.patron.patron_id);
        assert_eq!(add_res.patron.email, get_res.patron.email);
    }
}
