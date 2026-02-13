use crate::errors::AppError;
use crate::repositories::history_repository::HistoryListParams;
use crate::services::{HistoryResponse, HistoryService, PagedResponse};
use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};
use serde::Deserialize;
use std::sync::Arc;
use tracing::info;

/// List access histories with pagination
///
/// GET /api/histories
pub async fn list_histories(
    State(service): State<Arc<HistoryService>>,
    Query(params): Query<HistoryListParams>,
) -> Result<Json<PagedResponse<HistoryResponse>>, AppError> {
    info!(
        "Listing histories: page={}, per_page={}",
        params.page, params.page_size
    );

    let response = service.list_histories(params).await?;

    Ok(Json(response))
}

/// Request body for batch delete histories
#[derive(Debug, Deserialize)]
pub struct BatchDeleteHistoriesRequest {
    pub ids: Vec<i64>,
}

/// Delete multiple history records
///
/// POST /api/histories/batch-delete
pub async fn delete_histories(
    State(service): State<Arc<HistoryService>>,
    Json(req): Json<BatchDeleteHistoriesRequest>,
) -> Result<StatusCode, AppError> {
    if req.ids.is_empty() {
        return Err(AppError::BadRequest("ids cannot be empty".to_string()));
    }

    info!("Batch deleting {} history records", req.ids.len());

    service.delete_batch(req.ids).await?;

    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, DatabaseConfig, DatabaseType, SqliteConfig};
    use crate::db::DbFactory;
    use crate::geoip::NullGeoIp;
    use crate::repositories::history_repository::HistoryRepositoryImpl;
    use crate::repositories::url_repository::{UrlRepository, UrlRepositoryImpl};
    use axum::Router;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    async fn setup_test_app() -> (Router, Arc<dyn UrlRepository>) {
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
        let geoip = Some(Arc::new(NullGeoIp::new()) as Arc<dyn crate::geoip::GeoIp>);

        let service = Arc::new(HistoryService::new(history_repo, geoip));

        let app = Router::new()
            .route("/api/histories", axum::routing::get(list_histories))
            .route(
                "/api/histories/batch-delete",
                axum::routing::post(delete_histories),
            )
            .with_state(service);

        (app, url_repo)
    }

    #[tokio::test]
    async fn test_list_histories_handler() {
        let (app, _url_repo) = setup_test_app().await;

        // Test listing empty histories
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/histories?page=1&page_size=10")
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

        assert!(response_json["data"].is_array());
        assert!(response_json["meta"].is_object());
    }

    #[tokio::test]
    async fn test_delete_histories_handler() {
        let (app, _) = setup_test_app().await;

        // Test with valid IDs
        let delete_body = serde_json::json!({
            "ids": [999]
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/histories/batch-delete")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&delete_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn test_delete_histories_empty_ids() {
        let (app, _) = setup_test_app().await;

        // Test with empty IDs array
        let delete_body = serde_json::json!({
            "ids": []
        });

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/histories/batch-delete")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&delete_body).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }
}
