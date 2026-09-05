use crate::logging::LoggingConfig;
use config::{Config as ConfigBuilder, ConfigError, Environment, File};
use serde::de::Deserializer;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// 反序列化字符串列表：兼容 TOML 数组与字符串形式。
///
/// config crate 的 `Environment` 源只会把环境变量当作字符串交给 serde，
/// 因此 `OIDC__ALLOW_EMAILS="[]"` / `"a,b"` / `'["a","b"]'` 等写法默认
/// 都会报 `invalid type: string "...", expected a sequence`。这里统一处理：
/// - 空串或 `[]` → 空列表
/// - `["a","b"]`（JSON 风格）→ 去掉括号后按逗号拆分
/// - `a,b` → 按逗号拆分
fn deserialize_string_list<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringList {
        List(Vec<String>),
        Str(String),
    }

    match StringList::deserialize(deserializer)? {
        StringList::List(list) => Ok(list),
        StringList::Str(s) => {
            let s = s.trim();
            if s.is_empty() || s == "[]" {
                return Ok(Vec::new());
            }
            let inner = s
                .strip_prefix('[')
                .and_then(|rest| rest.strip_suffix(']'))
                .unwrap_or(s);
            if inner.trim().is_empty() {
                return Ok(Vec::new());
            }
            Ok(inner
                .split(',')
                .map(|item| {
                    item.trim()
                        .trim_matches('"')
                        .trim_matches('\'')
                        .to_string()
                })
                .collect())
        }
    }
}

/// 从监听地址推断默认短链域名：host 为空或通配地址（0.0.0.0/::）时回退 `localhost`。
/// 例如 `:8080` → `http://localhost:8080`，`0.0.0.0:8080` → `http://localhost:8080`，
/// `127.0.0.1:8080` → `http://127.0.0.1:8080`，`[::1]:8080` → `http://[::1]:8080`。
fn infer_short_url(address: &str) -> String {
    let Some((host, port)) = address.rsplit_once(':') else {
        return "http://localhost:8080".to_string();
    };
    match host {
        "" | "0.0.0.0" | "::" | "[::]" | "0:0:0:0:0:0:0:0" => {
            format!("http://localhost:{port}")
        }
        h => {
            // 剥离方括号后仍含冒号 → IPv6 字面量，URL 中需保留方括号
            let raw = h.trim_start_matches('[').trim_end_matches(']');
            if raw.contains(':') {
                format!("http://[{raw}]:{port}")
            } else {
                format!("http://{h}:{port}")
            }
        }
    }
}

/// Main configuration structure
///
/// 所有 section 均可缺省（`#[serde(default)]`），支持纯环境变量启动；
/// 字段级缺省值由 `apply_defaults()` 填充，必填项由 `validate()` 校验。
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    #[serde(default)]
    pub server: ServerConfig,
    #[serde(default)]
    pub slug: SlugConfig,
    #[serde(default)]
    pub admin: AdminConfig,
    #[serde(default)]
    pub oidc: OidcConfig,
    #[serde(default)]
    pub database: DatabaseConfig,
    #[serde(default)]
    pub cache: CacheConfig,
    #[serde(default = "default_geoip_config")]
    pub geoip: GeoIpConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
}

/// Server configuration
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ServerConfig {
    /// 监听地址；缺省时由 apply_defaults 填充 ":8080"
    #[serde(default)]
    pub address: String,
    #[serde(default, rename = "trusted-platform")]
    pub trusted_platform: Option<String>,
    /// 短址专用域名（可选）：未设置时从监听地址 `address` 推断（通配地址回退 localhost）
    #[serde(default)]
    pub short_url: String,
    /// API key（必填，敏感）；缺省空值由 validate 拦截
    #[serde(default)]
    pub api_key: String,
}

