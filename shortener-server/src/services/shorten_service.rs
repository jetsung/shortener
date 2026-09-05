use crate::cache::Cache;
use crate::config::SlugConfig;
use crate::errors::ServiceError;
use crate::models::url::{Model as UrlModel, UrlStatus};
use crate::repositories::url_repository::{CreateUrlDto, ListParams, UpdateUrlDto, UrlRepository};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Request DTO for creating a short URL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateShortenRequest {
    pub original_url: String,
    pub short_code: Option<String>,
    pub description: Option<String>,
}

/// Request DTO for updating a short URL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateShortenRequest {
    pub original_url: Option<String>,
    pub description: Option<String>,
    pub status: Option<i32>,
}

/// Response DTO for short URL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortenResponse {
    pub id: i64,
    pub short_code: String,
    pub short_url: String,
    pub original_url: String,
    pub description: Option<String>,
    pub status: i32,
    pub created_at: String,
    pub updated_at: String,
}

impl ShortenResponse {
    /// Convert URL model to response DTO
    pub fn from_model(model: UrlModel, short_url: &str) -> Self {
        Self {
            id: model.id,
            short_code: model.short_code.clone(),
            short_url: format!("{}/{}", short_url.trim_end_matches('/'), model.short_code),
            original_url: model.original_url,
            description: model.description,
            status: model.status,
            created_at: model.created_at.to_rfc3339(),
            updated_at: model.updated_at.to_rfc3339(),
        }
    }
}

/// Pagination metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageMeta {
    pub page: u64,
    pub per_page: u64,
    pub count: u64,
    pub total: u64,
    pub total_pages: u64,
}

/// Paginated response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PagedResponse<T> {
    pub data: Vec<T>,
    pub meta: PageMeta,
}

/// Shorten Service - handles business logic for short URL management
pub struct ShortenService {
    url_repo: Arc<dyn UrlRepository>,
    cache: Arc<dyn Cache>,
    config: SlugConfig,
    short_url: String,
}

impl ShortenService {
    /// Create a new ShortenService instance
    pub fn new(
        url_repo: Arc<dyn UrlRepository>,
        cache: Arc<dyn Cache>,
        config: SlugConfig,
        short_url: String,
    ) -> Self {
        Self {
            url_repo,
            cache,
            config,
            short_url,
        }
    }

    /// Create a new short URL
    ///
    /// # Arguments
    ///
    /// * `req` - Create request containing original URL and optional code
    ///
    /// # Returns
    ///
    /// * `Ok(ShortenResponse)` - Successfully created short URL
    /// * `Err(ServiceError)` - Creation failed
    pub async fn create_shorten(
        &self,
        req: CreateShortenRequest,
    ) -> Result<ShortenResponse, ServiceError> {
        // Validate URL
        if req.original_url.is_empty() {
            return Err(ServiceError::InvalidInput(
                "Original URL cannot be empty".to_string(),
            ));
        }

        // Validate URL format
        if !self.is_valid_url(&req.original_url) {
            return Err(ServiceError::InvalidInput(format!(
                "Invalid URL format: {}",
                req.original_url
            )));
        }

        // Generate or use provided code
        let code = if let Some(provided_code) = req.short_code {
            // Validate provided code
            if !self.is_valid_code(&provided_code) {
                return Err(ServiceError::InvalidInput(format!(
                    "Invalid code format: {}",
                    provided_code
                )));
            }

            // Check if code already exists
            if let Ok(Some(_)) = self.url_repo.find_by_code(&provided_code).await {
                return Err(ServiceError::AlreadyExists(format!(
                    "Code '{}' already exists",
                    provided_code
                )));
            }

            provided_code
        } else {
            // Generate unique code
            self.generate_unique_code().await?
        };

        // Create URL in database
        let create_dto = CreateUrlDto {
            short_code: code.clone(),
            original_url: req.original_url.clone(),
            description: req.description,
            status: UrlStatus::Enabled as i32,
        };

        let url_model = self.url_repo.create(create_dto).await?;

        info!("Created short URL: {} -> {}", code, req.original_url);

        // Cache the URL
        if let Err(e) = self.cache_url(&url_model).await {
            warn!("Failed to cache URL {}: {}", code, e);
            // Don't fail the request if caching fails
        }

        Ok(ShortenResponse::from_model(url_model, &self.short_url))
    }

