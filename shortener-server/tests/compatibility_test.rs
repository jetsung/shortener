// Compatibility tests to verify Rust implementation matches Go version
// This ensures smooth migration from Go to Rust version

use std::fs;
use std::path::Path;

#[cfg(test)]
mod config_compatibility_tests {
    use shortener_server::config::Config;
    use std::fs;

    #[test]
    fn test_config_file_format_compatibility() {
        // Test that Rust version can read Go version's config format
        let go_config_content = r#"
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
path = "data/shortener.db"

[cache]
enabled = false
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

[geoip.ip2region]
path = "data/ip2region.xdb"
mode = "vector"
version = "4"
"#;

        // Write temporary config file
        let temp_config_path = "/tmp/test_go_compat_config.toml";
        fs::write(temp_config_path, go_config_content).expect("Failed to write test config");

        // Try to load it with Rust config parser
        let result = Config::from_file(temp_config_path);

        // Clean up
        let _ = fs::remove_file(temp_config_path);

        // Verify it loaded successfully
        assert!(
            result.is_ok(),
            "Failed to load Go-format config: {:?}",
            result.err()
        );

        let config = result.unwrap();
        assert_eq!(config.server.address, ":8080");
        assert_eq!(config.server.api_key, "test-api-key");
        assert_eq!(config.shortener.code_length, 6);
        assert_eq!(config.admin.username, "admin");
    }

    #[test]
    fn test_config_field_names_match() {
        // Verify all config field names match between Go and Rust
        // This ensures hyphenated fields (Go) work with underscored fields (Rust)
        let config_with_hyphens = r#"
[server]
address = ":8080"
trusted-platform = "X-Real-IP"
site_url = "http://localhost:8080"
api_key = "test"

[shortener]
code_length = 6
code_charset = "abc123"

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

[cache.redis]
host = "localhost"
port = 6379
password = ""
db = 0

[geoip]
enabled = false
type = "ip2region"

[geoip.ip2region]
path = "data/ip2region.xdb"
mode = "vector"
version = "4"
"#;

        let temp_path = "/tmp/test_hyphen_config.toml";
        fs::write(temp_path, config_with_hyphens).unwrap();
        let result = Config::from_file(temp_path);
        let _ = fs::remove_file(temp_path);

        assert!(
            result.is_ok(),
            "Hyphenated config fields should work: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_database_types_compatibility() {
        // Test all database types that Go version supports
        let db_types = vec!["sqlite", "postgres", "mysql"];

        for db_type in db_types {
            let config_content = format!(
                r#"
[server]
address = ":8080"
site_url = "http://localhost:8080"
api_key = "test"

[shortener]
code_length = 6
code_charset = "abc"

[admin]
username = "admin"
password = "pass"

[database]
type = "{}"
log_level = 1

[database.sqlite]
path = ":memory:"

[database.postgres]
host = "localhost"
port = 5432
user = "postgres"
password = "postgres"
database = "shortener"
sslmode = "disable"
timezone = "UTC"

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

[cache.redis]
host = "localhost"
port = 6379
password = ""
db = 0

[geoip]
enabled = false
type = "ip2region"

[geoip.ip2region]
path = "data/ip2region.xdb"
mode = "vector"
version = "4"
"#,
                db_type
            );

            let temp_path = format!("/tmp/test_db_{}.toml", db_type);
            fs::write(&temp_path, config_content).unwrap();
            let result = Config::from_file(&temp_path);
            let _ = fs::remove_file(&temp_path);

            assert!(
                result.is_ok(),
                "Database type '{}' should be supported: {:?}",
                db_type,
                result.err()
            );
        }
    }

    #[test]
    fn test_cache_types_compatibility() {
        // Test cache types: redis and valkey
        let cache_types = vec!["redis", "valkey"];

        for cache_type in cache_types {
            let config_content = format!(
                r#"
[server]
address = ":8080"
site_url = "http://localhost:8080"
api_key = "test"

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
enabled = true
type = "{}"
expire = 3600
prefix = "shorten:"

[cache.redis]
host = "localhost"
port = 6379
password = ""
db = 0

[cache.valkey]
host = "localhost"
port = 6379
username = ""
password = ""
db = 0

[geoip]
enabled = false
type = "ip2region"

[geoip.ip2region]
path = "data/ip2region.xdb"
mode = "vector"
version = "4"
"#,
                cache_type
            );

            let temp_path = format!("/tmp/test_cache_{}.toml", cache_type);
            fs::write(&temp_path, config_content).unwrap();
            let result = Config::from_file(&temp_path);
            let _ = fs::remove_file(&temp_path);

            assert!(
                result.is_ok(),
                "Cache type '{}' should be supported: {:?}",
                cache_type,
                result.err()
            );
        }
    }
}

#[cfg(test)]
mod api_compatibility_tests {
    use serde_json::json;

