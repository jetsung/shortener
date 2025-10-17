use super::{Cache, CacheError, CacheResult};
use async_trait::async_trait;
use redis::{AsyncCommands, Client, aio::ConnectionManager};
use tracing::{debug, error, warn};

/// Valkey cache implementation
///
/// Valkey is a Redis-compatible key-value store, so we can reuse the Redis client.
/// This implementation is essentially identical to RedisCache but provides a
/// separate type for clarity and potential future Valkey-specific features.
#[derive(Clone)]
pub struct ValkeyCache {
    manager: ConnectionManager,
    prefix: String,
    expire: u64,
}

impl ValkeyCache {
    /// Create a new Valkey cache instance
    ///
    /// # Arguments
    /// * `url` - Valkey connection URL (e.g., "redis://localhost:6379/0")
    /// * `prefix` - Key prefix for all cache keys
    /// * `expire` - Default expiration time in seconds
    ///
    /// # Returns
    /// * `Ok(ValkeyCache)` - Successfully connected to Valkey
    /// * `Err(CacheError)` - Failed to connect
    pub async fn new(url: &str, prefix: String, expire: u64) -> CacheResult<Self> {
        debug!("Connecting to Valkey at {}", url);

        let client = Client::open(url).map_err(|e| {
            error!("Failed to create Valkey client: {}", e);
            CacheError::Connection(format!("Failed to create Valkey client: {}", e))
        })?;

        let manager = ConnectionManager::new(client).await.map_err(|e| {
            error!("Failed to connect to Valkey: {}", e);
            CacheError::Connection(format!("Failed to connect to Valkey: {}", e))
        })?;

        debug!("Successfully connected to Valkey");

        Ok(Self {
            manager,
            prefix,
            expire,
        })
    }

    /// Build the full cache key with prefix
    fn build_key(&self, key: &str) -> String {
        format!("{}{}", self.prefix, key)
    }
}

#[async_trait]
impl Cache for ValkeyCache {
    async fn get(&self, key: &str) -> CacheResult<Option<String>> {
        let full_key = self.build_key(key);
        debug!("Getting cache key from Valkey: {}", full_key);

        let mut conn = self.manager.clone();
        let result: Option<String> = conn.get(&full_key).await.map_err(|e| {
            warn!("Failed to get cache key {} from Valkey: {}", full_key, e);
            CacheError::Operation(format!("Failed to get key: {}", e))
        })?;

        if result.is_some() {
            debug!("Cache hit for key: {}", full_key);
        } else {
            debug!("Cache miss for key: {}", full_key);
        }

        Ok(result)
    }

    async fn set(&self, key: &str, value: &str, expire: u64) -> CacheResult<()> {
        let full_key = self.build_key(key);
        let expire_seconds = if expire > 0 { expire } else { self.expire };

        debug!(
            "Setting cache key in Valkey: {} with expiration: {}s",
            full_key, expire_seconds
        );

        let mut conn = self.manager.clone();

        // Use SETEX to set value with expiration atomically
        let _: () = conn
            .set_ex(&full_key, value, expire_seconds)
            .await
            .map_err(|e| {
                error!("Failed to set cache key {} in Valkey: {}", full_key, e);
                CacheError::Operation(format!("Failed to set key: {}", e))
            })?;

        debug!("Successfully set cache key in Valkey: {}", full_key);
        Ok(())
    }

    async fn delete(&self, key: &str) -> CacheResult<()> {
        let full_key = self.build_key(key);
        debug!("Deleting cache key from Valkey: {}", full_key);

        let mut conn = self.manager.clone();
        let _: () = conn.del(&full_key).await.map_err(|e| {
            warn!("Failed to delete cache key {} from Valkey: {}", full_key, e);
            CacheError::Operation(format!("Failed to delete key: {}", e))
        })?;

        debug!("Successfully deleted cache key from Valkey: {}", full_key);
        Ok(())
    }

