use crate::errors::AppError;
use crate::router::AppState;
use axum::{Json, extract::State};
use serde::Serialize;
use tracing::info;

/// Response body for cache refresh
#[derive(Debug, Serialize)]
pub struct CacheRefreshResponse {
    /// Number of stale keys removed before reloading
    pub cleared_keys: u64,
    /// Number of short URLs re-cached from the database
    pub warmed_urls: usize,
}

/// Refresh the URL cache
///
/// Clears all cache keys under the configured prefix, then reloads every
/// short URL from the database into the cache.
///
/// POST /api/cache/refresh
pub async fn refresh_cache(
    State(state): State<AppState>,
) -> Result<Json<CacheRefreshResponse>, AppError> {
    info!("Refreshing cache (prefix: {})", state.config.cache.prefix);

    let (cleared_keys, warmed_urls) = state
        .shorten_service
        .refresh_cache(&state.config.cache.prefix)
        .await?;

    info!(
        "Cache refreshed: {} keys cleared, {} URLs cached",
        cleared_keys, warmed_urls
    );

    Ok(Json(CacheRefreshResponse {
        cleared_keys,
        warmed_urls,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_refresh_response_serialization() {
        let resp = CacheRefreshResponse {
            cleared_keys: 4,
            warmed_urls: 3,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"cleared_keys\":4"));
        assert!(json.contains("\"warmed_urls\":3"));
    }
}
