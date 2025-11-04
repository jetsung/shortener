use clap::Parser;
use shortener_server::{
    cache::create_cache,
    config::Config,
    db::DbFactory,
    geoip::create_geoip,
    repositories::{HistoryRepositoryImpl, UrlRepositoryImpl},
    router::{AppState, create_router},
    services::{HistoryService, ShortenService},
};
use std::sync::Arc;
use tokio::signal;
use tracing::{error, info};

/// Shortener Server
#[derive(Parser)]
#[command(name = "shortener-server")]
#[command(about = "A URL shortener service")]
#[command(version)]
struct Args {
    /// Configuration file path
    #[arg(short, long, default_value = "config/config.toml")]
    config: String,
}

#[tokio::main]
async fn main() {
    // Parse command line arguments
    let args = Args::parse();

    // Load configuration first (before logging initialization)
    let config = match Config::from_file(&args.config) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("✗ Failed to load configuration from '{}': {}", args.config, e);
            std::process::exit(1);
        }
    };

    // Initialize logging with configuration
    if let Err(e) = config.logging.init() {
        eprintln!("✗ Failed to initialize logging: {}", e);
        std::process::exit(1);
    }

    info!("========================================");
    info!("Shortener Server");
    info!("========================================");
    info!("Version: {}", env!("CARGO_PKG_VERSION"));
    info!("✓ Configuration loaded successfully");
    info!(
        "✓ Logging initialized (level: {}, format: {:?})",
        config.logging.level, config.logging.format
    );

    // 显示配置信息
    display_config_info(&config);

    // 初始化数据库
    let db = match DbFactory::create_connection(&config).await {
        Ok(connection) => {
            info!("✓ Database connection established");
            connection
        }
        Err(e) => {
            error!("✗ Failed to connect to database: {}", e);
            std::process::exit(1);
        }
    };

    // 运行数据库迁移
    if let Err(e) = DbFactory::run_migrations(&db).await {
        error!("✗ Failed to run database migrations: {}", e);
        std::process::exit(1);
    }
    info!("✓ Database migrations completed");

    // 初始化缓存
    let cache = create_cache(&config.cache).await;
    info!("✓ Cache initialized");

    // 初始化 GeoIP
    let geoip = create_geoip(&config.geoip).await;
    if geoip.is_some() {
        info!("✓ GeoIP initialized");
    }

    // 初始化 repositories
    let url_repo = Arc::new(UrlRepositoryImpl::new(db.clone()));
    let history_repo = Arc::new(HistoryRepositoryImpl::new(db));

    // 初始化 services
    let shorten_service = Arc::new(ShortenService::new(
        url_repo,
        cache,
        config.shortener.clone(),
        config.server.site_url.clone(),
    ));

    let history_service = Arc::new(HistoryService::new(history_repo, geoip));

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
            // 如果解析失败，尝试添加默认 IP
            format!("0.0.0.0{}", config.server.address)
                .parse()
                .unwrap_or_else(|_| {
                    error!("✗ Invalid server address: {}", config.server.address);
                    std::process::exit(1);
                })
        });

    info!("========================================");
    info!("Server Configuration:");
    info!("  Address: {}", addr);
    info!("  Site URL: {}", config.server.site_url);
    info!("  API Key: {}", mask_api_key(&config.server.api_key));
    info!("  Admin User: {}", config.admin.username);
    info!("========================================");
    info!("Starting HTTP server...");

    // 创建 TCP listener
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => {
            info!("✓ Server listening on {}", addr);
            listener
        }
        Err(e) => {
            error!("✗ Failed to bind to {}: {}", addr, e);
            std::process::exit(1);
        }
    };

    info!("========================================");
    info!("Server is ready to accept connections!");
    info!("Press Ctrl+C to shutdown");
    info!("========================================");

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

/// Display configuration information
fn display_config_info(config: &Config) {
    info!("Configuration:");
    info!("  Database: {:?}", config.database.db_type);
    info!(
        "  Cache: {} ({:?})",
        if config.cache.enabled {
            "enabled"
        } else {
            "disabled"
        },
        if config.cache.enabled {
            Some(&config.cache.cache_type)
        } else {
            None
        }
    );
    info!(
        "  GeoIP: {} ({:?})",
        if config.geoip.enabled {
            "enabled"
        } else {
            "disabled"
        },
        if config.geoip.enabled {
            Some(&config.geoip.geoip_type)
        } else {
            None
        }
    );
    info!("  Short code length: {}", config.shortener.code_length);
}

/// Mask API key for display (show first 4 and last 4 characters)
fn mask_api_key(api_key: &str) -> String {
    if api_key.len() < 8 {
        "*".repeat(api_key.len())
    } else {
        format!("{}...{}", &api_key[..4], &api_key[api_key.len() - 4..])
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
            info!("Received Ctrl+C signal, shutting down gracefully...");
        },
        _ = terminate => {
            info!("Received terminate signal, shutting down gracefully...");
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_api_key() {
        assert_eq!(mask_api_key("12345678"), "1234...5678");
        assert_eq!(mask_api_key("1234567890abcdef"), "1234...cdef");
        assert_eq!(mask_api_key("short"), "*****");
        assert_eq!(mask_api_key(""), "");
    }
}
