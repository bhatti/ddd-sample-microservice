use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::patrons::dto::PatronDto;
use crate::core::command::{Command, CommandError};
use crate::patrons::domain::PatronService;

pub(crate) struct AddPatronCommand {
    patron_service: Box<dyn PatronService>,
}

impl AddPatronCommand {
    pub(crate) fn new(patron_service: Box<dyn PatronService>) -> Self {
        Self {
            patron_service,
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct AddPatronCommandRequest {
    pub email: String,
}

impl AddPatronCommandRequest {
    pub fn new(email: &str) -> Self {
        Self {
            email: email.to_string(),
        }
    }
    pub fn build_patron(&self) -> PatronDto {
        PatronDto::new(self.email.as_str())
    }
}


#[derive(Debug, Serialize)]
pub(crate) struct AddPatronCommandResponse {
    pub patron: PatronDto,
}

impl AddPatronCommandResponse {
    pub fn new(patron: PatronDto) -> Self {
        Self {
            patron,
        }
    }
}

#[async_trait]
impl Command<AddPatronCommandRequest, AddPatronCommandResponse> for AddPatronCommand {
    async fn execute(&self, req: AddPatronCommandRequest) -> Result<AddPatronCommandResponse, CommandError> {
        let patron = req.build_patron();
        self.patron_service.add_patron(&patron).await.map_err(CommandError::from).map(|_|AddPatronCommandResponse::new(patron))
    }
}

#[cfg(test)]
mod tests {
    use async_once::AsyncOnce;
    use lazy_static::lazy_static;
    use crate::patrons::command::add_patron_cmd::{AddPatronCommand, AddPatronCommandRequest};
    use crate::patrons::factory;
    use crate::core::command::Command;
    use crate::core::domain::Configuration;
    use crate::core::repository::RepositoryStore;

    lazy_static! {
        static ref SUT_CMD : AsyncOnce<AddPatronCommand> = AsyncOnce::new(async {
                let svc = factory::create_patron_service(&Configuration::new("test"), RepositoryStore::LocalDynamoDB).await;
                AddPatronCommand::new(svc)
            });
    }

    #[tokio::test]
    async fn test_should_run_add_patron() {
        let cmd = SUT_CMD.get().await.clone();

        let _ = cmd.execute(AddPatronCommandRequest::new("test-email")).await.expect("should add patron");
    }

}
