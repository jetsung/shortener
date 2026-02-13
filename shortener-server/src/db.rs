use crate::config::{Config, DatabaseType};
use crate::migration::Migrator;
use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use sea_orm_migration::MigratorTrait;
use std::fs;
use std::path::Path;
use std::time::Duration;
use tracing::{info, warn};

/// Database connection factory
pub struct DbFactory;

impl DbFactory {
    /// Create a database connection from configuration
    pub async fn create_connection(config: &Config) -> Result<DatabaseConnection, DbErr> {
        let database_url = config.get_database_url();

        // For SQLite, ensure the parent directory exists
        if config.database.db_type == DatabaseType::Sqlite
            && let Some(path) = config.database.sqlite.as_ref().map(|s| &s.path)
            && path != ":memory:"
            && let Some(parent) = Path::new(path).parent()
            && !parent.exists()
        {
            if let Err(e) = fs::create_dir_all(parent) {
                warn!("Failed to create database directory: {}", e);
            } else {
                info!("Created database directory: {:?}", parent);
            }
        }

        info!(
            "Connecting to {} database...",
            match config.database.db_type {
                DatabaseType::Sqlite => "SQLite",
                DatabaseType::Postgres => "PostgreSQL",
                DatabaseType::Mysql => "MySQL",
            }
        );

        let mut opt = ConnectOptions::new(database_url);

        // Configure connection pool
        opt.max_connections(100)
            .min_connections(5)
            .connect_timeout(Duration::from_secs(8))
            .acquire_timeout(Duration::from_secs(8))
            .idle_timeout(Duration::from_secs(8))
            .max_lifetime(Duration::from_secs(8));

        // Configure logging based on log_level
        let log_level = match config.database.log_level {
            0 => tracing::log::LevelFilter::Off,
            1 => tracing::log::LevelFilter::Error,
            2 => tracing::log::LevelFilter::Warn,
            3 => tracing::log::LevelFilter::Info,
            4 => tracing::log::LevelFilter::Debug,
            _ => tracing::log::LevelFilter::Trace,
        };

        opt.sqlx_logging(true).sqlx_logging_level(log_level);

        let db = Database::connect(opt).await?;

        info!("Database connection established successfully");

        Ok(db)
    }

    /// Test database connection
    pub async fn test_connection(db: &DatabaseConnection) -> Result<(), DbErr> {
        match db.ping().await {
            Ok(_) => {
                info!("Database ping successful");
                Ok(())
            }
            Err(e) => {
                warn!("Database ping failed: {}", e);
                Err(e)
            }
        }
    }

    /// Run database migrations
    pub async fn run_migrations(db: &DatabaseConnection) -> Result<(), DbErr> {
        info!("Running database migrations...");

        Migrator::up(db, None).await?;

        info!("Database migrations completed successfully");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, DatabaseConfig, DatabaseType, SqliteConfig};

    fn create_test_config() -> Config {
        Config {
            server: crate::config::ServerConfig {
                address: ":8080".to_string(),
                trusted_platform: None,
                site_url: "http://localhost:8080".to_string(),
                api_key: "test-key".to_string(),
            },
            shortener: crate::config::ShortenerConfig {
                code_length: 6,
                code_charset: "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"
                    .to_string(),
            },
            admin: crate::config::AdminConfig {
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
            cache: crate::config::CacheConfig {
                enabled: false,
                cache_type: crate::config::CacheType::Redis,
                expire: 3600,
                prefix: "shorten:".to_string(),
                redis: None,
                valkey: None,
            },
            geoip: crate::config::GeoIpConfig {
                enabled: false,
                geoip_type: crate::config::GeoIpType::Ip2region,
                ip2region: None,
            },
            logging: crate::logging::LoggingConfig::default(),
        }
    }

    #[tokio::test]
    async fn test_create_sqlite_connection() {
        let config = create_test_config();
        let result = DbFactory::create_connection(&config).await;

        if let Err(e) = &result {
            eprintln!("Connection error: {:?}", e);
        }
        assert!(result.is_ok());

        let db = result.unwrap();
        let ping_result = DbFactory::test_connection(&db).await;
        assert!(ping_result.is_ok());
    }

    #[tokio::test]
    async fn test_invalid_connection() {
        // Test with an invalid file path (not a valid SQLite database)
        let mut config = create_test_config();
        config.database.sqlite = Some(SqliteConfig {
            path: "/dev/null/test.db".to_string(),
        });

        // This should fail because /dev/null is not a valid SQLite database location
        let result = DbFactory::create_connection(&config).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_run_migrations() {
        let config = create_test_config();
        let db = DbFactory::create_connection(&config).await.unwrap();

        let result = DbFactory::run_migrations(&db).await;
        assert!(result.is_ok());

        // Verify tables were created by checking if we can query them
        use crate::models::{HistoryEntity, UrlEntity};
        use sea_orm::EntityTrait;

        let urls = UrlEntity::find().all(&db).await;
        assert!(urls.is_ok());

        let histories = HistoryEntity::find().all(&db).await;
        assert!(histories.is_ok());
    }

    #[tokio::test]
    async fn test_connection_with_different_log_levels() {
        for log_level in 0..=5 {
            let mut config = create_test_config();
            config.database.log_level = log_level;

            let result = DbFactory::create_connection(&config).await;
            assert!(result.is_ok(), "Failed with log_level {}", log_level);
        }
    }

    #[tokio::test]
    async fn test_test_connection() {
        let config = create_test_config();
        let db = DbFactory::create_connection(&config).await.unwrap();

        let result = DbFactory::test_connection(&db).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_multiple_connections() {
        let config = create_test_config();

        // Create multiple connections
        let db1 = DbFactory::create_connection(&config).await.unwrap();
        let db2 = DbFactory::create_connection(&config).await.unwrap();

        // Both should work
        assert!(DbFactory::test_connection(&db1).await.is_ok());
        assert!(DbFactory::test_connection(&db2).await.is_ok());
    }

    #[tokio::test]
    async fn test_migrations_idempotent() {
        let config = create_test_config();
        let db = DbFactory::create_connection(&config).await.unwrap();

        // Run migrations twice
        let result1 = DbFactory::run_migrations(&db).await;
        assert!(result1.is_ok());

        let result2 = DbFactory::run_migrations(&db).await;
        assert!(result2.is_ok());
    }
}
