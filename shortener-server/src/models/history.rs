use chrono::{DateTime, Utc};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

/// History entity model
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "histories")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,

    #[sea_orm(indexed)]
    pub url_id: i32,

    #[sea_orm(indexed)]
    pub short_code: String,

    pub ip_address: String,
    pub user_agent: String,
    pub referer: Option<String>,

    // GeoIP information
    pub country: Option<String>,
    pub region: Option<String>,
    pub province: Option<String>,
    pub city: Option<String>,
    pub isp: Option<String>,

    // User-Agent parsed information
    pub device_type: Option<String>,
    pub os: Option<String>,
    pub browser: Option<String>,

    #[sea_orm(indexed)]
    pub accessed_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::url::Entity",
        from = "Column::UrlId",
        to = "super::url::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Url,
}

impl Related<super::url::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Url.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_history_model_clone() {
        let model = Model {
            id: 1,
            url_id: 1,
            short_code: "test123".to_string(),
            ip_address: "192.168.1.1".to_string(),
            user_agent: "Mozilla/5.0".to_string(),
            referer: Some("https://google.com".to_string()),
            country: Some("US".to_string()),
            region: Some("California".to_string()),
            province: Some("CA".to_string()),
            city: Some("San Francisco".to_string()),
            isp: Some("Comcast".to_string()),
            device_type: Some("Desktop".to_string()),
            os: Some("Windows".to_string()),
            browser: Some("Chrome".to_string()),
            accessed_at: chrono::Utc::now(),
            created_at: chrono::Utc::now(),
        };

        let cloned = model.clone();
        assert_eq!(model.id, cloned.id);
        assert_eq!(model.url_id, cloned.url_id);
        assert_eq!(model.short_code, cloned.short_code);
        assert_eq!(model.ip_address, cloned.ip_address);
    }

    #[test]
    fn test_history_model_with_minimal_fields() {
        let model = Model {
            id: 1,
            url_id: 1,
            short_code: "test123".to_string(),
            ip_address: "192.168.1.1".to_string(),
            user_agent: "Mozilla/5.0".to_string(),
            referer: None,
            country: None,
            region: None,
            province: None,
            city: None,
            isp: None,
            device_type: None,
            os: None,
            browser: None,
            accessed_at: chrono::Utc::now(),
            created_at: chrono::Utc::now(),
        };

        assert!(!model.user_agent.is_empty());
        assert!(model.country.is_none());
        assert!(model.device_type.is_none());
    }

    #[test]
    fn test_history_model_partial_eq() {
        let now = chrono::Utc::now();
        let model1 = Model {
            id: 1,
            url_id: 1,
            short_code: "test123".to_string(),
            ip_address: "192.168.1.1".to_string(),
            user_agent: "Mozilla/5.0".to_string(),
            referer: None,
            country: Some("US".to_string()),
            region: None,
            province: None,
            city: None,
            isp: None,
            device_type: Some("Desktop".to_string()),
            os: Some("Windows".to_string()),
            browser: Some("Chrome".to_string()),
            accessed_at: now,
            created_at: now,
        };

        let model2 = model1.clone();
        assert_eq!(model1, model2);
    }

    #[test]
    fn test_history_model_serialization() {
        let model = Model {
            id: 1,
            url_id: 1,
            short_code: "test123".to_string(),
            ip_address: "192.168.1.1".to_string(),
            user_agent: "Mozilla/5.0".to_string(),
            referer: None,
            country: Some("US".to_string()),
            region: None,
            province: None,
            city: None,
            isp: None,
            device_type: Some("Desktop".to_string()),
            os: Some("Windows".to_string()),
            browser: Some("Chrome".to_string()),
            accessed_at: chrono::Utc::now(),
            created_at: chrono::Utc::now(),
        };

        let json = serde_json::to_string(&model).unwrap();
        assert!(json.contains("test123"));
        assert!(json.contains("192.168.1.1"));

        let deserialized: Model = serde_json::from_str(&json).unwrap();
        assert_eq!(model.id, deserialized.id);
        assert_eq!(model.short_code, deserialized.short_code);
    }

    #[test]
    fn test_history_model_with_ipv6() {
        let model = Model {
            id: 1,
            url_id: 1,
            short_code: "test123".to_string(),
            ip_address: "2001:0db8:85a3:0000:0000:8a2e:0370:7334".to_string(),
            user_agent: "Mozilla/5.0".to_string(),
            referer: None,
            country: None,
            region: None,
            province: None,
            city: None,
            isp: None,
            device_type: None,
            os: None,
            browser: None,
            accessed_at: chrono::Utc::now(),
            created_at: chrono::Utc::now(),
        };

        assert!(model.ip_address.contains("2001"));
    }
}
