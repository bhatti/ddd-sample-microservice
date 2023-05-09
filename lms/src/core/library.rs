use std::fmt;
use std::fmt::{Display, Formatter};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum LibraryError {
    Database {
        message: String,
        reason_code: Option<String>,
        retryable: bool,
    },
    AccessDenied {
        message: String,
        reason_code: Option<String>,
    },
    NotGranted {
        message: String,
        reason_code: Option<String>,
    },
    DuplicateKey {
        message: String,
    },
    NotFound {
        message: String,
    },
    // This is a retry-able error, which indicates that the lock being requested has already been
    // held by another worker and has not been released yet and the lease duration has not expired
    // since the lock was last updated by the current tenant_id.
    // The caller can retry acquiring the lock with or without a backoff.
    CurrentlyUnavailable {
        message: String,
        reason_code: Option<String>,
        retryable: bool,
    },
    Validation {
        message: String,
        reason_code: Option<String>,
    },
    Serialization {
        message: String,
    },
    Runtime {
        message: String,
        reason_code: Option<String>,
    },
}

impl LibraryError {
    pub fn database(message: &str, reason_code: Option<String>, retryable: bool) -> LibraryError {
        LibraryError::Database { message: message.to_string(), reason_code, retryable }
    }

    pub fn access_denied(message: &str, reason_code: Option<String>) -> LibraryError {
        LibraryError::AccessDenied { message: message.to_string(), reason_code }
    }

    pub fn not_granted(message: &str, reason_code: Option<String>) -> LibraryError {
        LibraryError::NotGranted { message: message.to_string(), reason_code }
    }

    pub fn duplicate_key(message: &str) -> LibraryError {
        LibraryError::DuplicateKey { message: message.to_string() }
    }

    pub fn not_found(message: &str) -> LibraryError {
        LibraryError::NotFound { message: message.to_string() }
    }

    pub fn unavailable(message: &str, reason_code: Option<String>, retryable: bool) -> LibraryError {
        LibraryError::CurrentlyUnavailable { message: message.to_string(), reason_code, retryable }
    }

    pub fn database_or_unavailable(message: &str, reason: Option<String>, retryable: bool) -> LibraryError {
        if retryable {
            LibraryError::unavailable(
                format!("ddb database unavailable error {:?} {:?}", message, reason).as_str(), reason, true)
        } else if let Some(ref reason_val) = reason {
            if reason_val.as_str().contains("404") {
                LibraryError::not_found(
                    format!("not found error {:?} {:?}", message, reason).as_str())
            } else if reason_val.as_str().contains("400") {
                LibraryError::access_denied(
                    format!("access-denied error {:?} {:?}", message, reason).as_str(), reason)
            } else {
                LibraryError::database(
                    format!("ddb database error {:?} {:?}", message, reason).as_str(), reason, false)
            }
        } else {
            LibraryError::database(
                format!("ddb database error {:?} {:?}", message, reason).as_str(), reason, false)
        }
    }

    pub fn validation(message: &str, reason_code: Option<String>) -> LibraryError {
        LibraryError::Validation { message: message.to_string(), reason_code }
    }

    pub fn serialization(message: &str) -> LibraryError {
        LibraryError::Serialization { message: message.to_string() }
    }
    pub fn runtime(message: &str, reason_code: Option<String>) -> LibraryError {
        LibraryError::Runtime { message: message.to_string(), reason_code }
    }

    pub fn retryable(&self) -> bool {
        match self {
            LibraryError::Database { retryable, .. } => { *retryable }
            LibraryError::AccessDenied { .. } => { false }
            LibraryError::NotGranted { .. } => { false }
            LibraryError::DuplicateKey { .. } => { false }
            LibraryError::NotFound { .. } => { false }
            LibraryError::CurrentlyUnavailable { retryable, .. } => { *retryable }
            LibraryError::Validation { .. } => { false }
            LibraryError::Serialization { .. } => { false }
            LibraryError::Runtime { .. } => { false }
        }
    }
}

impl From<std::io::Error> for LibraryError {
    fn from(err: std::io::Error) -> Self {
        LibraryError::runtime(
            format!("serde io {:?}", err).as_str(), None)
    }
}

impl From<serde_json::Error> for LibraryError {
    fn from(err: serde_json::Error) -> Self {
        LibraryError::serialization(
            format!("serde json parsing {:?}", err).as_str())
    }
}


impl From<String> for LibraryError {
    fn from(err: String) -> Self {
        LibraryError::serialization(
            format!("serde parsing {:?}", err).as_str())
    }
}


