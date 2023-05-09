use crate::core::repository::RepositoryStore;
use crate::gateway::ddb::publisher::DDBPublisher;
use crate::gateway::events::EventPublisher;
use crate::gateway::GatewayPublisherVia;
use crate::gateway::sns::publisher::SESPublisher;
use crate::utils::ddb::{build_db_client, build_ses_client};

pub(crate) async fn create_publisher(via: GatewayPublisherVia) -> Box<dyn EventPublisher> {
    match via {
        GatewayPublisherVia::Sns => {
            let client = build_ses_client().await;
            Box::new(SESPublisher::new(client))
        }
        GatewayPublisherVia::LocalDynamoDB => {
            let client = build_db_client(RepositoryStore::LocalDynamoDB).await;
            Box::new(DDBPublisher::new(client, "events", "events_ndx"))
        }
    }
}
