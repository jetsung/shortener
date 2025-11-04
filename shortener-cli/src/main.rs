mod client;
mod config;

use clap::{Parser, ValueEnum};
use client::{ApiClient, CreateShortenRequest, ListParams, UpdateShortenRequest};
use config::CliConfig;

#[derive(Debug, Clone, ValueEnum)]
enum OutputFormat {
    /// Full table with all columns
    Table,
    /// Compact table for narrow terminals
    Compact,
    /// List format for very narrow terminals
    List,
}

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
    /// Find short URLs by original URL
    Find {
        /// Original URL to search for
        #[arg(short = 'r', long)]
        original_url: String,

        /// Show all matches (auto-paginate)
        #[arg(short = 'a', long)]
        all: bool,
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

        /// Filter by original URL
        #[arg(short = 'r', long)]
        original_url: Option<String>,

        /// Output format (table, compact, list)
        #[arg(short = 'f', long, value_enum)]
        format: Option<OutputFormat>,
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
        Some(Commands::Find { original_url, all }) => {
            handle_find(cli.url, cli.key, original_url, all).await
        }
        Some(Commands::List {
            all,
            page,
            psize,
            sort,
            order,
            status,
            original_url,
            format,
        }) => {
            let options = ListOptions {
                all,
                page,
                psize,
                sort,
                order,
                status,
                original_url,
                format,
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

async fn handle_find(
    url_arg: Option<String>,
    key_arg: Option<String>,
    original_url: String,
    all: bool,
) -> anyhow::Result<()> {
    let config = CliConfig::load(url_arg, key_arg)?;
    let client = ApiClient::new(config.url, config.key);

    if all {
        // Fetch all pages
        let mut all_items = Vec::new();
        let mut current_page = 1u64;
        let page_size = 50u64; // Use larger page size for --all

        loop {
            let params = ListParams {
                page: Some(current_page),
                page_size: Some(page_size),
                status: None,
                sort: Some("created_at".to_string()),
                order: Some("desc".to_string()),
                original_url: Some(original_url.clone()),
            };

            let response = client.list_shortens(params).await?;
            all_items.extend(response.data);

            if current_page >= response.meta.total_pages {
                break;
            }
            current_page += 1;
        }

        if all_items.is_empty() {
            println!("No short URLs found for original URL: {}", original_url);
        } else {
            println!(
                "Found {} short URL(s) for: {}",
                all_items.len(),
                original_url
            );
            println!();
            print_shorten_table_with_format(&all_items, None);
        }
    } else {
        // Fetch first page only
        let params = ListParams {
            page: Some(1),
            page_size: Some(10),
            status: None,
            sort: Some("created_at".to_string()),
            order: Some("desc".to_string()),
            original_url: Some(original_url.clone()),
        };

        let response = client.list_shortens(params).await?;

        if response.data.is_empty() {
            println!("No short URLs found for original URL: {}", original_url);
        } else {
            println!(
                "Found {} short URL(s) for: {} (showing page 1 of {})",
                response.meta.total, original_url, response.meta.total_pages
            );
            println!();
            print_shorten_table_with_format(&response.data, None);

            if response.meta.total_pages > 1 {
                println!();
                println!(
                    "Use --all flag to see all results, or use 'list -r \"{}\"' for pagination.",
                    original_url
                );
            }
        }
    }

    Ok(())
}

struct ListOptions {
    all: bool,
    page: Option<u64>,
    psize: Option<u64>,
    sort: Option<String>,
    order: Option<String>,
    status: Option<i32>,
    original_url: Option<String>,
    format: Option<OutputFormat>,
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
                original_url: options.original_url.clone(),
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
        print_shorten_table_with_format(&all_items, options.format.as_ref());
    } else {
        // Fetch single page
        let params = ListParams {
            page: options.page,
            page_size: options.psize,
            status: options.status,
            sort: options.sort,
            order: options.order,
            original_url: options.original_url,
        };

        let response = client.list_shortens(params).await?;

        println!(
            "Page {}/{} (showing {} of {} total)",
            response.meta.page,
            response.meta.total_pages,
            response.meta.count,
            response.meta.total
        );
        println!();
        print_shorten_table_with_format(&response.data, options.format.as_ref());
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
    println!("Created:      {}", format_datetime(&shorten.created_at));
    println!("Updated:      {}", format_datetime(&shorten.updated_at));
}

/// Print a table of short URLs with optional format override
fn print_shorten_table_with_format(shortens: &[ShortenResponse], format: Option<&OutputFormat>) {
    if shortens.is_empty() {
        println!("No short URLs found.");
        return;
    }

    match format {
        Some(OutputFormat::Table) => print_shorten_table_full(shortens),
        Some(OutputFormat::Compact) => print_shorten_table_compact(shortens),
        Some(OutputFormat::List) => print_shorten_list(shortens),
        None => {
            // Default to list format
            print_shorten_list(shortens);
        }
    }
}

/// Print a full table of short URLs (for wide terminals)
fn print_shorten_table_full(shortens: &[ShortenResponse]) {
    #[derive(Tabled)]
    struct ShortenRow {
        #[tabled(rename = "ID")]
        id: i64,
        #[tabled(rename = "Code")]
        code: String,
        #[tabled(rename = "Short URL")]
        short_url: String,
        #[tabled(rename = "Original URL")]
        original_url: String,
        #[tabled(rename = "Description")]
        description: String,
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
            short_url: s.short_url.clone(),
            original_url: truncate_url_smart(&s.original_url, 40),
            description: truncate_string(s.describe.as_deref().unwrap_or("-"), 15),
            status: status_name(s.status).to_string(),
            created_at: format_datetime(&s.created_at),
        })
        .collect();

    let mut table = Table::new(rows);
    table.with(Style::rounded());
    println!("{}", table);
}

/// Print a compact table of short URLs (for narrow terminals)
fn print_shorten_table_compact(shortens: &[ShortenResponse]) {
    #[derive(Tabled)]
    struct ShortenRowCompact {
        #[tabled(rename = "Code")]
        code: String,
        #[tabled(rename = "Short URL")]
        short_url: String,
        #[tabled(rename = "Original URL")]
        original_url: String,
        #[tabled(rename = "Status")]
        status: String,
    }

    let rows: Vec<ShortenRowCompact> = shortens
        .iter()
        .map(|s| ShortenRowCompact {
            code: s.code.clone(),
            short_url: s.short_url.clone(),
            original_url: s.original_url.clone(), // 显示完整 URL
            status: status_name(s.status).to_string(),
        })
        .collect();

    let mut table = Table::new(rows);
    table.with(Style::rounded());
    println!("{}", table);
}

/// Print short URLs as a list (for very narrow terminals)
fn print_shorten_list(shortens: &[ShortenResponse]) {
    for (i, s) in shortens.iter().enumerate() {
        if i > 0 {
            println!();
        }

        println!("{}. {} ({})", i + 1, s.code, status_name(s.status));
        println!("   Short URL: {}", s.short_url);
        println!("   Original:  {}", s.original_url);

        if let Some(desc) = &s.describe {
            if !desc.is_empty() {
                println!("   Desc:      {}", desc);
            }
        }

        println!("   Created:   {}", format_datetime(&s.created_at));
    }
}

/// Get human-readable status name
fn status_name(status: i32) -> &'static str {
    match status {
        0 => "Enabled",
        1 => "Disabled",
        _ => "Unknown",
    }
}

/// Truncate a string to a maximum length with ellipsis (Unicode-safe)
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.chars().count() <= max_len {
        s.to_string()
    } else if max_len <= 3 {
        "...".to_string()
    } else {
        let truncated: String = s.chars().take(max_len - 3).collect();
        format!("{}...", truncated)
    }
}

