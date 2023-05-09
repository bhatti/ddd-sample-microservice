use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use crate::patrons::dto::PatronDto;
use crate::core::command::{Command, CommandError};
use crate::patrons::domain::PatronService;

pub(crate) struct UpdatePatronCommand {
    patron_service: Box<dyn PatronService>,
}

impl UpdatePatronCommand {
    pub(crate) fn new(patron_service: Box<dyn PatronService>) -> Self {
        Self {
            patron_service,
        }
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct UpdatePatronCommandRequest {
    pub patron_id: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
}

impl UpdatePatronCommandRequest {
    pub fn new(patron_id: &str, email: &str, first_name: &str, last_name: &str) -> Self {
        Self {
            patron_id: patron_id.to_string(),
            email: email.to_string(),
            first_name: first_name.to_string(),
            last_name: last_name.to_string(),
        }
    }
    pub fn build_patron(&self) -> PatronDto {
        PatronDto {
            patron_id: self.patron_id.to_string(),
            version: 0,
            first_name: self.first_name.to_string(),
            last_name: self.last_name.to_string(),
            email: self.email.to_string(),
            under_13: false,
            group_roles: vec![],
            num_holds: 0,
            num_overdue: 0,
            home_phone: None,
            cell_phone: None,
            work_phone: None,
            street_address: None,
            city: None,
            zip_code: None,
            state: None,
            country: None,
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
        }
    }
}


#[derive(Debug, Serialize)]
pub(crate) struct UpdatePatronCommandResponse {
    patron: PatronDto,
}

impl UpdatePatronCommandResponse {
    pub fn new(patron: PatronDto) -> Self {
        Self {
            patron,
        }
    }
}

#[async_trait]
impl Command<UpdatePatronCommandRequest, UpdatePatronCommandResponse> for UpdatePatronCommand {
    async fn execute(&self, req: UpdatePatronCommandRequest) -> Result<UpdatePatronCommandResponse, CommandError> {
        let patron = req.build_patron();
        self.patron_service.update_patron(&patron).await.map_err(CommandError::from).map(|_| UpdatePatronCommandResponse::new(patron))
    }
}

#[cfg(test)]
mod tests {
    use async_once::AsyncOnce;
    use lazy_static::lazy_static;
    use crate::patrons::dto::PatronDto;
    use crate::core::command::Command;
    use crate::core::domain::Configuration;
    use crate::core::repository::RepositoryStore;
    use crate::patrons::command::add_patron_cmd::{AddPatronCommand, AddPatronCommandRequest};
    use crate::patrons::command::update_patron_cmd::{UpdatePatronCommand, UpdatePatronCommandRequest};
    use crate::patrons::factory;

    lazy_static! {
        static ref ADD_CMD : AsyncOnce<AddPatronCommand> = AsyncOnce::new(async {
                let svc = factory::create_patron_service(&Configuration::new("test"), RepositoryStore::LocalDynamoDB).await;
                AddPatronCommand::new(svc)
            });
        static ref UPDATE_CMD : AsyncOnce<UpdatePatronCommand> = AsyncOnce::new(async {
                let svc = factory::create_patron_service(&Configuration::new("test"), RepositoryStore::LocalDynamoDB).await;
                UpdatePatronCommand::new(svc)
            });
    }

    #[tokio::test]
    async fn test_should_run_update_patron() {
        let add_cmd = ADD_CMD.get().await.clone();
        let update_cmd = UPDATE_CMD.get().await.clone();

        let mut patron = PatronDto::new("email");
        patron.email = "old_email".to_string();
        let _ = add_cmd.execute(AddPatronCommandRequest::new(patron.email.as_str())).await.expect("should add patron");

        let _ = update_cmd.execute(UpdatePatronCommandRequest::new(patron.patron_id.as_str(), "new-email",
        "new-first", patron.last_name.as_str())).await.expect("should update patron");
    }
}