    async fn exists(&self, key: &str) -> CacheResult<bool> {
        let full_key = self.build_key(key);
        debug!("Checking existence of cache key in Valkey: {}", full_key);

        let mut conn = self.manager.clone();
        let exists: bool = conn.exists(&full_key).await.map_err(|e| {
            warn!("Failed to check cache key {} in Valkey: {}", full_key, e);
            CacheError::Operation(format!("Failed to check key existence: {}", e))
        })?;

        debug!("Cache key {} exists in Valkey: {}", full_key, exists);
        Ok(exists)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require a running Valkey instance
    // They are integration tests and should be run with `cargo test --ignored`
    // Since Valkey is Redis-compatible, you can use Redis for testing

    #[tokio::test]
    #[ignore]
    async fn test_valkey_set_and_get() {
        let cache = ValkeyCache::new("redis://localhost:6379/1", "valkey:".to_string(), 60)
            .await
            .expect("Failed to connect to Valkey");

        // Set a value
        cache
            .set("key1", "value1", 0)
            .await
            .expect("Failed to set value");

        // Get the value
        let result = cache.get("key1").await.expect("Failed to get value");
        assert_eq!(result, Some("value1".to_string()));
    }

    #[tokio::test]
    #[ignore]
    async fn test_valkey_get_nonexistent() {
        let cache = ValkeyCache::new("redis://localhost:6379/1", "valkey:".to_string(), 60)
            .await
            .expect("Failed to connect to Valkey");

        let result = cache
            .get("nonexistent_key")
            .await
            .expect("Failed to get value");
        assert_eq!(result, None);
    }

    #[tokio::test]
    #[ignore]
    async fn test_valkey_delete() {
        let cache = ValkeyCache::new("redis://localhost:6379/1", "valkey:".to_string(), 60)
            .await
            .expect("Failed to connect to Valkey");

        // Set a value
        cache
            .set("key2", "value2", 0)
            .await
            .expect("Failed to set value");

        // Verify it exists
        let exists = cache.exists("key2").await.expect("Failed to check exists");
        assert!(exists);

        // Delete it
        cache.delete("key2").await.expect("Failed to delete key");

        // Verify it's gone
        let exists = cache.exists("key2").await.expect("Failed to check exists");
        assert!(!exists);
    }

    #[tokio::test]
    #[ignore]
    async fn test_valkey_exists() {
        let cache = ValkeyCache::new("redis://localhost:6379/1", "valkey:".to_string(), 60)
            .await
            .expect("Failed to connect to Valkey");

        // Check non-existent key
        let exists = cache
            .exists("nonexistent")
            .await
            .expect("Failed to check exists");
        assert!(!exists);

        // Set a value
        cache
            .set("key3", "value3", 0)
            .await
            .expect("Failed to set value");

        // Check it exists
        let exists = cache.exists("key3").await.expect("Failed to check exists");
        assert!(exists);
    }

    #[tokio::test]
    #[ignore]
    async fn test_valkey_expiration() {
        let cache = ValkeyCache::new("redis://localhost:6379/1", "valkey:".to_string(), 60)
            .await
            .expect("Failed to connect to Valkey");

        // Set a value with 1 second expiration
        cache
            .set("expire_key", "expire_value", 1)
            .await
            .expect("Failed to set value");

        // Verify it exists
        let result = cache.get("expire_key").await.expect("Failed to get value");
        assert_eq!(result, Some("expire_value".to_string()));

        // Wait for expiration
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Verify it's gone
        let result = cache.get("expire_key").await.expect("Failed to get value");
        assert_eq!(result, None);
    }

    #[tokio::test]
    #[ignore]
    async fn test_valkey_key_prefix() {
        let cache = ValkeyCache::new("redis://localhost:6379/1", "vkprefix:".to_string(), 60)
            .await
            .expect("Failed to connect to Valkey");

        cache
            .set("test", "value", 0)
            .await
            .expect("Failed to set value");

        // Verify the key has the prefix by checking with raw Redis client
        let client = Client::open("redis://localhost:6379/1").unwrap();
        let mut conn = client.get_connection_manager().await.unwrap();
        let exists: bool = conn.exists("vkprefix:test").await.unwrap();
        assert!(exists);

        // Clean up
        cache.delete("test").await.expect("Failed to delete key");
    }
}