    /// Get a short URL by code
    ///
    /// # Arguments
    ///
    /// * `code` - Short code to lookup
    ///
    /// # Returns
    ///
    /// * `Ok(ShortenResponse)` - URL found
    /// * `Err(ServiceError)` - URL not found or error occurred
    pub async fn get_shorten(&self, code: &str) -> Result<ShortenResponse, ServiceError> {
        // Try to get from cache first
        if let Ok(Some(cached_url)) = self.get_cached_url(code).await {
            debug!("Cache hit for code: {}", code);
            return Ok(ShortenResponse::from_model(cached_url, &self.short_url));
        }

        debug!("Cache miss for code: {}", code);

        // Get from database
        let url_model =
            self.url_repo.find_by_code(code).await?.ok_or_else(|| {
                ServiceError::NotFound(format!("URL with code '{}' not found", code))
            })?;

        // Update cache
        if let Err(e) = self.cache_url(&url_model).await {
            warn!("Failed to cache URL {}: {}", code, e);
        }

        Ok(ShortenResponse::from_model(url_model, &self.short_url))
    }

    /// List short URLs with pagination
    ///
    /// # Arguments
    ///
    /// * `params` - List parameters including pagination and filters
    ///
    /// # Returns
    ///
    /// * `Ok(PagedResponse<ShortenResponse>)` - Paginated list of URLs
    /// * `Err(ServiceError)` - Query failed
    pub async fn list_shortens(
        &self,
        params: ListParams,
    ) -> Result<PagedResponse<ShortenResponse>, ServiceError> {
        let (urls, total) = self.url_repo.list(params.clone()).await?;

        let data: Vec<ShortenResponse> = urls
            .into_iter()
            .map(|url| ShortenResponse::from_model(url, &self.short_url))
            .collect();

        let total_pages = (total as f64 / params.page_size as f64).ceil() as u64;

        let meta = PageMeta {
            page: params.page,
            per_page: params.page_size,
            count: data.len() as u64,
            total,
            total_pages,
        };

        Ok(PagedResponse { data, meta })
    }

    /// Update a short URL
    ///
    /// # Arguments
    ///
    /// * `code` - Short code to update
    /// * `req` - Update request with new values
    ///
    /// # Returns
    ///
    /// * `Ok(ShortenResponse)` - Successfully updated URL
    /// * `Err(ServiceError)` - Update failed
    pub async fn update_shorten(
        &self,
        code: &str,
        req: UpdateShortenRequest,
    ) -> Result<ShortenResponse, ServiceError> {
        // Validate URL if provided
        if let Some(ref url) = req.original_url
            && !self.is_valid_url(url)
        {
            return Err(ServiceError::InvalidInput(format!(
                "Invalid URL format: {}",
                url
            )));
        }

        // Update in database
        let update_dto = UpdateUrlDto {
            original_url: req.original_url,
            description: req.description,
            status: req.status,
        };

        let url_model = self.url_repo.update(code, update_dto).await?;

        info!("Updated short URL: {}", code);

        // Update cache
        if let Err(e) = self.cache_url(&url_model).await {
            warn!("Failed to update cache for URL {}: {}", code, e);
        }

        Ok(ShortenResponse::from_model(url_model, &self.short_url))
    }

    /// Delete a short URL
    ///
    /// # Arguments
    ///
    /// * `code` - Short code to delete
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Successfully deleted
    /// * `Err(ServiceError)` - Deletion failed
    pub async fn delete_shorten(&self, code: &str) -> Result<(), ServiceError> {
        // Delete from database
        self.url_repo.delete(code).await?;

        info!("Deleted short URL: {}", code);

        // Delete from cache
        if let Err(e) = self.delete_cached_url(code).await {
            warn!("Failed to delete cache for URL {}: {}", code, e);
        }

        Ok(())
    }

