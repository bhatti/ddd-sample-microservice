use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use crate::core::command::CommandError;
use crate::core::domain::Configuration;
use crate::core::repository::RepositoryStore;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct AppState {
    pub(crate) config: Configuration,
    pub(crate) store: RepositoryStore,
}

impl AppState {
    pub fn new(branch: &str, store: RepositoryStore) -> AppState {
        AppState {
            config: Configuration::new(branch),
            store,
        }
    }
}

pub(crate) type ServerError = (StatusCode, String);

pub fn json_to_server_error(err: serde_json::Error) -> ServerError {
    (StatusCode::BAD_REQUEST, format!("{}", err))
}

impl From<CommandError> for ServerError {
    fn from(err: CommandError) -> Self {
        match err {
            CommandError::Access { .. } => {
                (StatusCode::BAD_REQUEST, format!("{:?}", err))
            }
            CommandError::Database { .. } => {
                (StatusCode::INTERNAL_SERVER_ERROR, format!("{:?}", err))
            }
            CommandError::DuplicateKey { .. } => {
                (StatusCode::CONFLICT, format!("{:?}", err))
            }
            CommandError::NotFound { .. } => {
                (StatusCode::NOT_FOUND, format!("{:?}", err))
            }
            CommandError::Runtime { .. } => {
                (StatusCode::INTERNAL_SERVER_ERROR, format!("{:?}", err))
            }
            CommandError::Serialization { .. } => {
                (StatusCode::BAD_REQUEST, format!("{:?}", err))
            }
            CommandError::Validation { .. } => {
                (StatusCode::BAD_REQUEST, format!("{:?}", err))
            }
            CommandError::Other { .. } => {
                (StatusCode::INTERNAL_SERVER_ERROR, format!("{:?}", err))
            }
        }
    }
}

