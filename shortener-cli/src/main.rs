mod client;
mod config;

use clap::Parser;
use client::{ApiClient, CreateShortenRequest, ListParams, UpdateShortenRequest};
use config::CliConfig;

#[derive(Parser)]
#[command(name = "shortener-cli")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(about = "Short URL management CLI tool", long_about = None)]
struct Cli {
    /// Server URL (can also be set via SHORTENER_URL env var)
    #[arg(short = 'u', long, env = "SHORTENER_URL", global = true)]
    url: Option<String>,

    /// API Key (can also be set via SHORTENER_KEY env var)
    #[arg(short = 'k', long, env = "SHORTENER_KEY", global = true)]
    key: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Initialize configuration file
    Init {
        /// Server URL to save in config file
        #[arg(short = 'u', long)]
        url: Option<String>,

        /// API Key to save in config file
        #[arg(short = 'k', long)]
        key: Option<String>,
    },
    /// Show environment variables and configuration
    Env,
    /// Show version information
    Version,
    /// Create a new short URL
    Create {
        /// Original URL to shorten
        original_url: String,

        /// Custom short code (optional)
        #[arg(short = 'c', long)]
        code: Option<String>,

        /// Description for the short URL (optional)
        #[arg(short = 'd', long)]
        desc: Option<String>,
    },
    /// Get details of a short URL
    Get {
        /// Short code to retrieve
        code: String,
    },
    /// List short URLs
    List {
        /// Fetch all short URLs (auto-paginate)
        #[arg(short = 'a', long)]
        all: bool,

        /// Page number (default: 1)
        #[arg(short = 'p', long)]
        page: Option<u64>,

        /// Page size (default: 10)
        #[arg(short = 'z', long)]
        psize: Option<u64>,

        /// Sort field (e.g., created_at, updated_at)
        #[arg(short = 's', long)]
        sort: Option<String>,

        /// Sort order (asc or desc)
        #[arg(short = 'o', long)]
        order: Option<String>,

        /// Filter by status (0=enabled, 1=disabled)
        #[arg(short = 't', long)]
        status: Option<i32>,
    },
    /// Update a short URL
    Update {
        /// Short code to update
        code: String,

        /// New original URL
        #[arg(short = 'o', long)]
        ourl: Option<String>,

        /// New description
        #[arg(short = 'd', long)]
        desc: Option<String>,

        /// New status (0=enabled, 1=disabled)
        #[arg(short = 's', long)]
        status: Option<i32>,
    },
    /// Delete a short URL
    Delete {
        /// Short code to delete
        code: String,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Some(Commands::Init { url, key }) => handle_init(url, key),
        Some(Commands::Env) => {
            CliConfig::display_env_info(cli.url, cli.key);
            Ok(())
        }
        Some(Commands::Version) => {
            println!("shortener-cli version {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Some(Commands::Create {
            original_url,
            code,
            desc,
        }) => handle_create(cli.url, cli.key, original_url, code, desc).await,
        Some(Commands::Get { code }) => handle_get(cli.url, cli.key, code).await,
        Some(Commands::List {
            all,
            page,
            psize,
            sort,
            order,
            status,
        }) => {
            let options = ListOptions {
                all,
                page,
                psize,
                sort,
                order,
                status,
            };
            handle_list(cli.url, cli.key, options).await
        }
        Some(Commands::Update {
            code,
            ourl,
            desc,
            status,
        }) => handle_update(cli.url, cli.key, code, ourl, desc, status).await,
        Some(Commands::Delete { code }) => handle_delete(cli.url, cli.key, code).await,
        None => {
            println!("Shortener CLI - Rust implementation");
            println!("Use --help for more information");
            Ok(())
        }
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn handle_init(url: Option<String>, key: Option<String>) -> anyhow::Result<()> {
    println!("Initializing configuration...");

    let config_path = CliConfig::init(url.clone(), key.clone())?;

    println!("✓ Configuration file created at: {}", config_path.display());
    println!();
    println!("Configuration:");
    println!(
        "  URL: {}",
        url.unwrap_or_else(|| "http://localhost:8080".to_string())
    );
    println!(
        "  Key: {}",
        if key.is_some() { "****" } else { "(not set)" }
    );
    println!();
    println!("You can also set these values using environment variables:");
    println!("  export SHORTENER_URL=http://your-server:8080");
    println!("  export SHORTENER_KEY=your-api-key");
    println!();
    println!("Or pass them as command line arguments:");
    println!("  shortener-cli --url http://your-server:8080 --key your-api-key <command>");

    Ok(())
}

async fn handle_create(
    url_arg: Option<String>,
    key_arg: Option<String>,
    original_url: String,
    code: Option<String>,
    desc: Option<String>,
) -> anyhow::Result<()> {
    let config = CliConfig::load(url_arg, key_arg)?;
    let client = ApiClient::new(config.url, config.key);

    let request = CreateShortenRequest {
        original_url,
        code,
        describe: desc,
    };

    let response = client.create_shorten(request).await?;

    println!("✓ Short URL created successfully!");
    println!();
    print_shorten_details(&response);

    Ok(())
}

async fn handle_get(
    url_arg: Option<String>,
    key_arg: Option<String>,
    code: String,
) -> anyhow::Result<()> {
    let config = CliConfig::load(url_arg, key_arg)?;
    let client = ApiClient::new(config.url, config.key);

    let response = client.get_shorten(&code).await?;

    print_shorten_details(&response);

    Ok(())
}

struct ListOptions {
    all: bool,
    page: Option<u64>,
    psize: Option<u64>,
    sort: Option<String>,
    order: Option<String>,
    status: Option<i32>,
}

async fn handle_list(
    url_arg: Option<String>,
    key_arg: Option<String>,
    options: ListOptions,
) -> anyhow::Result<()> {
    let config = CliConfig::load(url_arg, key_arg)?;
    let client = ApiClient::new(config.url, config.key);

    if options.all {
        // Fetch all pages
        let mut all_items = Vec::new();
        let mut current_page = 1u64;
        let page_size = options.psize.unwrap_or(50); // Use larger page size for --all

        loop {
            let params = ListParams {
                page: Some(current_page),
                page_size: Some(page_size),
                status: options.status,
                sort: options.sort.clone(),
                order: options.order.clone(),
            };

            let response = client.list_shortens(params).await?;
            all_items.extend(response.data);

            if current_page >= response.meta.total_pages {
                break;
            }
            current_page += 1;
        }

        println!("Total: {} short URLs", all_items.len());
        println!();
        print_shorten_table(&all_items);
    } else {
        // Fetch single page
        let params = ListParams {
            page: options.page,
            page_size: options.psize,
            status: options.status,
            sort: options.sort,
            order: options.order,
        };

        let response = client.list_shortens(params).await?;

        println!(
            "Page {}/{} (showing {} of {} total)",
            response.meta.page,
            response.meta.total_pages,
            response.meta.current_count,
            response.meta.total_items
        );
        println!();
        print_shorten_table(&response.data);
    }

    Ok(())
}

async fn handle_update(
    url_arg: Option<String>,
    key_arg: Option<String>,
    code: String,
    ourl: Option<String>,
    desc: Option<String>,
    status: Option<i32>,
) -> anyhow::Result<()> {
    let config = CliConfig::load(url_arg, key_arg)?;
    let client = ApiClient::new(config.url, config.key);

    let request = UpdateShortenRequest {
        original_url: ourl,
        describe: desc,
        status,
    };

    let response = client.update_shorten(&code, request).await?;

    println!("✓ Short URL updated successfully!");
    println!();
    print_shorten_details(&response);

    Ok(())
}

async fn handle_delete(
    url_arg: Option<String>,
    key_arg: Option<String>,
    code: String,
) -> anyhow::Result<()> {
    let config = CliConfig::load(url_arg, key_arg)?;
    let client = ApiClient::new(config.url, config.key);

    client.delete_shorten(&code).await?;

    println!("✓ Short URL '{}' deleted successfully!", code);

    Ok(())
}

// ============================================================================
// Output Formatting Functions (Task 15.3)
// ============================================================================

use client::ShortenResponse;
use tabled::{Table, Tabled, settings::Style};

/// Print detailed information about a single short URL
fn print_shorten_details(shorten: &ShortenResponse) {
    println!("ID:           {}", shorten.id);
    println!("Code:         {}", shorten.code);
    println!("Short URL:    {}", shorten.short_url);
    println!("Original URL: {}", shorten.original_url);
    println!(
        "Description:  {}",
        shorten.describe.as_deref().unwrap_or("(none)")
    );
    println!(
        "Status:       {} ({})",
        shorten.status,
        status_name(shorten.status)
    );
    println!("Created:      {}", shorten.created_at);
    println!("Updated:      {}", shorten.updated_at);
}

/// Print a table of short URLs
fn print_shorten_table(shortens: &[ShortenResponse]) {
    if shortens.is_empty() {
        println!("No short URLs found.");
        return;
    }

    // Create a simplified view for table display
    #[derive(Tabled)]
    struct ShortenRow {
        #[tabled(rename = "ID")]
        id: i64,
        #[tabled(rename = "Code")]
        code: String,
        #[tabled(rename = "Original URL")]
        original_url: String,
        #[tabled(rename = "Status")]
        status: String,
        #[tabled(rename = "Created")]
        created_at: String,
    }

    let rows: Vec<ShortenRow> = shortens
        .iter()
        .map(|s| ShortenRow {
            id: s.id,
            code: s.code.clone(),
            original_url: s.original_url.clone(),
            status: status_name(s.status).to_string(),
            created_at: format_datetime(&s.created_at),
        })
        .collect();

    let mut table = Table::new(rows);
    table.with(Style::rounded());
    println!("{}", table);
}

/// Get human-readable status name
fn status_name(status: i32) -> &'static str {
    match status {
        0 => "Enabled",
        1 => "Disabled",
        _ => "Unknown",
    }
}

/// Truncate a string to a maximum length with ellipsis
#[allow(dead_code)]
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

/// Format datetime string for display (extract date and time)
fn format_datetime(dt: &str) -> String {
    // Input format: "2024-01-15T10:30:45Z" or similar
    // Output format: "2024-01-15 10:30"
    if let Some(t_pos) = dt.find('T') {
        let date = &dt[..t_pos];
        let time_part = &dt[t_pos + 1..];
        if let Some(colon_pos) = time_part.rfind(':') {
            let time = &time_part[..colon_pos];
            return format!("{} {}", date, time);
        }
    }
    dt.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_name() {
        assert_eq!(status_name(0), "Enabled");
        assert_eq!(status_name(1), "Disabled");
        assert_eq!(status_name(99), "Unknown");
    }

    #[test]
    fn test_truncate_string() {
        assert_eq!(truncate_string("short", 10), "short");
        assert_eq!(
            truncate_string("this is a very long string", 10),
            "this is..."
        );
        assert_eq!(truncate_string("exactly10!", 10), "exactly10!");
    }

    #[test]
    fn test_format_datetime() {
        assert_eq!(format_datetime("2024-01-15T10:30:45Z"), "2024-01-15 10:30");
        assert_eq!(
            format_datetime("2024-01-15T10:30:45.123Z"),
            "2024-01-15 10:30"
        );
        assert_eq!(format_datetime("invalid"), "invalid");
    }

    #[test]
    fn test_print_shorten_details() {
        let shorten = ShortenResponse {
            id: 1,
            code: "test123".to_string(),
            short_url: "http://localhost:8080/test123".to_string(),
            original_url: "https://example.com".to_string(),
            describe: Some("Test URL".to_string()),
            status: 1,
            created_at: "2024-01-15T10:30:45Z".to_string(),
            updated_at: "2024-01-15T10:30:45Z".to_string(),
        };

        // This test just verifies the function doesn't panic
        print_shorten_details(&shorten);
    }

    #[test]
    fn test_print_shorten_table_empty() {
        // This test just verifies the function doesn't panic with empty list
        print_shorten_table(&[]);
    }

    #[test]
    fn test_print_shorten_table_with_data() {
        let shortens = vec![
            ShortenResponse {
                id: 1,
                code: "test1".to_string(),
                short_url: "http://localhost:8080/test1".to_string(),
                original_url: "https://example.com/1".to_string(),
                describe: Some("Test 1".to_string()),
                status: 1,
                created_at: "2024-01-15T10:30:45Z".to_string(),
                updated_at: "2024-01-15T10:30:45Z".to_string(),
            },
            ShortenResponse {
                id: 2,
                code: "test2".to_string(),
                short_url: "http://localhost:8080/test2".to_string(),
                original_url: "https://example.com/2".to_string(),
                describe: None,
                status: 2,
                created_at: "2024-01-15T11:30:45Z".to_string(),
                updated_at: "2024-01-15T11:30:45Z".to_string(),
            },
        ];

        // This test just verifies the function doesn't panic
        print_shorten_table(&shortens);
    }
}
