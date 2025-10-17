use crate::errors::ServiceError;
use crate::geoip::{GeoIp, GeoIpInfo};
use crate::models::history::Model as HistoryModel;
use crate::repositories::history_repository::{
    CreateHistoryDto, HistoryListParams, HistoryRepository,
};
use crate::services::shorten_service::{PageMeta, PagedResponse};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info};

/// Response DTO for history record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryResponse {
    pub id: i64,
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
    pub accessed_at: String,
    pub created_at: String,
}

impl HistoryResponse {
    /// Convert history model to response DTO
    pub fn from_model(model: HistoryModel) -> Self {
        Self {
            id: model.id,
            url_id: model.url_id,
            short_code: model.short_code,
            ip_address: model.ip_address,
            user_agent: model.user_agent,
            referer: model.referer,
            country: model.country,
            region: model.region,
            province: model.province,
            city: model.city,
            isp: model.isp,
            device_type: model.device_type,
            os: model.os,
            browser: model.browser,
            accessed_at: model.accessed_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            created_at: model.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        }
    }
}

/// User-Agent parsed information
#[derive(Debug, Clone)]
pub struct UserAgentInfo {
    pub device_type: Option<String>,
    pub os: Option<String>,
    pub browser: Option<String>,
}

/// History Service - handles business logic for access history
pub struct HistoryService {
    history_repo: Arc<dyn HistoryRepository>,
    geoip: Option<Arc<dyn GeoIp>>,
}

impl HistoryService {
    /// Create a new HistoryService instance
    pub fn new(history_repo: Arc<dyn HistoryRepository>, geoip: Option<Arc<dyn GeoIp>>) -> Self {
        Self {
            history_repo,
            geoip,
        }
    }

    /// Record an access to a short URL
    ///
    /// # Arguments
    ///
    /// * `url_id` - ID of the URL being accessed
    /// * `code` - Short code being accessed
    /// * `ip` - IP address of the visitor
    /// * `user_agent` - User-Agent header from the request
    /// * `referer` - Referer header from the request
    ///
    /// # Returns
    ///
    /// * `Ok(())` - Successfully recorded access
    /// * `Err(ServiceError)` - Recording failed
    pub async fn record_access(
        &self,
        url_id: i64,
        code: &str,
        ip: &str,
        user_agent: Option<&str>,
        referer: Option<&str>,
    ) -> Result<(), ServiceError> {
        // Get GeoIP information
        let geoip_info = if let Some(ref geoip) = self.geoip {
            debug!("Looking up GeoIP for IP: {}", ip);
            geoip.lookup_or_empty(ip).await
        } else {
            debug!("GeoIP is disabled, using empty info");
            GeoIpInfo::empty()
        };

        // Parse User-Agent
        let ua_info = user_agent.map(|ua| self.parse_user_agent(ua));

        // Create history record
        let create_dto = CreateHistoryDto {
            url_id: url_id as i32,
            short_code: code.to_string(),
            ip_address: ip.to_string(),
            user_agent: user_agent.map(|s| s.to_string()),
            referer: referer.map(|s| s.to_string()),
            country: if geoip_info.country.is_empty() {
                None
            } else {
                Some(geoip_info.country)
            },
            region: if geoip_info.region.is_empty() {
                None
            } else {
                Some(geoip_info.region)
            },
            province: if geoip_info.province.is_empty() {
                None
            } else {
                Some(geoip_info.province)
            },
            city: if geoip_info.city.is_empty() {
                None
            } else {
                Some(geoip_info.city)
            },
            isp: if geoip_info.isp.is_empty() {
                None
            } else {
                Some(geoip_info.isp)
            },
            device_type: ua_info.as_ref().and_then(|ua| ua.device_type.clone()),
            os: ua_info.as_ref().and_then(|ua| ua.os.clone()),
            browser: ua_info.as_ref().and_then(|ua| ua.browser.clone()),
            accessed_at: chrono::Utc::now().naive_utc(),
        };

        self.history_repo.create(create_dto).await?;

        info!("Recorded access for code: {} from IP: {}", code, ip);

        Ok(())
    }

    /// List access history with pagination
    ///
    /// # Arguments
    ///
    /// * `params` - List parameters including pagination and filters
    ///
    /// # Returns
    ///
    /// * `Ok(PagedResponse<HistoryResponse>)` - Paginated list of history records
    /// * `Err(ServiceError)` - Query failed
    pub async fn list_histories(
        &self,
        params: HistoryListParams,
    ) -> Result<PagedResponse<HistoryResponse>, ServiceError> {
        let (histories, total) = self.history_repo.list(params.clone()).await?;

        let data: Vec<HistoryResponse> = histories
            .into_iter()
            .map(HistoryResponse::from_model)
            .collect();

        let total_pages = (total as f64 / params.page_size as f64).ceil() as u64;

        let meta = PageMeta {
            page: params.page,
            page_size: params.page_size,
            current_count: data.len() as u64,
            total_items: total,
            total_pages,
        };

        Ok(PagedResponse { data, meta })
    }

