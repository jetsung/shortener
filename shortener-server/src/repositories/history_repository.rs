use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, PaginatorTrait,
    QueryFilter, QueryOrder, Set,
};
use serde::{Deserialize, Serialize};

use crate::models::history::{ActiveModel, Column, Entity, Model};

/// DTO for creating a history record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateHistoryDto {
    pub url_id: i32,
    pub short_code: String,
    pub ip_address: String,
    pub user_agent: Option<String>,
    pub referer: Option<String>,
    pub country: Option<String>,
    pub region: Option<String>,
    pub province: Option<String>,
    pub city: Option<String>,
    pub isp: Option<String>,
    pub device_type: Option<String>,
    pub os: Option<String>,
    pub browser: Option<String>,
    pub accessed_at: chrono::NaiveDateTime,
}

/// Parameters for listing history records
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryListParams {
    pub page: u64,
    pub page_size: u64,
    pub short_code: Option<String>,
    pub url_id: Option<i32>,
    pub sort_by: Option<String>,
    pub order: Option<String>,
}

impl Default for HistoryListParams {
    fn default() -> Self {
        Self {
            page: 1,
            page_size: 10,
            short_code: None,
            url_id: None,
            sort_by: None,
            order: None,
        }
    }
}

/// History Repository trait
#[async_trait]
pub trait HistoryRepository: Send + Sync {
    /// Create a new history record
    async fn create(&self, history: CreateHistoryDto) -> Result<Model, DbErr>;

    /// List history records with pagination
    async fn list(&self, params: HistoryListParams) -> Result<(Vec<Model>, u64), DbErr>;

    /// Delete multiple history records by IDs
    async fn delete_batch(&self, ids: Vec<i64>) -> Result<u64, DbErr>;
}

/// History Repository implementation
pub struct HistoryRepositoryImpl {
    db: DatabaseConnection,
}

impl HistoryRepositoryImpl {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl HistoryRepository for HistoryRepositoryImpl {
    async fn create(&self, history: CreateHistoryDto) -> Result<Model, DbErr> {
        let now = chrono::Utc::now().naive_utc();

        let active_model = ActiveModel {
            url_id: Set(history.url_id),
            short_code: Set(history.short_code),
            ip_address: Set(history.ip_address),
            user_agent: Set(history.user_agent),
            referer: Set(history.referer),
            country: Set(history.country),
            region: Set(history.region),
            province: Set(history.province),
            city: Set(history.city),
            isp: Set(history.isp),
            device_type: Set(history.device_type),
            os: Set(history.os),
            browser: Set(history.browser),
            accessed_at: Set(history.accessed_at),
            created_at: Set(now),
            ..Default::default()
        };

        active_model.insert(&self.db).await
    }

    async fn list(&self, params: HistoryListParams) -> Result<(Vec<Model>, u64), DbErr> {
        let mut query = Entity::find();

        // Apply filters
        if let Some(short_code) = params.short_code {
            query = query.filter(Column::ShortCode.eq(short_code));
        }
        if let Some(url_id) = params.url_id {
            query = query.filter(Column::UrlId.eq(url_id));
        }

        // Apply sorting
        let sort_column = params.sort_by.as_deref().unwrap_or("accessed_at");
        let order = params.order.as_deref().unwrap_or("desc");

        query = match (sort_column, order) {
            ("id", "asc") => query.order_by_asc(Column::Id),
            ("id", "desc") => query.order_by_desc(Column::Id),
            ("accessed_at", "asc") => query.order_by_asc(Column::AccessedAt),
            ("accessed_at", "desc") => query.order_by_desc(Column::AccessedAt),
            ("created_at", "asc") => query.order_by_asc(Column::CreatedAt),
            ("created_at", "desc") => query.order_by_desc(Column::CreatedAt),
            _ => query.order_by_desc(Column::AccessedAt), // Default
        };

        // Get total count
        let total = query.clone().count(&self.db).await?;

        // Apply pagination
        let paginator = query.paginate(&self.db, params.page_size);
        let items = paginator.fetch_page(params.page - 1).await?;

        Ok((items, total))
    }

    async fn delete_batch(&self, ids: Vec<i64>) -> Result<u64, DbErr> {
        let result = Entity::delete_many()
            .filter(Column::Id.is_in(ids))
            .exec(&self.db)
            .await?;

        Ok(result.rows_affected)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, DatabaseConfig, DatabaseType, SqliteConfig};
    use crate::db::DbFactory;
    use crate::models::url::UrlStatus;
    use crate::repositories::url_repository::{CreateUrlDto, UrlRepository, UrlRepositoryImpl};

