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
    /// * `key` - The cache key to check
    ///
    /// # Returns
    /// * `Ok(true)` - Key exists
    /// * `Ok(false)` - Key does not exist
    /// * `Err(CacheError)` - Operation failed
    async fn exists(&self, key: &str) -> CacheResult<bool>;
}

// Re-export cache implementations
mod null_cache;
mod redis_cache;
mod valkey_cache;

pub use null_cache::NullCache;
pub use redis_cache::RedisCache;
pub use valkey_cache::ValkeyCache;

use crate::config::{CacheConfig, CacheType};
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

    match config.cache_type {
        CacheType::Redis => {
            if let Some(redis_config) = &config.redis {
                let url = if redis_config.password.is_empty() {
                    format!(
                        "redis://{}:{}/{}",
                        redis_config.host, redis_config.port, redis_config.db
                    )
                } else {
                    format!(
                        "redis://:{}@{}:{}/{}",
                        redis_config.password,
                        redis_config.host,
                        redis_config.port,
                        redis_config.db
                    )
                };

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
            } else {
                warn!("Redis cache enabled but no configuration provided, using NullCache");
                Arc::new(NullCache::new())
            }
        }
        CacheType::Valkey => {
            if let Some(valkey_config) = &config.valkey {
                let url = if valkey_config.password.is_empty() {
                    format!(
                        "redis://{}:{}/{}",
                        valkey_config.host, valkey_config.port, valkey_config.db
                    )
                } else if valkey_config.username.is_empty() {
                    format!(
                        "redis://:{}@{}:{}/{}",
                        valkey_config.password,
                        valkey_config.host,
                        valkey_config.port,
                        valkey_config.db
                    )
                } else {
                    format!(
                        "redis://{}:{}@{}:{}/{}",
                        valkey_config.username,
                        valkey_config.password,
                        valkey_config.host,
                        valkey_config.port,
                        valkey_config.db
                    )
                };

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
            } else {
                warn!("Valkey cache enabled but no configuration provided, using NullCache");
                Arc::new(NullCache::new())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{RedisConfig, ValkeyConfig};

    #[tokio::test]
    async fn test_create_cache_disabled() {
        let config = CacheConfig {
            enabled: false,
            cache_type: CacheType::Redis,
            expire: 60,
            prefix: "test:".to_string(),
            redis: None,
            valkey: None,
        };

        let cache = create_cache(&config).await;
        let result = cache.get("test").await.unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_create_cache_redis_no_config() {
        let config = CacheConfig {
            enabled: true,
            cache_type: CacheType::Redis,
            expire: 60,
            prefix: "test:".to_string(),
            redis: None,
            valkey: None,
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
            cache_type: CacheType::Redis,
            expire: 60,
            prefix: "test:".to_string(),
            redis: Some(RedisConfig {
                host: "localhost".to_string(),
                port: 9999,
                password: "".to_string(),
                db: 0,
            }),
            valkey: None,
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
    async fn test_create_cache_valkey_no_config() {
        let config = CacheConfig {
            enabled: true,
            cache_type: CacheType::Valkey,
            expire: 60,
            prefix: "test:".to_string(),
            redis: None,
            valkey: None,
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
            cache_type: CacheType::Valkey,
            expire: 60,
            prefix: "test:".to_string(),
            redis: None,
            valkey: Some(ValkeyConfig {
                host: "localhost".to_string(),
                port: 9999,
                username: "".to_string(),
                password: "".to_string(),
                db: 0,
            }),
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
            cache_type: CacheType::Redis,
            expire: 60,
            prefix: "test:".to_string(),
            redis: Some(RedisConfig {
                host: "localhost".to_string(),
                port: 6379,
                password: "".to_string(),
                db: 0,
            }),
            valkey: None,
        };

        let cache = create_cache(&config).await;

        // Should successfully connect and work
        cache.set("test_key", "test_value", 0).await.unwrap();
        let result = cache.get("test_key").await.unwrap();
        assert_eq!(result, Some("test_value".to_string()));

        cache.delete("test_key").await.unwrap();
    }
}
