use super::{Cache, CacheError, CacheResult};
use async_trait::async_trait;
use redis::{AsyncCommands, Client, aio::ConnectionManager};
use tracing::{debug, error, warn};

/// Redis cache implementation
#[derive(Clone)]
pub struct RedisCache {
    manager: ConnectionManager,
    prefix: String,
    expire: u64,
}

impl RedisCache {
    /// Create a new Redis cache instance
    ///
    /// # Arguments
    /// * `url` - Redis connection URL (e.g., "redis://localhost:6379/0")
    /// * `prefix` - Key prefix for all cache keys
    /// * `expire` - Default expiration time in seconds
    ///
    /// # Returns
    /// * `Ok(RedisCache)` - Successfully connected to Redis
    /// * `Err(CacheError)` - Failed to connect
    pub async fn new(url: &str, prefix: String, expire: u64) -> CacheResult<Self> {
        debug!("Connecting to Redis at {}", url);

        let client = Client::open(url).map_err(|e| {
            error!("Failed to create Redis client: {}", e);
            CacheError::Connection(format!("Failed to create Redis client: {}", e))
        })?;

        let manager = ConnectionManager::new(client).await.map_err(|e| {
            error!("Failed to connect to Redis: {}", e);
            CacheError::Connection(format!("Failed to connect to Redis: {}", e))
        })?;

        debug!("Successfully connected to Redis");

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
impl Cache for RedisCache {
    async fn get(&self, key: &str) -> CacheResult<Option<String>> {
        let full_key = self.build_key(key);
        debug!("Getting cache key: {}", full_key);

        let mut conn = self.manager.clone();
        let result: Option<String> = conn.get(&full_key).await.map_err(|e| {
            warn!("Failed to get cache key {}: {}", full_key, e);
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
            "Setting cache key: {} with expiration: {}s",
            full_key, expire_seconds
        );

        let mut conn = self.manager.clone();

        // Use SETEX to set value with expiration atomically
        let _: () = conn
            .set_ex(&full_key, value, expire_seconds)
            .await
            .map_err(|e| {
                error!("Failed to set cache key {}: {}", full_key, e);
                CacheError::Operation(format!("Failed to set key: {}", e))
            })?;

        debug!("Successfully set cache key: {}", full_key);
        Ok(())
    }

    async fn delete(&self, key: &str) -> CacheResult<()> {
        let full_key = self.build_key(key);
        debug!("Deleting cache key: {}", full_key);

        let mut conn = self.manager.clone();
        let _: () = conn.del(&full_key).await.map_err(|e| {
            warn!("Failed to delete cache key {}: {}", full_key, e);
            CacheError::Operation(format!("Failed to delete key: {}", e))
        })?;

        debug!("Successfully deleted cache key: {}", full_key);
        Ok(())
    }

    async fn exists(&self, key: &str) -> CacheResult<bool> {
        let full_key = self.build_key(key);
        debug!("Checking existence of cache key: {}", full_key);

        let mut conn = self.manager.clone();
        let exists: bool = conn.exists(&full_key).await.map_err(|e| {
            warn!("Failed to check cache key {}: {}", full_key, e);
            CacheError::Operation(format!("Failed to check key existence: {}", e))
        })?;

        debug!("Cache key {} exists: {}", full_key, exists);
        Ok(exists)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require a running Redis instance
    // They are integration tests and should be run with `cargo test --ignored`

    #[tokio::test]
    #[ignore]
    async fn test_redis_set_and_get() {
        let cache = RedisCache::new("redis://localhost:6379/0", "test:".to_string(), 60)
            .await
            .expect("Failed to connect to Redis");

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
    async fn test_redis_get_nonexistent() {
        let cache = RedisCache::new("redis://localhost:6379/0", "test:".to_string(), 60)
            .await
            .expect("Failed to connect to Redis");

        let result = cache
            .get("nonexistent_key")
            .await
            .expect("Failed to get value");
        assert_eq!(result, None);
    }

    #[tokio::test]
    #[ignore]
    async fn test_redis_delete() {
        let cache = RedisCache::new("redis://localhost:6379/0", "test:".to_string(), 60)
            .await
            .expect("Failed to connect to Redis");

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
    async fn test_redis_exists() {
        let cache = RedisCache::new("redis://localhost:6379/0", "test:".to_string(), 60)
            .await
            .expect("Failed to connect to Redis");

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
    async fn test_redis_expiration() {
        let cache = RedisCache::new("redis://localhost:6379/0", "test:".to_string(), 60)
            .await
            .expect("Failed to connect to Redis");

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
    async fn test_redis_key_prefix() {
        let cache = RedisCache::new("redis://localhost:6379/0", "myprefix:".to_string(), 60)
            .await
            .expect("Failed to connect to Redis");

        cache
            .set("test", "value", 0)
            .await
            .expect("Failed to set value");

        // Verify the key has the prefix by checking with raw Redis client
        let client = Client::open("redis://localhost:6379/0").unwrap();
        let mut conn = client.get_connection_manager().await.unwrap();
        let exists: bool = conn.exists("myprefix:test").await.unwrap();
        assert!(exists);

        // Clean up
        cache.delete("test").await.expect("Failed to delete key");
    }
}
