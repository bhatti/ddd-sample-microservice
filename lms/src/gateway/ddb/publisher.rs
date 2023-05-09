use async_trait::async_trait;
use aws_sdk_dynamodb::Client;
use crate::core::events::DomainEvent;
use crate::core::library::LibraryError;
use crate::gateway::events::EventPublisher;
use crate::utils::ddb::parse_item;

#[derive(Debug)]
pub struct DDBPublisher {
    client: Client,
    table_name: String,
}

impl DDBPublisher {
    pub(crate) fn new(client: Client, table_name: &str, _index_name: &str) -> Self {
        Self {
            client,
            table_name: table_name.to_string(),
        }
    }
}

#[async_trait]
impl EventPublisher for DDBPublisher {
    async fn create_topic(&mut self, _topic: &str) -> Result<String, LibraryError> {
        Ok("".to_string())
    }

    async fn get_topics(&mut self) -> Result<Vec<String>, LibraryError> {
        Ok(vec![])
    }

    async fn publish(&self, event: &DomainEvent) -> Result<(), LibraryError> {
        let table_name: &str = self.table_name.as_ref();
        let val = serde_json::to_value(event)?;
        self.client
            .put_item()
            .table_name(table_name)
            .condition_expression("attribute_not_exists(event_id)")
            .set_item(Some(parse_item(val)?))
            .send()
            .await.map(|_|()).map_err(LibraryError::from)
    }
}


#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use async_once::AsyncOnce;
    use aws_sdk_dynamodb::Client;
    use lazy_static::lazy_static;
    use crate::core::events::DomainEvent;
    use crate::core::repository::RepositoryStore;

    use crate::gateway::ddb::publisher::DDBPublisher;
    use crate::gateway::events::EventPublisher;
    use crate::utils::ddb::{build_db_client, create_table, delete_table};

    lazy_static! {
        static ref CLIENT: AsyncOnce<Client> = AsyncOnce::new(async {
                let client = build_db_client(RepositoryStore::LocalDynamoDB).await;
                let _ = delete_table(&client, "events").await;
                let _ = create_table(&client, "events", "event_id", "group", "key").await;
                client
            });
    }

    #[tokio::test]
    async fn test_should_publish_to_ddb() {
        let data = HashMap::from([("a", 1), ("b", 2)]);
        let event = DomainEvent::added("test-name", "group", "key", &HashMap::from([("k".to_string(), "v".to_string())]), &data).expect("build event");
        let mut publisher = DDBPublisher::new(CLIENT.get().await.clone(), "events", "events_ndx");
        let _arn = publisher.create_topic(event.name.as_str()).await.expect("should create topic");
        let _ = publisher.publish(&event).await.expect("should publish");
        let topics = publisher.get_topics().await.expect("should get topics");
        assert_eq!(0, topics.len());
    }
}
