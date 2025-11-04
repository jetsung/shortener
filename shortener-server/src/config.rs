use crate::logging::LoggingConfig;
use config::{Config as ConfigBuilder, ConfigError, Environment, File};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Main configuration structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub server: ServerConfig,
    pub shortener: ShortenerConfig,
    pub admin: AdminConfig,
    pub database: DatabaseConfig,
    pub cache: CacheConfig,
    pub geoip: GeoIpConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
}

/// Server configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub address: String,
    #[serde(rename = "trusted-platform")]
    pub trusted_platform: Option<String>,
    pub site_url: String,
    pub api_key: String,
}

/// Shortener configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ShortenerConfig {
    pub code_length: usize,
    pub code_charset: String,
}

/// Admin configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AdminConfig {
    pub username: String,
    pub password: String,
}

/// Database configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    #[serde(rename = "type")]
    pub db_type: DatabaseType,
    pub log_level: u8,
    pub sqlite: Option<SqliteConfig>,
    pub postgres: Option<PostgresConfig>,
    pub mysql: Option<MysqlConfig>,
}

/// Database type enum
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseType {
    Sqlite,
    Postgres,
    Mysql,
}

/// SQLite configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SqliteConfig {
    pub path: String,
}

/// PostgreSQL configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PostgresConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: String,
    pub sslmode: String,
    pub timezone: String,
}

/// MySQL configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MysqlConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: String,
    pub charset: String,
    pub parse_time: bool,
    pub loc: String,
}

/// Cache configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CacheConfig {
    pub enabled: bool,
    #[serde(rename = "type", default = "default_cache_type")]
    pub cache_type: CacheType,
    #[serde(default = "default_cache_expire")]
    pub expire: u64,
    #[serde(default = "default_cache_prefix")]
    pub prefix: String,
    pub redis: Option<RedisConfig>,
    pub valkey: Option<ValkeyConfig>,
}

fn default_cache_type() -> CacheType {
    CacheType::Redis
}

fn default_cache_expire() -> u64 {
    3600
}

fn default_cache_prefix() -> String {
    "shorten:".to_string()
}

/// Cache type enum
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CacheType {
    Redis,
    Valkey,
}

/// Redis configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RedisConfig {
    pub host: String,
    pub port: u16,
    pub password: String,
    pub db: u8,
}

/// Valkey configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ValkeyConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub db: u8,
}

/// GeoIP configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GeoIpConfig {
    pub enabled: bool,
    #[serde(rename = "type", default = "default_geoip_type")]
    pub geoip_type: GeoIpType,
    pub ip2region: Option<Ip2RegionConfig>,
}

fn default_geoip_type() -> GeoIpType {
    GeoIpType::Ip2region
}

/// GeoIP type enum
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum GeoIpType {
    Ip2region,
}

/// ip2region configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Ip2RegionConfig {
    pub path: String,
    pub mode: String,
    pub version: String,
}

impl Config {
    /// Load configuration from a file
    pub fn load() -> Result<Self, ConfigError> {
        Self::from_file("config/config.toml")
    }

    /// Load configuration from a specific file path
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let config = ConfigBuilder::builder()
            .add_source(File::from(path.as_ref()).format(config::FileFormat::Toml))
            .add_source(Environment::with_prefix("SHORTENER").separator("__"))
            .build()?;

        let mut cfg: Config = config.try_deserialize()?;

        // Apply defaults and validate
        cfg.apply_defaults();
        cfg.validate()?;

