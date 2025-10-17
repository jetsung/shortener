use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use thiserror::Error;

// Re-export error codes from common
pub use shortener_common::error_codes;

/// 应用错误类型
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),

    #[error("Cache error: {0}")]
    Cache(String),

    #[error("GeoIP error: {0}")]
    GeoIp(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Internal server error: {0}")]
    Internal(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
}

/// 错误响应结构体
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub errcode: String,
    pub errinfo: String,
}

impl ErrorResponse {
    pub fn new(errcode: impl Into<String>, errinfo: impl Into<String>) -> Self {
        Self {
            errcode: errcode.into(),
            errinfo: errinfo.into(),
        }
    }
}

/// 实现 IntoResponse trait 用于 HTTP 错误响应
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, errcode, errinfo) = match &self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, error_codes::NOT_FOUND, msg.clone()),
            AppError::Conflict(msg) => {
                (StatusCode::CONFLICT, error_codes::CODE_EXISTS, msg.clone())
            }
            AppError::Unauthorized(msg) => (
                StatusCode::UNAUTHORIZED,
                error_codes::UNAUTHORIZED,
                msg.clone(),
            ),
            AppError::Forbidden(msg) => {
                (StatusCode::FORBIDDEN, error_codes::FORBIDDEN, msg.clone())
            }
            AppError::BadRequest(msg) => (
                StatusCode::BAD_REQUEST,
                error_codes::INVALID_URL,
                msg.clone(),
            ),
            AppError::Validation(msg) => (
                StatusCode::BAD_REQUEST,
                error_codes::INVALID_URL,
                msg.clone(),
            ),
            AppError::Database(err) => {
                tracing::error!("Database error: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    error_codes::DATABASE_ERROR,
                    "Database operation failed".to_string(),
                )
            }
            AppError::Cache(msg) => {
                tracing::warn!("Cache error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    error_codes::CACHE_ERROR,
                    "Cache operation failed".to_string(),
                )
            }
            AppError::GeoIp(msg) => {
                tracing::warn!("GeoIP error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    error_codes::GEOIP_ERROR,
                    "GeoIP lookup failed".to_string(),
                )
            }
            AppError::Config(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                error_codes::CONFIG_ERROR,
                msg.clone(),
            ),
            AppError::ServiceUnavailable(msg) => (
                StatusCode::SERVICE_UNAVAILABLE,
                error_codes::SYSTEM_ERROR,
                msg.clone(),
            ),
            AppError::Internal(msg) => {
                tracing::error!("Internal error: {}", msg);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    error_codes::SYSTEM_ERROR,
                    "Internal server error".to_string(),
                )
            }
        };

        let body = Json(ErrorResponse::new(errcode, errinfo));
        (status, body).into_response()
    }
}

/// 缓存错误类型
#[derive(Error, Debug)]
pub enum CacheError {
    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Operation error: {0}")]
    Operation(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Not found")]
    NotFound,
}

impl From<CacheError> for AppError {
    fn from(err: CacheError) -> Self {
        AppError::Cache(err.to_string())
    }
}

/// GeoIP 错误类型
#[derive(Error, Debug)]
pub enum GeoIpError {
    #[error("Database not found: {0}")]
    DatabaseNotFound(String),

    #[error("Lookup failed: {0}")]
    LookupFailed(String),

    #[error("Invalid IP address: {0}")]
    InvalidIp(String),

