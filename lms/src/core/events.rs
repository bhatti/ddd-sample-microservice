use std::collections::HashMap;
use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::utils::date::{serializer};

// DomainEventType defines type of event for domain changes
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub enum DomainEventType {
    Added,
    Updated,
    Deleted,
}

// DomainEvent abstracts domain event for data changes
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct DomainEvent {
    pub event_id: String,
    pub name: String,
    pub group: String,
    pub key: String,
    pub kind: DomainEventType,
    pub metadata: HashMap<String, String>,
    pub json_data: String,
    #[serde(with = "serializer")]
    pub created_at: NaiveDateTime,
}

impl DomainEvent {
    pub fn added<T: Serialize>(name: &str, group: &str, key: &str, metadata: &HashMap<String, String>, data: &T) -> serde_json::Result<Self> {
        let json = serde_json::to_string(&data)?;
        Ok(Self::build(name, group, key, DomainEventType::Added, metadata, json))
    }

    pub fn updated<T: Serialize>(name: &str, group: &str, key: &str, metadata: &HashMap<String, String>, data: &T) -> serde_json::Result<Self> {
        let json = serde_json::to_string(&data)?;
        Ok(Self::build(name, group, key, DomainEventType::Updated, metadata, json))
    }

    pub fn deleted<T: Serialize>(name: &str, group: &str, key: &str, metadata: &HashMap<String, String>, data: &T) -> serde_json::Result<Self> {
        let json = serde_json::to_string(&data)?;
        Ok(Self::build(name, group, key, DomainEventType::Deleted, metadata, json))
    }

    fn build(name: &str, group: &str, key: &str, kind: DomainEventType, metadata: &HashMap<String, String>, json: String) -> DomainEvent {
        DomainEvent {
            event_id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            group: group.to_string(),
            key: key.to_string(),
            kind,
            metadata: metadata.clone(),
            json_data: json,
            created_at: Utc::now().naive_utc(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use crate::core::events::{DomainEvent, DomainEventType};

    #[tokio::test]
    async fn test_should_build_added() {
        let data = HashMap::from([("a", 1), ("b", 2)]);
        let event = DomainEvent::added("name", "group", "key", &HashMap::from([("k".to_string(), "v".to_string())]), &data).expect("build event");
        assert_eq!("name", event.name.as_str());
        assert_eq!("key", event.key.as_str());
        assert_eq!(DomainEventType::Added, event.kind);
    }

    #[tokio::test]
    async fn test_should_build_updated() {
        let data = HashMap::from([("a", 1), ("b", 2)]);
        let event = DomainEvent::updated("name", "group", "key", &HashMap::from([("k".to_string(), "v".to_string())]), &data).expect("build event");
        assert_eq!("name", event.name.as_str());
        assert_eq!("key", event.key.as_str());
        assert_eq!(DomainEventType::Updated, event.kind);
    }

    #[tokio::test]
    async fn test_should_build_deleted() {
        let data = HashMap::from([("a", 1), ("b", 2)]);
        let event = DomainEvent::deleted("name", "group", "key", &HashMap::from([("k".to_string(), "v".to_string())]), &data).expect("build event");
        assert_eq!("name", event.name.as_str());
        assert_eq!("key", event.key.as_str());
        assert_eq!(DomainEventType::Deleted, event.kind);
    }
}
