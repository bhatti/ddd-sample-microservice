use std::collections::HashMap;
use async_trait::async_trait;
use crate::core::domain::Configuration;
use crate::core::library::{LibraryResult, PartyKind, Role};
use crate::parties::domain::model::{AddressEntity, PartyEntity};
use crate::parties::repository::PartyRepository;
use crate::patrons::domain::PatronService;
use crate::patrons::dto::PatronDto;

pub(crate) struct PatronServiceImpl {
    party_repository: Box<dyn PartyRepository>,
}

impl PatronServiceImpl {
    pub(crate) fn new(_config: &Configuration, party_repository: Box<dyn PartyRepository>) -> Self {
        PatronServiceImpl {
            party_repository,
        }
    }
}

#[async_trait]
impl PatronService for PatronServiceImpl {
    async fn add_patron(&self, patron: &PatronDto) -> LibraryResult<()> {
        self.party_repository.create(&PartyEntity::from(patron)).await.map(|_| ())
    }

    async fn remove_patron(&self, id: &str) -> LibraryResult<()> {
        self.party_repository.delete(id).await.map(|_| ())
    }

    async fn update_patron(&self, patron: &PatronDto) -> LibraryResult<()> {
        self.party_repository.update(&PartyEntity::from(patron)).await.map(|_| ())
    }

    async fn find_patron_by_id(&self, id: &str) -> LibraryResult<PatronDto> {
        self.party_repository.get(id).await.map(|p| PatronDto::from(&p))
    }
    async fn find_patron_by_email(&self, email: &str) -> LibraryResult<Vec<PatronDto>> {
        let res = self.party_repository.query(
            &HashMap::from([("email".to_string(), email.to_string()),
                ("kind".to_string(), PartyKind::Patron.to_string())]), None, 100).await?;
        Ok(res.records.iter().map(PatronDto::from).collect())
    }
}

impl From<&PartyEntity> for PatronDto {
    fn from(other: &PartyEntity) -> Self {
        let mut patron = Self {
            patron_id: other.party_id.to_string(),
            version: other.version,
            first_name: other.first_name.to_string(),
            last_name: other.last_name.to_string(),
            email: other.email.to_string(),
            under_13: other.under_13,
            group_roles: other.group_roles.iter().map(|r| Role::from(r.to_string())).collect(),
            num_holds: other.num_holds,
            num_overdue: other.num_overdue,
            home_phone: other.home_phone.clone(),
            cell_phone: other.cell_phone.clone(),
            work_phone: other.work_phone.clone(),
            street_address: None,
            city: None,
            zip_code: None,
            state: None,
            country: None,
            created_at: other.created_at,
            updated_at: other.updated_at,
        };
        if let Some(address) = &other.address {
            patron.street_address = Some(address.street_address.to_string());
            patron.city = Some(address.city.to_string());
            patron.zip_code = Some(address.zip_code.to_string());
            patron.state = Some(address.state.to_string());
            patron.country = Some(address.country.to_string());
        }
        patron
    }
}

impl From<&PatronDto> for PartyEntity {
    fn from(other: &PatronDto) -> Self {
        let mut patron = PartyEntity {
            party_id: other.patron_id.to_string(),
            version: other.version,
            kind: PartyKind::Patron,
            first_name: other.first_name.to_string(),
            last_name: other.last_name.to_string(),
            email: other.email.to_string(),
            under_13: other.under_13,
            group_roles: other.group_roles.iter().map(|r| r.to_string()).collect(),
            num_holds: other.num_holds,
            num_overdue: other.num_overdue,
            home_phone: other.home_phone.clone(),
            cell_phone: other.cell_phone.clone(),
            work_phone: other.work_phone.clone(),
            address: None,
            created_at: other.created_at,
            updated_at: other.updated_at,
        };
        if let (Some(street_address), Some(city), Some(zip_code), Some(state), Some(country)) =
        (&other.street_address, &other.city, &other.zip_code, &other.state, &other.country) {
            patron.address = Some(AddressEntity {
                street_address: street_address.to_string(),
                city: city.to_string(),
                zip_code: zip_code.to_string(),
                state: state.to_string(),
                country: country.to_string(),
                created_at: other.created_at,
                updated_at: other.updated_at,
            });
        }
        patron
    }
}


#[cfg(test)]
mod tests {
    use async_once::AsyncOnce;
    use lazy_static::lazy_static;
    use crate::core::domain::Configuration;
    use crate::core::repository::RepositoryStore;
    use crate::patrons::domain::PatronService;
    use crate::patrons::dto::PatronDto;
    use crate::patrons::factory;

    lazy_static! {
        static ref SUT_SVC: AsyncOnce<Box<dyn PatronService>> = AsyncOnce::new(async {
                factory::create_patron_service(&Configuration::new("test"), RepositoryStore::LocalDynamoDB).await
            });
    }

    #[tokio::test]
    async fn test_should_add_patron() {
        let patron_svc = SUT_SVC.get().await.clone();

        let patron = PatronDto::new("email");
        let _ = patron_svc.add_patron(&patron).await.expect("should add parton");

        let loaded = patron_svc.find_patron_by_id(patron.patron_id.as_str()).await.expect("should return patron");
        assert_eq!(patron.patron_id, loaded.patron_id);
    }

    #[tokio::test]
    async fn test_should_update_patron() {
        let patron_svc = SUT_SVC.get().await.clone();

        let mut patron = PatronDto::new("email");
        let _ = patron_svc.add_patron(&patron).await.expect("should add patron");

        patron.email = "new_email".to_string();
        patron.first_name = "new_first".to_string();
        let _ = patron_svc.update_patron(&patron).await.expect("should update patron");

        let loaded = patron_svc.find_patron_by_id(patron.patron_id.as_str()).await.expect("should return patron");
        assert_eq!(patron.email, loaded.email);
        assert_eq!(patron.first_name, loaded.first_name);
    }


    #[tokio::test]
    async fn test_should_find_by_email() {
        let patron_svc = SUT_SVC.get().await.clone();

        let patron = PatronDto::new("email.xyz");
        let _ = patron_svc.add_patron(&patron).await.expect("should add patron");
        let res = patron_svc.find_patron_by_email(patron.email.as_str()).await.expect("should return patron");
        assert_eq!(1, res.len());
    }

    #[tokio::test]
    async fn test_should_remove_patron() {
        let patron_svc = SUT_SVC.get().await.clone();

        let patron = PatronDto::new("email");
        let _ = patron_svc.add_patron(&patron).await.expect("should add patron");

        let _ = patron_svc.remove_patron(patron.patron_id.as_str()).await.expect("should remove patron");

        let loaded = patron_svc.find_patron_by_id(patron.patron_id.as_str()).await;
        assert!(loaded.is_err());
    }
}
