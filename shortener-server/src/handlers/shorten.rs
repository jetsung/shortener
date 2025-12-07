use crate::errors::AppError;
use crate::repositories::url_repository::ListParams;
use crate::services::{
    CreateShortenRequest, PagedResponse, ShortenResponse, ShortenService, UpdateShortenRequest,
};
use axum::{
    Json,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::Redirect,
};
use serde::Deserialize;
use std::sync::Arc;
use tracing::info;

/// Create a new short URL
///
/// POST /api/shortens
pub async fn create_shorten(
    State(service): State<Arc<ShortenService>>,
    Json(req): Json<CreateShortenRequest>,
) -> Result<(StatusCode, Json<ShortenResponse>), AppError> {
    info!("Creating short URL for: {}", req.original_url);

    let response = service.create_shorten(req).await?;

    Ok((StatusCode::CREATED, Json(response)))
}

/// Get a short URL by short_code
///
/// GET /api/shortens/{short_code}
pub async fn get_shorten(
    State(service): State<Arc<ShortenService>>,
    Path(short_code): Path<String>,
) -> Result<Json<ShortenResponse>, AppError> {
    info!("Getting short URL: {}", short_code);

    let response = service.get_shorten(&short_code).await?;

    Ok(Json(response))
}

/// List short URLs with pagination
///
/// GET /api/shortens
pub async fn list_shortens(
    State(service): State<Arc<ShortenService>>,
    Query(params): Query<ListParams>,
) -> Result<Json<PagedResponse<ShortenResponse>>, AppError> {
    info!(
        "Listing short URLs: page={}, per_page={}",
        params.page, params.page_size
    );

    let response = service.list_shortens(params).await?;

    Ok(Json(response))
}

/// Update a short URL
///
/// PUT /api/shortens/{short_code}
pub async fn update_shorten(
    State(service): State<Arc<ShortenService>>,
    Path(short_code): Path<String>,
    Json(req): Json<UpdateShortenRequest>,
) -> Result<Json<ShortenResponse>, AppError> {
    info!("Updating short URL: {}", short_code);

    let response = service.update_shorten(&short_code, req).await?;

    Ok(Json(response))
}

