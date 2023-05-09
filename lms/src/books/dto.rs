use chrono::{NaiveDateTime, Utc};
use rand::Rng;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use crate::books::domain::Book;
use crate::core::domain::Identifiable;
use crate::core::library::BookStatus;
use crate::utils::date::serializer;

// BookDto is a data transfer object for Catalog service
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct BookDto {
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

impl BookDto {
    pub fn new(isbn: &str, title: &str, status: BookStatus) -> BookDto {
        BookDto {
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

impl Identifiable for BookDto {
    fn id(&self) -> String {
        self.book_id.to_string()
    }

    fn version(&self) -> i64 {
        self.version
    }
}

impl Book for BookDto {
    fn is_restricted(&self) -> bool {
        self.restricted
    }

    fn status(&self) -> BookStatus {
        self.book_status
    }
}

#[cfg(test)]
mod tests {
    use crate::books::dto::BookDto;
    use crate::core::library::BookStatus;

    #[tokio::test]
    async fn test_should_build_books() {
        let book = BookDto::new("isbn", "title", BookStatus::OnHold);
        assert_eq!("isbn", book.isbn.as_str());
        assert_eq!("title", book.title.as_str());
        assert_eq!("en", book.language.as_str());
    }
}