    /// Delete multiple history records by IDs
    ///
    /// # Arguments
    ///
    /// * `ids` - List of history IDs to delete
    ///
    /// # Returns
    ///
    /// * `Ok(u64)` - Number of records deleted
    /// * `Err(ServiceError)` - Deletion failed
    pub async fn delete_batch(&self, ids: Vec<i64>) -> Result<u64, ServiceError> {
        let deleted_count = self.history_repo.delete_batch(ids.clone()).await?;

        info!("Batch deleted {} history records", deleted_count);

        Ok(deleted_count)
    }

    /// Parse User-Agent string to extract device, OS, and browser information
    ///
    /// This is a simplified parser. In production, consider using a library like
    /// `woothee` or `uaparser` for more accurate parsing.
    ///
    /// # Arguments
    ///
    /// * `user_agent` - User-Agent string
    ///
    /// # Returns
    ///
    /// * `UserAgentInfo` - Parsed information
    fn parse_user_agent(&self, user_agent: &str) -> UserAgentInfo {
        let ua_lower = user_agent.to_lowercase();

        // Detect OS first (more specific checks first)
        let os = if ua_lower.contains("android") {
            Some("Android".to_string())
        } else if ua_lower.contains("iphone")
            || ua_lower.contains("ipad")
            || ua_lower.contains("ipod")
        {
            Some("iOS".to_string())
        } else if ua_lower.contains("windows") {
            Some("Windows".to_string())
        } else if ua_lower.contains("mac os") || ua_lower.contains("macos") {
            Some("macOS".to_string())
        } else if ua_lower.contains("linux") {
            Some("Linux".to_string())
        } else {
            None
        };

        // Detect device type (check for tablet before mobile)
        let device_type = if ua_lower.contains("ipad") || ua_lower.contains("tablet") {
            Some("Tablet".to_string())
        } else if ua_lower.contains("mobile")
            || ua_lower.contains("android") && !ua_lower.contains("tablet")
        {
            Some("Mobile".to_string())
        } else {
            Some("Desktop".to_string())
        };

        // Detect browser
        let browser = if ua_lower.contains("edg/") || ua_lower.contains("edge/") {
            Some("Edge".to_string())
        } else if ua_lower.contains("chrome/") && !ua_lower.contains("edg") {
            Some("Chrome".to_string())
        } else if ua_lower.contains("firefox/") {
            Some("Firefox".to_string())
        } else if ua_lower.contains("safari/") && !ua_lower.contains("chrome") {
            Some("Safari".to_string())
        } else if ua_lower.contains("opera/") || ua_lower.contains("opr/") {
            Some("Opera".to_string())
        } else {
            None
        };

        UserAgentInfo {
            device_type,
            os,
            browser,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, DatabaseConfig, DatabaseType, SqliteConfig};
    use crate::db::DbFactory;
    use crate::geoip::NullGeoIp;
    use crate::models::url::UrlStatus;
    use crate::repositories::history_repository::HistoryRepositoryImpl;
    use crate::repositories::url_repository::{CreateUrlDto, UrlRepository, UrlRepositoryImpl};

    async fn setup_test_service() -> (HistoryService, Arc<dyn UrlRepository>) {
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

        let url_repo = Arc::new(UrlRepositoryImpl::new(db.clone()));
        let history_repo = Arc::new(HistoryRepositoryImpl::new(db));
        let geoip = Some(Arc::new(NullGeoIp::new()) as Arc<dyn GeoIp>);

        let service = HistoryService::new(history_repo, geoip);

        (service, url_repo)
    }

    async fn create_test_url(url_repo: &Arc<dyn UrlRepository>) -> i64 {
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
    async fn test_record_access() {
        let (service, url_repo) = setup_test_service().await;
        let url_id = create_test_url(&url_repo).await;

        let result = service
            .record_access(
                url_id,
                "test123",
                "192.168.1.1",
                Some("Mozilla/5.0 (Windows NT 10.0; Win64; x64) Chrome/91.0"),
                Some("https://google.com"),
            )
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_record_access_without_user_agent() {
        let (service, url_repo) = setup_test_service().await;
        let url_id = create_test_url(&url_repo).await;

        let result = service
            .record_access(url_id, "test123", "192.168.1.1", None, None)
            .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_list_histories() {
        let (service, url_repo) = setup_test_service().await;
        let url_id = create_test_url(&url_repo).await;

        // Record multiple accesses
        for i in 1..=5 {
            service
                .record_access(
                    url_id,
                    "test123",
                    &format!("192.168.1.{}", i),
                    Some("Mozilla/5.0"),
                    None,
                )
                .await
                .unwrap();
        }

        // List all histories
        let params = HistoryListParams {
            page: 1,
            page_size: 10,
            ..Default::default()
        };

        let result = service.list_histories(params).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.data.len(), 5);
        assert_eq!(response.meta.total_items, 5);
        assert_eq!(response.meta.total_pages, 1);
    }

    #[tokio::test]
    async fn test_list_histories_pagination() {
        let (service, url_repo) = setup_test_service().await;
        let url_id = create_test_url(&url_repo).await;

        // Record multiple accesses
        for i in 1..=5 {
            service
                .record_access(url_id, "test123", &format!("192.168.1.{}", i), None, None)
                .await
                .unwrap();
        }

        // Get first page
        let params = HistoryListParams {
            page: 1,
            page_size: 2,
            ..Default::default()
        };

        let result = service.list_histories(params).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.data.len(), 2);
        assert_eq!(response.meta.total_items, 5);
        assert_eq!(response.meta.total_pages, 3);
        assert_eq!(response.meta.page, 1);
    }

    #[tokio::test]
    async fn test_list_histories_with_filter() {
        let (service, url_repo) = setup_test_service().await;
        let url_id = create_test_url(&url_repo).await;

        // Record accesses
        service
            .record_access(url_id, "test123", "192.168.1.1", None, None)
            .await
            .unwrap();

        // List with short_code filter
        let params = HistoryListParams {
            page: 1,
            page_size: 10,
            short_code: Some("test123".to_string()),
            ..Default::default()
        };

        let result = service.list_histories(params).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].short_code, "test123");
    }