/// Shortener configuration
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SlugConfig {
    /// 短码长度；缺省 0 由 apply_defaults 填充 6
    #[serde(default)]
    pub length: usize,
    /// 短码字母表；缺省空值由 apply_defaults 填充
    #[serde(default)]
    pub alphabet: String,
}

/// Admin configuration
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct AdminConfig {
    /// 管理员用户名；缺省 "admin"
    #[serde(default = "default_admin_username")]
    pub username: String,
    /// Argon2id password hash (PHC string format), generated via
    /// `shortener-server hash-password` or `shortener-cli hash-password`.
    /// 必填（敏感）；缺省空值由 validate 拦截
    #[serde(default)]
    pub password_hash: String,
}

fn default_admin_username() -> String {
    "admin".to_string()
}

/// OIDC / OAuth2.0 configuration for external identity provider (IdP) login.
///
/// Any standard OIDC IdP (Keycloak, Authelia, Okta, Entra ID, ...) is supported
/// via discovery. Login is allowed only when the authenticated user's email and/or
/// subject matches the configured allowlist.
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct OidcConfig {
    /// Master switch for OIDC login. When `false` (the default), the OIDC
    /// login/callback endpoints are unavailable. When `true`, `issuer`,
    /// `client_id` and at least one allowlist (`allow_emails` or
    /// `allow_subjects`) must also be set.
    #[serde(default)]
    pub enabled: bool,
    /// IdP issuer URL, e.g. `https://keycloak.example.com/realms/main`.
    /// Required when `enabled` is true.
    #[serde(default)]
    pub issuer: Option<String>,
    #[serde(default)]
    pub client_id: Option<String>,
    /// Sensitive: prefer the `OIDC_CLIENT_SECRET` environment variable.
    #[serde(default)]
    pub client_secret: Option<String>,
    /// Allowlist of user emails; any match grants login.
    /// At least one of `allow_emails` / `allow_subjects` must be non-empty.
    #[serde(default, deserialize_with = "deserialize_string_list")]
    pub allow_emails: Vec<String>,
    /// Allowlist of user subjects (sub claim); any match grants login.
    /// At least one of `allow_emails` / `allow_subjects` must be non-empty.
    #[serde(default, deserialize_with = "deserialize_string_list")]
    pub allow_subjects: Vec<String>,
}

/// Database configuration
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct DatabaseConfig {
    /// Connection string (e.g. "sqlite://data/app.db?mode=rwc",
    /// "postgres://user:pass@host:5432/db", "mysql://user:pass@host:3306/db").
    /// Can also be provided via the `DATABASE_URL` environment variable.
    #[serde(default)]
    pub url: Option<String>,
    /// 日志级别 1-4；缺省 0 由 apply_defaults 填充 1
    #[serde(default)]
    pub log_level: u8,
}

/// Database engine kind, derived from the connection URL scheme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbKind {
    Sqlite,
    Postgres,
    Mysql,
}

impl DbKind {
    /// Infer the database engine from a connection URL's scheme.
    pub fn from_url(url: &str) -> Option<DbKind> {
        let scheme = url
            .split("://")
            .next()
            .unwrap_or("")
            .to_ascii_lowercase();
        match scheme.as_str() {
            "sqlite" => Some(DbKind::Sqlite),
            "postgres" | "postgresql" => Some(DbKind::Postgres),
            "mysql" => Some(DbKind::Mysql),
            _ => None,
        }
    }

    /// Human-readable engine name for logging.
    pub fn label(&self) -> &'static str {
        match self {
            DbKind::Sqlite => "SQLite",
            DbKind::Postgres => "PostgreSQL",
            DbKind::Mysql => "MySQL",
        }
    }
}

/// Cache configuration
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct CacheConfig {
    /// 是否启用缓存；缺省 false
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_cache_expire")]
    pub expire: u64,
    #[serde(default = "default_cache_prefix")]
    pub prefix: String,
    /// Connection string (e.g. "redis://[:password@]host:port/db",
    /// "valkey://..."). Can also be provided via the `CACHE_URL` environment
    /// variable. Required when the cache is enabled.
    #[serde(default)]
    pub url: Option<String>,
}