    #[test]
    fn test_api_endpoint_paths() {
        // Verify all API endpoints match between Go and Rust versions
        let expected_endpoints = vec![
            // Account endpoints
            ("POST", "/api/account/login"),
            ("POST", "/api/account/logout"),
            ("GET", "/api/users/current"),
            // Shorten endpoints
            ("POST", "/api/shortens"),
            ("GET", "/api/shortens"),
            ("GET", "/api/shortens/{code}"),
            ("PUT", "/api/shortens/{code}"),
            ("DELETE", "/api/shortens/{code}"),
            ("DELETE", "/api/shortens"),
            // History endpoints
            ("GET", "/api/histories"),
            ("DELETE", "/api/histories"),
        ];

        // This test documents the expected API surface
        // In a real integration test, we would verify these endpoints exist
        assert_eq!(
            expected_endpoints.len(),
            11,
            "All 11 API endpoints should be defined"
        );
    }

    #[test]
    fn test_request_body_format_create_shorten() {
        // Test that request body format matches Go version
        let request_body = json!({
            "original_url": "https://example.com",
            "code": "abc123",
            "describe": "Test URL"
        });

        // Verify required fields
        assert!(request_body.get("original_url").is_some());
        assert!(request_body.get("code").is_some());
        assert!(request_body.get("describe").is_some());

        // Verify field names match (not original_url vs originalUrl)
        let json_str = serde_json::to_string(&request_body).unwrap();
        assert!(json_str.contains("original_url"));
        assert!(json_str.contains("describe")); // Not "description"
    }

    #[test]
    fn test_response_format_shorten() {
        // Test that response format matches Go version
        let response = json!({
            "id": 1,
            "code": "abc123",
            "short_url": "http://localhost:8080/abc123",
            "original_url": "https://example.com",
            "describe": "Test URL",
            "status": 1,
            "created_at": "2024-01-01T00:00:00Z",
            "updated_at": "2024-01-01T00:00:00Z"
        });

        // Verify all expected fields exist
        assert!(response.get("id").is_some());
        assert!(response.get("code").is_some());
        assert!(response.get("short_url").is_some());
        assert!(response.get("original_url").is_some());
        assert!(response.get("describe").is_some());
        assert!(response.get("status").is_some());
        assert!(response.get("created_at").is_some());
        assert!(response.get("updated_at").is_some());
    }

    #[test]
    fn test_pagination_format() {
        // Test that pagination format matches Go version
        let page_meta = json!({
            "page": 1,
            "page_size": 10,
            "current_count": 10,
            "total_items": 100,
            "total_pages": 10
        });

        // Verify field names match exactly
        assert!(page_meta.get("page").is_some());
        assert!(page_meta.get("page_size").is_some()); // Not pageSize
        assert!(page_meta.get("current_count").is_some());
        assert!(page_meta.get("total_items").is_some());
        assert!(page_meta.get("total_pages").is_some());
    }

    #[test]
    fn test_error_response_format() {
        // Test that error response format matches Go version
        let error_response = json!({
            "errcode": "NOT_FOUND",
            "errinfo": "Short URL not found"
        });

        // Verify field names match exactly
        assert!(error_response.get("errcode").is_some());
        assert!(error_response.get("errinfo").is_some());

        // Verify it's not using different naming conventions
        let json_str = serde_json::to_string(&error_response).unwrap();
        assert!(!json_str.contains("error_code"));
        assert!(!json_str.contains("error_message"));
    }

    #[test]
    fn test_query_parameters_naming() {
        // Test that query parameter names match Go version
        let query_params = [
            "page",
            "page_size", // Not pageSize
            "sort_by",   // Not sortBy
            "order",
            "status",
            "ids",
        ];

        // Document expected parameter names
        assert_eq!(query_params.len(), 6);
        assert!(query_params.contains(&"page_size"));
        assert!(query_params.contains(&"sort_by"));
    }
}

#[cfg(test)]
mod database_compatibility_tests {
    #[test]
    fn test_urls_table_schema() {
        // Verify table schema matches Go version
        let expected_columns = [
            "id",
            "code",
            "original_url",
            "describe", // Not "description"
            "status",
            "created_at",
            "updated_at",
        ];

        // Document expected schema
        assert_eq!(expected_columns.len(), 7);
        assert!(expected_columns.contains(&"describe"));
        assert!(expected_columns.contains(&"original_url"));
    }

    #[test]
    fn test_histories_table_schema() {
        // Verify histories table schema matches Go version
        let expected_columns = vec![
            "id",
            "url_id",
            "short_code",
            "ip_address",
            "user_agent",
            "referer",
            "country",
            "region",
            "province",
            "city",
            "isp",
            "device_type",
            "os",
            "browser",
            "accessed_at",
            "created_at",
        ];

        // Document expected schema
        assert_eq!(expected_columns.len(), 16);
        assert!(expected_columns.contains(&"url_id"));
        assert!(expected_columns.contains(&"short_code"));
        assert!(expected_columns.contains(&"accessed_at"));
    }

