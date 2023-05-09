use async_trait::async_trait;
use core::option::Option;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::core::library::{LibraryResult, PaginatedResult};
use crate::gateway::GatewayPublisherVia;

#[async_trait]
pub trait Repository<Entity>: Sync + Send {
    // create an entity
    async fn create(&self, entity: &Entity) -> LibraryResult<usize>;

    // updates an entity
    async fn update(&self, entity: &Entity) -> LibraryResult<usize>;

    // get an entity
    async fn get(&self, id: &str) -> LibraryResult<Entity>;

    // delete an entity
    async fn delete(&self, id: &str) -> LibraryResult<usize>;

    // find by tenant_id
    async fn query(&self, predicate: &HashMap::<String, String>,
                   page: Option<&str>, page_size: usize) -> LibraryResult<PaginatedResult<Entity>>;
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone, Copy)]
pub(crate) enum RepositoryStore {
    DynamoDB,
    LocalDynamoDB,
}

impl RepositoryStore {
    pub fn gateway_publisher(&self) -> GatewayPublisherVia  {
        match self {
            RepositoryStore::DynamoDB => {GatewayPublisherVia::Sns},
            RepositoryStore::LocalDynamoDB => {GatewayPublisherVia::LocalDynamoDB},
        }
    }
}