impl Display for LibraryError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            LibraryError::Database { message, reason_code, retryable } => {
                write!(f, "{} {:?} {}", message, reason_code, retryable)
            }
            LibraryError::AccessDenied { message, reason_code } => {
                write!(f, "{} {:?}", message, reason_code)
            }
            LibraryError::NotGranted { message, reason_code } => {
                write!(f, "{} {:?}", message, reason_code)
            }
            LibraryError::DuplicateKey { message } => {
                write!(f, "{}", message)
            }
            LibraryError::NotFound { message } => {
                write!(f, "{}", message)
            }
            LibraryError::CurrentlyUnavailable { message, reason_code, retryable } => {
                write!(f, "{} {:?} {}", message, reason_code, retryable)
            }
            LibraryError::Validation { message, reason_code } => {
                write!(f, "{} {:?}", message, reason_code)
            }
            LibraryError::Serialization { message } => {
                write!(f, "{}", message)
            }
            LibraryError::Runtime { message, reason_code } => {
                write!(f, "{} {:?}", message, reason_code)
            }
        }
    }
}

/// A specialized Result type for Repository .
pub type LibraryResult<T> = Result<T, LibraryError>;

// It defines abstraction for paginated result
#[derive(Debug, Clone)]
pub struct PaginatedResult<T> {
    // The page number or token
    pub page: Option<String>,
    // page size
    pub page_size: usize,
    // Next page if available
    pub next_page: Option<String>,
    // list of records
    pub records: Vec<T>,
}

impl<T> PaginatedResult<T> {
    pub(crate) fn new(page: Option<&str>, page_size: usize,
                      next_page: Option<String>, records: Vec<T>) -> Self {
        PaginatedResult {
            page: page.map(str::to_string),
            page_size,
            next_page,
            records,
        }
    }
}


#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub(crate) enum BookStatus {
    Available,
    CheckedOut,
    OnHold,
    Deleted,
    Unknown,
}

impl From<String> for BookStatus {
    fn from(s: String) -> Self {
        match s.as_str() {
            "Available" => BookStatus::Available,
            "CheckedOut" => BookStatus::CheckedOut,
            "OnHold" => BookStatus::OnHold,
            "Deleted" => BookStatus::Deleted,
            _ => BookStatus::Unknown,
        }
    }
}

impl Display for BookStatus {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            BookStatus::Available => write!(f, "Available"),
            BookStatus::CheckedOut => write!(f, "CheckedOut"),
            BookStatus::OnHold => write!(f, "OnHold"),
            BookStatus::Deleted => write!(f, "Deleted"),
            BookStatus::Unknown => write!(f, "Unknown"),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub(crate) enum Role {
    Admin,
    Regular,
    Child,
    Employee,
    Librarian,
}

impl From<String> for Role {
    fn from(s: String) -> Self {
        match s.as_str() {
            "Admin" => Role::Admin,
            "Regular" => Role::Regular,
            "Child" => Role::Child,
            "Employee" => Role::Employee,
            "Librarian" => Role::Librarian,
            _ => Role::Regular,
        }
    }
}

impl Display for Role {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Role::Admin => write!(f, "Admin"),
            Role::Regular => write!(f, "Regular"),
            Role::Child => write!(f, "Child"),
            Role::Employee => write!(f, "Employee"),
            Role::Librarian => write!(f, "Librarian"),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub(crate) enum CheckoutStatus {
    CheckedOut,
    Returned,
}

impl From<String> for CheckoutStatus {
    fn from(s: String) -> Self {
        match s.as_str() {
            "CheckedOut" => CheckoutStatus::CheckedOut,
            "Returned" => CheckoutStatus::Returned,
            _ => CheckoutStatus::CheckedOut,
        }
    }
}

impl Display for CheckoutStatus {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            CheckoutStatus::CheckedOut => write!(f, "CheckedOut"),
            CheckoutStatus::Returned => write!(f, "Returned"),
        }
    }
}


#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub(crate) enum HoldStatus {
    OnHold,
    Waiting,
    CheckedOut,
    Canceled,
}

impl From<String> for HoldStatus {
    fn from(s: String) -> Self {
        match s.as_str() {
            "OnHold" => HoldStatus::OnHold,
            "Waiting" => HoldStatus::Waiting,
            "CheckedOut" => HoldStatus::CheckedOut,
            "Canceled" => HoldStatus::Canceled,
            _ => HoldStatus::OnHold,
        }
    }
}