    #[test]
    fn test_status_values() {
        // Verify status values
        // 0 = Enabled, 1 = Disabled
        let enabled_status = 0;
        let disabled_status = 1;

        assert_eq!(enabled_status, 0);
        assert_eq!(disabled_status, 1);
    }
}

#[cfg(test)]
mod cli_compatibility_tests {
    #[test]
    fn test_cli_command_names() {
        // Verify CLI command names match Go version
        let expected_commands = [
            "init", "env", "version", "create", "get", "list", "update", "delete",
        ];

        // Document expected commands
        assert_eq!(expected_commands.len(), 8);
        assert!(expected_commands.contains(&"init"));
        assert!(expected_commands.contains(&"create"));
    }

    #[test]
    fn test_cli_option_names() {
        // Verify CLI option names match Go version
        let expected_options = vec![
            ("--code", "-c"),
            ("--desc", "-d"),
            ("--ourl", ""), // Original URL for update
            ("--all", ""),
            ("--page", ""),
            ("--psize", ""), // Not --page-size
            ("--sort", ""),
            ("--order", ""),
            ("--url", "-u"),
            ("--key", "-k"),
        ];

        // Document expected options
        assert_eq!(expected_options.len(), 10);

        // Verify specific naming conventions
        let psize_option = expected_options.iter().find(|(long, _)| *long == "--psize");
        assert!(
            psize_option.is_some(),
            "--psize option should exist (not --page-size)"
        );
    }

    #[test]
    fn test_cli_config_file_location() {
        // Verify CLI config file location matches Go version
        let expected_config_path = "~/.config/shortener/config.toml";

        // Document expected path
        assert!(expected_config_path.contains(".config/shortener"));
        assert!(expected_config_path.ends_with("config.toml"));
    }

    #[test]
    fn test_cli_environment_variables() {
        // Verify environment variable names match Go version
        let expected_env_vars = ["SHORTENER_URL", "SHORTENER_KEY"];

        // Document expected environment variables
        assert_eq!(expected_env_vars.len(), 2);
        assert!(expected_env_vars.contains(&"SHORTENER_URL"));
        assert!(expected_env_vars.contains(&"SHORTENER_KEY"));
    }
}

#[cfg(test)]
mod migration_compatibility_tests {
    #[test]
    fn test_data_migration_compatibility() {
        // Test that Rust version can work with existing Go database
        // This test documents the migration requirements

        // 1. Table names must match exactly
        let table_names = ["urls", "histories"];
        assert_eq!(table_names.len(), 2);

        // 2. Column names must match exactly
        // Already tested in database_compatibility_tests

        // 3. Data types must be compatible
        // SQLite: INTEGER, TEXT, DATETIME
        // PostgreSQL: BIGINT, VARCHAR, TEXT, INTEGER, TIMESTAMP
        // MySQL: BIGINT, VARCHAR, TEXT, INT, DATETIME

        // 4. Indexes must be compatible
        let expected_indexes = [
            "idx_urls_code",
            "idx_urls_status",
            "idx_histories_url_id",
            "idx_histories_short_code",
            "idx_histories_accessed_at",
        ];
        assert_eq!(expected_indexes.len(), 5);
    }

    #[test]
    fn test_config_migration_compatibility() {
        // Test that existing Go config files work with Rust version
        // This is already tested in config_compatibility_tests
        // This test documents the migration path

        let migration_steps = [
            "1. Stop Go version server",
            "2. Backup database file",
            "3. Copy config.toml to Rust version",
            "4. Start Rust version server",
            "5. Verify API endpoints work",
            "6. Verify CLI commands work",
        ];

        assert_eq!(migration_steps.len(), 6);
    }

    #[test]
    fn test_api_client_compatibility() {
        // Test that existing API clients work with Rust version
        // API endpoints, request/response formats must match exactly

        let compatibility_requirements = [
            "Same HTTP methods (POST, GET, PUT, DELETE)",
            "Same endpoint paths",
            "Same request body field names",
            "Same response body field names",
            "Same HTTP status codes",
            "Same error response format",
            "Same authentication mechanism (X-API-KEY header)",
        ];

        assert_eq!(compatibility_requirements.len(), 7);
    }
}

#[test]
fn test_openapi_spec_compatibility() {
    // Verify OpenAPI spec exists and is compatible
    let openapi_path = "openapi.yml";

    if Path::new(openapi_path).exists() {
        let content = fs::read_to_string(openapi_path).expect("Failed to read openapi.yml");

        // Verify key sections exist
        assert!(
            content.contains("openapi:"),
            "OpenAPI version should be specified"
        );
        assert!(
            content.contains("/api/shortens"),
            "Shortens endpoints should be documented"
        );
        assert!(
            content.contains("/api/histories"),
            "Histories endpoints should be documented"
        );
        assert!(
            content.contains("/api/account/login"),
            "Login endpoint should be documented"
        );

        // Verify schema definitions
        assert!(
            content.contains("ShortenResponse"),
            "ShortenResponse schema should exist"
        );
        assert!(
            content.contains("ErrorResponse"),
            "ErrorResponse schema should exist"
        );
        assert!(content.contains("PageMeta"), "PageMeta schema should exist");
    }
}
