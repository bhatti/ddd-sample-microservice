use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::core::domain::Identifiable;
use crate::core::library::PartyKind;
use crate::utils::date::serializer;

// Party abstracts person, patron, employee, branch, organization based on https://martinfowler.com/apsupp/accountability.pdf
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct PartyEntity {
    pub party_id: String,
    pub version: i64,
    pub kind: PartyKind,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub under_13: bool,
    pub group_roles: Vec<String>,
    pub num_holds: i64,
    pub num_overdue: i64,
    pub home_phone: Option<String>,
    pub cell_phone: Option<String>,
    pub work_phone: Option<String>,
    pub address: Option<AddressEntity>,
    #[serde(with = "serializer")]
    pub created_at: NaiveDateTime,
    #[serde(with = "serializer")]
    pub updated_at: NaiveDateTime,
}

// Address defines physical location
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub(crate) struct AddressEntity {
    pub street_address: String,
    pub city: String,
    pub zip_code: String,
    pub state: String,
    pub country: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl PartyEntity {
    pub fn new(kind: PartyKind, email: &str) -> Self {
        Self {
            party_id: Uuid::new_v4().to_string(),
            version: 0,
            kind,
            first_name: "".to_string(),
            last_name: "".to_string(),
            email: email.to_string(),
            under_13: false,
            group_roles: vec![],
            num_holds: 0,
            num_overdue: 0,
            home_phone: None,
            cell_phone: None,
            work_phone: None,
            address: None,
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
        }
    }
}

impl AddressEntity {
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }
    pub fn from_json(data: String) -> Option<AddressEntity> {
        if let Ok(addr) = serde_json::from_str(data.as_str()) {
            Some(addr)
        } else {
            None
        }
    }
}

impl Default for AddressEntity {
    fn default() -> Self {
        AddressEntity {
            street_address: "".to_string(),
            city: "".to_string(),
            zip_code: "".to_string(),
            state: "".to_string(),
            country: "".to_string(),
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
        }
    }
}

impl Identifiable for PartyEntity {
    fn id(&self) -> String {
        self.party_id.to_string()
    }

    fn version(&self) -> i64 {
        self.version
    }
}


#[cfg(test)]
mod tests {
    use chrono::Utc;
    use crate::core::library::PartyKind;
    use crate::parties::domain::model::{AddressEntity, PartyEntity};

    #[tokio::test]
    async fn test_should_build_party() {
        let patron = PartyEntity::new(PartyKind::Patron, "email@org.cc");
        assert_eq!("email@org.cc", patron.email.as_str());
        assert_eq!(PartyKind::Patron, patron.kind);
    }

    #[tokio::test]
    async fn test_should_serialize_address() {
        let address = AddressEntity {
            street_address: "100 main st.".to_string(),
            city: "Seattle".to_string(),
            zip_code: "980101".to_string(),
            state: "WA".to_string(),
            country: "US".to_string(),
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
        };
        let str = address.to_json();
        let des_address = AddressEntity::from_json(str).unwrap();
        assert_eq!(address.street_address, des_address.street_address);
    }
}
