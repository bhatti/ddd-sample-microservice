use chrono::{Duration, NaiveDateTime, Utc};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use crate::core::domain::Identifiable;
use crate::core::library::HoldStatus;
use crate::utils::date::serializer;

// HoldEntity abstracts the book that is on hold or waiting for on-hold
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub(crate) struct HoldEntity {
    pub hold_id: String,
    pub version: i64,
    pub branch_id: String,
    pub book_id: String,
    pub patron_id: String,
    pub hold_status: HoldStatus,
    #[serde(with = "serializer")]
    pub hold_at: NaiveDateTime,
    #[serde(with = "serializer")]
    pub expires_at: NaiveDateTime,
    pub canceled_at: Option<NaiveDateTime>,
    pub checked_out_at: Option<NaiveDateTime>,
    #[serde(with = "serializer")]
    pub created_at: NaiveDateTime,
    #[serde(with = "serializer")]
    pub updated_at: NaiveDateTime,
}

impl HoldEntity{
    pub fn new(book_id: &str, patron_id: &str) -> Self {
        Self {
            hold_id: Uuid::new_v4().to_string(),
            version: 0,
            branch_id: Uuid::new_v4().to_string(),
            book_id: book_id.to_string(),
            patron_id: patron_id.to_string(),
            hold_status: HoldStatus::OnHold,
            hold_at: Utc::now().naive_utc(),
            expires_at: Utc::now().naive_utc() + Duration::days(15),
            canceled_at: None,
            checked_out_at: None,
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
        }
    }
}

impl Identifiable for HoldEntity {
    fn id(&self) -> String {
        self.hold_id.to_string()
    }

    fn version(&self) -> i64 {
        self.version
    }
}


#[cfg(test)]
mod tests {
    use crate::core::library::HoldStatus;
    use crate::hold::domain::model::HoldEntity;

    #[tokio::test]
    async fn test_should_build_hold() {
        let hold = HoldEntity::new("book1", "patron1");
        assert_eq!("book1", hold.book_id.as_str());
        assert_eq!("patron1", hold.patron_id.as_str());
        assert_eq!(HoldStatus::OnHold, hold.hold_status);
    }
}
