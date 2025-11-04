use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use shortener_server::config::{Config, DatabaseConfig, DatabaseType, SqliteConfig};
use shortener_server::db::DbFactory;
use shortener_server::models::url::UrlStatus;
use shortener_server::repositories::url_repository::{
    CreateUrlDto, ListParams, UrlRepository, UrlRepositoryImpl,
};
use std::hint::black_box;
use tokio::runtime::Runtime;

/// Setup test database with migrations
async fn setup_test_db() -> sea_orm::DatabaseConnection {
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
    db
}

/// Benchmark database create operations
fn benchmark_db_create(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let db = rt.block_on(setup_test_db());
    let repo = UrlRepositoryImpl::new(db);

    let mut counter = 0;

    c.bench_function("db_create_single", |b| {
        b.iter(|| {
            counter += 1;
            let create_dto = CreateUrlDto {
                short_code: format!("bench{}", counter),
                original_url: "https://example.com".to_string(),
                description: Some("Benchmark URL".to_string()),
                status: UrlStatus::Enabled as i32,
            };

            rt.block_on(async { black_box(repo.create(create_dto).await.unwrap()) })
        });
    });
}

/// Benchmark database find operations
fn benchmark_db_find(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let db = rt.block_on(setup_test_db());
    let repo = UrlRepositoryImpl::new(db);

    // Pre-populate database with test data
    rt.block_on(async {
        for i in 0..100 {
            let create_dto = CreateUrlDto {
                short_code: format!("find{}", i),
                original_url: format!("https://example{}.com", i),
                description: Some(format!("URL {}", i)),
                status: UrlStatus::Enabled as i32,
            };
            repo.create(create_dto).await.unwrap();
        }
    });

    c.bench_function("db_find_by_code", |b| {
        b.iter(|| rt.block_on(async { black_box(repo.find_by_code("find50").await.unwrap()) }));
    });

    c.bench_function("db_find_by_id", |b| {
        b.iter(|| rt.block_on(async { black_box(repo.find_by_id(50).await.unwrap()) }));
    });
}

/// Benchmark database list operations with different page sizes
fn benchmark_db_list(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let db = rt.block_on(setup_test_db());
    let repo = UrlRepositoryImpl::new(db);

    // Pre-populate database with test data
    rt.block_on(async {
        for i in 0..1000 {
            let create_dto = CreateUrlDto {
                short_code: format!("list{}", i),
                original_url: format!("https://example{}.com", i),
                description: Some(format!("URL {}", i)),
                status: if i % 2 == 0 {
                    UrlStatus::Enabled as i32
                } else {
                    UrlStatus::Disabled as i32
                },
            };
            repo.create(create_dto).await.unwrap();
        }
    });

    let mut group = c.benchmark_group("db_list");

    for page_size in [10, 50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(page_size),
            page_size,
            |b, &page_size| {
                b.iter(|| {
                    let params = ListParams {
                        page: 1,
                        page_size,
                        ..Default::default()
                    };
                    rt.block_on(async { black_box(repo.list(params).await.unwrap()) })
                });
            },
        );
    }

    group.finish();
}

/// Benchmark database list with filters
fn benchmark_db_list_filtered(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let db = rt.block_on(setup_test_db());
    let repo = UrlRepositoryImpl::new(db);

    // Pre-populate database with test data
    rt.block_on(async {
        for i in 0..1000 {
            let create_dto = CreateUrlDto {
                short_code: format!("filter{}", i),
                original_url: format!("https://example{}.com", i),
                description: Some(format!("URL {}", i)),
                status: if i % 2 == 0 {
                    UrlStatus::Enabled as i32
                } else {
                    UrlStatus::Disabled as i32
                },
            };
            repo.create(create_dto).await.unwrap();
        }
    });

    c.bench_function("db_list_with_status_filter", |b| {
        b.iter(|| {
            let params = ListParams {
                page: 1,
                page_size: 50,
                status: Some(UrlStatus::Enabled as i32),
                ..Default::default()
            };
            rt.block_on(async { black_box(repo.list(params).await.unwrap()) })
        });
    });

    c.bench_function("db_list_with_sorting", |b| {
        b.iter(|| {
            let params = ListParams {
                page: 1,
                page_size: 50,
                sort_by: Some("created_at".to_string()),
                order: Some("desc".to_string()),
                ..Default::default()
            };
            rt.block_on(async { black_box(repo.list(params).await.unwrap()) })
        });
    });
}

/// Benchmark database update operations
fn benchmark_db_update(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let db = rt.block_on(setup_test_db());
    let repo = UrlRepositoryImpl::new(db);

    // Pre-populate database with test data
    rt.block_on(async {
        for i in 0..100 {
            let create_dto = CreateUrlDto {
                short_code: format!("update{}", i),
                original_url: format!("https://example{}.com", i),
                description: Some(format!("URL {}", i)),
                status: UrlStatus::Enabled as i32,
            };
            repo.create(create_dto).await.unwrap();
        }
    });

    c.bench_function("db_update_single", |b| {
        b.iter(|| {
            let update_dto = shortener_server::repositories::url_repository::UpdateUrlDto {
                original_url: Some("https://updated.com".to_string()),
                description: Some("Updated".to_string()),
                status: Some(UrlStatus::Disabled as i32),
            };
            rt.block_on(async { black_box(repo.update("update50", update_dto).await.unwrap()) })
        });
    });
}

/// Benchmark database delete operations
fn benchmark_db_delete(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("db_delete_single", |b| {
        b.iter_batched(
            || {
                // Setup: create a new database and URL for each iteration
                let db = rt.block_on(setup_test_db());
                let repo = UrlRepositoryImpl::new(db);
                rt.block_on(async {
                    let create_dto = CreateUrlDto {
                        short_code: "delete_test".to_string(),
                        original_url: "https://example.com".to_string(),
                        description: None,
                        status: UrlStatus::Enabled as i32,
                    };
                    repo.create(create_dto).await.unwrap();
                    repo
                })
            },
            |repo| {
                // Benchmark: delete the URL
                rt.block_on(async {
                    repo.delete("delete_test").await.unwrap();
                    black_box(())
                })
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

/// Benchmark batch delete operations
fn benchmark_db_delete_batch(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("db_delete_batch");

    for batch_size in [10, 50, 100].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(batch_size),
            batch_size,
            |b, &batch_size| {
                b.iter_batched(
                    || {
                        // Setup: create database and URLs
                        let db = rt.block_on(setup_test_db());
                        let repo = UrlRepositoryImpl::new(db);
                        let ids = rt.block_on(async {
                            let mut ids = Vec::new();
                            for i in 0..batch_size {
                                let create_dto = CreateUrlDto {
                                    short_code: format!("batch{}", i),
                                    original_url: format!("https://example{}.com", i),
                                    description: None,
                                    status: UrlStatus::Enabled as i32,
                                };
                                let url = repo.create(create_dto).await.unwrap();
                                ids.push(url.id);
                            }
                            ids
                        });
                        (repo, ids)
                    },
                    |(repo, ids)| {
                        // Benchmark: batch delete
                        rt.block_on(async { black_box(repo.delete_batch(ids).await.unwrap()) })
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_db_create,
    benchmark_db_find,
    benchmark_db_list,
    benchmark_db_list_filtered,
    benchmark_db_update,
    benchmark_db_delete,
    benchmark_db_delete_batch
);
criterion_main!(benches);
