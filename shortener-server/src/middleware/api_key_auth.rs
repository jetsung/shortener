use axum::{extract::Request, http::HeaderMap, middleware::Next, response::Response};
use std::sync::Arc;

use crate::errors::AppError;

/// API Key 认证中间件
#[derive(Clone)]
pub struct ApiKeyAuth {
    #[allow(dead_code)]
    api_key: Arc<String>,
}

impl ApiKeyAuth {
    /// 创建新的 API Key 认证中间件
    pub fn new(api_key: String) -> Self {
        Self {
            api_key: Arc::new(api_key),
        }
    }

    /// 验证 API Key
    pub async fn check_api_key(
        api_key: Arc<String>,
        headers: HeaderMap,
        request: Request,
        next: Next,
    ) -> Result<Response, AppError> {
        // 从请求头中获取 X-API-KEY
        let provided_key = headers
            .get("X-API-KEY")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        // 验证 API Key
        if provided_key.is_empty() {
            tracing::warn!("API Key missing in request");
            return Err(AppError::Unauthorized("API Key is required".to_string()));
        }

        if provided_key != api_key.as_str() {
            tracing::warn!("Invalid API Key provided: {}", provided_key);
            return Err(AppError::Unauthorized("Invalid API Key".to_string()));
        }

        // API Key 验证通过，继续处理请求
        tracing::debug!("API Key validated successfully");
        Ok(next.run(request).await)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        Router,
        body::Body,
        http::{Request, StatusCode},
        middleware,
        routing::get,
    };
    use tower::ServiceExt;

    async fn test_handler() -> &'static str {
        "OK"
    }

    #[tokio::test]
    async fn test_api_key_auth_success() {
        let api_key = "test-api-key-123".to_string();
        let auth = ApiKeyAuth::new(api_key.clone());

        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(middleware::from_fn(move |headers, req, next| {
                ApiKeyAuth::check_api_key(auth.api_key.clone(), headers, req, next)
            }));

        let request = Request::builder()
            .uri("/test")
            .header("X-API-KEY", api_key)
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_api_key_auth_missing_key() {
        let api_key = "test-api-key-123".to_string();
        let auth = ApiKeyAuth::new(api_key);

        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(middleware::from_fn(move |headers, req, next| {
                ApiKeyAuth::check_api_key(auth.api_key.clone(), headers, req, next)
            }));

        let request = Request::builder().uri("/test").body(Body::empty()).unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_api_key_auth_invalid_key() {
        let api_key = "test-api-key-123".to_string();
        let auth = ApiKeyAuth::new(api_key);

        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(middleware::from_fn(move |headers, req, next| {
                ApiKeyAuth::check_api_key(auth.api_key.clone(), headers, req, next)
            }));

        let request = Request::builder()
            .uri("/test")
            .header("X-API-KEY", "wrong-key")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_api_key_auth_empty_key() {
        let api_key = "test-api-key-123".to_string();
        let auth = ApiKeyAuth::new(api_key);

        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(middleware::from_fn(move |headers, req, next| {
                ApiKeyAuth::check_api_key(auth.api_key.clone(), headers, req, next)
            }));

        let request = Request::builder()
            .uri("/test")
            .header("X-API-KEY", "")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
