use serde::{Deserialize, Serialize};

// Identifiable defines common traits that can be shared by persistent objects
pub trait Identifiable : Sync + Send {
    fn id(&self) -> String;
    fn version(&self) -> i64;
}


// Configuration abstracts config options for library system
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub(crate) struct Configuration {
    pub branch_id: String,
    pub max_holds: i64,
    pub book_loan_days: i64,
    pub bool_hold_days: i64,
}

impl Configuration {
    pub fn new(branch_id: &str) -> Self {
        Configuration {
            branch_id: branch_id.to_string(),
            max_holds: 4,
            book_loan_days: 15,
            bool_hold_days: 10,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::domain::Configuration;

    #[tokio::test]
    async fn test_should_build_config() {
        let config = Configuration::new("test");
        assert_eq!(4, config.max_holds);
        assert_eq!(15, config.book_loan_days);
        assert_eq!(10, config.bool_hold_days);
    }
}