        Ok(cfg)
    }

    /// Apply default values to configuration
    fn apply_defaults(&mut self) {
        // Server defaults
        if self.server.address.is_empty() {
            self.server.address = ":8080".to_string();
        }
        if self.server.site_url.is_empty() {
            self.server.site_url = "http://localhost:8080".to_string();
        }

        // Shortener defaults
        if self.shortener.code_length == 0 {
            self.shortener.code_length = 6;
        }
        if self.shortener.code_charset.is_empty() {
            self.shortener.code_charset =
                "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ".to_string();
        }

        // Cache defaults
        if self.cache.expire == 0 {
            self.cache.expire = 3600;
        }
        if self.cache.prefix.is_empty() {
            self.cache.prefix = "shorten:".to_string();
        }

        // Database defaults
        if self.database.log_level == 0 {
            self.database.log_level = 1;
        }
    }

    /// Validate configuration
    fn validate(&self) -> Result<(), ConfigError> {
        // Validate logging configuration
        if let Err(e) = self.logging.validate() {
            return Err(ConfigError::Message(format!("logging: {}", e)));
        }

        // Validate server configuration
        if self.server.api_key.is_empty() {
            return Err(ConfigError::Message(
                "server.api_key is required".to_string(),
            ));
        }

        // Validate admin configuration
        if self.admin.username.is_empty() {
            return Err(ConfigError::Message(
                "admin.username is required".to_string(),
            ));
        }
        if self.admin.password.is_empty() {
            return Err(ConfigError::Message(
                "admin.password is required".to_string(),
            ));
        }

        // Validate shortener configuration
        if self.shortener.code_length < 4 || self.shortener.code_length > 16 {
            return Err(ConfigError::Message(
                "shortener.code_length must be between 4 and 16".to_string(),
            ));
        }
        if self.shortener.code_charset.is_empty() {
            return Err(ConfigError::Message(
                "shortener.code_charset cannot be empty".to_string(),
            ));
        }

        // Validate database configuration
        match self.database.db_type {
            DatabaseType::Sqlite => {
                if self.database.sqlite.is_none() {
                    return Err(ConfigError::Message(
                        "database.sqlite configuration is required when type is sqlite".to_string(),
                    ));
                }
            }
            DatabaseType::Postgres => {
                if self.database.postgres.is_none() {
                    return Err(ConfigError::Message(
                        "database.postgres configuration is required when type is postgres"
                            .to_string(),
                    ));
                }
            }
            DatabaseType::Mysql => {
                if self.database.mysql.is_none() {
                    return Err(ConfigError::Message(
                        "database.mysql configuration is required when type is mysql".to_string(),
                    ));
                }
            }
        }

        // Validate cache configuration
        if self.cache.enabled {
            match self.cache.cache_type {
                CacheType::Redis => {
                    if self.cache.redis.is_none() {
                        return Err(ConfigError::Message(
                            "cache.redis configuration is required when type is redis".to_string(),
                        ));
                    }
                }
                CacheType::Valkey => {
                    if self.cache.valkey.is_none() {
                        return Err(ConfigError::Message(
                            "cache.valkey configuration is required when type is valkey"
                                .to_string(),
                        ));
                    }
                }
            }
        }

        // Validate GeoIP configuration
        if self.geoip.enabled {
            match self.geoip.geoip_type {
                GeoIpType::Ip2region => {
                    if self.geoip.ip2region.is_none() {
                        return Err(ConfigError::Message(
                            "geoip.ip2region configuration is required when type is ip2region"
                                .to_string(),
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    /// Get database connection string
    pub fn get_database_url(&self) -> String {
        match self.database.db_type {
            DatabaseType::Sqlite => {
                let sqlite_config = self.database.sqlite.as_ref().unwrap();
                // For SQLite, use the path directly without the sqlite:// prefix for SeaORM
                if sqlite_config.path == ":memory:" {
                    "sqlite::memory:".to_string()
                } else {
                    format!("sqlite://{}?mode=rwc", sqlite_config.path)
                }
            }
            DatabaseType::Postgres => {
                let pg_config = self.database.postgres.as_ref().unwrap();
                format!(
                    "postgres://{}:{}@{}:{}/{}?sslmode={}",
                    pg_config.user,
                    pg_config.password,
                    pg_config.host,
                    pg_config.port,
                    pg_config.database,
                    pg_config.sslmode
                )
            }
            DatabaseType::Mysql => {
                let mysql_config = self.database.mysql.as_ref().unwrap();
                format!(
                    "mysql://{}:{}@{}:{}/{}?charset={}",
                    mysql_config.user,
                    mysql_config.password,
                    mysql_config.host,
                    mysql_config.port,
                    mysql_config.database,
                    mysql_config.charset
                )
            }
        }
    }

    /// Get cache connection string
    pub fn get_cache_url(&self) -> Option<String> {
        if !self.cache.enabled {
            return None;
        }

        match self.cache.cache_type {
            CacheType::Redis => {
                let redis_config = self.cache.redis.as_ref()?;
                let auth = if redis_config.password.is_empty() {
                    String::new()
                } else {
                    format!(":{}@", redis_config.password)
                };
                Some(format!(
                    "redis://{}{}:{}/{}",
                    auth, redis_config.host, redis_config.port, redis_config.db
                ))
            }
            CacheType::Valkey => {
                let valkey_config = self.cache.valkey.as_ref()?;
                let auth = if valkey_config.password.is_empty() {
                    String::new()
                } else if valkey_config.username.is_empty() {
                    format!(":{}@", valkey_config.password)
                } else {
                    format!("{}:{}@", valkey_config.username, valkey_config.password)
                };
                Some(format!(
                    "redis://{}{}:{}/{}",
                    auth, valkey_config.host, valkey_config.port, valkey_config.db
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_config_file(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file.flush().unwrap();
        file
    }

    #[test]
    fn test_load_valid_config() {
        let config_content = r#"
[server]
address = ":8080"
trusted-platform = ""
site_url = "http://localhost:8080"
api_key = "test-api-key"

[shortener]
code_length = 6
code_charset = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"

[admin]
username = "admin"
password = "admin123"

[database]
type = "sqlite"
log_level = 1

[database.sqlite]
path = "data/test.db"

[cache]
enabled = false
type = "redis"
expire = 3600
prefix = "shorten:"

[geoip]
enabled = false
type = "ip2region"
"#;

        let file = create_test_config_file(config_content);
        let config = Config::from_file(file.path()).unwrap();

        assert_eq!(config.server.address, ":8080");
        assert_eq!(config.server.api_key, "test-api-key");
        assert_eq!(config.shortener.code_length, 6);
        assert_eq!(config.admin.username, "admin");
        assert_eq!(config.database.db_type, DatabaseType::Sqlite);
        assert!(!config.cache.enabled);
        assert!(!config.geoip.enabled);
    }

    #[test]
    fn test_config_with_defaults() {
        let config_content = r#"
[server]
address = ""
site_url = ""
api_key = "test-key"

[shortener]
code_length = 0
code_charset = ""

[admin]
username = "admin"
password = "pass"

[database]
type = "sqlite"
log_level = 0

[database.sqlite]
path = "test.db"

[cache]
enabled = false
type = "redis"
expire = 0
prefix = ""

[geoip]
enabled = false
type = "ip2region"
"#;

        let file = create_test_config_file(config_content);
        let config = Config::from_file(file.path()).unwrap();

        // Check defaults are applied
        assert_eq!(config.server.address, ":8080");
        assert_eq!(config.server.site_url, "http://localhost:8080");
        assert_eq!(config.shortener.code_length, 6);
        assert_eq!(
            config.shortener.code_charset,
            "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"
        );
        assert_eq!(config.cache.expire, 3600);
        assert_eq!(config.cache.prefix, "shorten:");
        assert_eq!(config.database.log_level, 1);
    }

    #[test]
    fn test_missing_api_key() {
        let config_content = r#"
[server]
address = ":8080"
site_url = "http://localhost:8080"
api_key = ""

[shortener]
code_length = 6
code_charset = "abc"

[admin]
username = "admin"
password = "pass"

[database]
type = "sqlite"
log_level = 1

[database.sqlite]
path = "test.db"

[cache]
enabled = false
type = "redis"
expire = 3600
prefix = "shorten:"

[geoip]
enabled = false
type = "ip2region"
"#;

        let file = create_test_config_file(config_content);
        let result = Config::from_file(file.path());

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("server.api_key is required")
        );
    }

    #[test]
    fn test_missing_admin_username() {
        let config_content = r#"
[server]
address = ":8080"
site_url = "http://localhost:8080"
api_key = "test-key"

[shortener]
code_length = 6
code_charset = "abc"

[admin]
username = ""
password = "pass"

[database]
type = "sqlite"
log_level = 1

[database.sqlite]
path = "test.db"

[cache]
enabled = false
type = "redis"
expire = 3600
prefix = "shorten:"

[geoip]
enabled = false
type = "ip2region"
"#;

        let file = create_test_config_file(config_content);
        let result = Config::from_file(file.path());

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("admin.username is required")
        );
    }

    #[test]
    fn test_invalid_code_length() {
        let config_content = r#"
[server]
address = ":8080"
site_url = "http://localhost:8080"
api_key = "test-key"

[shortener]
code_length = 20
code_charset = "abc"

[admin]
username = "admin"
password = "pass"

[database]
type = "sqlite"
log_level = 1

[database.sqlite]
path = "test.db"

[cache]
enabled = false
type = "redis"
expire = 3600
prefix = "shorten:"

[geoip]
enabled = false
type = "ip2region"
"#;

        let file = create_test_config_file(config_content);
        let result = Config::from_file(file.path());

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("code_length must be between 4 and 16")
        );
    }

    #[test]
    fn test_postgres_config() {
        let config_content = r#"
[server]
address = ":8080"
site_url = "http://localhost:8080"
api_key = "test-key"

[shortener]
code_length = 6
code_charset = "abc"

[admin]
username = "admin"
password = "pass"

[database]
type = "postgres"
log_level = 1

[database.postgres]
host = "localhost"
port = 5432
user = "postgres"
password = "postgres"
database = "shortener"
sslmode = "disable"
timezone = "UTC"

[cache]
enabled = false
type = "redis"
expire = 3600
prefix = "shorten:"

[geoip]
enabled = false
type = "ip2region"
"#;

        let file = create_test_config_file(config_content);
        let config = Config::from_file(file.path()).unwrap();

        assert_eq!(config.database.db_type, DatabaseType::Postgres);
        assert!(config.database.postgres.is_some());

        let pg_config = config.database.postgres.as_ref().unwrap();
        assert_eq!(pg_config.host, "localhost");
        assert_eq!(pg_config.port, 5432);
        assert_eq!(pg_config.database, "shortener");
    }

    #[test]
    fn test_mysql_config() {
        let config_content = r#"
[server]
address = ":8080"
site_url = "http://localhost:8080"
api_key = "test-key"

[shortener]
code_length = 6
code_charset = "abc"

[admin]
username = "admin"
password = "pass"

[database]
type = "mysql"
log_level = 1

[database.mysql]
host = "localhost"
port = 3306
user = "root"
password = "root"
database = "shortener"
charset = "utf8mb4"
parse_time = true
loc = "Local"

[cache]
enabled = false
type = "redis"
expire = 3600
prefix = "shorten:"

[geoip]
enabled = false
type = "ip2region"
"#;

        let file = create_test_config_file(config_content);
        let config = Config::from_file(file.path()).unwrap();

        assert_eq!(config.database.db_type, DatabaseType::Mysql);
        assert!(config.database.mysql.is_some());

        let mysql_config = config.database.mysql.as_ref().unwrap();
        assert_eq!(mysql_config.host, "localhost");
        assert_eq!(mysql_config.port, 3306);
        assert_eq!(mysql_config.charset, "utf8mb4");
    }

    #[test]
    fn test_cache_enabled_redis() {
        let config_content = r#"
[server]
address = ":8080"
site_url = "http://localhost:8080"
api_key = "test-key"

[shortener]
code_length = 6
code_charset = "abc"

[admin]
username = "admin"
password = "pass"

[database]
type = "sqlite"
log_level = 1

[database.sqlite]
path = "test.db"

[cache]
enabled = true
type = "redis"
expire = 3600
prefix = "shorten:"

[cache.redis]
host = "localhost"
port = 6379
password = ""
db = 0

[geoip]
enabled = false
type = "ip2region"
"#;

        let file = create_test_config_file(config_content);
        let config = Config::from_file(file.path()).unwrap();

        assert!(config.cache.enabled);
        assert_eq!(config.cache.cache_type, CacheType::Redis);
        assert!(config.cache.redis.is_some());
    }

    #[test]
    fn test_cache_enabled_valkey() {
        let config_content = r#"
[server]
address = ":8080"
site_url = "http://localhost:8080"
api_key = "test-key"

[shortener]
code_length = 6
code_charset = "abc"

[admin]
username = "admin"
password = "pass"

[database]
type = "sqlite"
log_level = 1

[database.sqlite]
path = "test.db"

[cache]
enabled = true
type = "valkey"
expire = 3600
prefix = "shorten:"

[cache.valkey]
host = "localhost"
port = 6379
username = ""
password = ""
db = 0

[geoip]
enabled = false
type = "ip2region"
"#;

        let file = create_test_config_file(config_content);
        let config = Config::from_file(file.path()).unwrap();

        assert!(config.cache.enabled);
        assert_eq!(config.cache.cache_type, CacheType::Valkey);
        assert!(config.cache.valkey.is_some());
    }

    #[test]
    fn test_geoip_enabled() {
        let config_content = r#"
[server]
address = ":8080"
site_url = "http://localhost:8080"
api_key = "test-key"

[shortener]
code_length = 6
code_charset = "abc"

[admin]
username = "admin"
password = "pass"

[database]
type = "sqlite"
log_level = 1

[database.sqlite]
path = "test.db"

[cache]
enabled = false
type = "redis"
expire = 3600
prefix = "shorten:"

[geoip]
enabled = true
type = "ip2region"

[geoip.ip2region]
path = "data/ip2region.xdb"
mode = "vector"
version = "4"
"#;

        let file = create_test_config_file(config_content);
        let config = Config::from_file(file.path()).unwrap();

        assert!(config.geoip.enabled);
        assert_eq!(config.geoip.geoip_type, GeoIpType::Ip2region);
        assert!(config.geoip.ip2region.is_some());

        let ip2region = config.geoip.ip2region.as_ref().unwrap();
        assert_eq!(ip2region.version, "4");
    }

    #[test]
    fn test_get_database_url_sqlite() {
        let config_content = r#"
[server]
address = ":8080"
site_url = "http://localhost:8080"
api_key = "test-key"

[shortener]
code_length = 6
code_charset = "abc"

[admin]
username = "admin"
password = "pass"

[database]
type = "sqlite"
log_level = 1

[database.sqlite]
path = "data/test.db"

[cache]
enabled = false
type = "redis"
expire = 3600
prefix = "shorten:"

[geoip]
enabled = false
type = "ip2region"
"#;

        let file = create_test_config_file(config_content);
        let config = Config::from_file(file.path()).unwrap();

        let url = config.get_database_url();
        assert_eq!(url, "sqlite://data/test.db?mode=rwc");
    }

    #[test]
    fn test_get_database_url_sqlite_memory() {
        let config_content = r#"
[server]
address = ":8080"
site_url = "http://localhost:8080"
api_key = "test-key"

[shortener]
code_length = 6
code_charset = "abc"

[admin]
username = "admin"
password = "pass"

[database]
type = "sqlite"
log_level = 1

[database.sqlite]
path = ":memory:"

[cache]
enabled = false
type = "redis"
expire = 3600
prefix = "shorten:"

[geoip]
enabled = false
type = "ip2region"
"#;

        let file = create_test_config_file(config_content);
        let config = Config::from_file(file.path()).unwrap();

        let url = config.get_database_url();
        assert_eq!(url, "sqlite::memory:");
    }

    #[test]
    fn test_get_database_url_postgres() {
        let config_content = r#"
[server]
address = ":8080"
site_url = "http://localhost:8080"
api_key = "test-key"

[shortener]
code_length = 6
code_charset = "abc"

[admin]
username = "admin"
password = "pass"

[database]
type = "postgres"
log_level = 1

[database.postgres]
host = "localhost"
port = 5432
user = "postgres"
password = "secret"
database = "shortener"
sslmode = "disable"
timezone = "UTC"

[cache]
enabled = false
type = "redis"
expire = 3600
prefix = "shorten:"

[geoip]
enabled = false
type = "ip2region"
"#;

        let file = create_test_config_file(config_content);
        let config = Config::from_file(file.path()).unwrap();

        let url = config.get_database_url();
        assert_eq!(
            url,
            "postgres://postgres:secret@localhost:5432/shortener?sslmode=disable"
        );
    }

    #[test]
    fn test_get_cache_url_redis() {
        let config_content = r#"
[server]
address = ":8080"
site_url = "http://localhost:8080"
api_key = "test-key"

[shortener]
code_length = 6
code_charset = "abc"

[admin]
username = "admin"
password = "pass"

[database]
type = "sqlite"
log_level = 1

[database.sqlite]
path = "test.db"

[cache]
enabled = true
type = "redis"
expire = 3600
prefix = "shorten:"

[cache.redis]
host = "localhost"
port = 6379
password = "secret"
db = 1

[geoip]
enabled = false
type = "ip2region"
"#;

        let file = create_test_config_file(config_content);
        let config = Config::from_file(file.path()).unwrap();

        let url = config.get_cache_url();
        assert_eq!(url, Some("redis://:secret@localhost:6379/1".to_string()));
    }

    #[test]
    fn test_get_cache_url_disabled() {
        let config_content = r#"
[server]
address = ":8080"
site_url = "http://localhost:8080"
api_key = "test-key"

[shortener]
code_length = 6
code_charset = "abc"

[admin]
username = "admin"
password = "pass"

[database]
type = "sqlite"
log_level = 1

[database.sqlite]
path = "test.db"

[cache]
enabled = false
type = "redis"
expire = 3600
prefix = "shorten:"

[geoip]
enabled = false
type = "ip2region"
"#;

        let file = create_test_config_file(config_content);
        let config = Config::from_file(file.path()).unwrap();

        let url = config.get_cache_url();
        assert_eq!(url, None);
    }

    #[test]
    fn test_missing_database_config() {
        let config_content = r#"
[server]
address = ":8080"
site_url = "http://localhost:8080"
api_key = "test-key"

[shortener]
code_length = 6
code_charset = "abc"

[admin]
username = "admin"
password = "pass"

[database]
type = "postgres"
log_level = 1

[cache]
enabled = false
type = "redis"
expire = 3600
prefix = "shorten:"

[geoip]
enabled = false
type = "ip2region"
"#;

        let file = create_test_config_file(config_content);
        let result = Config::from_file(file.path());

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("database.postgres configuration is required")
        );
    }

    #[test]
    fn test_missing_cache_config() {
        let config_content = r#"
[server]
address = ":8080"
site_url = "http://localhost:8080"
api_key = "test-key"

[shortener]
code_length = 6
code_charset = "abc"

[admin]
username = "admin"
password = "pass"

[database]
type = "sqlite"
log_level = 1

[database.sqlite]
path = "test.db"

[cache]
enabled = true
type = "redis"
expire = 3600
prefix = "shorten:"

[geoip]
enabled = false
type = "ip2region"
"#;

        let file = create_test_config_file(config_content);
        let result = Config::from_file(file.path());

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("cache.redis configuration is required")
        );
    }
}