    #[tokio::test]
    async fn test_delete_batch() {
        let (service, url_repo) = setup_test_service().await;
        let url_id = create_test_url(&url_repo).await;

        // Record multiple accesses
        let mut ids = Vec::new();
        for i in 1..=5 {
            service
                .record_access(url_id, "test123", &format!("192.168.1.{}", i), None, None)
                .await
                .unwrap();
        }

        // Get IDs
        let params = HistoryListParams::default();
        let list_result = service.list_histories(params).await.unwrap();
        for history in list_result.data {
            ids.push(history.id);
        }

        // Delete first 3 histories
        let delete_ids = ids[0..3].to_vec();
        let result = service.delete_batch(delete_ids).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);

        // Verify remaining histories
        let params = HistoryListParams::default();
        let list_result = service.list_histories(params).await.unwrap();
        assert_eq!(list_result.data.len(), 2);
    }

    #[tokio::test]
    async fn test_parse_user_agent_chrome_windows() {
        let (service, _) = setup_test_service().await;

        let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36";
        let info = service.parse_user_agent(ua);

        assert_eq!(info.device_type, Some("Desktop".to_string()));
        assert_eq!(info.os, Some("Windows".to_string()));
        assert_eq!(info.browser, Some("Chrome".to_string()));
    }

    #[tokio::test]
    async fn test_parse_user_agent_firefox_linux() {
        let (service, _) = setup_test_service().await;

        let ua = "Mozilla/5.0 (X11; Linux x86_64; rv:89.0) Gecko/20100101 Firefox/89.0";
        let info = service.parse_user_agent(ua);

        assert_eq!(info.device_type, Some("Desktop".to_string()));
        assert_eq!(info.os, Some("Linux".to_string()));
        assert_eq!(info.browser, Some("Firefox".to_string()));
    }

    #[tokio::test]
    async fn test_parse_user_agent_safari_macos() {
        let (service, _) = setup_test_service().await;

        let ua = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.1.1 Safari/605.1.15";
        let info = service.parse_user_agent(ua);

        assert_eq!(info.device_type, Some("Desktop".to_string()));
        assert_eq!(info.os, Some("macOS".to_string()));
        assert_eq!(info.browser, Some("Safari".to_string()));
    }

    #[tokio::test]
    async fn test_parse_user_agent_mobile_android() {
        let (service, _) = setup_test_service().await;

        let ua = "Mozilla/5.0 (Linux; Android 11; SM-G991B) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.120 Mobile Safari/537.36";
        let info = service.parse_user_agent(ua);

        assert_eq!(info.device_type, Some("Mobile".to_string()));
        assert_eq!(info.os, Some("Android".to_string()));
        assert_eq!(info.browser, Some("Chrome".to_string()));
    }

    #[tokio::test]
    async fn test_parse_user_agent_iphone() {
        let (service, _) = setup_test_service().await;

        let ua = "Mozilla/5.0 (iPhone; CPU iPhone OS 14_6 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.1.1 Mobile/15E148 Safari/604.1";
        let info = service.parse_user_agent(ua);

        assert_eq!(info.device_type, Some("Mobile".to_string()));
        assert_eq!(info.os, Some("iOS".to_string()));
        assert_eq!(info.browser, Some("Safari".to_string()));
    }

    #[tokio::test]
    async fn test_parse_user_agent_edge() {
        let (service, _) = setup_test_service().await;

        let ua = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36 Edg/91.0.864.59";
        let info = service.parse_user_agent(ua);

        assert_eq!(info.device_type, Some("Desktop".to_string()));
        assert_eq!(info.os, Some("Windows".to_string()));
        assert_eq!(info.browser, Some("Edge".to_string()));
    }

    #[tokio::test]
    async fn test_parse_user_agent_tablet() {
        let (service, _) = setup_test_service().await;

        let ua = "Mozilla/5.0 (iPad; CPU OS 14_6 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.1.1 Mobile/15E148 Safari/604.1";
        let info = service.parse_user_agent(ua);

        assert_eq!(info.device_type, Some("Tablet".to_string()));
        assert_eq!(info.os, Some("iOS".to_string()));
        assert_eq!(info.browser, Some("Safari".to_string()));
    }
}
