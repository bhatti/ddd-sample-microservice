use async_trait::async_trait;
use crate::core::events::DomainEvent;
use crate::core::library::LibraryError;

#[async_trait]
pub(crate) trait EventPublisher: Sync + Send {
    async fn create_topic(&mut self, topic: &str) -> Result<String, LibraryError>;
    async fn get_topics(&mut self) -> Result<Vec<String>, LibraryError>;
    async fn publish(&self, event: &DomainEvent) -> Result<(), LibraryError>;
}

