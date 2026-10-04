use crate::config::Config;
use crate::handlers::{
    create_shorten, current_user, delete_batch, delete_histories, delete_shorten, get_shorten,
    list_histories, list_shortens, login, logout, oidc_callback, oidc_login, redirect_to_url,
    refresh_cache, update_shorten,
};
use crate::middleware::{HybridAuth, error_handler_middleware, logging_middleware};
use crate::services::{HistoryService, ShortenService};
use axum::{
    Router, extract::State, http::Request, middleware, response::{IntoResponse, Response},
    routing::{delete, get, post, put},
};
use std::path::PathBuf;
use std::sync::Arc;
use tower::util::ServiceExt;
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};

/// Application state shared across handlers
#[derive(Clone)]
pub struct AppState {
    pub shorten_service: Arc<ShortenService>,
    pub history_service: Arc<HistoryService>,
    pub config: Arc<Config>,
}

/// 解析后端托管的静态资源目录（可选）。
///
/// 未配置或值为空返回 `None`；配置后由 fallback handler 托管该目录。
fn static_dir_of(config: &Config) -> Option<PathBuf> {
    config
        .server
        .static_dir
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(PathBuf::from)
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
    // 短码跳转统一走 /go/ 前缀，避免与静态资源在根命名空间冲突
    let redirect_routes = Router::new()
        .route("/go/{short_code}", get(redirect_to_url))
        .with_state(state.clone());

    // Create health check routes（全部收敛到 /api 命名空间）
    let health_routes = Router::new()
        .route("/api", get(root))
        .route("/api/ping", get(ping));

    // Combine all routes
    Router::new()
        .merge(health_routes)
        .merge(protected_api)
        .merge(public_api)
        .merge(redirect_routes)
        .fallback(fallback_handler)
        .with_state(state.clone())
        // Add CORS layer
        .layer(CorsLayer::permissive())
        // Add logging middleware
        .layer(middleware::from_fn(logging_middleware))
        // Add error handler middleware
        .layer(middleware::from_fn(error_handler_middleware))
}

/// 未匹配任何路由的兜底处理。
///
/// 配置了 `static-dir` 时托管前端静态资源：命中文件直接返回，
/// 未命中回退 `index.html`（SPA 回退，200），使前端路由可接管；
/// 未配置时返回结构化 404。
async fn fallback_handler(State(state): State<AppState>, request: Request<axum::body::Body>) -> Response {
    let Some(static_dir) = static_dir_of(&state.config) else {
        let path = request.uri().path().to_owned();
        return (
            axum::http::StatusCode::NOT_FOUND,
            axum::Json(serde_json::json!({
                "code": 404,
                "message": format!("路径不存在: {path}"),
            })),
        )
            .into_response();
    };
    // 用 `fallback` 而不是 `not_found_service`：后者会把回退响应状态码
    // 强制改写为 404，而 SPA 深链回退必须以 200 交出入口 HTML。
    let index = static_dir.join("index.html");
    let service = ServeDir::new(&static_dir).fallback(ServeFile::new(index));
    match service.oneshot(request).await {
        Ok(response) => response.into_response(),
        Err(never) => match never {},
    }
}

/// Health check handler
///
/// GET /api/ping
async fn ping() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "message": "pong"
    }))
}

/// Root handler - service information
///
/// GET /api
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
                static_dir: None,
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
            .uri("/go/test123")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_root_path_single_segment_is_not_redirect() {
        let state = setup_test_state().await;
        let app = create_router(state);

        // 先创建短码 redirect123，但根路径 /redirect123 不再触发跳转
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

        let request = Request::builder()
            .method("GET")
            .uri("/redirect123")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        // 未配置 static_dir 时兜底返回结构化 404，而非 302 跳转
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
            .uri("/go/redirect123")
            .body(Body::empty())
            .unwrap();

        let redirect_response = app.oneshot(redirect_request).await.unwrap();
        assert_eq!(redirect_response.status(), StatusCode::PERMANENT_REDIRECT);

        // Check the Location header
        let location = redirect_response.headers().get("location").unwrap();
        assert_eq!(location.to_str().unwrap(), "https://example.com");
    }

    #[tokio::test]
    async fn test_static_dir_serves_files_and_spa_fallback() {
        let mut state = setup_test_state().await;
        // 构造临时静态目录：index.html + assets/app.js
        let dir = std::env::temp_dir().join(format!("shortener-static-test-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("assets")).unwrap();
        std::fs::write(dir.join("index.html"), "<html>spa</html>").unwrap();
        std::fs::write(dir.join("assets/app.js"), "console.log(1)").unwrap();

        let config = Arc::make_mut(&mut state.config);
        config.server.static_dir = Some(dir.to_string_lossy().into_owned());

        let app = create_router(state);

        // 命中静态文件
        let request = Request::builder()
            .method("GET")
            .uri("/assets/app.js")
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // SPA 深链回退：无对应文件时以 200 返回 index.html
        let request = Request::builder()
            .method("GET")
            .uri("/some/spa/route")
            .body(Body::empty())
            .unwrap();
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(&body[..], b"<html>spa</html>");

        std::fs::remove_dir_all(&dir).ok();
    }

    #[tokio::test]
    async fn test_api_route_takes_precedence_over_static_dir() {
        let mut state = setup_test_state().await;
        let dir = std::env::temp_dir().join(format!("shortener-static-api-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();

        let config = Arc::make_mut(&mut state.config);
        config.server.static_dir = Some(dir.to_string_lossy().into_owned());

        let app = create_router(state);

        // 即使配置了 static_dir，API 路由仍优先进入处理逻辑（无凭证返回 401 而非静态内容）
        let request = Request::builder()
            .method("GET")
            .uri("/api/shortens")
            .body(Body::empty())
            .unwrap();
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        std::fs::remove_dir_all(&dir).ok();
    }

    #[tokio::test]
    async fn test_redirect_with_static_dir_still_works() {
        let mut state = setup_test_state().await;
        let dir = std::env::temp_dir().join(format!("shortener-static-to-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();

        let config = Arc::make_mut(&mut state.config);
        config.server.static_dir = Some(dir.to_string_lossy().into_owned());

        let app = create_router(state);

        let create_request = Request::builder()
            .method("POST")
            .uri("/api/shortens")
            .header("X-API-KEY", "test-api-key")
            .header("content-type", "application/json")
            .body(Body::from(
                r#"{"original_url":"https://example.com","short_code":"mixabc"}"#,
            ))
            .unwrap();
        let create_response = app.clone().oneshot(create_request).await.unwrap();
        assert_eq!(create_response.status(), StatusCode::CREATED);

        let request = Request::builder()
            .method("GET")
            .uri("/go/mixabc")
            .body(Body::empty())
            .unwrap();
        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::PERMANENT_REDIRECT);

        std::fs::remove_dir_all(&dir).ok();
    }
}