impl Display for HoldStatus {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            HoldStatus::OnHold => write!(f, "OnHold"),
            HoldStatus::Waiting => write!(f, "Waiting"),
            HoldStatus::CheckedOut => write!(f, "CheckedOut"),
            HoldStatus::Canceled => write!(f, "Canceled"),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub(crate) enum PartyKind {
    Patron,
    Employee,
    Branch,
    Organization,
}

impl From<String> for PartyKind {
    fn from(s: String) -> Self {
        match s.as_str() {
            "Patron" => PartyKind::Patron,
            "Employee" => PartyKind::Employee,
            "Branch" => PartyKind::Branch,
            "Organization" => PartyKind::Organization,
            _ => PartyKind::Patron,
        }
    }
}

impl Display for PartyKind {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            PartyKind::Patron => write!(f, "Patron"),
            PartyKind::Employee => write!(f, "Employee"),
            PartyKind::Branch => write!(f, "Branch"),
            PartyKind::Organization => write!(f, "Organization"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::library::{BookStatus, LibraryError};

    #[tokio::test]
    async fn test_should_create_database_error() {
        assert!(matches!(LibraryError::database("test", None, false), LibraryError::Database{ message: _, reason_code: _, retryable: _ }));
    }

    #[tokio::test]
    async fn test_should_create_access_error() {
        assert!(matches!(LibraryError::access_denied("test", None), LibraryError::AccessDenied{ message: _, reason_code: _ }));
    }

    #[tokio::test]
    async fn test_should_create_not_granted_error() {
        assert!(matches!(LibraryError::not_granted("test", None), LibraryError::NotGranted{ message: _, reason_code: _ }));
    }

    #[tokio::test]
    async fn test_should_create_duplicate_key_error() {
        assert!(matches!(LibraryError::duplicate_key("test"), LibraryError::DuplicateKey{ message: _ }));
    }

    #[tokio::test]
    async fn test_should_create_not_found_error() {
        assert!(matches!(LibraryError::not_found("test"), LibraryError::NotFound{ message: _ }));
    }

    #[tokio::test]
    async fn test_should_create_unavailable_error() {
        assert!(matches!(LibraryError::unavailable("test", None, false), LibraryError::CurrentlyUnavailable{ message: _, reason_code: _, retryable: _ }));
    }

    #[tokio::test]
    async fn test_should_create_validation_error() {
        assert!(matches!(LibraryError::validation("test", None), LibraryError::Validation{ message: _, reason_code: _ }));
    }

    #[tokio::test]
    async fn test_should_create_serialization_error() {
        assert!(matches!(LibraryError::serialization("test"), LibraryError::Serialization{ message: _ }));
    }

    #[tokio::test]
    async fn test_should_create_runtime_error() {
        assert!(matches!(LibraryError::runtime("test", None), LibraryError::Runtime{ message: _, reason_code: _ }));
    }

    #[tokio::test]
    async fn test_should_create_database_or_unavailable_error() {
        assert!(matches!(LibraryError::database_or_unavailable("test", None, true), LibraryError::CurrentlyUnavailable{ message: _, reason_code: _, retryable: _ }));
        assert!(matches!(LibraryError::database_or_unavailable("test", Some("404".to_string()), false), LibraryError::NotFound{ message: _ }));
        assert!(matches!(LibraryError::database_or_unavailable("test", Some("400".to_string()), false), LibraryError::AccessDenied{ message: _, reason_code: _ }));
        assert!(matches!(LibraryError::database_or_unavailable("test", Some("500".to_string()), false), LibraryError::Database{ message: _, reason_code: _, retryable: _ }));
        assert!(matches!(LibraryError::database_or_unavailable("test", None, false), LibraryError::Database{ message: _, reason_code: _, retryable: _ }));
    }

    #[tokio::test]
    async fn test_should_create_retryable_error() {
        assert_eq!(false, LibraryError::database("test", None, false).retryable());
        assert_eq!(false, LibraryError::access_denied("test", None).retryable());
        assert_eq!(false, LibraryError::not_granted("test", None).retryable());
        assert_eq!(false, LibraryError::duplicate_key("test").retryable());
        assert_eq!(false, LibraryError::not_found("test").retryable());
        assert_eq!(false, LibraryError::unavailable("test", None, false).retryable());
        assert_eq!(true, LibraryError::unavailable("test", None, true).retryable());
        assert_eq!(false, LibraryError::validation("test", None).retryable());
        assert_eq!(false, LibraryError::serialization("test").retryable());
        assert_eq!(false, LibraryError::runtime("test", None).retryable());
    }

    #[tokio::test]
    async fn test_should_format_book_status() {
        let statuses = vec![
            BookStatus::Available,
            BookStatus::CheckedOut,
            BookStatus::OnHold,
            BookStatus::Deleted,
            BookStatus::Unknown,
        ];
        for status in statuses {
            let str = status.to_string();
            let str_status = BookStatus::from(str);
            assert_eq!(status, str_status);
        }
    }
}
