use shortener_server::cache::{Cache, NullCache, RedisCache, ValkeyCache};

/// Test NullCache behavior
#[tokio::test]
async fn test_null_cache_operations() {
    let cache = NullCache::new();

    // Test get - should always return None
    let result = cache.get("test_key").await.unwrap();
    assert_eq!(result, None);

    // Test set - should succeed but do nothing
    cache.set("test_key", "test_value", 60).await.unwrap();

    // Test get after set - should still return None
    let result = cache.get("test_key").await.unwrap();
    assert_eq!(result, None);

    // Test exists - should always return false
    let exists = cache.exists("test_key").await.unwrap();
    assert!(!exists);

    // Test delete - should succeed
    cache.delete("test_key").await.unwrap();
}

/// Test NullCache with multiple operations
#[tokio::test]
async fn test_null_cache_multiple_operations() {
    let cache = NullCache::new();

    // Perform multiple operations
    for i in 0..10 {
        let key = format!("key_{}", i);
        let value = format!("value_{}", i);

        cache.set(&key, &value, 60).await.unwrap();
        let result = cache.get(&key).await.unwrap();
        assert_eq!(result, None);
    }
}

/// Test cache degradation scenario
/// This test simulates what happens when cache is unavailable
#[tokio::test]
async fn test_cache_degradation_with_null_cache() {
    // When cache is unavailable, we can fall back to NullCache
    let cache: Box<dyn Cache> = Box::new(NullCache::new());

    // Application should continue to work
    let result = cache.get("some_key").await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), None);

    // Set operations should not fail
    let result = cache.set("some_key", "some_value", 60).await;
    assert!(result.is_ok());
}

/// Integration test for Redis cache
/// Requires a running Redis instance at localhost:6379
#[tokio::test]
#[ignore]
async fn test_redis_cache_integration() {
    let cache = RedisCache::new("redis://localhost:6379/0", "test:".to_string(), 60)
        .await
        .expect("Failed to connect to Redis");

    // Test basic set and get
    cache
        .set("integration_key", "integration_value", 0)
        .await
        .unwrap();

    let result = cache.get("integration_key").await.unwrap();
    assert_eq!(result, Some("integration_value".to_string()));

    // Test exists
    let exists = cache.exists("integration_key").await.unwrap();
    assert!(exists);

    // Test delete
    cache.delete("integration_key").await.unwrap();
    let result = cache.get("integration_key").await.unwrap();
    assert_eq!(result, None);
}

/// Integration test for Valkey cache
/// Requires a running Valkey/Redis instance at localhost:6379
#[tokio::test]
#[ignore]
async fn test_valkey_cache_integration() {
    let cache = ValkeyCache::new("redis://localhost:6379/1", "valkey:".to_string(), 60)
        .await
        .expect("Failed to connect to Valkey");

    // Test basic set and get
    cache
        .set("integration_key", "integration_value", 0)
        .await
        .unwrap();

    let result = cache.get("integration_key").await.unwrap();
    assert_eq!(result, Some("integration_value".to_string()));

    // Test exists
    let exists = cache.exists("integration_key").await.unwrap();
    assert!(exists);

    // Test delete
    cache.delete("integration_key").await.unwrap();
    let result = cache.get("integration_key").await.unwrap();
    assert_eq!(result, None);
}

/// Test Redis cache with custom expiration
#[tokio::test]
#[ignore]
async fn test_redis_cache_custom_expiration() {
    let cache = RedisCache::new("redis://localhost:6379/0", "test:".to_string(), 60)
        .await
        .expect("Failed to connect to Redis");

    // Set with 2 second expiration
    cache.set("expire_test", "expire_value", 2).await.unwrap();

    // Should exist immediately
    let result = cache.get("expire_test").await.unwrap();
    assert_eq!(result, Some("expire_value".to_string()));

    // Wait for expiration
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

    // Should be gone
    let result = cache.get("expire_test").await.unwrap();
    assert_eq!(result, None);
}

/// Test Redis cache with default expiration
#[tokio::test]
#[ignore]
async fn test_redis_cache_default_expiration() {
    let cache = RedisCache::new("redis://localhost:6379/0", "test:".to_string(), 1)
        .await
        .expect("Failed to connect to Redis");

    // Set with 0 expiration (should use default of 1 second)
    cache
        .set("default_expire", "default_value", 0)
        .await
        .unwrap();

    // Should exist immediately
    let result = cache.get("default_expire").await.unwrap();
    assert_eq!(result, Some("default_value".to_string()));

    // Wait for default expiration
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Should be gone
    let result = cache.get("default_expire").await.unwrap();
    assert_eq!(result, None);
}

