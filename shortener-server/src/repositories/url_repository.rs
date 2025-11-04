use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, PaginatorTrait,
    QueryFilter, QueryOrder, Set,
};
use serde::{Deserialize, Serialize};

use crate::models::url::{ActiveModel, Column, Entity, Model};

/// DTO for creating a URL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUrlDto {
    pub short_code: String,
    pub original_url: String,
    pub description: Option<String>,
    pub status: i32,
}

/// DTO for updating a URL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUrlDto {
    pub original_url: Option<String>,
    pub description: Option<String>,
    pub status: Option<i32>,
}

/// Parameters for listing URLs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListParams {
    #[serde(default = "default_page")]
    pub page: u64,
    #[serde(default = "default_per_page", rename = "per_page")]
    pub page_size: u64,
    pub short_code: Option<String>,
    pub original_url: Option<String>,
    pub status: Option<i32>,
    #[serde(default = "default_sort_by")]
    pub sort_by: Option<String>,
    #[serde(default = "default_order")]
    pub order: Option<String>,
}

fn default_page() -> u64 {
    1
}

fn default_per_page() -> u64 {
    10
}

fn default_sort_by() -> Option<String> {
    Some("created_at".to_string())
}

fn default_order() -> Option<String> {
    Some("desc".to_string())
}

impl Default for ListParams {
    fn default() -> Self {
        Self {
            page: 1,
            page_size: 10,
            short_code: None,
            original_url: None,
            status: None,
            sort_by: Some("created_at".to_string()),
            order: Some("desc".to_string()),
        }
    }
}

/// URL Repository trait
#[async_trait]
pub trait UrlRepository: Send + Sync {
    /// Create a new URL
    async fn create(&self, url: CreateUrlDto) -> Result<Model, DbErr>;

    /// Find URL by code
    async fn find_by_code(&self, code: &str) -> Result<Option<Model>, DbErr>;

    /// Find URL by ID
    async fn find_by_id(&self, id: i64) -> Result<Option<Model>, DbErr>;

    /// List URLs with pagination
    async fn list(&self, params: ListParams) -> Result<(Vec<Model>, u64), DbErr>;

    /// Update URL by code
    async fn update(&self, code: &str, data: UpdateUrlDto) -> Result<Model, DbErr>;

    /// Delete URL by code
    async fn delete(&self, code: &str) -> Result<(), DbErr>;

    /// Delete multiple URLs by IDs
    async fn delete_batch(&self, ids: Vec<i64>) -> Result<u64, DbErr>;
}

/// URL Repository implementation
pub struct UrlRepositoryImpl {
    db: DatabaseConnection,
}

impl UrlRepositoryImpl {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }
}

#[async_trait]
impl UrlRepository for UrlRepositoryImpl {
    async fn create(&self, url: CreateUrlDto) -> Result<Model, DbErr> {
        let now = chrono::Utc::now();

        let active_model = ActiveModel {
            short_code: Set(url.short_code),
            original_url: Set(url.original_url),
            description: Set(url.description),
            status: Set(url.status),
            created_at: Set(now),
            updated_at: Set(now),
            ..Default::default()
        };

        active_model.insert(&self.db).await
    }

    async fn find_by_code(&self, code: &str) -> Result<Option<Model>, DbErr> {
        Entity::find()
            .filter(Column::ShortCode.eq(code))
            .one(&self.db)
            .await
    }

    async fn find_by_id(&self, id: i64) -> Result<Option<Model>, DbErr> {
        Entity::find_by_id(id).one(&self.db).await
    }