    /// Delete multiple short URLs by IDs
    ///
    /// # Arguments
    ///
    /// * `ids` - List of URL IDs to delete
    ///
    /// # Returns
    ///
    /// * `Ok(u64)` - Number of URLs deleted
    /// * `Err(ServiceError)` - Deletion failed
    pub async fn delete_batch(&self, ids: Vec<i64>) -> Result<u64, ServiceError> {
        // Get codes before deletion for cache cleanup
        let mut codes = Vec::new();
        for id in &ids {
            if let Ok(Some(url)) = self.url_repo.find_by_id(*id).await {
                codes.push(url.short_code);
            }
        }

        // Delete from database
        let deleted_count = self.url_repo.delete_batch(ids.clone()).await?;

        info!("Batch deleted {} short URLs", deleted_count);

        // Delete from cache
        for code in &codes {
            if let Err(e) = self.delete_cached_url(code).await {
                warn!("Failed to delete cache for URL {}: {}", code, e);
            }
        }

        Ok(deleted_count)
    }

    /// Warm up the cache by loading all short URLs from the database
    ///
    /// Called at startup after the cache has been cleared, so the cache
    /// reflects exactly the current database contents. Individual failures
    /// are logged and skipped; they will be re-cached on the next access.
    ///
    /// # Returns
    ///
    /// * `Ok(usize)` - Number of URLs successfully written to the cache
    /// * `Err(ServiceError)` - Failed to list URLs from the database
    pub async fn warm_up_cache(&self) -> Result<usize, ServiceError> {
        let urls = self.url_repo.list_all().await?;
        info!("Warming up cache with {} short URLs from database", urls.len());

        let mut cached = 0;
        for url in &urls {
            match self.cache_url(url).await {
                Ok(()) => cached += 1,
                Err(e) => warn!("Failed to cache URL {} during warm-up: {}", url.short_code, e),
            }
        }

        info!("Cache warm-up finished: {}/{} URLs cached", cached, urls.len());
        Ok(cached)
    }

    /// Refresh the cache: clear all keys under the given prefix, then reload
    /// all short URLs from the database
    ///
    /// # Arguments
    ///
    /// * `prefix` - Cache key prefix to clear (e.g. "shorten:")
    ///
    /// # Returns
    ///
    /// * `Ok((u64, usize))` - (keys cleared, URLs re-cached)
    /// * `Err(ServiceError)` - Cache not connected or operation failed
    pub async fn refresh_cache(&self, prefix: &str) -> Result<(u64, usize), ServiceError> {
        if self.cache.is_null() {
            return Err(ServiceError::Cache(
                "Cache is not enabled or not connected".to_string(),
            ));
        }

        let cleared = self
            .cache
            .clear_prefix(prefix)
            .await
            .map_err(|e| ServiceError::Cache(e.to_string()))?;

        info!(
            "Cache refresh: cleared {} keys with prefix '{}'",
            cleared, prefix
        );

        let warmed = self.warm_up_cache().await?;
        Ok((cleared, warmed))
    }

    /// Generate a random short code
    ///
    /// # Returns
    ///
    /// * `String` - Generated short code
    fn generate_code(&self) -> String {
        let mut rng = rand::rng();
        let chars: Vec<char> = self.config.alphabet.chars().collect();
        let charset_len = chars.len();

        (0..self.config.length)
            .map(|_| chars[rng.random_range(0..charset_len)])
            .collect()
    }

