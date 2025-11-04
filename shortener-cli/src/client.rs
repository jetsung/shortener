use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// API Client errors
#[derive(Error, Debug)]
pub enum ClientError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),

    #[error("API error ({code}): {message}")]
    ApiError { code: String, message: String },

    #[error("Unauthorized: Invalid API key")]
    Unauthorized,

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Server error: {0}")]
    ServerError(String),
}

/// Error response from API
#[derive(Debug, Deserialize)]
struct ErrorResponse {
    errcode: String,
    errinfo: String,
}

/// Request DTO for creating a short URL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateShortenRequest {
    pub original_url: String,
    #[serde(skip_serializing_if = "Option::is_none", rename = "short_code")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "description")]
    pub describe: Option<String>,
}

/// Request DTO for updating a short URL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateShortenRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "description")]
    pub describe: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<i32>,
}

/// Response DTO for short URL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortenResponse {
    pub id: i64,
    #[serde(rename = "short_code")]
    pub code: String,
    pub short_url: String,
    pub original_url: String,
    #[serde(rename = "description")]
    pub describe: Option<String>,
    pub status: i32,
    pub created_at: String,
    pub updated_at: String,
}

/// Pagination metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageMeta {
    pub page: u64,
    pub per_page: u64,
    pub count: u64,
    pub total: u64,
    pub total_pages: u64,
}

/// Paginated response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PagedResponse<T> {
    pub data: Vec<T>,
    pub meta: PageMeta,
}

/// Query parameters for listing short URLs
#[derive(Debug, Clone, Default)]
pub struct ListParams {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub status: Option<i32>,
    pub sort: Option<String>,
    pub order: Option<String>,
    pub original_url: Option<String>,
}

/// API Client for interacting with the shortener server
pub struct ApiClient {
    base_url: String,
    api_key: String,
    client: Client,
}

