use crate::config::Config;
use crate::handlers::{
    create_shorten, current_user, delete_batch, delete_histories, delete_shorten, get_shorten,
    list_histories, list_shortens, login, logout, oidc_callback, oidc_login, redirect_to_url,
    refresh_cache, update_shorten,
};
use crate::middleware::{HybridAuth, error_handler_middleware, logging_middleware};
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

    // Create shortener API routes (protected)
    let shortener_api = Router::new()
        .route("/api/shortens", post(create_shorten))
        .route("/api/shortens", get(list_shortens))
        .route("/api/shortens/batch-delete", post(delete_batch))
        .route("/api/shortens/{short_code}", get(get_shorten))
        .route("/api/shortens/{short_code}", put(update_shorten))
        .route("/api/shortens/{short_code}", delete(delete_shorten))
        .with_state(state.shorten_service.clone());

    // Create history API routes (protected)
    let history_api = Router::new()
        .route("/api/histories", get(list_histories))
        .route("/api/histories/batch-delete", post(delete_histories))
        .with_state(state.history_service.clone());

    // Create account API routes (protected)
    let account_api = Router::new()
        .route("/api/account/logout", post(logout))
        .route("/api/users/current", get(current_user));

    // Create cache API routes (protected)
    let cache_api = Router::new()
        .route("/api/cache/refresh", post(refresh_cache))
        .with_state(state.clone());

    // Combine protected API routes
    let protected_api = Router::new()
        .merge(shortener_api)
        .merge(history_api)
        .merge(account_api)
        .merge(cache_api)
        // Apply hybrid authentication middleware (supports both API key and JWT token)
        .layer(middleware::from_fn(move |headers, req, next| {
            let api_key = api_key.clone();
            async move { HybridAuth::check_auth(api_key, headers, req, next).await }
        }));

    // Create public API routes (no authentication required)
    let login_api = Router::new()
        .route("/api/account/login", post(login))
        .with_state(Arc::new(state.config.admin.clone()));

    let oidc_api = Router::new()
        .route("/api/oidc/login", get(oidc_login))
        .route("/api/oidc/callback", get(oidc_callback))
        .with_state(Arc::new(state.config.oidc.clone()));

    let public_api = Router::new().merge(login_api).merge(oidc_api);

    // Create redirect routes (public, for short URL redirection)
    let redirect_routes = Router::new()
        .route("/{short_code}", get(redirect_to_url))
        .with_state(state.clone());

    // Create health check route
    let health_routes = Router::new()
        .route("/", get(root))
        .route("/ping", get(ping));

    // Combine all routes
    Router::new()
        .merge(health_routes)
        .merge(protected_api)
        .merge(public_api)
        .merge(redirect_routes)
        // Add CORS layer
        .layer(CorsLayer::permissive())
        // Add logging middleware
        .layer(middleware::from_fn(logging_middleware))
        // Add error handler middleware
        .layer(middleware::from_fn(error_handler_middleware))
}

/// Health check handler
///
/// GET /ping
async fn ping() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "message": "pong"
    }))
}

/// Root handler - service information
///
/// GET /
async fn root() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "service": "URL Shortener API",
        "version": env!("CARGO_PKG_VERSION"),
        "status": "running"
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::NullCache;
    use crate::config::{
        AdminConfig, CacheConfig, DatabaseConfig, GeoIpConfig, GeoIpType, OidcConfig,
        ServerConfig, SlugConfig,
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
                short_url: "http://localhost:8080".to_string(),
                api_key: "test-api-key".to_string(),
            },
            slug: SlugConfig {
                length: 6,
                alphabet: "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"
                    .to_string(),
            },
            admin: AdminConfig {
                username: "admin".to_string(),
                password_hash: "".to_string(),
            },
            oidc: OidcConfig::default(),
            database: DatabaseConfig {
                url: Some("sqlite::memory:".to_string()),
                log_level: 0,
            },
            cache: CacheConfig {
                enabled: false,
                expire: 3600,
                prefix: "shorten:".to_string(),
                url: None,
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
            config.slug.clone(),
            config.server.short_url.clone(),
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

    #[tokio::test]
    async fn test_redirect_route() {
        let state = setup_test_state().await;
        let app = create_router(state);

        // This will return 404 since no short URL exists with code "test123"
        let request = Request::builder()
            .method("GET")
            .uri("/test123")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_redirect_route_with_existing_code() {
        let state = setup_test_state().await;
        let app = create_router(state);

        // First create a short URL
        let create_request = Request::builder()
            .method("POST")
            .uri("/api/shortens")
            .header("X-API-KEY", "test-api-key")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"original_url":"https://example.com","short_code":"redirect123"}"#,
            ))
            .unwrap();

        let create_response = app.clone().oneshot(create_request).await.unwrap();
        assert_eq!(create_response.status(), StatusCode::CREATED);

        // Now test the redirect
        let redirect_request = Request::builder()
            .method("GET")
            .uri("/redirect123")
            .body(Body::empty())
            .unwrap();

        let redirect_response = app.oneshot(redirect_request).await.unwrap();
        assert_eq!(redirect_response.status(), StatusCode::PERMANENT_REDIRECT);

        // Check the Location header
        let location = redirect_response.headers().get("location").unwrap();
        assert_eq!(location.to_str().unwrap(), "https://example.com");
    }
}