    async fn setup_test_db() -> DatabaseConnection {
        let config = Config {
            server: crate::config::ServerConfig {
                address: ":8080".to_string(),
                trusted_platform: None,
                site_url: "http://localhost:8080".to_string(),
                api_key: "test-key".to_string(),
            },
            shortener: crate::config::ShortenerConfig {
                code_length: 6,
                code_charset: "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"
                    .to_string(),
            },
            admin: crate::config::AdminConfig {
                username: "admin".to_string(),
                password: "admin123".to_string(),
            },
            database: DatabaseConfig {
                db_type: DatabaseType::Sqlite,
                log_level: 0,
                sqlite: Some(SqliteConfig {
                    path: ":memory:".to_string(),
                }),
                postgres: None,
                mysql: None,
            },
            cache: crate::config::CacheConfig {
                enabled: false,
                cache_type: crate::config::CacheType::Redis,
                expire: 3600,
                prefix: "shorten:".to_string(),
                redis: None,
                valkey: None,
            },
            geoip: crate::config::GeoIpConfig {
                enabled: false,
                geoip_type: crate::config::GeoIpType::Ip2region,
                ip2region: None,
            },
            logging: crate::logging::LoggingConfig::default(),
        };

        let db = DbFactory::create_connection(&config).await.unwrap();
        DbFactory::run_migrations(&db).await.unwrap();
        db
    }

    async fn create_test_url(db: &DatabaseConnection) -> i64 {
        let url_repo = UrlRepositoryImpl::new(db.clone());
        let create_dto = CreateUrlDto {
            code: "test123".to_string(),
            original_url: "https://example.com".to_string(),
            describe: Some("Test URL".to_string()),
            status: UrlStatus::Enabled as i32,
        };
        let url = url_repo.create(create_dto).await.unwrap();
        url.id
    }

    #[tokio::test]
    async fn test_create_history() {
        let db = setup_test_db().await;
        let url_id = create_test_url(&db).await;
        let repo = HistoryRepositoryImpl::new(db);

        let accessed_at = chrono::Utc::now().naive_utc();
        let create_dto = CreateHistoryDto {
            url_id: url_id as i32,
            short_code: "test123".to_string(),
            ip_address: "192.168.1.1".to_string(),
            user_agent: Some("Mozilla/5.0".to_string()),
            referer: Some("https://google.com".to_string()),
            country: Some("US".to_string()),
            region: Some("California".to_string()),
            province: Some("CA".to_string()),
            city: Some("San Francisco".to_string()),
            isp: Some("Comcast".to_string()),
            device_type: Some("Desktop".to_string()),
            os: Some("Windows".to_string()),
            browser: Some("Chrome".to_string()),
            accessed_at,
        };

        let result = repo.create(create_dto).await;
        assert!(result.is_ok());

        let history = result.unwrap();
        assert_eq!(history.url_id, url_id as i32);
        assert_eq!(history.short_code, "test123");
        assert_eq!(history.ip_address, "192.168.1.1");
        assert_eq!(history.country, Some("US".to_string()));
    }

    #[tokio::test]
    async fn test_list_histories() {
        let db = setup_test_db().await;
        let url_id = create_test_url(&db).await;
        let repo = HistoryRepositoryImpl::new(db);

        // Create multiple history records
        for i in 1..=5 {
            let accessed_at = chrono::Utc::now().naive_utc();
            let create_dto = CreateHistoryDto {
                url_id: url_id as i32,
                short_code: "test123".to_string(),
                ip_address: format!("192.168.1.{}", i),
                user_agent: Some(format!("Browser {}", i)),
                referer: None,
                country: Some("US".to_string()),
                region: None,
                province: None,
                city: None,
                isp: None,
                device_type: None,
                os: None,
                browser: None,
                accessed_at,
            };
            repo.create(create_dto).await.unwrap();
        }

        // List all histories
        let params = HistoryListParams {
            page: 1,
            page_size: 10,
            ..Default::default()
        };
        let (histories, total) = repo.list(params).await.unwrap();
        assert_eq!(histories.len(), 5);
        assert_eq!(total, 5);

        // List with short_code filter
        let params = HistoryListParams {
            page: 1,
            page_size: 10,
            short_code: Some("test123".to_string()),
            ..Default::default()
        };
        let (histories, total) = repo.list(params).await.unwrap();
        assert_eq!(histories.len(), 5);
        assert_eq!(total, 5);

        // List with pagination
        let params = HistoryListParams {
            page: 1,
            page_size: 2,
            ..Default::default()
        };
        let (histories, total) = repo.list(params).await.unwrap();
        assert_eq!(histories.len(), 2);
        assert_eq!(total, 5);
    }

