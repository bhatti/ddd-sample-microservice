use crate::catalog::factory::create_catalog_service;
use crate::checkout::domain::CheckoutService;
use crate::checkout::domain::service::CheckoutServiceImpl;
use crate::checkout::factory;
use crate::checkout::repository::CheckoutRepository;
use crate::checkout::repository::ddb_checkout_repository::DDBCheckoutRepository;
use crate::core::domain::Configuration;
use crate::core::repository::RepositoryStore;
use crate::gateway::factory::create_publisher;
use crate::patrons::factory::create_patron_service;
use crate::utils::ddb::{build_db_client, create_table};

pub(crate) async fn create_checkout_repository(store: RepositoryStore) -> Box<dyn CheckoutRepository> {
    match store {
        RepositoryStore::DynamoDB => {
            let client = build_db_client(store).await;
            Box::new(DDBCheckoutRepository::new(client, "checkout", "checkout_ndx"))
        }
        RepositoryStore::LocalDynamoDB => {
            let client = build_db_client(store).await;
            let _ = create_table(&client, "checkout", "checkout_id", "checkout_status", "patron_id").await;
            Box::new(DDBCheckoutRepository::new(client, "checkout", "checkout_ndx"))
        }
    }
}

pub(crate) async fn create_checkout_service(config: &Configuration, store: RepositoryStore) -> Box<dyn CheckoutService> {
    let checkout_repo = factory::create_checkout_repository(store).await;
    let catalog_svc = create_catalog_service(config, store).await;
    let patron_svc = create_patron_service(config, store).await;
    let publisher = create_publisher(store.gateway_publisher()).await;
    Box::new(CheckoutServiceImpl::new(config, checkout_repo,
                                      patron_svc, catalog_svc, publisher))
}
