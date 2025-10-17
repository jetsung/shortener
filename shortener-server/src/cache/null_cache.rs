use super::{Cache, CacheResult};
use async_trait::async_trait;
use tracing::debug;

/// Null cache implementation (no-op)
///
/// This cache implementation does nothing and is used when caching is disabled.
/// All operations succeed immediately without actually storing or retrieving data.
/// This allows the application to use the same Cache trait interface regardless
/// of whether caching is enabled or not.
#[derive(Debug, Clone, Default)]
pub struct NullCache;

impl NullCache {
    /// Create a new NullCache instance
    pub fn new() -> Self {
        debug!("Using NullCache (caching disabled)");
        Self
    }
}

#[async_trait]
impl Cache for NullCache {
    /// Always returns None (cache miss)
    async fn get(&self, key: &str) -> CacheResult<Option<String>> {
        debug!("NullCache: get({}) -> None", key);
        Ok(None)
    }

    /// Does nothing, always succeeds
    async fn set(&self, key: &str, _value: &str, _expire: u64) -> CacheResult<()> {
        debug!("NullCache: set({}) -> no-op", key);
        Ok(())
    }

    /// Does nothing, always succeeds
    async fn delete(&self, key: &str) -> CacheResult<()> {
        debug!("NullCache: delete({}) -> no-op", key);
        Ok(())
    }

    /// Always returns false (key doesn't exist)
    async fn exists(&self, key: &str) -> CacheResult<bool> {
        debug!("NullCache: exists({}) -> false", key);
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_null_cache_get() {
        let cache = NullCache::new();
        let result = cache.get("any_key").await.unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_null_cache_set() {
        let cache = NullCache::new();
        let result = cache.set("key", "value", 60).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_null_cache_delete() {
        let cache = NullCache::new();
        let result = cache.delete("key").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_null_cache_exists() {
        let cache = NullCache::new();
        let result = cache.exists("key").await.unwrap();
        assert!(!result);
    }

    #[tokio::test]
    async fn test_null_cache_set_then_get() {
        let cache = NullCache::new();

        // Set a value
        cache.set("key", "value", 60).await.unwrap();

        // Try to get it - should still return None
        let result = cache.get("key").await.unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_null_cache_default() {
        let cache = NullCache;
        let result = cache.get("key").await.unwrap();
        assert_eq!(result, None);
    }
}
