use serde::{Deserialize, Serialize};

/// API 响应的通用结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub data: T,
}

/// 错误响应结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub errcode: String,
    pub errinfo: String,
}

/// 分页元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageMeta {
    pub page: i64,
    pub page_size: i64,
    pub current_count: i64,
    pub total_items: i64,
    pub total_pages: i64,
}

/// 分页响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PagedResponse<T> {
    pub data: Vec<T>,
    pub meta: PageMeta,
}

/// URL 状态枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum UrlStatus {
    Enabled = 0,
    Disabled = 1,
}

impl From<i32> for UrlStatus {
    fn from(value: i32) -> Self {
        match value {
            0 => UrlStatus::Enabled,
            1 => UrlStatus::Disabled,
            _ => UrlStatus::Enabled,
        }
    }
}

impl From<UrlStatus> for i32 {
    fn from(status: UrlStatus) -> Self {
        status as i32
    }
}
