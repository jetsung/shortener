use clap::{Parser, Subcommand};
use shortener_server::{
    cache::create_cache,
    config::Config,
    db::DbFactory,
    geoip::create_geoip,
    jwt,
    repositories::{HistoryRepositoryImpl, UrlRepositoryImpl},
    router::{AppState, create_router},
    services::{HistoryService, ShortenService},
};
use std::sync::Arc;
use tokio::signal;
use tracing::{error, info, warn};

/// Shortener Server
#[derive(Parser)]
#[command(name = "shortener-server")]
#[command(about = "A URL shortener service")]
#[command(version)]
struct Args {
    /// Configuration file path
    #[arg(short, long, default_value = "config.toml", global = true)]
    config: String,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new config.toml file
    Init {
        /// Force overwrite if config.toml already exists
        #[arg(short, long)]
        force: bool,
    },
    /// Generate an Argon2id password hash (for [admin] password_hash)
    HashPassword {
        /// Plaintext password. If omitted, you will be prompted interactively
        /// (input is not echoed and not stored in shell history).
        #[arg(short, long)]
        password: Option<String>,
    },
}

#[tokio::main]
async fn main() {
    // Parse command line arguments
    let args = Args::parse();

    // Handle subcommands
    if let Some(command) = args.command {
        match command {
            Commands::Init { force } => {
                handle_init_command(force);
                return;
            }
            Commands::HashPassword { password } => {
                handle_hash_password_command(password);
                return;
            }
        }
    }

    // JWT secret is required for signing/verifying tokens.
    if let Err(e) = jwt::resolve_secret() {
        eprintln!("✗ {}", e);
        eprintln!("  Set JWT_SECRET to the secret, or JWT_SECRET_FILE to a file containing it.");
        eprintln!("  Generate one with: openssl rand -base64 48");
        std::process::exit(1);
    }

    // Load configuration first (before logging initialization)
    let config = match Config::from_file(&args.config) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!(
                "✗ Failed to load configuration from '{}': {}",
                args.config, e
            );
            std::process::exit(1);
        }
    };

    // Initialize logging with configuration
    if let Err(e) = config.logging.init() {
        eprintln!("✗ Failed to initialize logging: {}", e);
        std::process::exit(1);
    }

    info!("Shortener Server v{}", env!("CARGO_PKG_VERSION"));
    if std::path::Path::new(&args.config).exists() {
        info!("Configuration loaded from: {}", args.config);
    } else {
        info!(
            "Configuration file '{}' not found, using environment variables only",
            args.config
        );
    }

    // 初始化数据库
    let db = match DbFactory::create_connection(&config).await {
        Ok(connection) => connection,
        Err(e) => {
            error!("Failed to connect to database: {}", e);
            std::process::exit(1);
        }
    };

    // 运行数据库迁移
    if let Err(e) = DbFactory::run_migrations(&db).await {
        error!("Failed to run database migrations: {}", e);
        std::process::exit(1);
    }

    // 初始化缓存
    let cache = create_cache(&config.cache).await;

    // 初始化 GeoIP
    let geoip = create_geoip(&config.geoip).await;

    // 初始化 repositories
    let url_repo = Arc::new(UrlRepositoryImpl::new(db.clone()));
    let history_repo = Arc::new(HistoryRepositoryImpl::new(db));

    // 初始化 services
    let shorten_service = Arc::new(ShortenService::new(
        url_repo,
        cache.clone(),
        config.slug.clone(),
        config.server.short_url.clone(),
    ));

    let history_service = Arc::new(HistoryService::new(history_repo, geoip));

    // 启动时重建缓存：先清空前缀下的旧键，再从数据库预热全量短链，
    // 保证缓存与数据库内容一致（仅在真实缓存连接成功时执行）
    if !cache.is_null() {
        match cache.clear_prefix(&config.cache.prefix).await {
            Ok(n) => info!(
                "Cleared {} stale cache keys with prefix '{}'",
                n, config.cache.prefix
            ),
            Err(e) => warn!("Failed to clear stale cache keys: {}", e),
        }
        match shorten_service.warm_up_cache().await {
            Ok(n) => info!("Cache warmed up with {} short URLs", n),
            Err(e) => warn!("Failed to warm up cache: {}", e),
        }
    }

    // 创建应用状态
    let state = AppState {
        shorten_service,
        history_service,
        config: Arc::new(config.clone()),
    };

    // 创建路由
    let app = create_router(state);

    // 解析监听地址
    let addr = config
        .server
        .address
        .parse::<std::net::SocketAddr>()
        .unwrap_or_else(|_| {
            // 如果解析失败，检查是否只是端口号（如 ":8080"）
            if config.server.address.starts_with(':') {
                format!("0.0.0.0{}", config.server.address)
                    .parse()
                    .unwrap_or_else(|_| {
                        error!("✗ Invalid server address: {}", config.server.address);
                        std::process::exit(1);
                    })
            } else {
                error!("✗ Invalid server address: {}", config.server.address);
                std::process::exit(1);
            }
        });

    // 创建 TCP listener
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(e) => {
            error!("Failed to bind to {}: {}", addr, e);
            std::process::exit(1);
        }
    };

    info!("Server listening on http://{}", addr);
    info!("Short URL: {}", config.server.short_url);
    info!("Admin: {}", config.admin.username);

    // 启动服务器并处理优雅关闭
    if let Err(e) = axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
    {
        error!("✗ Server error: {}", e);
        std::process::exit(1);
    }

    info!("Server shutdown complete");
}