    #[error("Initialization error: {0}")]
    Initialization(String),
}

impl From<GeoIpError> for AppError {
    fn from(err: GeoIpError) -> Self {
        AppError::GeoIp(err.to_string())
    }
}

/// Service 层错误类型
#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("Repository error: {0}")]
    Repository(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Already exists: {0}")]
    AlreadyExists(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Cache error: {0}")]
    Cache(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl From<ServiceError> for AppError {
    fn from(err: ServiceError) -> Self {
        match err {
            ServiceError::NotFound(msg) => AppError::NotFound(msg),
            ServiceError::AlreadyExists(msg) => AppError::Conflict(msg),
            ServiceError::InvalidInput(msg) => AppError::BadRequest(msg),
            ServiceError::Cache(msg) => AppError::Cache(msg),
            ServiceError::Repository(msg) | ServiceError::Internal(msg) => AppError::Internal(msg),
        }
    }
}

impl From<sea_orm::DbErr> for ServiceError {
    fn from(err: sea_orm::DbErr) -> Self {
        ServiceError::Repository(err.to_string())
    }
}

impl From<CacheError> for ServiceError {
    fn from(err: CacheError) -> Self {
        ServiceError::Cache(err.to_string())
    }
}

/// Repository 层错误类型
#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("Database error: {0}")]
    Database(#[from] sea_orm::DbErr),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Constraint violation: {0}")]
    ConstraintViolation(String),

    #[error("Invalid query: {0}")]
    InvalidQuery(String),
}

impl From<RepositoryError> for ServiceError {
    fn from(err: RepositoryError) -> Self {
        match err {
            RepositoryError::NotFound(msg) => ServiceError::NotFound(msg),
            RepositoryError::ConstraintViolation(msg) => ServiceError::AlreadyExists(msg),
            RepositoryError::Database(db_err) => ServiceError::Repository(db_err.to_string()),
            RepositoryError::InvalidQuery(msg) => ServiceError::InvalidInput(msg),
        }
    }
}

impl From<RepositoryError> for AppError {
    fn from(err: RepositoryError) -> Self {
        match err {
            RepositoryError::NotFound(msg) => AppError::NotFound(msg),
            RepositoryError::ConstraintViolation(msg) => AppError::Conflict(msg),
            RepositoryError::Database(db_err) => AppError::Database(db_err),
            RepositoryError::InvalidQuery(msg) => AppError::BadRequest(msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_response_creation() {
        let response = ErrorResponse::new("TEST001", "Test error message");
        assert_eq!(response.errcode, "TEST001");
        assert_eq!(response.errinfo, "Test error message");
    }

    #[test]
    fn test_app_error_not_found() {
        let error = AppError::NotFound("Resource not found".to_string());
        assert_eq!(error.to_string(), "Not found: Resource not found");
    }

    #[test]
    fn test_app_error_conflict() {
        let error = AppError::Conflict("Code already exists".to_string());
        assert_eq!(error.to_string(), "Conflict: Code already exists");
    }

    #[test]
    fn test_app_error_unauthorized() {
        let error = AppError::Unauthorized("Invalid credentials".to_string());
        assert_eq!(error.to_string(), "Unauthorized: Invalid credentials");
    }

    #[test]
    fn test_app_error_bad_request() {
        let error = AppError::BadRequest("Invalid URL format".to_string());
        assert_eq!(error.to_string(), "Bad request: Invalid URL format");
    }

    #[test]
    fn test_cache_error_conversion() {
        let cache_error = CacheError::Connection("Redis connection failed".to_string());
        let app_error: AppError = cache_error.into();
        assert!(matches!(app_error, AppError::Cache(_)));
    }

    #[test]
    fn test_geoip_error_conversion() {
        let geoip_error = GeoIpError::DatabaseNotFound("/path/to/db".to_string());
        let app_error: AppError = geoip_error.into();
        assert!(matches!(app_error, AppError::GeoIp(_)));
    }

    #[test]
    fn test_service_error_not_found_conversion() {
        let service_error = ServiceError::NotFound("URL not found".to_string());
        let app_error: AppError = service_error.into();
        assert!(matches!(app_error, AppError::NotFound(_)));
    }

    #[test]
    fn test_service_error_already_exists_conversion() {
        let service_error = ServiceError::AlreadyExists("Code exists".to_string());
        let app_error: AppError = service_error.into();
        assert!(matches!(app_error, AppError::Conflict(_)));
    }

    #[test]
    fn test_service_error_invalid_input_conversion() {
        let service_error = ServiceError::InvalidInput("Invalid data".to_string());
        let app_error: AppError = service_error.into();
        assert!(matches!(app_error, AppError::BadRequest(_)));
    }

    #[test]
    fn test_repository_error_not_found_conversion() {
        let repo_error = RepositoryError::NotFound("Record not found".to_string());
        let service_error: ServiceError = repo_error.into();
        assert!(matches!(service_error, ServiceError::NotFound(_)));
    }

    #[test]
    fn test_repository_error_constraint_violation_conversion() {
        let repo_error = RepositoryError::ConstraintViolation("Unique constraint".to_string());
        let service_error: ServiceError = repo_error.into();
        assert!(matches!(service_error, ServiceError::AlreadyExists(_)));
    }

    #[test]
    fn test_error_codes_constants() {
        assert_eq!(error_codes::SYSTEM_ERROR, "00001");
        assert_eq!(error_codes::CONFIG_ERROR, "00002");
        assert_eq!(error_codes::URL_NOT_FOUND, "10001");
        assert_eq!(error_codes::CODE_EXISTS, "10002");
        assert_eq!(error_codes::INVALID_URL, "10003");
        assert_eq!(error_codes::UNAUTHORIZED, "40001");
        assert_eq!(error_codes::FORBIDDEN, "40003");
        assert_eq!(error_codes::NOT_FOUND, "40004");
        assert_eq!(error_codes::DATABASE_ERROR, "50001");
        assert_eq!(error_codes::CACHE_ERROR, "50002");
        assert_eq!(error_codes::GEOIP_ERROR, "50003");
    }

    #[tokio::test]
    async fn test_app_error_into_response_not_found() {
        let error = AppError::NotFound("Test resource".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_app_error_into_response_conflict() {
        let error = AppError::Conflict("Duplicate entry".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn test_app_error_into_response_unauthorized() {
        let error = AppError::Unauthorized("No token".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_app_error_into_response_bad_request() {
        let error = AppError::BadRequest("Invalid input".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_app_error_into_response_internal() {
        let error = AppError::Internal("Something went wrong".to_string());
        let response = error.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
