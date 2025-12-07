use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// URL entity model
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "urls")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,

    #[sea_orm(unique, indexed)]
    pub short_code: String,

    pub original_url: String,

    pub description: Option<String>,

    #[sea_orm(indexed)]
    pub status: i32,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// URL status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UrlStatus {
    Enabled = 0,
    Disabled = 1,
}

impl From<i32> for UrlStatus {
    fn from(value: i32) -> Self {
        match value {
            0 => UrlStatus::Enabled,
            1 => UrlStatus::Disabled,
            _ => UrlStatus::Enabled, // Default to Enabled
        }
    }
}

impl From<UrlStatus> for i32 {
    fn from(status: UrlStatus) -> Self {
        status as i32
    }
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::history::Entity")]
    History,
}

impl Related<super::history::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::History.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_status_conversion() {
        assert_eq!(UrlStatus::from(0), UrlStatus::Enabled);
        assert_eq!(UrlStatus::from(1), UrlStatus::Disabled);
        assert_eq!(UrlStatus::from(999), UrlStatus::Enabled); // Unknown defaults to Enabled

        assert_eq!(i32::from(UrlStatus::Enabled), 0);
        assert_eq!(i32::from(UrlStatus::Disabled), 1);
    }

    #[test]
    fn test_url_status_equality() {
        assert_eq!(UrlStatus::Enabled, UrlStatus::Enabled);
        assert_eq!(UrlStatus::Disabled, UrlStatus::Disabled);
        assert_ne!(UrlStatus::Enabled, UrlStatus::Disabled);
    }

    #[test]
    fn test_url_status_copy() {
        let status1 = UrlStatus::Enabled;
        let status2 = status1;
        assert_eq!(status1, status2);
    }

    #[test]
    fn test_url_model_clone() {
        let model = Model {
            id: 1,
            short_code: "test123".to_string(),
            original_url: "https://example.com".to_string(),
            description: Some("Test".to_string()),
            status: UrlStatus::Enabled as i32,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let cloned = model.clone();
        assert_eq!(model.id, cloned.id);
        assert_eq!(model.short_code, cloned.short_code);
        assert_eq!(model.original_url, cloned.original_url);
    }

    #[test]
    fn test_url_model_partial_eq() {
        let now = chrono::Utc::now();
        let model1 = Model {
            id: 1,
            short_code: "test123".to_string(),
            original_url: "https://example.com".to_string(),
            description: Some("Test".to_string()),
            status: UrlStatus::Enabled as i32,
            created_at: now,
            updated_at: now,
        };

        let model2 = Model {
            id: 1,
            short_code: "test123".to_string(),
            original_url: "https://example.com".to_string(),
            description: Some("Test".to_string()),
            status: UrlStatus::Enabled as i32,
            created_at: now,
            updated_at: now,
        };

        assert_eq!(model1, model2);
    }

    #[test]
    fn test_url_model_with_none_description() {
        let model = Model {
            id: 1,
            short_code: "test123".to_string(),
            original_url: "https://example.com".to_string(),
            description: None,
            status: UrlStatus::Enabled as i32,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        assert!(model.description.is_none());
    }

    #[test]
    fn test_url_status_serialization() {
        let status = UrlStatus::Enabled;
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("Enabled"));

        let deserialized: UrlStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status, deserialized);
    }
}