/// Smart truncate URL with format: prefix + ... + filename
/// Prioritizes preserving the complete filename
fn truncate_url_smart(url: &str, max_len: usize) -> String {
    if url.chars().count() <= max_len {
        return url.to_string();
    }

    if max_len <= 7 {
        return "...".to_string();
    }

    // Try to find the filename (after the last slash)
    if let Some(last_slash_pos) = url.rfind('/') {
        let filename = &url[last_slash_pos..]; // Include the slash
        let prefix_part = &url[..last_slash_pos];

        // Calculate space needed
        let ellipsis_len = 3; // "..."
        let filename_len = filename.chars().count();
        let available_for_prefix = max_len.saturating_sub(ellipsis_len + filename_len);

        // If we can fit the complete filename
        if available_for_prefix > 0 && ellipsis_len + filename_len <= max_len {
            let prefix: String = prefix_part.chars().take(available_for_prefix).collect();
            return format!("{}...{}", prefix, filename);
        }
    }

    // Fallback: use the original logic with 15 character suffix
    let suffix_len = 15.min(max_len - 4); // 4 for "..." minimum
    let ellipsis_len = 3; // "..."
    let prefix_len = max_len - suffix_len - ellipsis_len;

    if prefix_len < 1 {
        return truncate_string(url, max_len);
    }

    let chars: Vec<char> = url.chars().collect();
    let total_chars = chars.len();

    let prefix: String = chars.iter().take(prefix_len).collect();
    let suffix: String = chars.iter().skip(total_chars - suffix_len).collect();

    format!("{}...{}", prefix, suffix)
}

