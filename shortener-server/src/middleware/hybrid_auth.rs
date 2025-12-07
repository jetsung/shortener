use crate::errors::AppError;
use crate::handlers::account::{User, verify_token};
use axum::{extract::Request, http::HeaderMap, middleware::Next, response::Response};
use std::sync::Arc;

/// 混合认证中间件 - 同时支持API Key和JWT Token
pub struct HybridAuth;

impl HybridAuth {
    /// 检查API Key或JWT Token认证
    /// 优先检查JWT Token，如果没有或无效则检查API Key
    pub async fn check_auth(
        api_key: Arc<String>,
        headers: HeaderMap,
        mut request: Request,
        next: Next,
    ) -> Result<Response, AppError> {
        // 首先尝试JWT Token认证
        if let Some(auth_header) = headers.get("Authorization")
            && let Ok(auth_str) = auth_header.to_str()
            && let Some(token) = auth_str.strip_prefix("Bearer ")
        {
            // 尝试验证JWT Token
            match verify_token(token) {
                Ok(username) => {
                    // JWT Token验证成功，添加用户信息到请求扩展
                    let user = User { username };
                    request.extensions_mut().insert(user);
                    tracing::debug!("JWT token validated successfully");
                    return Ok(next.run(request).await);
                }
                Err(_) => {
                    // JWT Token无效，继续尝试API Key
                    tracing::debug!("JWT token invalid, trying API Key");
                }
            }
        }

        // JWT Token不存在或无效，尝试API Key认证
        let provided_key = headers
            .get("X-API-KEY")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        // 验证 API Key
        if provided_key.is_empty() {
            tracing::warn!("Neither valid JWT token nor API Key provided");
            return Err(AppError::Unauthorized(
                "Authentication required: provide either Bearer token or X-API-KEY".to_string(),
            ));
        }

        if provided_key != api_key.as_str() {
            tracing::warn!("Invalid API Key provided: {}", provided_key);
            return Err(AppError::Unauthorized("Invalid API Key".to_string()));
        }

        // API Key 验证通过
        tracing::debug!("API Key validated successfully");
        Ok(next.run(request).await)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handlers::account::generate_token;
    use axum::{
        Extension, Router,
        body::Body,
        http::{Request, StatusCode},
        middleware,
        routing::get,
    };
    use tower::ServiceExt;

    async fn test_handler() -> &'static str {
        "OK"
    }

    async fn test_handler_with_user(Extension(user): Extension<User>) -> String {
        format!("Hello, {}", user.username)
    }

    #[tokio::test]
    async fn test_hybrid_auth_with_valid_jwt() {
        let api_key = "test-api-key-123".to_string();
        let username = "testuser";
        let token = generate_token(username).unwrap();

        let app = Router::new()
            .route("/test", get(test_handler_with_user))
            .layer(middleware::from_fn(move |headers, req, next| {
                let api_key = Arc::new(api_key.clone());
                HybridAuth::check_auth(api_key, headers, req, next)
            }));

        let request = Request::builder()
            .uri("/test")
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_hybrid_auth_with_valid_api_key() {
        let api_key = "test-api-key-123".to_string();

        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(middleware::from_fn(move |headers, req, next| {
                let api_key = Arc::new(api_key.clone());
                HybridAuth::check_auth(api_key, headers, req, next)
            }));

        let request = Request::builder()
            .uri("/test")
            .header("X-API-KEY", "test-api-key-123")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_hybrid_auth_jwt_priority_over_api_key() {
        let api_key = "test-api-key-123".to_string();
        let username = "testuser";
        let token = generate_token(username).unwrap();

        let app = Router::new()
            .route("/test", get(test_handler_with_user))
            .layer(middleware::from_fn(move |headers, req, next| {
                let api_key = Arc::new(api_key.clone());
                HybridAuth::check_auth(api_key, headers, req, next)
            }));

        let request = Request::builder()
            .uri("/test")
            .header("Authorization", format!("Bearer {}", token))
            .header("X-API-KEY", "test-api-key-123") // 同时提供两种认证
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // 应该使用JWT认证，因为它有更高优先级
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        assert_eq!(body_str, "Hello, testuser");
    }

    #[tokio::test]
    async fn test_hybrid_auth_invalid_jwt_fallback_to_api_key() {
        let api_key = "test-api-key-123".to_string();

        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(middleware::from_fn(move |headers, req, next| {
                let api_key = Arc::new(api_key.clone());
                HybridAuth::check_auth(api_key, headers, req, next)
            }));

        let request = Request::builder()
            .uri("/test")
            .header("Authorization", "Bearer invalid.token.here")
            .header("X-API-KEY", "test-api-key-123")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_hybrid_auth_no_auth_provided() {
        let api_key = "test-api-key-123".to_string();

        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(middleware::from_fn(move |headers, req, next| {
                let api_key = Arc::new(api_key.clone());
                HybridAuth::check_auth(api_key, headers, req, next)
            }));

        let request = Request::builder().uri("/test").body(Body::empty()).unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_hybrid_auth_invalid_both() {
        let api_key = "test-api-key-123".to_string();

        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(middleware::from_fn(move |headers, req, next| {
                let api_key = Arc::new(api_key.clone());
                HybridAuth::check_auth(api_key, headers, req, next)
            }));

        let request = Request::builder()
            .uri("/test")
            .header("Authorization", "Bearer invalid.token.here")
            .header("X-API-KEY", "wrong-api-key")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