    async fn list(&self, params: ListParams) -> Result<(Vec<Model>, u64), DbErr> {
        let mut query = Entity::find();

        // Apply code filter if provided
        if let Some(code) = params.short_code {
            query = query.filter(Column::ShortCode.eq(code));
        }

        // Apply original_url filter if provided (fuzzy search using LIKE)
        if let Some(original_url) = params.original_url {
            query = query.filter(Column::OriginalUrl.contains(&original_url));
        }

        // Apply status filter if provided
        if let Some(status) = params.status {
            query = query.filter(Column::Status.eq(status));
        }

        // Apply sorting
        let sort_column = params.sort_by.as_deref().unwrap_or("created_at");
        let order = params.order.as_deref().unwrap_or("desc");

        query = match (sort_column, order) {
            ("id", "asc") => query.order_by_asc(Column::Id),
            ("id", "desc") => query.order_by_desc(Column::Id),
            ("short_code", "asc") => query.order_by_asc(Column::ShortCode),
            ("short_code", "desc") => query.order_by_desc(Column::ShortCode),
            ("created_at", "asc") => query.order_by_asc(Column::CreatedAt),
            ("created_at", "desc") => query.order_by_desc(Column::CreatedAt),
            ("updated_at", "asc") => query.order_by_asc(Column::UpdatedAt),
            ("updated_at", "desc") => query.order_by_desc(Column::UpdatedAt),
            _ => query.order_by_desc(Column::CreatedAt), // Default
        };

        // Get total count
        let total = query.clone().count(&self.db).await?;

        // Apply pagination
        let paginator = query.paginate(&self.db, params.page_size);
        let items = paginator.fetch_page(params.page - 1).await?;

        Ok((items, total))
    }

    async fn update(&self, code: &str, data: UpdateUrlDto) -> Result<Model, DbErr> {
        // First find the URL
        let url = self
            .find_by_code(code)
            .await?
            .ok_or_else(|| DbErr::RecordNotFound(format!("URL with code '{}' not found", code)))?;

        // Create active model for update
        let mut active_model: ActiveModel = url.into();

        // Update fields if provided
        if let Some(original_url) = data.original_url {
            active_model.original_url = Set(original_url);
        }
        if let Some(description) = data.description {
            active_model.description = Set(Some(description));
        }
        if let Some(status) = data.status {
            active_model.status = Set(status);
        }

        // Always update the updated_at timestamp
        active_model.updated_at = Set(chrono::Utc::now());

        active_model.update(&self.db).await
    }