fn default_cache_expire() -> u64 {
    3600
}

fn default_cache_prefix() -> String {
    "shorten:".to_string()
}

/// Cache engine kind, derived from the connection URL scheme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CacheKind {
    Redis,
    Valkey,
}

impl CacheKind {
    /// Infer the cache engine from a connection URL's scheme.
    /// Anything other than `valkey://` is treated as Redis (Valkey is
    /// wire-compatible with the Redis protocol).
    pub fn from_url(url: &str) -> CacheKind {
        let scheme = url
            .split("://")
            .next()
            .unwrap_or("")
            .to_ascii_lowercase();
        match scheme.as_str() {
            "valkey" => CacheKind::Valkey,
            _ => CacheKind::Redis,
        }
    }
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

fn default_geoip_config() -> GeoIpConfig {
    GeoIpConfig {
        enabled: false,
        geoip_type: GeoIpType::Ip2region,
        ip2region: None,
    }
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
        Self::from_file("config.toml")
    }

    /// Load configuration from a specific file path
    ///
    /// The file source is optional: when the file does not exist the
    /// configuration is built from environment variables only
    /// (`SERVER__ADDRESS`, `DATABASE__URL`, ...).
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let config = ConfigBuilder::builder()
            .add_source(
                File::from(path.as_ref())
                    .format(config::FileFormat::Toml)
                    .required(false),
            )
            .add_source(Environment::default().separator("__"))
            .build()?;

        let mut cfg: Config = config.try_deserialize()?;

        // `DATABASE__URL` / `CACHE__URL` / `OIDC__CLIENT_SECRET` are the
        // canonical nested forms, mapped automatically by the `config` crate
        // (separator `__`). The flat `DATABASE_URL` / `CACHE_URL` /
        // `OIDC_CLIENT_SECRET` remain compatibility aliases (e.g. PaaS that
        // only inject the flat var). When both forms are set, the `__` form
        // wins, so the flat fallback only applies when the `__` form is absent.
        if std::env::var("DATABASE__URL").is_err() {
            if let Ok(v) = std::env::var("DATABASE_URL") {
                let v = v.trim().to_string();
                if !v.is_empty() {
                    cfg.database.url = Some(v);
                }
            }
        }
        if std::env::var("CACHE__URL").is_err() {
            if let Ok(v) = std::env::var("CACHE_URL") {
                let v = v.trim().to_string();
                if !v.is_empty() {
                    cfg.cache.url = Some(v);
                }
            }
        }
        if std::env::var("OIDC__CLIENT_SECRET").is_err() {
            if let Ok(v) = std::env::var("OIDC_CLIENT_SECRET") {
                let v = v.trim().to_string();
                if !v.is_empty() {
                    cfg.oidc.client_secret = Some(v);
                }
            }
        }

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
        // 短址专用域名未单独配置时，从监听地址推断（host 为空或通配地址时回退 localhost）
        if self.server.short_url.is_empty() {
            self.server.short_url = infer_short_url(&self.server.address);
        }

