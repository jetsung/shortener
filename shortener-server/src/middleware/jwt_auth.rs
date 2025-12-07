use crate::errors::AppError;
use crate::handlers::account::{User, verify_token};
use axum::{extract::Request, http::HeaderMap, middleware::Next, response::Response};

/// JWT 认证中间件
pub struct JwtAuth;

impl JwtAuth {
    /// 验证 JWT Token
    pub async fn check_jwt_token(
        headers: HeaderMap,
        mut request: Request,
        next: Next,
    ) -> Result<Response, AppError> {
        // 从请求头中获取 Authorization Bearer token
        let auth_header = headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        // 检查是否有 Bearer token
        if !auth_header.starts_with("Bearer ") {
            tracing::warn!("Missing or invalid Authorization header");
            return Err(AppError::Unauthorized(
                "Bearer token is required".to_string(),
            ));
        }

        // 提取 token
        let token = &auth_header[7..]; // 去掉 "Bearer " 前缀

        // 验证 token 并获取用户名
        let username = verify_token(token)?;

        // 创建用户对象并添加到请求扩展中
        let user = User { username };
        request.extensions_mut().insert(user);

        // Token 验证通过，继续处理请求
        tracing::debug!("JWT token validated successfully");
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

    async fn test_handler(Extension(user): Extension<User>) -> String {
        format!("Hello, {}", user.username)
    }

    #[tokio::test]
    async fn test_jwt_auth_success() {
        let username = "testuser";
        let token = generate_token(username).unwrap();

        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(middleware::from_fn(JwtAuth::check_jwt_token));

        let request = Request::builder()
            .uri("/test")
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_jwt_auth_missing_header() {
        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(middleware::from_fn(JwtAuth::check_jwt_token));

        let request = Request::builder().uri("/test").body(Body::empty()).unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_jwt_auth_invalid_token() {
        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(middleware::from_fn(JwtAuth::check_jwt_token));

        let request = Request::builder()
            .uri("/test")
            .header("Authorization", "Bearer invalid.token.here")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_jwt_auth_missing_bearer_prefix() {
        let username = "testuser";
        let token = generate_token(username).unwrap();

        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(middleware::from_fn(JwtAuth::check_jwt_token));

        let request = Request::builder()
            .uri("/test")
            .header("Authorization", token) // 没有 "Bearer " 前缀
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