    async fn delete(&self, code: &str) -> Result<(), DbErr> {
        let url = self
            .find_by_code(code)
            .await?
            .ok_or_else(|| DbErr::RecordNotFound(format!("URL with code '{}' not found", code)))?;

        let active_model: ActiveModel = url.into();
        active_model.delete(&self.db).await?;

        Ok(())
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

    #[tokio::test]
    async fn test_create_url() {
        let db = setup_test_db().await;
        let repo = UrlRepositoryImpl::new(db);

        let create_dto = CreateUrlDto {
            short_code: "test123".to_string(),
            original_url: "https://example.com".to_string(),
            description: Some("Test URL".to_string()),
            status: UrlStatus::Enabled as i32,
        };

        let result = repo.create(create_dto).await;
        assert!(result.is_ok());

        let url = result.unwrap();
        assert_eq!(url.short_code, "test123");
        assert_eq!(url.original_url, "https://example.com");
        assert_eq!(url.description, Some("Test URL".to_string()));
        assert_eq!(url.status, UrlStatus::Enabled as i32);
    }

    #[tokio::test]
    async fn test_find_by_code() {
        let db = setup_test_db().await;
        let repo = UrlRepositoryImpl::new(db);

        // Create a URL first
        let create_dto = CreateUrlDto {
            short_code: "find123".to_string(),
            original_url: "https://example.com".to_string(),
            description: None,
            status: UrlStatus::Enabled as i32,
        };
        repo.create(create_dto).await.unwrap();

        // Find by code
        let result = repo.find_by_code("find123").await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());

        // Find non-existent code
        let result = repo.find_by_code("nonexistent").await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_find_by_id() {
        let db = setup_test_db().await;
        let repo = UrlRepositoryImpl::new(db);

        // Create a URL first
        let create_dto = CreateUrlDto {
            short_code: "id123".to_string(),
            original_url: "https://example.com".to_string(),
            description: None,
            status: UrlStatus::Enabled as i32,
        };
        let created = repo.create(create_dto).await.unwrap();

        // Find by ID
        let result = repo.find_by_id(created.id).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());

        // Find non-existent ID
        let result = repo.find_by_id(99999).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_list_urls() {
        let db = setup_test_db().await;
        let repo = UrlRepositoryImpl::new(db);

        // Create multiple URLs
        for i in 1..=5 {
            let create_dto = CreateUrlDto {
                short_code: format!("list{}", i),
                original_url: format!("https://example{}.com", i),
                description: Some(format!("URL {}", i)),
                status: if i % 2 == 0 {
                    UrlStatus::Disabled as i32
                } else {
                    UrlStatus::Enabled as i32
                },
            };
            repo.create(create_dto).await.unwrap();
        }

        // List all URLs
        let params = ListParams {
            page: 1,
            page_size: 10,
            ..Default::default()
        };
        let (urls, total) = repo.list(params).await.unwrap();
        assert_eq!(urls.len(), 5);
        assert_eq!(total, 5);

        // List with status filter
        let params = ListParams {
            page: 1,
            page_size: 10,
            status: Some(UrlStatus::Enabled as i32),
            ..Default::default()
        };
        let (urls, total) = repo.list(params).await.unwrap();
        assert_eq!(urls.len(), 3);
        assert_eq!(total, 3);

        // List with pagination
        let params = ListParams {
            page: 1,
            page_size: 2,
            ..Default::default()
        };
        let (urls, total) = repo.list(params).await.unwrap();
        assert_eq!(urls.len(), 2);
        assert_eq!(total, 5);
    }

    #[tokio::test]
    async fn test_update_url() {
        let db = setup_test_db().await;
        let repo = UrlRepositoryImpl::new(db);

        // Create a URL first
        let create_dto = CreateUrlDto {
            short_code: "update123".to_string(),
            original_url: "https://example.com".to_string(),
            description: Some("Original".to_string()),
            status: UrlStatus::Enabled as i32,
        };
        repo.create(create_dto).await.unwrap();

        // Update the URL
        let update_dto = UpdateUrlDto {
            original_url: Some("https://updated.com".to_string()),
            description: Some("Updated".to_string()),
            status: Some(UrlStatus::Disabled as i32),
        };
        let result = repo.update("update123", update_dto).await;
        assert!(result.is_ok());

        let updated = result.unwrap();
        assert_eq!(updated.original_url, "https://updated.com");
        assert_eq!(updated.description, Some("Updated".to_string()));
        assert_eq!(updated.status, UrlStatus::Disabled as i32);

        // Try to update non-existent URL
        let update_dto = UpdateUrlDto {
            original_url: Some("https://test.com".to_string()),
            description: None,
            status: None,
        };
        let result = repo.update("nonexistent", update_dto).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_url() {
        let db = setup_test_db().await;
        let repo = UrlRepositoryImpl::new(db);

        // Create a URL first
        let create_dto = CreateUrlDto {
            short_code: "delete123".to_string(),
            original_url: "https://example.com".to_string(),
            description: None,
            status: UrlStatus::Enabled as i32,
        };
        repo.create(create_dto).await.unwrap();

        // Delete the URL
        let result = repo.delete("delete123").await;
        assert!(result.is_ok());

        // Verify it's deleted
        let found = repo.find_by_code("delete123").await.unwrap();
        assert!(found.is_none());

        // Try to delete non-existent URL
        let result = repo.delete("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_batch() {
        let db = setup_test_db().await;
        let repo = UrlRepositoryImpl::new(db);

        // Create multiple URLs
        let mut ids = Vec::new();
        for i in 1..=5 {
            let create_dto = CreateUrlDto {
                short_code: format!("batch{}", i),
                original_url: format!("https://example{}.com", i),
                description: None,
                status: UrlStatus::Enabled as i32,
            };
            let created = repo.create(create_dto).await.unwrap();
            ids.push(created.id);
        }

        // Delete first 3 URLs
        let delete_ids = ids[0..3].to_vec();
        let result = repo.delete_batch(delete_ids).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);

        // Verify remaining URLs
        let params = ListParams::default();
        let (urls, total) = repo.list(params).await.unwrap();
        assert_eq!(urls.len(), 2);
        assert_eq!(total, 2);
    }

    #[tokio::test]
    async fn test_list_urls_with_code_filter() {
        let db = setup_test_db().await;
        let repo = UrlRepositoryImpl::new(db);

        // Create multiple URLs with different codes
        for i in 1..=3 {
            let create_dto = CreateUrlDto {
                short_code: format!("test{}", i),
                original_url: format!("https://example{}.com", i),
                description: Some(format!("Test URL {}", i)),
                status: UrlStatus::Enabled as i32,
            };
            repo.create(create_dto).await.unwrap();
        }

        // List with code filter
        let params = ListParams {
            page: 1,
            page_size: 10,
            short_code: Some("test2".to_string()),
            ..Default::default()
        };
        let (urls, total) = repo.list(params).await.unwrap();
        assert_eq!(urls.len(), 1);
        assert_eq!(total, 1);
        assert_eq!(urls[0].short_code, "test2");

        // List with non-existent code
        let params = ListParams {
            page: 1,
            page_size: 10,
            short_code: Some("nonexistent".to_string()),
            ..Default::default()
        };
        let (urls, total) = repo.list(params).await.unwrap();
        assert_eq!(urls.len(), 0);
        assert_eq!(total, 0);
    }

    #[tokio::test]
    async fn test_list_urls_with_combined_filters() {
        let db = setup_test_db().await;
        let repo = UrlRepositoryImpl::new(db);

        // Create URLs with different statuses
        let create_dto1 = CreateUrlDto {
            short_code: "enabled1".to_string(),
            original_url: "https://example1.com".to_string(),
            description: None,
            status: UrlStatus::Enabled as i32,
        };
        repo.create(create_dto1).await.unwrap();

        let create_dto2 = CreateUrlDto {
            short_code: "disabled1".to_string(),
            original_url: "https://example2.com".to_string(),
            description: None,
            status: UrlStatus::Disabled as i32,
        };
        repo.create(create_dto2).await.unwrap();

        // List with status filter
        let params = ListParams {
            page: 1,
            page_size: 10,
            status: Some(UrlStatus::Enabled as i32),
            ..Default::default()
        };
        let (urls, total) = repo.list(params).await.unwrap();
        assert_eq!(urls.len(), 1);
        assert_eq!(total, 1);
        assert_eq!(urls[0].short_code, "enabled1");

        // List with both code and status filters
        let params = ListParams {
            page: 1,
            page_size: 10,
            short_code: Some("disabled1".to_string()),
            status: Some(UrlStatus::Disabled as i32),
            ..Default::default()
        };
        let (urls, total) = repo.list(params).await.unwrap();
        assert_eq!(urls.len(), 1);
        assert_eq!(total, 1);
        assert_eq!(urls[0].short_code, "disabled1");
    }

    #[tokio::test]
    async fn test_list_urls_with_original_url_filter() {
        let db = setup_test_db().await;
        let repo = UrlRepositoryImpl::new(db);

        // Create URLs with different original URLs
        let create_dto1 = CreateUrlDto {
            short_code: "github1".to_string(),
            original_url: "https://github.com/user/repo1".to_string(),
            description: None,
            status: UrlStatus::Enabled as i32,
        };
        repo.create(create_dto1).await.unwrap();

        let create_dto2 = CreateUrlDto {
            short_code: "gitlab1".to_string(),
            original_url: "https://gitlab.com/user/repo2".to_string(),
            description: None,
            status: UrlStatus::Enabled as i32,
        };
        repo.create(create_dto2).await.unwrap();

        let create_dto3 = CreateUrlDto {
            short_code: "github2".to_string(),
            original_url: "https://github.com/another/project".to_string(),
            description: None,
            status: UrlStatus::Enabled as i32,
        };
        repo.create(create_dto3).await.unwrap();

        // Test fuzzy search for "github"
        let params = ListParams {
            page: 1,
            page_size: 10,
            short_code: None,
            original_url: Some("github".to_string()),
            status: None,
            sort_by: None,
            order: None,
        };
        let (urls, total) = repo.list(params).await.unwrap();
        assert_eq!(urls.len(), 2);
        assert_eq!(total, 2);
        // Both github URLs should be returned
        let codes: Vec<&String> = urls.iter().map(|u| &u.short_code).collect();
        assert!(codes.contains(&&"github1".to_string()));
        assert!(codes.contains(&&"github2".to_string()));

        // Test fuzzy search for "gitlab"
        let params = ListParams {
            page: 1,
            page_size: 10,
            short_code: None,
            original_url: Some("gitlab".to_string()),
            status: None,
            sort_by: None,
            order: None,
        };
        let (urls, total) = repo.list(params).await.unwrap();
        assert_eq!(urls.len(), 1);
        assert_eq!(total, 1);
        assert_eq!(urls[0].short_code, "gitlab1");

        // Test fuzzy search for "user"
        let params = ListParams {
            page: 1,
            page_size: 10,
            short_code: None,
            original_url: Some("user".to_string()),
            status: None,
            sort_by: None,
            order: None,
        };
        let (urls, total) = repo.list(params).await.unwrap();
        assert_eq!(urls.len(), 2);
        assert_eq!(total, 2);

        // Test search with no matches
        let params = ListParams {
            page: 1,
            page_size: 10,
            short_code: None,
            original_url: Some("nonexistent".to_string()),
            status: None,
            sort_by: None,
            order: None,
        };
        let (urls, total) = repo.list(params).await.unwrap();
        assert_eq!(urls.len(), 0);
        assert_eq!(total, 0);
    }

    #[tokio::test]
    async fn test_list_urls_with_all_filters() {
        let db = setup_test_db().await;
        let repo = UrlRepositoryImpl::new(db);

        // Create URLs with different properties
        let create_dto1 = CreateUrlDto {
            short_code: "test1".to_string(),
            original_url: "https://github.com/test/repo".to_string(),
            description: None,
            status: UrlStatus::Enabled as i32,
        };
        repo.create(create_dto1).await.unwrap();

        let create_dto2 = CreateUrlDto {
            short_code: "test2".to_string(),
            original_url: "https://github.com/another/project".to_string(),
            description: None,
            status: UrlStatus::Disabled as i32,
        };
        repo.create(create_dto2).await.unwrap();

        // Test combining original_url and status filters
        let params = ListParams {
            page: 1,
            page_size: 10,
            short_code: None,
            original_url: Some("github".to_string()),
            status: Some(UrlStatus::Enabled as i32),
            sort_by: None,
            order: None,
        };
        let (urls, total) = repo.list(params).await.unwrap();
        assert_eq!(urls.len(), 1);
        assert_eq!(total, 1);
        assert_eq!(urls[0].short_code, "test1");

        // Test combining all filters
        let params = ListParams {
            page: 1,
            page_size: 10,
            short_code: Some("test2".to_string()),
            original_url: Some("github".to_string()),
            status: Some(UrlStatus::Disabled as i32),
            sort_by: None,
            order: None,
        };
        let (urls, total) = repo.list(params).await.unwrap();
        assert_eq!(urls.len(), 1);
        assert_eq!(total, 1);
        assert_eq!(urls[0].short_code, "test2");
    }
}