        // Shortener defaults
        if self.slug.length == 0 {
            self.slug.length = 6;
        }
        if self.slug.alphabet.is_empty() {
            self.slug.alphabet =
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
        if self.admin.password_hash.is_empty() {
            return Err(ConfigError::Message(
                "admin.password_hash is required (generate with: shortener-server hash-password)"
                    .to_string(),
            ));
        }

        // Validate OIDC configuration
        if self.oidc.enabled {
            if self.oidc.issuer.is_none() || self.oidc.issuer.as_ref().unwrap().is_empty() {
                return Err(ConfigError::Message(
                    "oidc.issuer is required when oidc.enabled is set".to_string(),
                ));
            }
            if self.oidc.client_id.is_none() || self.oidc.client_id.as_ref().unwrap().is_empty() {
                return Err(ConfigError::Message(
                    "oidc.client_id is required when oidc.enabled is set".to_string(),
                ));
            }
            if self.oidc.allow_emails.is_empty() && self.oidc.allow_subjects.is_empty() {
                return Err(ConfigError::Message(
                    "oidc.allow_emails and oidc.allow_subjects cannot both be empty when oidc.enabled is set"
                        .to_string(),
                ));
            }
        }

        // Validate shortener configuration
        if self.slug.length < 4 || self.slug.length > 16 {
            return Err(ConfigError::Message(
                "slug.length must be between 4 and 16".to_string(),
            ));
        }
        if self.slug.alphabet.is_empty() {
            return Err(ConfigError::Message(
                "slug.alphabet cannot be empty".to_string(),
            ));
        }

        // Validate database configuration
        if self
            .database
            .url
            .as_ref()
            .map(|s| s.trim())
            .unwrap_or("")
            .is_empty()
        {
            return Err(ConfigError::Message(
                "database.url is required (set [database] url or DATABASE_URL)".to_string(),
            ));
        }

        // Validate cache configuration
        if self.cache.enabled
            && self
                .cache
                .url
                .as_ref()
                .map(|s| s.trim())
                .unwrap_or("")
                .is_empty()
        {
            return Err(ConfigError::Message(
                "cache.url is required when cache is enabled (set [cache] url or CACHE_URL)"
                    .to_string(),
            ));
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

    /// Get database connection string.
    ///
    /// Returns the configured connection URL (`[database] url` or the
    /// `DATABASE_URL` environment variable). The engine is inferred from the
    /// URL scheme by the caller (see `DbKind`).
    pub fn get_database_url(&self) -> String {
        self.database
            .url
            .as_ref()
            .map(|s| s.trim().to_string())
            .unwrap_or_default()
    }

    /// Get cache connection string.
    ///
    /// Returns `None` when the cache is disabled, otherwise the configured
    /// connection URL (`[cache] url` or the `CACHE_URL` environment variable).
    pub fn get_cache_url(&self) -> Option<String> {
        if !self.cache.enabled {
            return None;
        }

        self.cache.url.as_ref().map(|s| s.trim().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::sync::Mutex;
    use tempfile::NamedTempFile;

    // Serializes tests that read process-global env vars (DATABASE_URL /
    // CACHE_URL) so env-mutating tests cannot pollute URL-asserting tests.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    /// Neutralize ambient OIDC env vars (e.g. direnv/.env may set
    /// `OIDC__ENABLED=true` with empty allowlists, which fails the new
    /// non-empty allowlist validation). These tests do not exercise OIDC, so
    /// the file-provided `[oidc] enabled = false` should take effect.
    fn disable_oidc_env() {
        unsafe {
            std::env::remove_var("OIDC__ENABLED");
            std::env::remove_var("OIDC__ISSUER");
            std::env::remove_var("OIDC__CLIENT_ID");
            std::env::remove_var("OIDC__CLIENT_SECRET");
            std::env::remove_var("OIDC__REDIRECT_URI");
            std::env::remove_var("OIDC__ALLOW_EMAILS");
            std::env::remove_var("OIDC__ALLOW_SUBJECTS");
        }
    }

    fn create_test_config_file(content: &str) -> NamedTempFile {
        disable_oidc_env();
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file.flush().unwrap();
        file
    }

    /// A minimal valid config body (server/admin/shortener/geoip) shared by
    /// tests. Database/cache sections are appended per-test.
    fn base_config() -> String {
        r#"
[server]
address = ":8080"
trusted-platform = ""
api_key = "test-api-key"

[slug]
length = 6
alphabet = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"

[admin]
username = "admin"
password_hash = "admin123"

[geoip]
enabled = false
type = "ip2region"

[oidc]
enabled = false
allow_emails = []
allow_subjects = []
"#
        .to_string()
    }

    #[test]
    fn test_load_valid_config() {
        let _guard = ENV_LOCK.lock().unwrap();
        let config_content = format!(
            "{}\n{}\n{}",
            base_config(),
            "[database]\nlog_level = 1\nurl = \"sqlite://data/test.db?mode=rwc\"\n",
            "[cache]\nenabled = false\nexpire = 3600\nprefix = \"shorten:\"\n"
        );

        let file = create_test_config_file(&config_content);
        let config = Config::from_file(file.path()).unwrap();

        assert_eq!(config.server.address, ":8080");
        assert_eq!(config.server.api_key, "test-api-key");
        assert_eq!(config.slug.length, 6);
        assert_eq!(config.admin.username, "admin");
        assert!(!config.cache.enabled);
        assert_eq!(
            config.get_database_url(),
            "sqlite://data/test.db?mode=rwc"
        );
        assert!(!config.geoip.enabled);
    }

    #[test]
    fn test_config_with_defaults() {
        let config_content = format!(
            "{}\n{}\n{}",
            base_config(),
            "[database]\nlog_level = 0\nurl = \"sqlite://data/test.db\"\n",
            "[cache]\nenabled = false\nexpire = 0\nprefix = \"\"\n"
        );

        let file = create_test_config_file(&config_content);
        let config = Config::from_file(file.path()).unwrap();

        // Check defaults are applied
        assert_eq!(config.server.address, ":8080");
        assert_eq!(config.server.short_url, "http://localhost:8080");
        assert_eq!(config.slug.length, 6);
        assert_eq!(
            config.slug.alphabet,
            "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"
        );
        assert_eq!(config.cache.expire, 3600);
        assert_eq!(config.cache.prefix, "shorten:");
        assert_eq!(config.database.log_level, 1);
    }

    #[test]
    fn test_infer_short_url() {
        // 端口形式：host 为空 → localhost
        assert_eq!(infer_short_url(":8080"), "http://localhost:8080");
        // 通配地址 → localhost
        assert_eq!(infer_short_url("0.0.0.0:8080"), "http://localhost:8080");
        assert_eq!(infer_short_url("[::]:8080"), "http://localhost:8080");
        // 具体 host → 保留原样
        assert_eq!(infer_short_url("127.0.0.1:8080"), "http://127.0.0.1:8080");
        assert_eq!(
            infer_short_url("localhost:3000"),
            "http://localhost:3000"
        );
        // IPv6：保留方括号
        assert_eq!(infer_short_url("[::1]:8080"), "http://[::1]:8080");
        // 无法解析 → 兜底
        assert_eq!(infer_short_url(""), "http://localhost:8080");
    }

    #[test]
    fn test_missing_api_key() {
        let config_content = format!(
            r#"
[server]
address = ":8080"
api_key = ""

[slug]
length = 6
alphabet = "abc"

[admin]
username = "admin"
password_hash = "pass"

[database]
log_level = 1
url = "sqlite://data/test.db"

[cache]
enabled = false
expire = 3600
prefix = "shorten:"
"#
        );

        let file = create_test_config_file(&config_content);
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
        let config_content = format!(
            "{}\n{}\n{}",
            r#"
[server]
address = ":8080"
api_key = "test-key"

[slug]
length = 6
alphabet = "abc"

[admin]
username = ""
password_hash = "pass"
"#,
            "[database]\nlog_level = 1\nurl = \"sqlite://data/test.db\"\n",
            "[cache]\nenabled = false\nexpire = 3600\nprefix = \"shorten:\"\n"
        );

        let file = create_test_config_file(&config_content);
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
    fn test_invalid_length() {
        let config_content = format!(
            "{}\n{}\n{}",
            r#"
[server]
address = ":8080"
api_key = "test-key"

[slug]
length = 20
alphabet = "abc"

[admin]
username = "admin"
password_hash = "pass"
"#,
            "[database]\nlog_level = 1\nurl = \"sqlite://data/test.db\"\n",
            "[cache]\nenabled = false\nexpire = 3600\nprefix = \"shorten:\"\n"
        );

        let file = create_test_config_file(&config_content);
        let result = Config::from_file(file.path());

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("length must be between 4 and 16")
        );
    }

    #[test]
    fn test_postgres_url_config() {
        let _guard = ENV_LOCK.lock().unwrap();
        let config_content = format!(
            "{}\n{}\n{}",
            base_config(),
            "[database]\nlog_level = 1\nurl = \"postgres://postgres:secret@localhost:5432/shortener?sslmode=disable\"\n",
            "[cache]\nenabled = false\nexpire = 3600\nprefix = \"shorten:\"\n"
        );

        let file = create_test_config_file(&config_content);
        let config = Config::from_file(file.path()).unwrap();

        assert_eq!(
            config.get_database_url(),
            "postgres://postgres:secret@localhost:5432/shortener?sslmode=disable"
        );
        assert_eq!(DbKind::from_url(&config.get_database_url()), Some(DbKind::Postgres));
    }

    #[test]
    fn test_mysql_url_config() {
        let _guard = ENV_LOCK.lock().unwrap();
        let config_content = format!(
            "{}\n{}\n{}",
            base_config(),
            "[database]\nlog_level = 1\nurl = \"mysql://root:root@localhost:3306/shortener?charset=utf8mb4\"\n",
            "[cache]\nenabled = false\nexpire = 3600\nprefix = \"shorten:\"\n"
        );

        let file = create_test_config_file(&config_content);
        let config = Config::from_file(file.path()).unwrap();

        assert_eq!(
            config.get_database_url(),
            "mysql://root:root@localhost:3306/shortener?charset=utf8mb4"
        );
        assert_eq!(DbKind::from_url(&config.get_database_url()), Some(DbKind::Mysql));
    }

    #[test]
    fn test_cache_enabled_redis() {
        let _guard = ENV_LOCK.lock().unwrap();
        let config_content = format!(
            "{}\n{}\n{}",
            base_config(),
            "[database]\nlog_level = 1\nurl = \"sqlite://data/test.db\"\n",
            "[cache]\nenabled = true\nexpire = 3600\nprefix = \"shorten:\"\nurl = \"redis://:secret@localhost:6379/1\"\n"
        );

        let file = create_test_config_file(&config_content);
        let config = Config::from_file(file.path()).unwrap();

        assert!(config.cache.enabled);
        assert_eq!(
            config.get_cache_url(),
            Some("redis://:secret@localhost:6379/1".to_string())
        );
        assert_eq!(
            CacheKind::from_url(config.get_cache_url().unwrap().as_str()),
            CacheKind::Redis
        );
    }

    #[test]
    fn test_cache_enabled_valkey() {
        let _guard = ENV_LOCK.lock().unwrap();
        let config_content = format!(
            "{}\n{}\n{}",
            base_config(),
            "[database]\nlog_level = 1\nurl = \"sqlite://data/test.db\"\n",
            "[cache]\nenabled = true\nexpire = 3600\nprefix = \"shorten:\"\nurl = \"valkey://:secret@localhost:6379/0\"\n"
        );

        let file = create_test_config_file(&config_content);
        let config = Config::from_file(file.path()).unwrap();

        assert!(config.cache.enabled);
        assert_eq!(
            config.get_cache_url(),
            Some("valkey://:secret@localhost:6379/0".to_string())
        );
        assert_eq!(
            CacheKind::from_url(config.get_cache_url().unwrap().as_str()),
            CacheKind::Valkey
        );
    }

    #[test]
    fn test_geoip_enabled() {
        let config_content = format!(
            "{}\n{}\n{}",
            r#"
[server]
address = ":8080"
api_key = "test-key"

[slug]
length = 6
alphabet = "abc"

[admin]
username = "admin"
password_hash = "pass"

[geoip]
enabled = true
type = "ip2region"

[geoip.ip2region]
path = "data/ip2region.xdb"
mode = "vector"
version = "4"
"#,
            "[database]\nlog_level = 1\nurl = \"sqlite://data/test.db\"\n",
            "[cache]\nenabled = false\nexpire = 3600\nprefix = \"shorten:\"\n"
        );

        let file = create_test_config_file(&config_content);
        let config = Config::from_file(file.path()).unwrap();

        assert!(config.geoip.enabled);
        assert_eq!(config.geoip.geoip_type, GeoIpType::Ip2region);
        assert!(config.geoip.ip2region.is_some());

        let ip2region = config.geoip.ip2region.as_ref().unwrap();
        assert_eq!(ip2region.version, "4");
    }

    #[test]
    fn test_get_database_url_sqlite() {
        let config_content = format!(
            "{}\n{}\n{}",
            base_config(),
            "[database]\nlog_level = 1\nurl = \"sqlite://data/test.db?mode=rwc\"\n",
            "[cache]\nenabled = false\nexpire = 3600\nprefix = \"shorten:\"\n"
        );

        let file = create_test_config_file(&config_content);
        let config = Config::from_file(file.path()).unwrap();

        let url = config.get_database_url();
        assert_eq!(url, "sqlite://data/test.db?mode=rwc");
        assert_eq!(DbKind::from_url(&url), Some(DbKind::Sqlite));
    }

    #[test]
    fn test_get_database_url_sqlite_memory() {
        let config_content = format!(
            "{}\n{}\n{}",
            base_config(),
            "[database]\nlog_level = 1\nurl = \"sqlite::memory:\"\n",
            "[cache]\nenabled = false\nexpire = 3600\nprefix = \"shorten:\"\n"
        );

        let file = create_test_config_file(&config_content);
        let config = Config::from_file(file.path()).unwrap();

        let url = config.get_database_url();
        assert_eq!(url, "sqlite::memory:");
    }

    #[test]
    fn test_get_cache_url_redis() {
        let config_content = format!(
            "{}\n{}\n{}",
            base_config(),
            "[database]\nlog_level = 1\nurl = \"sqlite://data/test.db\"\n",
            "[cache]\nenabled = true\nexpire = 3600\nprefix = \"shorten:\"\nurl = \"redis://:secret@localhost:6379/1\"\n"
        );

        let file = create_test_config_file(&config_content);
        let config = Config::from_file(file.path()).unwrap();

        let url = config.get_cache_url();
        assert_eq!(url, Some("redis://:secret@localhost:6379/1".to_string()));
    }

    #[test]
    fn test_get_cache_url_disabled() {
        let config_content = format!(
            "{}\n{}\n{}",
            base_config(),
            "[database]\nlog_level = 1\nurl = \"sqlite://data/test.db\"\n",
            "[cache]\nenabled = false\nexpire = 3600\nprefix = \"shorten:\"\nurl = \"redis://localhost:6379/0\"\n"
        );

        let file = create_test_config_file(&config_content);
        let config = Config::from_file(file.path()).unwrap();

        let url = config.get_cache_url();
        assert_eq!(url, None);
    }

    #[test]
    fn test_missing_database_url() {
        let _guard = ENV_LOCK.lock().unwrap();
        let config_content = format!(
            "{}\n{}\n{}",
            base_config(),
            "[database]\nlog_level = 1\n",
            "[cache]\nenabled = false\nexpire = 3600\nprefix = \"shorten:\"\n"
        );

        let file = create_test_config_file(&config_content);
        let result = Config::from_file(file.path());

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("database.url is required")
        );
    }

    #[test]
    fn test_missing_cache_url_when_enabled() {
        let _guard = ENV_LOCK.lock().unwrap();
        let config_content = format!(
            "{}\n{}\n{}",
            base_config(),
            "[database]\nlog_level = 1\nurl = \"sqlite://data/test.db\"\n",
            "[cache]\nenabled = true\nexpire = 3600\nprefix = \"shorten:\"\n"
        );

        let file = create_test_config_file(&config_content);
        let result = Config::from_file(file.path());

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("cache.url is required")
        );
    }

    #[test]
    fn test_database_url_from_env() {
        let _guard = ENV_LOCK.lock().unwrap();
        unsafe {
            std::env::set_var("DATABASE_URL", "postgres://u:p@host:5432/db");
        }
        let config_content = format!(
            "{}\n{}\n{}",
            base_config(),
            "[database]\nlog_level = 1\n",
            "[cache]\nenabled = false\nexpire = 3600\nprefix = \"shorten:\"\n"
        );

        let file = create_test_config_file(&config_content);
        let config = Config::from_file(file.path()).unwrap();
        unsafe {
            std::env::remove_var("DATABASE_URL");
        }

        assert_eq!(config.get_database_url(), "postgres://u:p@host:5432/db");
    }

    #[test]
    fn test_database_url_nested_precedence_over_flat() {
        let _guard = ENV_LOCK.lock().unwrap();
        unsafe {
            std::env::set_var("DATABASE__URL", "postgres://nested:host:5432/db");
            std::env::set_var("DATABASE_URL", "postgres://flat:host:5432/db");
        }
        let config_content = format!(
            "{}\n{}\n{}",
            base_config(),
            "[database]\nlog_level = 1\n",
            "[cache]\nenabled = false\nexpire = 3600\nprefix = \"shorten:\"\n"
        );

        let file = create_test_config_file(&config_content);
        let config = Config::from_file(file.path()).unwrap();
        unsafe {
            std::env::remove_var("DATABASE__URL");
            std::env::remove_var("DATABASE_URL");
        }

        // The nested `__` form wins over the flat alias.
        assert_eq!(
            config.get_database_url(),
            "postgres://nested:host:5432/db"
        );
    }

    #[test]
    fn test_cache_url_from_env() {
        let _guard = ENV_LOCK.lock().unwrap();
        unsafe {
            std::env::set_var("CACHE_URL", "redis://:pw@host:6379/2");
        }
        let config_content = format!(
            "{}\n{}\n{}",
            base_config(),
            "[database]\nlog_level = 1\nurl = \"sqlite://data/test.db\"\n",
            "[cache]\nenabled = true\nexpire = 3600\nprefix = \"shorten:\"\n"
        );

        let file = create_test_config_file(&config_content);
        let config = Config::from_file(file.path()).unwrap();
        unsafe {
            std::env::remove_var("CACHE_URL");
        }

        assert_eq!(
            config.get_cache_url(),
            Some("redis://:pw@host:6379/2".to_string())
        );
    }

    #[test]
    fn test_cache_url_nested_precedence_over_flat() {
        let _guard = ENV_LOCK.lock().unwrap();
        unsafe {
            std::env::set_var("CACHE__URL", "redis://nested:host:6379/3");
            std::env::set_var("CACHE_URL", "redis://flat:host:6379/2");
        }
        let config_content = format!(
            "{}\n{}\n{}",
            base_config(),
            "[database]\nlog_level = 1\nurl = \"sqlite://data/test.db\"\n",
            "[cache]\nenabled = true\nexpire = 3600\nprefix = \"shorten:\"\n"
        );

        let file = create_test_config_file(&config_content);
        let config = Config::from_file(file.path()).unwrap();
        unsafe {
            std::env::remove_var("CACHE__URL");
            std::env::remove_var("CACHE_URL");
        }

        // The nested `__` form wins over the flat alias.
        assert_eq!(
            config.get_cache_url(),
            Some("redis://nested:host:6379/3".to_string())
        );
    }
}
