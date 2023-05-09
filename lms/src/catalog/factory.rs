use crate::books::factory;
use crate::catalog::domain::CatalogService;
use crate::catalog::domain::service::CatalogServiceImpl;
use crate::core::domain::Configuration;
use crate::core::repository::RepositoryStore;
use crate::gateway::factory::create_publisher;

pub(crate) async fn create_catalog_service(config: &Configuration, store: RepositoryStore) -> Box<dyn CatalogService> {
    let book_repo = factory::create_book_repository(store).await;
    let publisher = create_publisher(store.gateway_publisher()).await;
    Box::new(CatalogServiceImpl::new(config, book_repo, publisher))
}
