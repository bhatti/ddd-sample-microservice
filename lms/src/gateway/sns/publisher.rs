use std::collections::HashMap;
use aws_sdk_sns::Client;
use async_trait::async_trait;
use aws_sdk_sns::error::SdkError;
use aws_sdk_sns::operation::create_topic::CreateTopicError;
use aws_sdk_sns::operation::list_topics::ListTopicsError;
use aws_sdk_sns::operation::publish::PublishError;
use tracing::log::info;
use crate::core::events::DomainEvent;
use crate::core::library::LibraryError;
use crate::gateway::events::EventPublisher;

#[derive(Debug)]
pub struct SESPublisher {
    client: Client,
    topics: HashMap<String, String>,
}

impl SESPublisher {
    pub(crate) fn new(client: Client) -> Self {
        Self {
            client,
            topics: HashMap::new(),
        }
    }
}

#[async_trait]
impl EventPublisher for SESPublisher {
    async fn create_topic(&mut self, topic: &str) -> Result<String, LibraryError> {
        let resp = self.client.create_topic().name(topic).send().await?;
        let arn = resp.topic_arn().unwrap_or_default();
        self.topics.insert(topic.to_string(), arn.to_string());
        info!("Created topic with ARN: {}", arn);
        Ok(arn.to_string())
    }

    async fn get_topics(&mut self) -> Result<Vec<String>, LibraryError> {
        let mut topics = vec![];
        let resp = self.client.list_topics().send().await?;
        for topic in resp.topics().unwrap_or_default() {
            topics.push(topic.topic_arn().unwrap_or_default().to_string());
        }
        Ok(topics)
    }

    async fn publish(&self, event: &DomainEvent) -> Result<(), LibraryError> {
        let topic = self.topics.get(event.name.as_str());
        if let Some(arn) = topic {
            let json = serde_json::to_string(event)?;
            self.client.publish().topic_arn(arn).message(json).send().await?;
            Ok(())
        } else {
            Err(LibraryError::runtime(format!("topic is not found {}", event.name).as_str(), None))
        }
    }
}

impl From<SdkError<CreateTopicError>> for LibraryError {
    fn from(err: SdkError<CreateTopicError>) -> Self {
        LibraryError::runtime(format!("{:?}", err).as_str(), None)
    }
}

impl From<SdkError<ListTopicsError>> for LibraryError {
    fn from(err: SdkError<ListTopicsError>) -> Self {
        LibraryError::runtime(format!("{:?}", err).as_str(), None)
    }
}

impl From<SdkError<PublishError>> for LibraryError {
    fn from(err: SdkError<PublishError>) -> Self {
        LibraryError::runtime(format!("{:?}", err).as_str(), None)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use crate::core::events::DomainEvent;
    use crate::gateway::{factory, GatewayPublisherVia};

    #[tokio::test]
    async fn test_should_publish_to_sns() {
        let data = HashMap::from([("a", 1), ("b", 2)]);
        let event = DomainEvent::added("test-name", "group", "key", &HashMap::from([("k".to_string(), "v".to_string())]), &data).expect("build event");
        let mut publisher = factory::create_publisher(GatewayPublisherVia::Sns).await;
        let arn = publisher.create_topic(event.name.as_str()).await.expect("should create topic");
        let _ = publisher.publish(&event).await.expect("should publish");
        let topics = publisher.get_topics().await.expect("should get topics");
        assert!(topics.contains(&arn));
    }
}