/// Handle init command to create config.toml
fn handle_init_command(force: bool) {
    const DEFAULT_CONFIG: &str = r#"# Shortener Server Configuration
# This is the default configuration file
# Edit this file directly; environment variables (e.g. LOGGING__LEVEL) override values here

# ============================================================================
# Server Configuration
# ============================================================================
[server]
# Server listen address (use 0.0.0.0 for Docker/production to accept external connections)
address = "0.0.0.0:8080"

# Trusted platform header for getting real client IP (optional)
trusted-platform = ""

# Short URL base (短址专用域名，用于生成短链接；未设置时回退到默认值)
short_url = ""

# API key for authentication (REQUIRED)
# Generate with: openssl rand -base64 32
# IMPORTANT: Change this in production!
api_key = "your-secret-api-key-change-me"

# ============================================================================
# Shortener Configuration
# ============================================================================
[slug]
# Length of the generated short slug (4-16)
length = 6

# Characters (alphabet) used when generating a short slug
alphabet = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ"

# ============================================================================
# Admin Account Configuration
# ============================================================================
[admin]
# Admin username (REQUIRED)
username = "admin"

# Admin password hash (REQUIRED) - Argon2id PHC string.
# Generate with: shortener-server hash-password "your-secure-password"
# IMPORTANT: Change this in production!
password_hash = ""

# ============================================================================
# OIDC / OAuth2.0 Configuration (optional, for SSO login)
# ============================================================================
[oidc]
# IdP issuer URL, e.g. "https://keycloak.example.com/realms/main".
# Leave empty to disable OIDC login.
issuer = ""

# OAuth2 client credentials registered with the IdP.
client_id = ""
# client_secret can also be set via the OIDC__CLIENT_SECRET environment variable.
client_secret = ""

# Allowlist: only users whose email OR subject (sub) matches may log in.
# At least one of allow_emails / allow_subjects must be non-empty.
# The OIDC callback URL is derived from the request Host header, so no
# redirect_uri needs to be configured here.
allow_emails = []
allow_subjects = []

# ============================================================================
# Database Configuration
# ============================================================================
[database]
# Database connection string (REQUIRED). The engine is inferred from the scheme:
#   sqlite://   e.g. "sqlite://data/shortener.db?mode=rwc" or "sqlite::memory:"
#   postgres:// e.g. "postgres://user:pass@host:5432/shortener?sslmode=disable"
#   mysql://    e.g. "mysql://user:pass@host:3306/shortener?charset=utf8mb4"
# Can also be provided via the DATABASE__URL environment variable.
url = "sqlite://data/shortener.db?mode=rwc"

# Database log level: 1=Silent, 2=Error, 3=Warn, 4=Info
log_level = 1

# ============================================================================
# Cache Configuration
# ============================================================================
[cache]
# Enable caching (recommended for production)
enabled = false

# Cache connection string (required when enabled). The engine is inferred from
# the scheme: "redis://" or "valkey://". Can also be provided via CACHE__URL.
# Examples:
#   redis://[:password@]host:port/db
#   valkey://[:password@]host:port/db
url = "redis://localhost:6379/0"

# Cache expiration time in seconds (default: 1 hour)
expire = 3600

# Cache key prefix
prefix = "shorten:"

# ============================================================================
# GeoIP Configuration
# ============================================================================
[geoip]
# Enable GeoIP lookup for visitor location tracking
# NOTE: Requires ip2region database file (see docs for setup instructions)
enabled = false

# GeoIP provider type (currently only "ip2region" is supported)
type = "ip2region"

# ----------------------------------------------------------------------------
# ip2region Configuration
# ----------------------------------------------------------------------------
[geoip.ip2region]
# Path to ip2region.xdb database file
# Download from: https://github.com/lionsoul2014/ip2region/raw/master/data/ip2region_v4.xdb
path = "data/ip2region.xdb"

# Search mode: "vector" (fastest), "btree" (balanced), or "binary" (smallest memory)
mode = "vector"

# IP version: "4" for IPv4, "6" for IPv6
version = "4"

# ============================================================================
# Logging Configuration
# ============================================================================
[logging]
# Log level: "error", "warn", "info", "debug", "trace"
level = "info"

# Log format: "json", "pretty", or "compact"
format = "json"

# Include timestamp in logs
with_timestamp = true

# Include module/target name in logs
with_target = true

# Include thread ID in logs
with_thread_ids = false

# Include thread name in logs
with_thread_names = false

# Include source file name in logs
with_file = false

# Include line number in logs
with_line_number = false

# Use ANSI colors in logs (disable for log files)
with_ansi = true
"#;
    const CONFIG_FILE: &str = "config.toml";
    const DATA_DIR: &str = "data";

    // Check if file already exists
    if std::path::Path::new(CONFIG_FILE).exists() && !force {
        eprintln!("✗ File '{}' already exists", CONFIG_FILE);
        eprintln!("  Use --force to overwrite");
        std::process::exit(1);
    }

    // Create data directory if not exists
    if !std::path::Path::new(DATA_DIR).exists() {
        if let Err(e) = std::fs::create_dir(DATA_DIR) {
            eprintln!("✗ Failed to create '{}' directory: {}", DATA_DIR, e);
            std::process::exit(1);
        }
        println!("✓ Created '{}' directory", DATA_DIR);
    }

    // Write config file
    match std::fs::write(CONFIG_FILE, DEFAULT_CONFIG) {
        Ok(_) => {
            println!("✓ Created '{}'", CONFIG_FILE);
            println!();
            println!("Next steps:");
            println!("  1. Edit '{}' and update the following:", CONFIG_FILE);
            println!("     - server.api_key (generate with: openssl rand -base64 32)");
            println!("     - admin.password_hash (generate with: shortener-server hash-password \"your-password\")");
            println!("     - server.short_url (your public URL; optional, defaults to http://localhost:8080)");
            println!("     - JWT_SECRET environment variable (generate with: openssl rand -base64 48)");
            println!("       or JWT_SECRET_FILE pointing at a file holding the secret");
            println!("  2. (Optional) Configure [oidc] for SSO login");
            println!("  3. Run the server: shortener-server");
        }
        Err(e) => {
            eprintln!("✗ Failed to create '{}': {}", CONFIG_FILE, e);
            std::process::exit(1);
        }
    }
}

