use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use shortener_server::cache::{Cache, NullCache};
use std::hint::black_box;
use tokio::runtime::Runtime;

/// Benchmark cache set operations
fn benchmark_cache_set(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let cache = NullCache::new();

    c.bench_function("cache_set_small_value", |b| {
        b.iter(|| {
            rt.block_on(async {
                cache.set("test_key", "small_value", 3600).await.unwrap();
                black_box(())
            })
        });
    });

    c.bench_function("cache_set_medium_value", |b| {
        let medium_value = "x".repeat(1024); // 1KB
        b.iter(|| {
            rt.block_on(async {
                cache.set("test_key", &medium_value, 3600).await.unwrap();
                black_box(())
            })
        });
    });

    c.bench_function("cache_set_large_value", |b| {
        let large_value = "x".repeat(10240); // 10KB
        b.iter(|| {
            rt.block_on(async {
                cache.set("test_key", &large_value, 3600).await.unwrap();
                black_box(())
            })
        });
    });
}

/// Benchmark cache get operations
fn benchmark_cache_get(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let cache = NullCache::new();

    c.bench_function("cache_get_nonexistent", |b| {
        b.iter(|| rt.block_on(async { black_box(cache.get("nonexistent").await.unwrap()) }));
    });
}

/// Benchmark cache delete operations
fn benchmark_cache_delete(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let cache = NullCache::new();

    c.bench_function("cache_delete", |b| {
        b.iter(|| {
            rt.block_on(async {
                cache.delete("test_key").await.unwrap();
                black_box(())
            })
        });
    });
}

/// Benchmark cache exists operations
fn benchmark_cache_exists(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let cache = NullCache::new();

    c.bench_function("cache_exists", |b| {
        b.iter(|| rt.block_on(async { black_box(cache.exists("test_key").await.unwrap()) }));
    });
}

/// Benchmark cache operations with different key patterns
fn benchmark_cache_key_patterns(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let cache = NullCache::new();

    let mut group = c.benchmark_group("cache_key_patterns");

    let key_patterns = [
        ("short", "abc"),
        ("medium", "url:abc123def456"),
        ("long", "shortener:production:url:abc123def456ghi789"),
        ("with_prefix", "shorten:url:test123"),
    ];

    for (name, key) in key_patterns.iter() {
        group.bench_with_input(BenchmarkId::from_parameter(name), key, |b, key| {
            b.iter(|| {
                rt.block_on(async {
                    cache.set(key, "test_value", 3600).await.unwrap();
                    black_box(())
                })
            });
        });
    }

    group.finish();
}

/// Benchmark concurrent cache operations
fn benchmark_cache_concurrent(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let cache = std::sync::Arc::new(NullCache::new());

    c.bench_function("cache_concurrent_10", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut handles = vec![];
                for i in 0..10 {
                    let cache_clone = cache.clone();
                    let handle = tokio::spawn(async move {
                        cache_clone
                            .set(&format!("key{}", i), "value", 3600)
                            .await
                            .unwrap();
                    });
                    handles.push(handle);
                }
                for handle in handles {
                    handle.await.unwrap();
                }
            })
        });
    });

    c.bench_function("cache_concurrent_50", |b| {
        b.iter(|| {
            rt.block_on(async {
                let mut handles = vec![];
                for i in 0..50 {
                    let cache_clone = cache.clone();
                    let handle = tokio::spawn(async move {
                        cache_clone
                            .set(&format!("key{}", i), "value", 3600)
                            .await
                            .unwrap();
                    });
                    handles.push(handle);
                }
                for handle in handles {
                    handle.await.unwrap();
                }
            })
        });
    });
}

/// Benchmark cache hit rate simulation
fn benchmark_cache_hit_rate(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let cache = NullCache::new();

    let mut group = c.benchmark_group("cache_hit_rate");

    // Simulate different cache hit rates
    for hit_rate in [0, 50, 80, 95, 100].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}%", hit_rate)),
            hit_rate,
            |b, &_hit_rate| {
                b.iter(|| {
                    rt.block_on(async {
                        // In NullCache, all operations are no-ops
                        // This simulates the overhead of cache operations
                        for i in 0..100 {
                            let key = format!("key{}", i);
                            let _ = cache.get(&key).await;
                        }
                    })
                });
            },
        );
    }

    group.finish();
}

/// Benchmark serialization overhead for cache values
fn benchmark_cache_serialization(c: &mut Criterion) {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct UrlModel {
        id: i64,
        code: String,
        original_url: String,
        describe: Option<String>,
        status: i32,
    }

    let url = UrlModel {
        id: 1,
        code: "abc123".to_string(),
        original_url: "https://example.com".to_string(),
        describe: Some("Test URL".to_string()),
        status: 1,
    };

    c.bench_function("cache_serialize_url", |b| {
        b.iter(|| black_box(serde_json::to_string(&url).unwrap()));
    });

    let serialized = serde_json::to_string(&url).unwrap();

    c.bench_function("cache_deserialize_url", |b| {
        b.iter(|| black_box(serde_json::from_str::<UrlModel>(&serialized).unwrap()));
    });
}

criterion_group!(
    benches,
    benchmark_cache_set,
    benchmark_cache_get,
    benchmark_cache_delete,
    benchmark_cache_exists,
    benchmark_cache_key_patterns,
    benchmark_cache_concurrent,
    benchmark_cache_hit_rate,
    benchmark_cache_serialization
);
criterion_main!(benches);
