use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use shortener_server::cache::NullCache;
use shortener_server::config::{Config, DatabaseConfig, DatabaseType, SqliteConfig};
use shortener_server::db::DbFactory;
use shortener_server::repositories::url_repository::UrlRepositoryImpl;
use shortener_server::services::{CreateShortenRequest, ShortenService, UpdateShortenRequest};
use std::hint::black_box;
use std::sync::Arc;
use tokio::runtime::Runtime;

/// Setup test service with in-memory database
async fn setup_test_service() -> ShortenService {
    let config = Config {
        server: shortener_server::config::ServerConfig {
            address: ":8080".to_string(),
            trusted_platform: None,
            site_url: "http://localhost:8080".to_string(),
            api_key: "test-key".to_string(),
        },
        shortener: shortener_server::config::ShortenerConfig {
            code_length: 6,
            code_charset: "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"
                .to_string(),
        },
        admin: shortener_server::config::AdminConfig {
            username: "admin".to_string(),
            password: "admin123".to_string(),
        },
        database: DatabaseConfig {
            db_type: DatabaseType::Sqlite,
            log_level: 0,
            sqlite: Some(SqliteConfig {
                path: ":memory:".to_string(),
            }),
            postgres: None,
            mysql: None,
        },
        cache: shortener_server::config::CacheConfig {
            enabled: false,
            cache_type: shortener_server::config::CacheType::Redis,
            expire: 3600,
            prefix: "shorten:".to_string(),
            redis: None,
            valkey: None,
        },
        geoip: shortener_server::config::GeoIpConfig {
            enabled: false,
            geoip_type: shortener_server::config::GeoIpType::Ip2region,
            ip2region: None,
        },
        logging: shortener_server::logging::LoggingConfig::default(),
    };

    let db = DbFactory::create_connection(&config).await.unwrap();
    DbFactory::run_migrations(&db).await.unwrap();

    let url_repo = Arc::new(UrlRepositoryImpl::new(db));
    let cache = Arc::new(NullCache::new());

    ShortenService::new(
        url_repo,
        cache,
        config.shortener.clone(),
        config.server.site_url.clone(),
    )
}

/// Benchmark service create operations
fn benchmark_service_create(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let service = rt.block_on(setup_test_service());

    let mut counter = 0;

    c.bench_function("service_create_auto_code", |b| {
        b.iter(|| {
            counter += 1;
            let req = CreateShortenRequest {
                original_url: format!("https://example{}.com", counter),
                code: None,
                describe: Some("Benchmark URL".to_string()),
            };
            rt.block_on(async { black_box(service.create_shorten(req).await.unwrap()) })
        });
    });

    let mut custom_counter = 0;

    c.bench_function("service_create_custom_code", |b| {
        b.iter(|| {
            custom_counter += 1;
            let req = CreateShortenRequest {
                original_url: "https://example.com".to_string(),
                code: Some(format!("custom{}", custom_counter)),
                describe: Some("Benchmark URL".to_string()),
            };
            rt.block_on(async { black_box(service.create_shorten(req).await.unwrap()) })
        });
    });
}

/// Benchmark service get operations (cache miss scenario)
fn benchmark_service_get(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let service = rt.block_on(setup_test_service());

    // Pre-populate with test data
    rt.block_on(async {
        for i in 0..100 {
            let req = CreateShortenRequest {
                original_url: format!("https://example{}.com", i),
                code: Some(format!("get{}", i)),
                describe: None,
            };
            service.create_shorten(req).await.unwrap();
        }
    });

    c.bench_function("service_get_existing", |b| {
        b.iter(|| rt.block_on(async { black_box(service.get_shorten("get50").await.unwrap()) }));
    });
}

/// Benchmark service list operations
fn benchmark_service_list(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let service = rt.block_on(setup_test_service());

    // Pre-populate with test data
    rt.block_on(async {
        for i in 0..500 {
            let req = CreateShortenRequest {
                original_url: format!("https://example{}.com", i),
                code: Some(format!("list{}", i)),
                describe: Some(format!("URL {}", i)),
            };
            service.create_shorten(req).await.unwrap();
        }
    });

    let mut group = c.benchmark_group("service_list");

    for page_size in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(page_size),
            page_size,
            |b, &page_size| {
                b.iter(|| {
                    let params = shortener_server::repositories::url_repository::ListParams {
                        page: 1,
                        page_size,
                        ..Default::default()
                    };
                    rt.block_on(async { black_box(service.list_shortens(params).await.unwrap()) })
                });
            },
        );
    }

    group.finish();
}

/// Benchmark service update operations
fn benchmark_service_update(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let service = rt.block_on(setup_test_service());

    // Pre-populate with test data
    rt.block_on(async {
        for i in 0..100 {
            let req = CreateShortenRequest {
                original_url: format!("https://example{}.com", i),
                code: Some(format!("update{}", i)),
                describe: Some(format!("URL {}", i)),
            };
            service.create_shorten(req).await.unwrap();
        }
    });

    c.bench_function("service_update_url", |b| {
        b.iter(|| {
            let req = UpdateShortenRequest {
                original_url: Some("https://updated.com".to_string()),
                describe: Some("Updated".to_string()),
                status: None,
            };
            rt.block_on(async { black_box(service.update_shorten("update50", req).await.unwrap()) })
        });
    });
}

/// Benchmark code generation and uniqueness checking
fn benchmark_service_code_generation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let service = rt.block_on(setup_test_service());

    // Pre-populate to increase collision probability
    rt.block_on(async {
        for i in 0..100 {
            let req = CreateShortenRequest {
                original_url: format!("https://example{}.com", i),
                code: None,
                describe: None,
            };
            service.create_shorten(req).await.unwrap();
        }
    });

    let mut counter = 0;

    c.bench_function("service_generate_unique_code", |b| {
        b.iter(|| {
            counter += 1;
            let req = CreateShortenRequest {
                original_url: format!("https://unique{}.com", counter),
                code: None,
                describe: None,
            };
            rt.block_on(async { black_box(service.create_shorten(req).await.unwrap()) })
        });
    });
}

/// Benchmark validation operations
fn benchmark_service_validation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let service = rt.block_on(setup_test_service());

    c.bench_function("service_validate_and_create", |b| {
        let mut counter = 0;
        b.iter(|| {
            counter += 1;
            let req = CreateShortenRequest {
                original_url: format!("https://example{}.com", counter),
                code: Some(format!("valid{}", counter)),
                describe: Some("Test description".to_string()),
            };
            rt.block_on(async { black_box(service.create_shorten(req).await.unwrap()) })
        });
    });
}

criterion_group!(
    benches,
    benchmark_service_create,
    benchmark_service_get,
    benchmark_service_list,
    benchmark_service_update,
    benchmark_service_code_generation,
    benchmark_service_validation
);
criterion_main!(benches);