/// Test cache with key prefix
#[tokio::test]
#[ignore]
async fn test_redis_cache_key_prefix() {
    let cache1 = RedisCache::new("redis://localhost:6379/0", "app1:".to_string(), 60)
        .await
        .expect("Failed to connect to Redis");

    let cache2 = RedisCache::new("redis://localhost:6379/0", "app2:".to_string(), 60)
        .await
        .expect("Failed to connect to Redis");

    // Set same key in both caches
    cache1.set("shared_key", "value1", 0).await.unwrap();
    cache2.set("shared_key", "value2", 0).await.unwrap();

    // Should get different values due to different prefixes
    let result1 = cache1.get("shared_key").await.unwrap();
    let result2 = cache2.get("shared_key").await.unwrap();

    assert_eq!(result1, Some("value1".to_string()));
    assert_eq!(result2, Some("value2".to_string()));

    // Clean up
    cache1.delete("shared_key").await.unwrap();
    cache2.delete("shared_key").await.unwrap();
}

/// Test cache degradation: fallback to NullCache when Redis is unavailable
#[tokio::test]
async fn test_cache_degradation_fallback() {
    // Try to connect to non-existent Redis with timeout
    let redis_result = tokio::time::timeout(
        tokio::time::Duration::from_secs(2),
        RedisCache::new("redis://localhost:9999/0", "test:".to_string(), 60),
    )
    .await;

    // Should timeout or fail to connect
    assert!(redis_result.is_err() || redis_result.unwrap().is_err());

    // Fallback to NullCache
    let cache: Box<dyn Cache> = Box::new(NullCache::new());

    // Application continues to work
    let result = cache.get("test_key").await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), None);
}

/// Test concurrent cache operations
#[tokio::test]
#[ignore]
async fn test_redis_cache_concurrent_operations() {
    let cache = RedisCache::new("redis://localhost:6379/0", "concurrent:".to_string(), 60)
        .await
        .expect("Failed to connect to Redis");

    // Spawn multiple concurrent operations
    let mut handles = vec![];

    for i in 0..10 {
        let cache_clone = cache.clone();
        let handle = tokio::spawn(async move {
            let key = format!("key_{}", i);
            let value = format!("value_{}", i);

            // Set value
            cache_clone.set(&key, &value, 0).await.unwrap();

            // Get value
            let result = cache_clone.get(&key).await.unwrap();
            assert_eq!(result, Some(value.clone()));

            // Delete value
            cache_clone.delete(&key).await.unwrap();
        });
        handles.push(handle);
    }

    // Wait for all operations to complete
    for handle in handles {
        handle.await.unwrap();
    }
}

/// Test cache with large values
#[tokio::test]
#[ignore]
async fn test_redis_cache_large_values() {
    let cache = RedisCache::new("redis://localhost:6379/0", "large:".to_string(), 60)
        .await
        .expect("Failed to connect to Redis");

    // Create a large value (1MB)
    let large_value = "x".repeat(1024 * 1024);

    cache.set("large_key", &large_value, 0).await.unwrap();

    let result = cache.get("large_key").await.unwrap();
    assert_eq!(result, Some(large_value));

    cache.delete("large_key").await.unwrap();
}

/// Test cache with special characters in keys and values
#[tokio::test]
#[ignore]
async fn test_redis_cache_special_characters() {
    let cache = RedisCache::new("redis://localhost:6379/0", "special:".to_string(), 60)
        .await
        .expect("Failed to connect to Redis");

    let special_key = "key:with:colons:and-dashes_and_underscores";
    let special_value = r#"{"json": "value", "with": ["special", "chars"]}"#;

    cache.set(special_key, special_value, 0).await.unwrap();

    let result = cache.get(special_key).await.unwrap();
    assert_eq!(result, Some(special_value.to_string()));

    cache.delete(special_key).await.unwrap();
}

/// Test that Cache trait is object-safe (can be used as trait object)
#[tokio::test]
async fn test_cache_trait_object_safety() {
    let null_cache: Box<dyn Cache> = Box::new(NullCache::new());

    // Should be able to call methods through trait object
    let result = null_cache.get("test").await;
    assert!(result.is_ok());

    let result = null_cache.set("test", "value", 60).await;
    assert!(result.is_ok());

    let result = null_cache.delete("test").await;
    assert!(result.is_ok());

    let result = null_cache.exists("test").await;
    assert!(result.is_ok());
}

/// Test cache factory pattern
#[tokio::test]
async fn test_cache_factory_pattern() {
    // Simulate creating cache based on configuration
    fn create_cache(enabled: bool) -> Box<dyn Cache> {
        if enabled {
            // In real scenario, would create RedisCache or ValkeyCache
            // For this test, we use NullCache
            Box::new(NullCache::new())
        } else {
            Box::new(NullCache::new())
        }
    }

    let cache_enabled = create_cache(true);
    let cache_disabled = create_cache(false);

    // Both should work the same way through the trait
    assert!(cache_enabled.get("test").await.is_ok());
    assert!(cache_disabled.get("test").await.is_ok());
}
