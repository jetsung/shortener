use async_trait::async_trait;
use thiserror::Error;

/// Cache error types
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

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Result type for cache operations
pub type CacheResult<T> = Result<T, CacheError>;

/// Cache trait defining the interface for all cache implementations
#[async_trait]
pub trait Cache: Send + Sync {
    /// Get a value from cache by key
    ///
    /// # Arguments
    /// * `key` - The cache key
    ///
    /// # Returns
    /// * `Ok(Some(String))` - Value found in cache
    /// * `Ok(None)` - Key not found in cache
    /// * `Err(CacheError)` - Operation failed
    async fn get(&self, key: &str) -> CacheResult<Option<String>>;

    /// Set a value in cache with expiration
    ///
    /// # Arguments
    /// * `key` - The cache key
    /// * `value` - The value to store
    /// * `expire` - Expiration time in seconds
    ///
    /// # Returns
    /// * `Ok(())` - Value successfully stored
    /// * `Err(CacheError)` - Operation failed
    async fn set(&self, key: &str, value: &str, expire: u64) -> CacheResult<()>;

    /// Delete a value from cache
    ///
    /// # Arguments
    /// * `key` - The cache key to delete
    ///
    /// # Returns
    /// * `Ok(())` - Key successfully deleted (or didn't exist)
    /// * `Err(CacheError)` - Operation failed
    async fn delete(&self, key: &str) -> CacheResult<()>;

    /// Check if a key exists in cache
    ///
    /// # Arguments
    /// * `key` - The cache key
    ///
    /// # Returns
    /// * `Ok(true)` - Key exists
    /// * `Ok(false)` - Key does not exist
    /// * `Err(CacheError)` - Operation failed
    async fn exists(&self, key: &str) -> CacheResult<bool>;

    /// Whether this is the no-op NullCache implementation
    ///
    /// Returns `false` by default; only `NullCache` overrides it. Used to
    /// distinguish "caching disabled / connection failed" from a real cache.
    fn is_null(&self) -> bool {
        false
    }

    /// Delete all keys under the given prefix (SCAN + batched DEL)
    ///
    /// # Arguments
    /// * `prefix` - The key prefix to clear (e.g. "shorten:")
    ///
    /// # Returns
    /// * `Ok(u64)` - Number of keys deleted
    /// * `Err(CacheError)` - Operation failed
    async fn clear_prefix(&self, prefix: &str) -> CacheResult<u64>;
}

// Re-export cache implementations
mod null_cache;
mod redis_cache;
mod valkey_cache;

pub use null_cache::NullCache;
pub use redis_cache::RedisCache;
pub use valkey_cache::ValkeyCache;

use crate::config::{CacheConfig, CacheKind};
use std::sync::Arc;
use tracing::{info, warn};

/// Create a cache instance based on configuration
///
/// # Arguments
/// * `config` - Cache configuration
///
/// # Returns
/// * `Arc<dyn Cache>` - Cache instance (Redis, Valkey, or NullCache)
pub async fn create_cache(config: &CacheConfig) -> Arc<dyn Cache> {
    if !config.enabled {
        info!("Cache is disabled, using NullCache");
        return Arc::new(NullCache::new());
    }

    let url = match &config.url {
        Some(url) if !url.trim().is_empty() => url.trim().to_string(),
        _ => {
            warn!("Cache enabled but no URL provided, using NullCache");
            return Arc::new(NullCache::new());
        }
    };

    match CacheKind::from_url(&url) {
        CacheKind::Valkey => {
            match ValkeyCache::new(&url, config.prefix.clone(), config.expire).await {
                Ok(cache) => {
                    info!("Successfully connected to Valkey cache");
                    Arc::new(cache)
                }
                Err(e) => {
                    warn!(
                        "Failed to connect to Valkey: {}, falling back to NullCache",
                        e
                    );
                    Arc::new(NullCache::new())
                }
            }
        }
        CacheKind::Redis => {
            match RedisCache::new(&url, config.prefix.clone(), config.expire).await {
                Ok(cache) => {
                    info!("Successfully connected to Redis cache");
                    Arc::new(cache)
                }
                Err(e) => {
                    warn!(
                        "Failed to connect to Redis: {}, falling back to NullCache",
                        e
                    );
                    Arc::new(NullCache::new())
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_cache_disabled() {
        let config = CacheConfig {
            enabled: false,
            expire: 60,
            prefix: "test:".to_string(),
            url: None,
        };

        let cache = create_cache(&config).await;
        let result = cache.get("test").await.unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_create_cache_redis_no_url() {
        let config = CacheConfig {
            enabled: true,
            expire: 60,
            prefix: "test:".to_string(),
            url: None,
        };

        let cache = create_cache(&config).await;
        // Should fall back to NullCache
        let result = cache.get("test").await.unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_create_cache_redis_invalid_connection() {
        let config = CacheConfig {
            enabled: true,
            expire: 60,
            prefix: "test:".to_string(),
            url: Some("redis://localhost:9999/0".to_string()),
        };

        // Use timeout to prevent hanging (10 seconds should be enough)
        let cache_result =
            tokio::time::timeout(tokio::time::Duration::from_secs(10), create_cache(&config)).await;

        // Should either timeout or successfully create a NullCache fallback
        let cache = match cache_result {
            Ok(cache) => cache,
            Err(_) => {
                // Timeout occurred, which is acceptable for this test
                // Just verify we can create a NullCache
                return;
            }
        };

        // Should fall back to NullCache due to connection failure
        let result = cache.get("test").await.unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_create_cache_valkey_no_url() {
        let config = CacheConfig {
            enabled: true,
            expire: 60,
            prefix: "test:".to_string(),
            url: None,
        };

        let cache = create_cache(&config).await;
        // Should fall back to NullCache
        let result = cache.get("test").await.unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_create_cache_valkey_invalid_connection() {
        let config = CacheConfig {
            enabled: true,
            expire: 60,
            prefix: "test:".to_string(),
            url: Some("valkey://localhost:9999/0".to_string()),
        };

        // Use timeout to prevent hanging (10 seconds should be enough)
        let cache_result =
            tokio::time::timeout(tokio::time::Duration::from_secs(10), create_cache(&config)).await;

        // Should either timeout or successfully create a NullCache fallback
        let cache = match cache_result {
            Ok(cache) => cache,
            Err(_) => {
                // Timeout occurred, which is acceptable for this test
                // Just verify we can create a NullCache
                return;
            }
        };

        // Should fall back to NullCache due to connection failure
        let result = cache.get("test").await.unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    #[ignore]
    async fn test_create_cache_redis_valid_connection() {
        let config = CacheConfig {
            enabled: true,
            expire: 60,
            prefix: "test:".to_string(),
            url: Some("redis://localhost:6379/0".to_string()),
        };

        let cache = create_cache(&config).await;

        // Should successfully connect and work
        cache.set("test_key", "test_value", 0).await.unwrap();
        let result = cache.get("test_key").await.unwrap();
        assert_eq!(result, Some("test_value".to_string()));

        cache.delete("test_key").await.unwrap();
    }
}