    /// Generate a unique short code (not already in database)
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - Generated unique code
    /// * `Err(ServiceError)` - Failed to generate unique code after max attempts
    async fn generate_unique_code(&self) -> Result<String, ServiceError> {
        const MAX_ATTEMPTS: u32 = 10;

        for attempt in 1..=MAX_ATTEMPTS {
            let code = self.generate_code();

            // Check if code exists
            match self.url_repo.find_by_code(&code).await {
                Ok(None) => return Ok(code),
                Ok(Some(_)) => {
                    debug!("Code collision on attempt {}: {}", attempt, code);
                    continue;
                }
                Err(e) => {
                    return Err(ServiceError::Repository(format!(
                        "Failed to check code existence: {}",
                        e
                    )));
                }
            }
        }

        Err(ServiceError::Internal(format!(
            "Failed to generate unique code after {} attempts",
            MAX_ATTEMPTS
        )))
    }

    /// Validate URL format
    fn is_valid_url(&self, url: &str) -> bool {
        url.starts_with("http://") || url.starts_with("https://")
    }

    /// Validate code format
    fn is_valid_code(&self, code: &str) -> bool {
        if code.is_empty() || code.len() > 16 {
            return false;
        }

        // Check if all characters are in the charset
        let charset: std::collections::HashSet<char> = self.config.alphabet.chars().collect();
        code.chars().all(|c| charset.contains(&c))
    }

    /// Cache a URL model
    async fn cache_url(&self, url: &UrlModel) -> Result<(), ServiceError> {
        let cache_key = format!("url:{}", url.short_code);
        let cache_value = serde_json::to_string(url)
            .map_err(|e| ServiceError::Cache(format!("Failed to serialize URL: {}", e)))?;

        self.cache
            .set(
                &cache_key,
                &cache_value,
                self.config.length as u64 * 3600,
            )
            .await
            .map_err(|e| ServiceError::Cache(e.to_string()))?;

        Ok(())
    }

    /// Get a URL from cache
    async fn get_cached_url(&self, code: &str) -> Result<Option<UrlModel>, ServiceError> {
        let cache_key = format!("url:{}", code);

        match self.cache.get(&cache_key).await {
            Ok(Some(cached_value)) => {
                let url: UrlModel = serde_json::from_str(&cached_value).map_err(|e| {
                    ServiceError::Cache(format!("Failed to deserialize URL: {}", e))
                })?;
                Ok(Some(url))
            }
            Ok(None) => Ok(None),
            Err(e) => {
                warn!("Cache get error: {}", e);
                Ok(None) // Don't fail on cache errors
            }
        }
    }