/// Delete a short URL
///
/// DELETE /api/shortens/{short_code}
pub async fn delete_shorten(
    State(service): State<Arc<ShortenService>>,
    Path(short_code): Path<String>,
) -> Result<StatusCode, AppError> {
    info!("Deleting short URL: {}", short_code);

    service.delete_shorten(&short_code).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Query parameters for batch delete
#[derive(Debug, Deserialize)]
pub struct DeleteBatchParams {
    pub ids: String, // Comma-separated list of IDs
}

/// Delete multiple short URLs
///
/// DELETE /api/shortens?ids=1,2,3
pub async fn delete_batch(
    State(service): State<Arc<ShortenService>>,
    Query(params): Query<DeleteBatchParams>,
) -> Result<StatusCode, AppError> {
    // Parse comma-separated IDs
    let ids: Result<Vec<i64>, _> = params
        .ids
        .split(',')
        .map(|s| s.trim().parse::<i64>())
        .collect();

    let ids = ids.map_err(|_| AppError::BadRequest("Invalid ID format".to_string()))?;

    info!("Batch deleting {} short URLs", ids.len());

    service.delete_batch(ids).await?;

    Ok(StatusCode::NO_CONTENT)
}

/// Redirect to original URL by short code
///
/// GET /{short_code}
pub async fn redirect_to_url(
    State(state): State<crate::router::AppState>,
    Path(short_code): Path<String>,
    headers: HeaderMap,
) -> Result<Redirect, AppError> {
    info!("Redirecting short code: {}", short_code);

    // Get the short URL info
    let shorten_response = state.shorten_service.get_shorten(&short_code).await?;

    // Check if the URL is disabled
    if shorten_response.status != 0 {
        return Err(AppError::NotFound("Short URL is disabled".to_string()));
    }

    // Extract client information for history logging
    let user_agent = headers.get("user-agent").and_then(|h| h.to_str().ok());

    let referer = headers.get("referer").and_then(|h| h.to_str().ok());

    // Extract IP address from headers
    // Try to get real IP from common proxy headers
    let ip_address = headers
        .get("x-forwarded-for")
        .and_then(|h| h.to_str().ok())
        .and_then(|s| s.split(',').next())
        .or_else(|| headers.get("x-real-ip").and_then(|h| h.to_str().ok()))
        .unwrap_or("unknown");

    // Record access history asynchronously (don't block the redirect)
    let history_service = state.history_service.clone();
    let url_id = shorten_response.id;
    let code = short_code.clone();
    let ip = ip_address.to_string();
    let ua = user_agent.map(|s| s.to_string());
    let ref_url = referer.map(|s| s.to_string());

    tokio::spawn(async move {
        if let Err(e) = history_service
            .record_access(url_id, &code, &ip, ua.as_deref(), ref_url.as_deref())
            .await
        {
            tracing::error!("Failed to record access history: {:?}", e);
        }
    });

    info!(
        "Redirecting: short_code={}, ip={}, user_agent={:?}, referer={:?}",
        short_code, ip_address, user_agent, referer
    );

    // Redirect to the original URL
    Ok(Redirect::permanent(&shorten_response.original_url))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::NullCache;
    use crate::config::{Config, DatabaseConfig, DatabaseType, SqliteConfig};
    use crate::db::DbFactory;
    use crate::repositories::url_repository::UrlRepositoryImpl;
    use axum::Router;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use serde_json::json;
    use tower::ServiceExt;

    async fn setup_test_app() -> Router {
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

        let url_repo = Arc::new(UrlRepositoryImpl::new(db));
        let cache = Arc::new(NullCache::new());

        let service = Arc::new(ShortenService::new(
            url_repo,
            cache,
            config.shortener.clone(),
            config.server.site_url.clone(),
        ));

        Router::new()
            .route("/api/shortens", axum::routing::post(create_shorten))
            .route("/api/shortens", axum::routing::get(list_shortens))
            .route("/api/shortens", axum::routing::delete(delete_batch))
            .route("/api/shortens/{code}", axum::routing::get(get_shorten))
            .route("/api/shortens/{code}", axum::routing::put(update_shorten))
            .route(
                "/api/shortens/{code}",
                axum::routing::delete(delete_shorten),
            )
            .with_state(service)
    }

    #[tokio::test]
    async fn test_create_shorten_handler() {
        let app = setup_test_app().await;

        let request_body = json!({
            "original_url": "https://example.com",
            "short_code": "test123",
            "description": "Test URL"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/shortens")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(response_json["short_code"], "test123");
        assert_eq!(response_json["original_url"], "https://example.com");
        assert_eq!(response_json["description"], "Test URL");
    }

    #[tokio::test]
    async fn test_create_shorten_auto_code() {
        let app = setup_test_app().await;

        let request_body = json!({
            "original_url": "https://example.com"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/shortens")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::CREATED);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(response_json["original_url"], "https://example.com");
        assert_eq!(response_json["short_code"].as_str().unwrap().len(), 6);
    }

    #[tokio::test]
    async fn test_create_shorten_invalid_url() {
        let app = setup_test_app().await;

        let request_body = json!({
            "original_url": "not-a-url"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/shortens")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&request_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_get_shorten_handler() {
        let app = setup_test_app().await;

        // Create a URL first
        let create_body = json!({
            "original_url": "https://example.com",
            "short_code": "gettest"
        });

        app.clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/shortens")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&create_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        // Get the URL
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/shortens/gettest")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(response_json["short_code"], "gettest");
        assert_eq!(response_json["original_url"], "https://example.com");
    }

    #[tokio::test]
    async fn test_get_shorten_not_found() {
        let app = setup_test_app().await;

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/shortens/nonexistent")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_list_shortens_handler() {
        let app = setup_test_app().await;

        // Create multiple URLs
        for i in 1..=3 {
            let create_body = json!({
                "original_url": format!("https://example{}.com", i),
                "short_code": format!("list{}", i)
            });

            app.clone()
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/api/shortens")
                        .header("content-type", "application/json")
                        .body(Body::from(serde_json::to_string(&create_body).unwrap()))
                        .unwrap(),
                )
                .await
                .unwrap();
        }

        // List URLs
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/shortens?page=1&page_size=10")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(response_json["data"].as_array().unwrap().len(), 3);
        assert_eq!(response_json["meta"]["total"], 3);
    }

    #[tokio::test]
    async fn test_update_shorten_handler() {
        let app = setup_test_app().await;

        // Create a URL first
        let create_body = json!({
            "original_url": "https://example.com",
            "short_code": "update"
        });

        app.clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/shortens")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&create_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        // Update the URL
        let update_body = json!({
            "original_url": "https://updated.com",
            "description": "Updated description"
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri("/api/shortens/update")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&update_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(response_json["original_url"], "https://updated.com");
        assert_eq!(response_json["description"], "Updated description");
    }

    #[tokio::test]
    async fn test_delete_shorten_handler() {
        let app = setup_test_app().await;

        // Create a URL first
        let create_body = json!({
            "original_url": "https://example.com",
            "short_code": "delete"
        });

        app.clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/shortens")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&create_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        // Delete the URL
        let response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri("/api/shortens/delete")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn test_delete_batch_handler() {
        let app = setup_test_app().await;

        // Create multiple URLs and collect their IDs
        let mut ids = Vec::new();
        for i in 1..=3 {
            let create_body = json!({
                "original_url": format!("https://example{}.com", i),
                "short_code": format!("batch{}", i)
            });

            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/api/shortens")
                        .header("content-type", "application/json")
                        .body(Body::from(serde_json::to_string(&create_body).unwrap()))
                        .unwrap(),
                )
                .await
                .unwrap();

            let body = axum::body::to_bytes(response.into_body(), usize::MAX)
                .await
                .unwrap();
            let response_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
            ids.push(response_json["id"].as_i64().unwrap());
        }

        // Delete batch
        let ids_str = ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(",");

        let response = app
            .oneshot(
                Request::builder()
                    .method("DELETE")
                    .uri(format!("/api/shortens?ids={}", ids_str))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }
}
