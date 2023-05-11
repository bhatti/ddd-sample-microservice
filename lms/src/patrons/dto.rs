use chrono::{NaiveDateTime, Utc};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use crate::core::domain::Identifiable;
use crate::core::library::Role;
use crate::patrons::Patron;


// Patron abstracts library member.
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub(crate) struct PatronDto {
    pub patron_id: String,
    pub version: i64,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub under_13: bool,
    pub group_roles: Vec<Role>,
    pub num_holds: i64,
    pub num_overdue: i64,
    pub home_phone: Option<String>,
    pub cell_phone: Option<String>,
    pub work_phone: Option<String>,
    pub street_address: Option<String>,
    pub city: Option<String>,
    pub zip_code: Option<String>,
    pub state: Option<String>,
    pub country: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl PatronDto {
    pub(crate) fn new(email: &str) -> Self {
        Self {
            patron_id: Uuid::new_v4().to_string(),
            version: 0,
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
            street_address: None,
            city: None,
            zip_code: None,
            state: None,
            country: None,
            created_at: Utc::now().naive_utc(),
            updated_at: Utc::now().naive_utc(),
        }
    }
}

impl Identifiable for PatronDto {
    fn id(&self) -> String {
        self.patron_id.to_string()
    }

    fn version(&self) -> i64 {
        self.version
    }
}

impl Patron for PatronDto {
    fn is_admin(&self) -> bool {
        self.is_role(Role::Admin)
    }
    fn is_child(&self) -> bool {
        self.is_role(Role::Child)
    }
    fn is_employee(&self) -> bool {
        self.is_role(Role::Employee)
    }
    fn is_librarian(&self) -> bool {
        self.is_role(Role::Librarian)
    }
    fn is_role(&self, match_role: Role) -> bool {
        for role in self.group_roles.iter() {
            if *role == match_role {
                return true;
            }
        }
        false
    }
    fn is_regular(&self) -> bool {
        self.group_roles.is_empty() || self.is_role(Role::Regular)
    }
}

#[cfg(test)]
mod tests {
    use crate::core::library::Role;
    use crate::patrons::Patron;
    use crate::patrons::dto::PatronDto;

    #[tokio::test]
    async fn test_should_build_patron() {
        let patron = PatronDto::new("email@org.cc");
        assert_eq!("email@org.cc", patron.email.as_str());
        assert!(patron.is_regular());
        assert!(!patron.is_admin());
        assert!(!patron.is_employee());
        assert!(!patron.is_child());
        assert!(!patron.is_librarian());
    }

    #[tokio::test]
    async fn test_should_format_roles() {
        let roles = vec![
            Role::Admin,
            Role::Regular,
            Role::Child,
            Role::Employee,
            Role::Librarian,
        ];
        for role in roles {
            let str = role.to_string();
            let str_role = Role::from(str);
            assert_eq!(role, str_role);
        }
    }
}