/// Format datetime string for display (convert to local timezone)
fn format_datetime(dt: &str) -> String {
    // Input format: "2024-01-15T10:30:45Z" (RFC 3339 / ISO 8601)
    // Output format: "2024-01-15 10:30 +08:00" (local time with timezone)
    
    use chrono::{DateTime, Local};
    
    // Try to parse as RFC 3339
    if let Ok(utc_time) = DateTime::parse_from_rfc3339(dt) {
        let local_time: DateTime<Local> = utc_time.with_timezone(&Local);
        return local_time.format("%Y-%m-%d %H:%M %:z").to_string();
    }
    
    // Fallback: simple string manipulation if parsing fails
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
        // Test valid RFC 3339 format (will convert to local timezone)
        let result = format_datetime("2024-01-15T10:30:45Z");
        assert!(result.contains("2024-01-15"));
        assert!(result.contains(":"));
        
        // Test with milliseconds
        let result2 = format_datetime("2024-01-15T10:30:45.123Z");
        assert!(result2.contains("2024-01-15"));
        
        // Test invalid format (fallback)
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
        print_shorten_table_with_format(&[], None);
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
        print_shorten_table_with_format(&shortens, None);
    }

    #[test]
    fn test_truncate_url_smart() {
        // Test normal truncation
        assert_eq!(truncate_url_smart("short", 10), "short");

        // Test smart truncation with domain + /filename
        let long_url = "https://example.com/very/long/path/to/file.html";
        let result = truncate_url_smart(long_url, 35);
        assert!(result.contains("..."));
        assert!(result.starts_with("https://example.com/"));
        assert!(result.ends_with("/file.html"));
        assert!(result.len() <= 35);

        // Test edge cases
        assert_eq!(truncate_url_smart("test", 4), "test");
        assert_eq!(truncate_url_smart("test", 3), "...");

        // Test URL without path
        assert_eq!(
            truncate_url_smart("https://example.com", 20),
            "https://example.com"
        );

        // Test specific example from user
        let git_url = "https://gist.asfd.cn/jetsung/git-mirror/raw/HEAD/git-mirror.sh";
        let result = truncate_url_smart(git_url, 40);
        assert!(result.starts_with("https://gist.asfd.cn/"));
        assert!(result.ends_with("/git-mirror.sh"));
        assert!(result.contains("..."));
    }

    #[test]
    fn test_truncate_url_smart_github_raw() {
        // Test the new universal truncation rule: prefix + ... + last 15 chars

        // Test the specific example from user
        let framagit_url =
            "https://framagit.org/jetsung/sh/-/raw/main/install/static-web-server.sh";
        let result = truncate_url_smart(framagit_url, 50);
        // Expected: https://framagit.org/.../static-web-server.sh
        assert_eq!(result.len(), 50);
        assert!(result.ends_with("/static-web-server.sh"));
        assert!(result.starts_with("https://framagit.org/"));
        assert!(result.contains("..."));

        // Test other examples with the new rule
        let github_url =
            "https://raw.githubusercontent.com/jetsung/sh/refs/heads/main/shell/gitlab-backup.sh";
        let result2 = truncate_url_smart(github_url, 50);
        assert_eq!(result2.len(), 50);
        assert!(result2.ends_with("/gitlab-backup.sh"));

        // Test regular URL
        let regular_url = "https://example.com/very/long/path/to/some/file.txt";
        let result3 = truncate_url_smart(regular_url, 40);
        assert_eq!(result3.len(), 40);
        assert!(result3.ends_with("/file.txt"));
    }
}
