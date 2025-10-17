use axum::{extract::Request, middleware::Next, response::Response};
use std::time::Instant;

/// 日志中间件
/// 记录请求方法、路径、状态码、耗时
pub async fn logging_middleware(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let start = Instant::now();

    // 处理请求
    let response = next.run(request).await;

    // 计算耗时
    let duration = start.elapsed();
    let status = response.status();

    // 记录日志
    tracing::info!(
        method = %method,
        uri = %uri,
        status = %status.as_u16(),
        duration_ms = %duration.as_millis(),
        "HTTP request processed"
    );

    response
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

    async fn test_handler_error() -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }

    #[tokio::test]
    async fn test_logging_middleware_success() {
        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(middleware::from_fn(logging_middleware));

        let request = Request::builder().uri("/test").body(Body::empty()).unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_logging_middleware_error() {
        let app = Router::new()
            .route("/error", get(test_handler_error))
            .layer(middleware::from_fn(logging_middleware));

        let request = Request::builder()
            .uri("/error")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_logging_middleware_post_request() {
        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(middleware::from_fn(logging_middleware));

        let request = Request::builder()
            .method("POST")
            .uri("/test")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        // POST to GET route should return 405 Method Not Allowed
        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio::test]
    async fn test_logging_middleware_not_found() {
        let app = Router::new()
            .route("/test", get(test_handler))
            .layer(middleware::from_fn(logging_middleware));

        let request = Request::builder()
            .uri("/nonexistent")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