    /// Delete a URL from cache
    async fn delete_cached_url(&self, code: &str) -> Result<(), ServiceError> {
        let cache_key = format!("url:{}", code);

        self.cache
            .delete(&cache_key)
            .await
            .map_err(|e| ServiceError::Cache(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::NullCache;
    use crate::config::{Config, DatabaseConfig};
    use crate::db::DbFactory;
    use crate::repositories::url_repository::UrlRepositoryImpl;

    async fn setup_test_service() -> ShortenService {
        let config = Config {
            server: crate::config::ServerConfig {
                address: ":8080".to_string(),
                trusted_platform: None,
                short_url: "http://localhost:8080".to_string(),
                api_key: "test-key".to_string(),
            },
            slug: crate::config::SlugConfig {
                length: 6,
                alphabet: "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"
                    .to_string(),
            },
            admin: crate::config::AdminConfig {
                username: "admin".to_string(),
                password_hash: "".to_string(),
            },
            oidc: crate::config::OidcConfig::default(),
            database: DatabaseConfig {
                url: Some("sqlite::memory:".to_string()),
                log_level: 0,
            },
            cache: crate::config::CacheConfig {
                enabled: false,
                expire: 3600,
                prefix: "shorten:".to_string(),
                url: None,
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

        let url_repo = Arc::new(UrlRepositoryImpl::new(db));
        let cache = Arc::new(NullCache::new());

        ShortenService::new(
            url_repo,
            cache,
            config.slug.clone(),
            config.server.short_url.clone(),
        )
    }

    #[tokio::test]
    async fn test_create_shorten_with_auto_code() {
        let service = setup_test_service().await;

        let req = CreateShortenRequest {
            original_url: "https://example.com".to_string(),
            short_code: None,
            description: Some("Test URL".to_string()),
        };

        let result = service.create_shorten(req).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.original_url, "https://example.com");
        assert_eq!(response.description, Some("Test URL".to_string()));
        assert_eq!(response.status, UrlStatus::Enabled as i32);
        assert_eq!(response.short_code.len(), 6);
    }

    #[tokio::test]
    async fn test_create_shorten_with_custom_code() {
        let service = setup_test_service().await;

        let req = CreateShortenRequest {
            original_url: "https://example.com".to_string(),
            short_code: Some("custom".to_string()),
            description: None,
        };

        let result = service.create_shorten(req).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.short_code, "custom");
        assert_eq!(response.original_url, "https://example.com");
    }

    #[tokio::test]
    async fn test_create_shorten_duplicate_code() {
        let service = setup_test_service().await;

        // Create first URL
        let req1 = CreateShortenRequest {
            original_url: "https://example.com".to_string(),
            short_code: Some("duplicate".to_string()),
            description: None,
        };
        service.create_shorten(req1).await.unwrap();

        // Try to create with same code
        let req2 = CreateShortenRequest {
            original_url: "https://another.com".to_string(),
            short_code: Some("duplicate".to_string()),
            description: None,
        };

        let result = service.create_shorten(req2).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ServiceError::AlreadyExists(_)
        ));
    }

    #[tokio::test]
    async fn test_create_shorten_invalid_url() {
        let service = setup_test_service().await;

        let req = CreateShortenRequest {
            original_url: "not-a-url".to_string(),
            short_code: None,
            description: None,
        };

        let result = service.create_shorten(req).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ServiceError::InvalidInput(_)));
    }

    #[tokio::test]
    async fn test_create_shorten_empty_url() {
        let service = setup_test_service().await;

        let req = CreateShortenRequest {
            original_url: "".to_string(),
            short_code: None,
            description: None,
        };

        let result = service.create_shorten(req).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ServiceError::InvalidInput(_)));
    }

    #[tokio::test]
    async fn test_get_shorten() {
        let service = setup_test_service().await;

        // Create a URL first
        let req = CreateShortenRequest {
            original_url: "https://example.com".to_string(),
            short_code: Some("gettest".to_string()),
            description: Some("Get test".to_string()),
        };
        service.create_shorten(req).await.unwrap();

        // Get the URL
        let result = service.get_shorten("gettest").await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.short_code, "gettest");
        assert_eq!(response.original_url, "https://example.com");
        assert_eq!(response.description, Some("Get test".to_string()));
    }

    #[tokio::test]
    async fn test_get_shorten_not_found() {
        let service = setup_test_service().await;

        let result = service.get_shorten("nonexistent").await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ServiceError::NotFound(_)));
    }

    #[tokio::test]
    async fn test_list_shortens() {
        let service = setup_test_service().await;

        // Create multiple URLs
        for i in 1..=5 {
            let req = CreateShortenRequest {
                original_url: format!("https://example{}.com", i),
                short_code: Some(format!("list{}", i)),
                description: Some(format!("URL {}", i)),
            };
            service.create_shorten(req).await.unwrap();
        }

        // List all URLs
        let params = ListParams {
            page: 1,
            page_size: 10,
            ..Default::default()
        };

        let result = service.list_shortens(params).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.data.len(), 5);
        assert_eq!(response.meta.total, 5);
        assert_eq!(response.meta.total_pages, 1);
    }

    #[tokio::test]
    async fn test_list_shortens_pagination() {
        let service = setup_test_service().await;

        // Create multiple URLs
        for i in 1..=5 {
            let req = CreateShortenRequest {
                original_url: format!("https://example{}.com", i),
                short_code: Some(format!("page{}", i)),
                description: None,
            };
            service.create_shorten(req).await.unwrap();
        }

        // Get first page
        let params = ListParams {
            page: 1,
            page_size: 2,
            ..Default::default()
        };

        let result = service.list_shortens(params).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.data.len(), 2);
        assert_eq!(response.meta.total, 5);
        assert_eq!(response.meta.total_pages, 3);
        assert_eq!(response.meta.page, 1);
    }

    #[tokio::test]
    async fn test_update_shorten() {
        let service = setup_test_service().await;

        // Create a URL first
        let req = CreateShortenRequest {
            original_url: "https://example.com".to_string(),
            short_code: Some("update".to_string()),
            description: Some("Original".to_string()),
        };
        service.create_shorten(req).await.unwrap();

        // Update the URL
        let update_req = UpdateShortenRequest {
            original_url: Some("https://updated.com".to_string()),
            description: Some("Updated".to_string()),
            status: Some(UrlStatus::Disabled as i32),
        };

        let result = service.update_shorten("update", update_req).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.original_url, "https://updated.com");
        assert_eq!(response.description, Some("Updated".to_string()));
        assert_eq!(response.status, UrlStatus::Disabled as i32);
    }

    #[tokio::test]
    async fn test_update_shorten_not_found() {
        let service = setup_test_service().await;

        let update_req = UpdateShortenRequest {
            original_url: Some("https://test.com".to_string()),
            description: None,
            status: None,
        };

        let result = service.update_shorten("nonexistent", update_req).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_shorten() {
        let service = setup_test_service().await;

        // Create a URL first
        let req = CreateShortenRequest {
            original_url: "https://example.com".to_string(),
            short_code: Some("delete".to_string()),
            description: None,
        };
        service.create_shorten(req).await.unwrap();

        // Delete the URL
        let result = service.delete_shorten("delete").await;
        assert!(result.is_ok());

        // Verify it's deleted
        let get_result = service.get_shorten("delete").await;
        assert!(get_result.is_err());
        assert!(matches!(get_result.unwrap_err(), ServiceError::NotFound(_)));
    }

    #[tokio::test]
    async fn test_delete_shorten_not_found() {
        let service = setup_test_service().await;

        let result = service.delete_shorten("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_batch() {
        let service = setup_test_service().await;

        // Create multiple URLs
        let mut ids = Vec::new();
        for i in 1..=5 {
            let req = CreateShortenRequest {
                original_url: format!("https://example{}.com", i),
                short_code: Some(format!("batch{}", i)),
                description: None,
            };
            let response = service.create_shorten(req).await.unwrap();
            ids.push(response.id);
        }

        // Delete first 3 URLs
        let delete_ids = ids[0..3].to_vec();
        let result = service.delete_batch(delete_ids).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);

        // Verify remaining URLs
        let params = ListParams::default();
        let list_result = service.list_shortens(params).await.unwrap();
        assert_eq!(list_result.data.len(), 2);
    }

    #[tokio::test]
    async fn test_generate_code() {
        let service = setup_test_service().await;

        let code = service.generate_code();
        assert_eq!(code.len(), 6);

        // Check all characters are from charset
        let charset: std::collections::HashSet<char> =
            service.config.alphabet.chars().collect();
        assert!(code.chars().all(|c| charset.contains(&c)));
    }

    #[tokio::test]
    async fn test_is_valid_url() {
        let service = setup_test_service().await;

        assert!(service.is_valid_url("http://example.com"));
        assert!(service.is_valid_url("https://example.com"));
        assert!(!service.is_valid_url("ftp://example.com"));
        assert!(!service.is_valid_url("example.com"));
        assert!(!service.is_valid_url(""));
    }

    #[tokio::test]
    async fn test_is_valid_code() {
        let service = setup_test_service().await;

        assert!(service.is_valid_code("abc123"));
        assert!(service.is_valid_code("ABC"));
        assert!(!service.is_valid_code("")); // Empty
        assert!(!service.is_valid_code("12345678901234567")); // Too long (>16)
        assert!(!service.is_valid_code("abc-123")); // Invalid character
        assert!(!service.is_valid_code("abc@123")); // Invalid character
    }
}
