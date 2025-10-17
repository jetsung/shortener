use crate::config::Config;
use crate::handlers::{
    create_shorten, current_user, delete_batch, delete_histories, delete_shorten, get_shorten,
    list_histories, list_shortens, login, logout, update_shorten,
};
use crate::middleware::{ApiKeyAuth, error_handler_middleware, logging_middleware};
use crate::services::{HistoryService, ShortenService};
use axum::{
    Router, middleware,
    routing::{delete, get, post, put},
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub shorten_service: Arc<ShortenService>,
    pub history_service: Arc<HistoryService>,
    pub config: Arc<Config>,
}

/// Create the main application router with all routes and middleware
pub fn create_router(state: AppState) -> Router {
    let api_key = Arc::new(state.config.server.api_key.clone());

    // Create protected routes (require API key)
    let protected_routes = Router::new()
        // Short URL management routes
        .route("/api/shortens", post(create_shorten))
        .route("/api/shortens", get(list_shortens))
        .route("/api/shortens", delete(delete_batch))
        .route("/api/shortens/{code}", get(get_shorten))
        .route("/api/shortens/{code}", put(update_shorten))
        .route("/api/shortens/{code}", delete(delete_shorten))
        .with_state(state.shorten_service.clone())
        // History routes
        .route("/api/histories", get(list_histories))
        .route("/api/histories", delete(delete_histories))
        .with_state(state.history_service.clone())
        // Account routes that require authentication
        .route("/api/account/logout", post(logout))
        .route("/api/users/current", get(current_user))
        // Apply API key authentication middleware
        .layer(middleware::from_fn(move |headers, req, next| {
            let api_key = api_key.clone();
            async move { ApiKeyAuth::check_api_key(api_key, headers, req, next).await }
        }));

    // Create public routes (no API key required)
    let public_routes = Router::new()
        .route("/api/account/login", post(login))
        .with_state(Arc::new(state.config.admin.clone()));

    // Combine all routes
    Router::new()
        .merge(protected_routes)
        .merge(public_routes)
        // Add CORS layer
        .layer(CorsLayer::permissive())
        // Add logging middleware
        .layer(middleware::from_fn(logging_middleware))
        // Add error handler middleware
        .layer(middleware::from_fn(error_handler_middleware))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::NullCache;
    use crate::config::{
        AdminConfig, CacheConfig, CacheType, DatabaseConfig, DatabaseType, GeoIpConfig, GeoIpType,
        ServerConfig, ShortenerConfig, SqliteConfig,
    };
    use crate::db::DbFactory;
    use crate::geoip::NullGeoIp;
    use crate::repositories::{HistoryRepositoryImpl, UrlRepositoryImpl};
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    async fn setup_test_state() -> AppState {
        let config = Config {
            server: ServerConfig {
                address: ":8080".to_string(),
                trusted_platform: None,
                site_url: "http://localhost:8080".to_string(),
                api_key: "test-api-key".to_string(),
            },
            shortener: ShortenerConfig {
                code_length: 6,
                code_charset: "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"
                    .to_string(),
            },
            admin: AdminConfig {
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
            cache: CacheConfig {
                enabled: false,
                cache_type: CacheType::Redis,
                expire: 3600,
                prefix: "shorten:".to_string(),
                redis: None,
                valkey: None,
            },
            geoip: GeoIpConfig {
                enabled: false,
                geoip_type: GeoIpType::Ip2region,
                ip2region: None,
            },
            logging: crate::logging::LoggingConfig::default(),
        };

        let db = DbFactory::create_connection(&config).await.unwrap();
        DbFactory::run_migrations(&db).await.unwrap();

        let url_repo = Arc::new(UrlRepositoryImpl::new(db.clone()));
        let history_repo = Arc::new(HistoryRepositoryImpl::new(db));
        let cache = Arc::new(NullCache::new());
        let geoip = Some(Arc::new(NullGeoIp::new()) as Arc<dyn crate::geoip::GeoIp>);

        let shorten_service = Arc::new(ShortenService::new(
            url_repo,
            cache,
            config.shortener.clone(),
            config.server.site_url.clone(),
        ));

        let history_service = Arc::new(HistoryService::new(history_repo, geoip));

        AppState {
            shorten_service,
            history_service,
            config: Arc::new(config),
        }
    }

    #[tokio::test]
    async fn test_router_protected_route_without_api_key() {
        let state = setup_test_state().await;
        let app = create_router(state);

        let request = Request::builder()
            .method("GET")
            .uri("/api/shortens")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_router_protected_route_with_api_key() {
        let state = setup_test_state().await;
        let app = create_router(state);

        let request = Request::builder()
            .method("GET")
            .uri("/api/shortens?page=1&page_size=10")
            .header("X-API-KEY", "test-api-key")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_router_public_route() {
        let state = setup_test_state().await;
        let app = create_router(state);

        let request = Request::builder()
            .method("POST")
            .uri("/api/account/login")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"username":"admin","password":"admin123"}"#))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        // Will fail password verification but should not be unauthorized
        assert_ne!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_router_not_found() {
        let state = setup_test_state().await;
        let app = create_router(state);

        let request = Request::builder()
            .method("GET")
            .uri("/api/nonexistent")
            .header("X-API-KEY", "test-api-key")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
