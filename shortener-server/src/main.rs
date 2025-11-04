use clap::{Parser, Subcommand};
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
    #[arg(short, long, default_value = "config/config.toml", global = true)]
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
        }
    }

    // Load configuration first (before logging initialization)
    // Try default path first, fallback to config.toml if not exists
    let config_path = if std::path::Path::new(&args.config).exists() {
        args.config.clone()
    } else if args.config == "config/config.toml" && std::path::Path::new("config.toml").exists() {
        "config.toml".to_string()
    } else {
        args.config.clone()
    };

    let config = match Config::from_file(&config_path) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!(
                "✗ Failed to load configuration from '{}': {}",
                config_path, e
            );
            if config_path != args.config {
                eprintln!("  (fallback from '{}')", args.config);
            }
            std::process::exit(1);
        }
    };

    // Initialize logging with configuration
    if let Err(e) = config.logging.init() {
        eprintln!("✗ Failed to initialize logging: {}", e);
        std::process::exit(1);
    }

    info!("Shortener Server v{}", env!("CARGO_PKG_VERSION"));
    info!("Configuration loaded from: {}", config_path);

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

    // 创建 TCP listener
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => listener,
        Err(e) => {
            error!("Failed to bind to {}: {}", addr, e);
            std::process::exit(1);
        }
    };

    info!("Server listening on http://{}", addr);
    info!("Site URL: {}", config.server.site_url);
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
    const DEFAULT_CONFIG: &str = include_str!("../../config/config.toml");
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
            println!("     - admin.password");
            println!("     - server.site_url (your public URL)");
            println!("  2. Run the server: shortener-server");
        }
        Err(e) => {
            eprintln!("✗ Failed to create '{}': {}", CONFIG_FILE, e);
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
