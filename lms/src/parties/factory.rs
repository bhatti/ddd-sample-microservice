use crate::parties::repository::ddb_party_repository::DDBPartyRepository;
use crate::core::repository::RepositoryStore;
use crate::parties::repository::PartyRepository;
use crate::utils::ddb::{build_db_client, create_table};

pub(crate) async fn create_party_repository(store: RepositoryStore) -> Box<dyn PartyRepository> {
    match store {
        RepositoryStore::DynamoDB => {
            let client = build_db_client(store).await;
            Box::new(DDBPartyRepository::new(client, "parties", "parties_ndx"))
        }
        RepositoryStore::LocalDynamoDB => {
            let client = build_db_client(store).await;
            let _ = create_table(&client, "parties", "party_id", "kind", "email").await;
            Box::new(DDBPartyRepository::new(client, "parties", "parties_ndx"))
        }
    }
}
