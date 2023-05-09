use async_trait::async_trait;
use crate::core::library::LibraryError;

#[derive(Debug)]
pub enum CommandError {
    Access {
        message: String,
        reason_code: Option<String>,
    },
    Database {
        message: String,
        reason_code: Option<String>,
        retryable: bool,
    },
    DuplicateKey {
        message: String,
    },
    NotFound {
        message: String,
    },
    Runtime {
        message: String,
        reason_code: Option<String>,
        retryable: bool,
    },
    Serialization {
        message: String,
    },
    Validation {
        message: String,
        reason_code: Option<String>,
    },
    Other {
        message: String,
        reason_code: Option<String>,
    },
}

#[async_trait]
pub trait Command<Request, Response> {
    async fn execute(&self, req: Request) -> Result<Response, CommandError>;
}

impl From<LibraryError> for CommandError {
    fn from(other: LibraryError) -> Self {
        match other {
            LibraryError::Database { message, reason_code, retryable } => {
                CommandError::Database { message, reason_code, retryable }
            }
            LibraryError::AccessDenied { message, reason_code } => {
                CommandError::Access { message, reason_code }
            }
            LibraryError::NotGranted { message, reason_code } => {
                CommandError::Access { message, reason_code }
            }
            LibraryError::DuplicateKey { message } => {
                CommandError::DuplicateKey { message }
            }
            LibraryError::NotFound { message } => {
                CommandError::NotFound { message }
            }
            LibraryError::CurrentlyUnavailable { message, reason_code, retryable } => {
                CommandError::Runtime { message, reason_code, retryable }
            }
            LibraryError::Validation { message, reason_code } => {
                CommandError::Validation { message, reason_code }
            }
            LibraryError::Serialization { message } => {
                CommandError::Serialization { message }
            }
            LibraryError::Runtime { message, reason_code } => {
                CommandError::Runtime { message, reason_code, retryable: true }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::command::CommandError;

    #[tokio::test]
    async fn test_should_build_command_error() {
        let _ = CommandError::Access { message: "test".to_string(), reason_code: None };
        let _ = CommandError::Database { message: "test".to_string(), reason_code: None, retryable: false };
        let _ = CommandError::Runtime { message: "test".to_string(), reason_code: None, retryable: false };
        let _ = CommandError::Serialization { message: "test".to_string() };
        let _ = CommandError::Validation { message: "test".to_string(), reason_code: None };
        let _ = CommandError::Other { message: "test".to_string(), reason_code: None };
    }
}
