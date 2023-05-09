pub mod ddb;
pub mod events;
pub mod logs;
pub mod sns;
pub mod factory;

#[derive(Debug, PartialEq)]
pub(crate) enum GatewayPublisherVia {
    Sns,
    LocalDynamoDB,
}

#[cfg(test)]
mod tests {
    use crate::gateway::GatewayPublisherVia;

    #[tokio::test]
    async fn test_should_create_sns_via() {
        let _ = GatewayPublisherVia::Sns;
        let _ = GatewayPublisherVia::LocalDynamoDB;
    }
}

