use chrono::{Duration, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::core::library::CheckoutStatus;
use crate::utils::date::serializer;

// CheckoutEntity abstracts the book that is checked out or borrowed.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub(crate) struct CheckoutEntity {
    pub checkout_id: String,
    pub version: i64,
    pub branch_id: String,
    pub book_id: String,
    pub patron_id: String,
    pub checkout_status: CheckoutStatus,
    #[serde(with = "serializer")]
    pub checkout_at: NaiveDateTime,
    #[serde(with = "serializer")]
    pub due_at: NaiveDateTime,
    pub returned_at: Option<NaiveDateTime>,
    #[serde(with = "serializer")]
    pub created_at: NaiveDateTime,
    #[serde(with = "serializer")]
    pub updated_at: NaiveDateTime,
}

impl CheckoutEntity {
    pub fn new(book_id: &str, patron_id: &str) -> Self {
        Self {
            checkout_id: Uuid::new_v4().to_string(),
            version: 0,
            branch_id: Uuid::new_v4().to_string(),
            book_id: book_id.to_string(),
            patron_id: patron_id.to_string(),
            checkout_status: CheckoutStatus::CheckedOut,
            checkout_at: Utc::now().naive_utc(),
            due_at: Utc::now().naive_utc() + Duration::days(15),
            returned_at: None,
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::checkout::domain::model::CheckoutEntity;
    use crate::core::library::CheckoutStatus;

    #[tokio::test]
    async fn test_should_build_checkout() {
        let checkout = CheckoutEntity::new("book1", "patron1");
        assert_eq!("book1", checkout.book_id.as_str());
        assert_eq!("patron1", checkout.patron_id.as_str());
        assert_eq!(CheckoutStatus::CheckedOut, checkout.checkout_status);
    }
}
