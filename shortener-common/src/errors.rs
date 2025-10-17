use thiserror::Error;

/// 通用错误类型
#[derive(Error, Debug)]
pub enum CommonError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// 错误码常量
pub mod error_codes {
    // 系统错误 (00xxx)
    pub const SYSTEM_ERROR: &str = "00001";
    pub const CONFIG_ERROR: &str = "00002";

    // 业务错误 (10xxx)
    pub const URL_NOT_FOUND: &str = "10001";
    pub const CODE_EXISTS: &str = "10002";
    pub const INVALID_URL: &str = "10003";

    // HTTP 错误 (40xxx)
    pub const UNAUTHORIZED: &str = "40001";
    pub const FORBIDDEN: &str = "40003";
    pub const NOT_FOUND: &str = "40004";

    // 第三方服务错误 (50xxx)
    pub const DATABASE_ERROR: &str = "50001";
    pub const CACHE_ERROR: &str = "50002";
    pub const GEOIP_ERROR: &str = "50003";
}
