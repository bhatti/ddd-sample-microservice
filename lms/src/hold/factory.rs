use crate::catalog::factory::create_catalog_service;
use crate::core::domain::Configuration;
use crate::hold::domain::HoldService;
use crate::hold::domain::service::HoldServiceImpl;
use crate::hold::repository::ddb_hold_repository::DDBHoldRepository;
use crate::hold::repository::HoldRepository;
use crate::core::repository::RepositoryStore;
use crate::gateway::factory::create_publisher;
use crate::patrons::factory::create_patron_service;
use crate::utils::ddb::{build_db_client, create_table};

pub(crate) async fn create_hold_repository(store: RepositoryStore) -> Box<dyn HoldRepository> {
    match store {
        RepositoryStore::DynamoDB => {
            let client = build_db_client(store).await;
            Box::new(DDBHoldRepository::new(client, "hold", "hold_ndx"))
        }
        RepositoryStore::LocalDynamoDB => {
            let client = build_db_client(store).await;
            let _ = create_table(&client, "hold", "hold_id", "hold_status", "patron_id").await;
            Box::new(DDBHoldRepository::new(client, "hold", "hold_ndx"))
        }
    }
}

pub(crate) async fn create_hold_service(config: &Configuration, store: RepositoryStore) -> Box<dyn HoldService> {
    let hold_repository = create_hold_repository(store).await;
    let catalog_svc = create_catalog_service(config, store).await;
    let patron_svc = create_patron_service(config, store).await;
    let publisher = create_publisher(store.gateway_publisher()).await;
    Box::new(HoldServiceImpl::new(config, hold_repository, patron_svc, catalog_svc, publisher))
}