    #[tokio::test]
    async fn test_list_histories_with_url_id_filter() {
        let db = setup_test_db().await;
        let url_id = create_test_url(&db).await;
        let repo = HistoryRepositoryImpl::new(db);

        // Create history records
        let accessed_at = chrono::Utc::now().naive_utc();
        let create_dto = CreateHistoryDto {
            url_id: url_id as i32,
            short_code: "test123".to_string(),
            ip_address: "192.168.1.1".to_string(),
            user_agent: None,
            referer: None,
            country: None,
            region: None,
            province: None,
            city: None,
            isp: None,
            device_type: None,
            os: None,
            browser: None,
            accessed_at,
        };
        repo.create(create_dto).await.unwrap();

        // List with url_id filter
        let params = HistoryListParams {
            page: 1,
            page_size: 10,
            url_id: Some(url_id as i32),
            ..Default::default()
        };
        let (histories, total) = repo.list(params).await.unwrap();
        assert_eq!(histories.len(), 1);
        assert_eq!(total, 1);

        // List with non-existent url_id
        let params = HistoryListParams {
            page: 1,
            page_size: 10,
            url_id: Some(99999),
            ..Default::default()
        };
        let (histories, total) = repo.list(params).await.unwrap();
        assert_eq!(histories.len(), 0);
        assert_eq!(total, 0);
    }

    #[tokio::test]
    async fn test_delete_batch_histories() {
        let db = setup_test_db().await;
        let url_id = create_test_url(&db).await;
        let repo = HistoryRepositoryImpl::new(db);

        // Create multiple history records
        let mut ids = Vec::new();
        for i in 1..=5 {
            let accessed_at = chrono::Utc::now().naive_utc();
            let create_dto = CreateHistoryDto {
                url_id: url_id as i32,
                short_code: "test123".to_string(),
                ip_address: format!("192.168.1.{}", i),
                user_agent: None,
                referer: None,
                country: None,
                region: None,
                province: None,
                city: None,
                isp: None,
                device_type: None,
                os: None,
                browser: None,
                accessed_at,
            };
            let created = repo.create(create_dto).await.unwrap();
            ids.push(created.id);
        }

        // Delete first 3 histories
        let delete_ids = ids[0..3].to_vec();
        let result = repo.delete_batch(delete_ids).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);

        // Verify remaining histories
        let params = HistoryListParams::default();
        let (histories, total) = repo.list(params).await.unwrap();
        assert_eq!(histories.len(), 2);
        assert_eq!(total, 2);
    }

    #[tokio::test]
    async fn test_list_histories_sorting() {
        let db = setup_test_db().await;
        let url_id = create_test_url(&db).await;
        let repo = HistoryRepositoryImpl::new(db);

        // Create history records with different timestamps
        for i in 1..=3 {
            let accessed_at = chrono::Utc::now().naive_utc() - chrono::Duration::hours(i);
            let create_dto = CreateHistoryDto {
                url_id: url_id as i32,
                short_code: "test123".to_string(),
                ip_address: format!("192.168.1.{}", i),
                user_agent: None,
                referer: None,
                country: None,
                region: None,
                province: None,
                city: None,
                isp: None,
                device_type: None,
                os: None,
                browser: None,
                accessed_at,
            };
            repo.create(create_dto).await.unwrap();
        }

        // List with ascending order
        let params = HistoryListParams {
            page: 1,
            page_size: 10,
            sort_by: Some("accessed_at".to_string()),
            order: Some("asc".to_string()),
            ..Default::default()
        };
        let (histories, _) = repo.list(params).await.unwrap();
        assert_eq!(histories.len(), 3);
        // First should be oldest
        assert!(histories[0].accessed_at < histories[1].accessed_at);
        assert!(histories[1].accessed_at < histories[2].accessed_at);

        // List with descending order (default)
        let params = HistoryListParams {
            page: 1,
            page_size: 10,
            sort_by: Some("accessed_at".to_string()),
            order: Some("desc".to_string()),
            ..Default::default()
        };
        let (histories, _) = repo.list(params).await.unwrap();
        assert_eq!(histories.len(), 3);
        // First should be newest
        assert!(histories[0].accessed_at > histories[1].accessed_at);
        assert!(histories[1].accessed_at > histories[2].accessed_at);
    }
}
