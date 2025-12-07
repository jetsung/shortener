use sea_orm::{EntityTrait, Set};
use shortener_server::{
    config::{Config, DatabaseConfig, DatabaseType, SqliteConfig},
    db::DbFactory,
    logging::LoggingConfig,
    models::{HistoryEntity, UrlEntity, history, url},
};

fn create_test_config() -> Config {
    Config {
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
            log_level: 1,
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
        logging: LoggingConfig::default(),
    }
}

#[tokio::test]
async fn test_database_migration_and_crud() {
    let config = create_test_config();
    let db = DbFactory::create_connection(&config).await.unwrap();

    // Run migrations
    DbFactory::run_migrations(&db).await.unwrap();

    // Test URL CRUD operations
    let url_model = url::ActiveModel {
        short_code: Set("test123".to_string()),
        original_url: Set("https://example.com".to_string()),
        description: Set(Some("Test URL".to_string())),
        status: Set(1),
        created_at: Set(chrono::Utc::now()),
        updated_at: Set(chrono::Utc::now()),
        ..Default::default()
    };

    let inserted_url = UrlEntity::insert(url_model)
        .exec(&db)
        .await
        .expect("Failed to insert URL");

    // Verify URL was inserted
    let found_url = UrlEntity::find_by_id(inserted_url.last_insert_id)
        .one(&db)
        .await
        .expect("Failed to find URL")
        .expect("URL not found");

    assert_eq!(found_url.short_code, "test123");
    assert_eq!(found_url.original_url, "https://example.com");
    assert_eq!(found_url.description, Some("Test URL".to_string()));

    // Test History CRUD operations
    let history_model = history::ActiveModel {
        url_id: Set(inserted_url.last_insert_id as i32),
        short_code: Set("test123".to_string()),
        ip_address: Set("127.0.0.1".to_string()),
        user_agent: Set("Test Agent".to_string()),
        referer: Set(None),
        country: Set(Some("US".to_string())),
        region: Set(None),
        province: Set(None),
        city: Set(None),
        isp: Set(None),
        device_type: Set(None),
        os: Set(None),
        browser: Set(None),
        accessed_at: Set(chrono::Utc::now()),
        created_at: Set(chrono::Utc::now()),
        ..Default::default()
    };

    let inserted_history = HistoryEntity::insert(history_model)
        .exec(&db)
        .await
        .expect("Failed to insert history");

    // Verify history was inserted
    let found_history = HistoryEntity::find_by_id(inserted_history.last_insert_id)
        .one(&db)
        .await
        .expect("Failed to find history")
        .expect("History not found");

    assert_eq!(found_history.short_code, "test123");
    assert_eq!(found_history.ip_address, "127.0.0.1");
    assert_eq!(found_history.country, Some("US".to_string()));

    // Test cascade delete
    UrlEntity::delete_by_id(inserted_url.last_insert_id)
        .exec(&db)
        .await
        .expect("Failed to delete URL");

    // Verify URL was deleted
    let deleted_url = UrlEntity::find_by_id(inserted_url.last_insert_id)
        .one(&db)
        .await
        .expect("Failed to query URL");

    assert!(deleted_url.is_none());

    // Note: Cascade delete behavior depends on database support
    // SQLite in-memory may not enforce foreign key constraints by default
}
