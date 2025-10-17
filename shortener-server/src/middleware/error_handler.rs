use axum::{
    Json,
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use futures_util::FutureExt;
use std::panic::AssertUnwindSafe;

use crate::errors::ErrorResponse;

/// 错误处理中间件
/// 统一捕获和处理未处理的错误，记录错误日志，返回统一格式的错误响应
pub async fn error_handler_middleware(request: Request, next: Next) -> Response {
    // 使用 catch_unwind 捕获 panic
    let result = AssertUnwindSafe(next.run(request)).catch_unwind().await;

    match result {
        Ok(response) => response,
        Err(panic_err) => {
            // 记录 panic 错误
            let panic_msg = if let Some(s) = panic_err.downcast_ref::<&str>() {
                s.to_string()
            } else if let Some(s) = panic_err.downcast_ref::<String>() {
                s.clone()
            } else {
                "Unknown panic occurred".to_string()
            };

            tracing::error!("Panic caught in error handler: {}", panic_msg);

            // 返回统一格式的错误响应
            let error_response = ErrorResponse::new(
                "00001", // SYSTEM_ERROR
                "Internal server error occurred",
            );

            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response)).into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Router, body::Body, http::Request, middleware, routing::get};
    use tower::ServiceExt;

    async fn test_handler_ok() -> &'static str {
        "OK"
    }

    async fn test_handler_panic() -> &'static str {
        panic!("Test panic");
    }

    #[tokio::test]
    async fn test_error_handler_middleware_success() {
        let app = Router::new()
            .route("/test", get(test_handler_ok))
            .layer(middleware::from_fn(error_handler_middleware));

        let request = Request::builder().uri("/test").body(Body::empty()).unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_error_handler_middleware_panic() {
        let app = Router::new()
            .route("/panic", get(test_handler_panic))
            .layer(middleware::from_fn(error_handler_middleware));

        let request = Request::builder()
            .uri("/panic")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_error_handler_middleware_not_found() {
        let app = Router::new()
            .route("/test", get(test_handler_ok))
            .layer(middleware::from_fn(error_handler_middleware));

        let request = Request::builder()
            .uri("/nonexistent")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
