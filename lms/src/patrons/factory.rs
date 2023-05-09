use crate::core::domain::Configuration;
use crate::parties::factory;
use crate::core::repository::RepositoryStore;
use crate::patrons::domain::PatronService;
use crate::patrons::domain::service::PatronServiceImpl;

pub(crate) async fn create_patron_service(config: &Configuration, store: RepositoryStore) -> Box<dyn PatronService> {
    let party_repo = factory::create_party_repository(store).await;
    Box::new(PatronServiceImpl::new(config, party_repo))
}