impl ApiClient {
    /// Create a new API client
    pub fn new(base_url: String, api_key: String) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
            client: Client::new(),
        }
    }

    /// Create a new short URL
    ///
    /// POST /api/shortens
    pub async fn create_shorten(
        &self,
        req: CreateShortenRequest,
    ) -> Result<ShortenResponse, ClientError> {
        let url = format!("{}/api/shortens", self.base_url);

        let response = self
            .client
            .post(&url)
            .header("X-API-KEY", &self.api_key)
            .json(&req)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Get a short URL by short_code
    ///
    /// GET /api/shortens/{short_code}
    pub async fn get_shorten(&self, code: &str) -> Result<ShortenResponse, ClientError> {
        let url = format!("{}/api/shortens/{}", self.base_url, code);

        let response = self
            .client
            .get(&url)
            .header("X-API-KEY", &self.api_key)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// List short URLs with pagination
    ///
    /// GET /api/shortens
    pub async fn list_shortens(
        &self,
        params: ListParams,
    ) -> Result<PagedResponse<ShortenResponse>, ClientError> {
        let url = format!("{}/api/shortens", self.base_url);

        let mut request = self.client.get(&url).header("X-API-KEY", &self.api_key);

        // Add query parameters
        let mut query_params = vec![];
        if let Some(page) = params.page {
            query_params.push(("page", page.to_string()));
        }
        if let Some(page_size) = params.page_size {
            query_params.push(("per_page", page_size.to_string()));
        }
        if let Some(status) = params.status {
            query_params.push(("status", status.to_string()));
        }
        if let Some(sort) = params.sort {
            query_params.push(("sort", sort));
        }
        if let Some(order) = params.order {
            query_params.push(("order", order));
        }
        if let Some(original_url) = params.original_url {
            query_params.push(("original_url", original_url));
        }

        if !query_params.is_empty() {
            request = request.query(&query_params);
        }

        let response = request.send().await?;

        self.handle_response(response).await
    }

    /// Update a short URL
    ///
    /// PUT /api/shortens/{short_code}
    pub async fn update_shorten(
        &self,
        code: &str,
        req: UpdateShortenRequest,
    ) -> Result<ShortenResponse, ClientError> {
        let url = format!("{}/api/shortens/{}", self.base_url, code);

        let response = self
            .client
            .put(&url)
            .header("X-API-KEY", &self.api_key)
            .json(&req)
            .send()
            .await?;

        self.handle_response(response).await
    }

    /// Delete a short URL
    ///
    /// DELETE /api/shortens/{short_code}
    pub async fn delete_shorten(&self, code: &str) -> Result<(), ClientError> {
        let url = format!("{}/api/shortens/{}", self.base_url, code);

        let response = self
            .client
            .delete(&url)
            .header("X-API-KEY", &self.api_key)
            .send()
            .await?;

        match response.status() {
            StatusCode::NO_CONTENT => Ok(()),
            StatusCode::NOT_FOUND => Err(ClientError::NotFound(format!(
                "Short URL '{}' not found",
                code
            ))),
            StatusCode::UNAUTHORIZED => Err(ClientError::Unauthorized),
            status if status.is_client_error() => {
                let error_body = response.json::<ErrorResponse>().await.ok();
                Err(ClientError::BadRequest(
                    error_body
                        .map(|e| e.errinfo)
                        .unwrap_or_else(|| "Bad request".to_string()),
                ))
            }
            status if status.is_server_error() => {
                let error_body = response.json::<ErrorResponse>().await.ok();
                Err(ClientError::ServerError(
                    error_body
                        .map(|e| e.errinfo)
                        .unwrap_or_else(|| "Server error".to_string()),
                ))
            }
            _ => Err(ClientError::ServerError(format!(
                "Unexpected status code: {}",
                response.status()
            ))),
        }
    }

    /// Delete multiple short URLs by IDs
    ///
    /// DELETE /api/shortens?ids=1,2,3
    #[allow(dead_code)]
    pub async fn delete_batch(&self, ids: Vec<i64>) -> Result<(), ClientError> {
        let url = format!("{}/api/shortens", self.base_url);
        let ids_str = ids
            .iter()
            .map(|id| id.to_string())
            .collect::<Vec<_>>()
            .join(",");

        let response = self
            .client
            .delete(&url)
            .header("X-API-KEY", &self.api_key)
            .query(&[("ids", ids_str)])
            .send()
            .await?;

        match response.status() {
            StatusCode::NO_CONTENT => Ok(()),
            StatusCode::UNAUTHORIZED => Err(ClientError::Unauthorized),
            status if status.is_client_error() => {
                let error_body = response.json::<ErrorResponse>().await.ok();
                Err(ClientError::BadRequest(
                    error_body
                        .map(|e| e.errinfo)
                        .unwrap_or_else(|| "Bad request".to_string()),
                ))
            }
            status if status.is_server_error() => {
                let error_body = response.json::<ErrorResponse>().await.ok();
                Err(ClientError::ServerError(
                    error_body
                        .map(|e| e.errinfo)
                        .unwrap_or_else(|| "Server error".to_string()),
                ))
            }
            _ => Err(ClientError::ServerError(format!(
                "Unexpected status code: {}",
                response.status()
            ))),
        }
    }

    /// Handle HTTP response and parse JSON or error
    async fn handle_response<T>(&self, response: reqwest::Response) -> Result<T, ClientError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let status = response.status();

        match status {
            StatusCode::OK | StatusCode::CREATED => {
                let data = response.json::<T>().await.map_err(|e| {
                    ClientError::ServerError(format!("Failed to parse response: {}", e))
                })?;
                Ok(data)
            }
            StatusCode::UNAUTHORIZED => Err(ClientError::Unauthorized),
            StatusCode::NOT_FOUND => {
                let error_body = response.json::<ErrorResponse>().await.ok();
                Err(ClientError::NotFound(
                    error_body
                        .map(|e| e.errinfo)
                        .unwrap_or_else(|| "Resource not found".to_string()),
                ))
            }
            StatusCode::CONFLICT => {
                let error_body = response.json::<ErrorResponse>().await.ok();
                Err(ClientError::Conflict(
                    error_body
                        .map(|e| e.errinfo)
                        .unwrap_or_else(|| "Conflict".to_string()),
                ))
            }
            status if status.is_client_error() => {
                let error_body = response.json::<ErrorResponse>().await.ok();
                Err(ClientError::BadRequest(
                    error_body
                        .map(|e| e.errinfo)
                        .unwrap_or_else(|| "Bad request".to_string()),
                ))
            }
            status if status.is_server_error() => {
                let error_body = response.json::<ErrorResponse>().await.ok();
                Err(ClientError::ServerError(
                    error_body
                        .map(|e| e.errinfo)
                        .unwrap_or_else(|| "Server error".to_string()),
                ))
            }
            _ => {
                let error_body = response.json::<ErrorResponse>().await.ok();
                Err(ClientError::ApiError {
                    code: error_body
                        .as_ref()
                        .map(|e| e.errcode.clone())
                        .unwrap_or_else(|| "UNKNOWN".to_string()),
                    message: error_body
                        .map(|e| e.errinfo)
                        .unwrap_or_else(|| format!("Unexpected status: {}", status)),
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = ApiClient::new("http://localhost:8080".to_string(), "test-key".to_string());
        assert_eq!(client.base_url, "http://localhost:8080");
        assert_eq!(client.api_key, "test-key");
    }

    #[test]
    fn test_client_trims_trailing_slash() {
        let client = ApiClient::new("http://localhost:8080/".to_string(), "test-key".to_string());
        assert_eq!(client.base_url, "http://localhost:8080");
    }

    #[test]
    fn test_create_request_serialization() {
        let req = CreateShortenRequest {
            original_url: "https://example.com".to_string(),
            code: Some("test123".to_string()),
            describe: Some("Test URL".to_string()),
        };

        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("original_url"));
        assert!(json.contains("test123"));
    }

    #[test]
    fn test_create_request_optional_fields() {
        let req = CreateShortenRequest {
            original_url: "https://example.com".to_string(),
            code: None,
            describe: None,
        };

        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("original_url"));
        // Optional fields should not be serialized when None
        assert!(!json.contains("code"));
        assert!(!json.contains("describe"));
    }

    #[test]
    fn test_update_request_serialization() {
        let req = UpdateShortenRequest {
            original_url: Some("https://updated.com".to_string()),
            describe: Some("Updated".to_string()),
            status: Some(1),
        };

        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("updated.com"));
        assert!(json.contains("Updated"));
    }

    #[test]
    fn test_list_params_default() {
        let params = ListParams::default();
        assert!(params.page.is_none());
        assert!(params.page_size.is_none());
        assert!(params.status.is_none());
        assert!(params.sort.is_none());
        assert!(params.order.is_none());
    }
}