/// Handle the `hash-password` subcommand: generate an Argon2id (PHC) hash.
fn handle_hash_password_command(password: Option<String>) {
    use std::io::{IsTerminal, Write};

    let plaintext = match password {
        Some(p) => p,
        None => {
            if !std::io::stdin().is_terminal() {
                eprintln!("✗ No password provided and stdin is not a terminal");
                std::process::exit(1);
            }
            print!("Enter password: ");
            let _ = std::io::stdout().flush();
            let mut buf = String::new();
            if std::io::stdin().read_line(&mut buf).is_err() {
                eprintln!("✗ Failed to read password");
                std::process::exit(1);
            }
            buf.trim_end().to_string()
        }
    };

    if plaintext.is_empty() {
        eprintln!("✗ Password must not be empty");
        std::process::exit(1);
    }

    match shortener_server::handlers::account::hash_password(&plaintext) {
        Ok(hash) => {
            println!("{}", hash);
            println!(
                "\nCopy the line above into your config as: password_hash = \"{}\"",
                hash
            );
        }
        Err(e) => {
            eprintln!("✗ Failed to hash password: {}", e);
            std::process::exit(1);
        }
    }
}

/// Handle graceful shutdown signal
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Shutting down gracefully...");
        },
        _ = terminate => {
            info!("Shutting down gracefully...");
        },
    }
}
