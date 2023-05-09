use chrono::{NaiveDateTime, Utc};
use rand::Rng;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use crate::core::domain::Identifiable;
use crate::core::library::BookStatus;
use crate::utils::date::serializer;

// BookEntity abstracts physical book in library management system and there can be
// many copies of the same book with different identifier.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct BookEntity {
    pub dewey_decimal_id: String,
    pub book_id: String,
    pub version: i64,
    pub author_id: String,
    pub publisher_id: String,
    pub language: String,
    pub isbn: String,
    pub title: String,
    pub book_status: BookStatus,
    pub restricted: bool,
    #[serde(with = "serializer")]
    pub published_at: NaiveDateTime,
    #[serde(with = "serializer")]
    pub created_at: NaiveDateTime,
    #[serde(with = "serializer")]
    pub updated_at: NaiveDateTime,
}

impl BookEntity {
    pub fn new(isbn: &str, title: &str, status: BookStatus) -> Self {
        // dewey_decimal_id:
        // 000–099: general works
        // 100–199: philosophy and psychology
        // 200–299: religion
        // 300–399: social sciences
        // 400–499: language
        // 500–599: natural sciences and mathematics
        // 600–699: technology
        // 700–799: the arts
        // 800–899: literature and rhetoric
        // 900–999: history, biography, and geography
        Self {
            dewey_decimal_id: format!("{}", rand::thread_rng().gen_range(0..1000)),
            version: 0,
            book_id: Uuid::new_v4().to_string(),
            author_id: Uuid::new_v4().to_string(), // random for testing purpose
            publisher_id: Uuid::new_v4().to_string(), // random for testing purpose
            language: "en".to_string(), // random for testing purpose
            isbn: isbn.to_string(),
            title: title.to_string(),
            book_status: status,
            restricted: false,
            published_at: Utc::now().naive_utc(), // for testing purpose
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
        }
    }
}

impl Identifiable for BookEntity {
    fn id(&self) -> String {
        self.book_id.to_string()
    }

    fn version(&self) -> i64 {
        self.version
    }
}


#[cfg(test)]
mod tests {
    use crate::books::domain::model::BookEntity;
    use crate::core::library::BookStatus;

    #[tokio::test]
    async fn test_should_build_books() {
        let book = BookEntity::new("isbn", "title", BookStatus::OnHold);
        assert_eq!("isbn", book.isbn.as_str());
        assert_eq!("title", book.title.as_str());
        assert_eq!("en", book.language.as_str());
    }
}
