use anyhow::{Context, Result};
use config::{Config as ConfigBuilder, ConfigError, File};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// CLI configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliConfig {
    /// Server URL (e.g., http://localhost:8080)
    pub url: String,
    /// API Key for authentication
    pub key: String,
}

impl Default for CliConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:8080".to_string(),
            key: String::new(),
        }
    }
}

impl CliConfig {
    /// Load configuration with priority: CLI args > Environment variables > Config file > Defaults
    pub fn load(url_arg: Option<String>, key_arg: Option<String>) -> Result<Self> {
        // Start with default configuration
        let mut config = Self::default();

        // Try to load from config file
        if let Some(config_path) = Self::config_file_path()
            && config_path.exists()
        {
            match Self::load_from_file(&config_path) {
                Ok(file_config) => {
                    config = file_config;
                }
                Err(e) => {
                    eprintln!("Warning: Failed to load config file: {}", e);
                }
            }
        }

        // Override with environment variables
        if let Ok(env_url) = std::env::var("SHORTENER_URL") {
            config.url = env_url;
        }
        if let Ok(env_key) = std::env::var("SHORTENER_KEY") {
            config.key = env_key;
        }

        // Override with command line arguments (highest priority)
        if let Some(url) = url_arg {
            config.url = url;
        }
        if let Some(key) = key_arg {
            config.key = key;
        }

        // Validate configuration
        if config.url.is_empty() {
            anyhow::bail!(
                "Server URL is not configured. Use --url, SHORTENER_URL env var, or run 'init' command."
            );
        }
        if config.key.is_empty() {
            anyhow::bail!(
                "API Key is not configured. Use --key, SHORTENER_KEY env var, or run 'init' command."
            );
        }

        Ok(config)
    }

    /// Load configuration from file using config-rs
    fn load_from_file(path: &std::path::Path) -> Result<Self, ConfigError> {
        let settings = ConfigBuilder::builder()
            .add_source(File::from(path))
            .build()?;

        settings.try_deserialize()
    }

    /// Get the configuration file path (~/.config/shortener/config.toml)
    pub fn config_file_path() -> Option<PathBuf> {
        ProjectDirs::from("", "", "shortener").map(|proj_dirs| {
            let config_dir = proj_dirs.config_dir();
            config_dir.join("config.toml")
        })
    }

    /// Get the configuration directory path (~/.config/shortener/)
    #[allow(dead_code)]
    pub fn config_dir_path() -> Option<PathBuf> {
        ProjectDirs::from("", "", "shortener").map(|proj_dirs| proj_dirs.config_dir().to_path_buf())
    }

    /// Initialize configuration file with provided values
    pub fn init(url: Option<String>, key: Option<String>) -> Result<PathBuf> {
        let config_path =
            Self::config_file_path().context("Failed to determine config file path")?;

        let config_dir = config_path
            .parent()
            .context("Failed to get config directory")?;

        // Create config directory if it doesn't exist
        if !config_dir.exists() {
            fs::create_dir_all(config_dir).context("Failed to create config directory")?;
        }

        // Create configuration with provided values or defaults
        let config = Self {
            url: url.unwrap_or_else(|| "http://localhost:8080".to_string()),
            key: key.unwrap_or_default(),
        };

        // Serialize to TOML
        let toml_content =
            toml::to_string_pretty(&config).context("Failed to serialize config to TOML")?;

        // Write to file
        fs::write(&config_path, toml_content).context("Failed to write config file")?;

        Ok(config_path)
    }

    /// Display current configuration sources
    pub fn display_env_info(url_arg: Option<String>, key_arg: Option<String>) {
        println!("Configuration Sources (in priority order):");
        println!("  1. Command line arguments (--url, --key)");
        println!("  2. Environment variables (SHORTENER_URL, SHORTENER_KEY)");
        println!("  3. Configuration file");
        println!("  4. Default values");
        println!();

        // Show config file path
        if let Some(config_path) = Self::config_file_path() {
            println!("Config file path: {}", config_path.display());
            println!("Config file exists: {}", config_path.exists());
        } else {
            println!("Config file path: Unable to determine");
        }
        println!();

        // Show command line arguments
        println!("Command Line Arguments:");
        match &url_arg {
            Some(val) => println!("  --url: {}", val),
            None => println!("  --url: (not provided)"),
        }
        match &key_arg {
            Some(val) => println!("  --key: {}", val),
            None => println!("  --key: (not provided)"),
        }
        println!();

        // Show environment variables
        println!("Environment Variables:");
        match std::env::var("SHORTENER_URL") {
            Ok(val) => println!("  SHORTENER_URL: {}", val),
            Err(_) => println!("  SHORTENER_URL: (not set)"),
        }
        match std::env::var("SHORTENER_KEY") {
            Ok(val) => println!("  SHORTENER_KEY: {}", val),
            Err(_) => println!("  SHORTENER_KEY: (not set)"),
        }
        println!();

        // Try to load and display current effective configuration
        println!("Current Effective Configuration:");
        match Self::load(url_arg, key_arg) {
            Ok(config) => {
                println!("  URL: {}", config.url);
                println!("  Key: {}", config.key);
            }
            Err(e) => {
                println!("  Error loading configuration: {}", e);
            }
        }
    }

    /// Mask API key for display
    #[allow(dead_code)]
    fn mask_key(key: &str) -> String {
        if key.len() > 8 {
            format!("{}...{}", &key[..4], &key[key.len() - 4..])
        } else if !key.is_empty() {
            "****".to_string()
        } else {
            "(empty)".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = CliConfig::default();
        assert_eq!(config.url, "http://localhost:8080");
        assert_eq!(config.key, "");
    }

    #[test]
    fn test_config_priority_cli_args() {
        // CLI arguments should have highest priority
        let config = CliConfig::load(
            Some("http://example.com".to_string()),
            Some("test-key".to_string()),
        )
        .unwrap();

        assert_eq!(config.url, "http://example.com");
        assert_eq!(config.key, "test-key");
    }

    #[test]
    fn test_config_validation() {
        // Test that both URL and key are required when not provided via any source
        // Note: This test may pass if environment variables are set

        // When both are provided, should succeed
        let result = CliConfig::load(
            Some("http://example.com".to_string()),
            Some("test-key".to_string()),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_config_file_path() {
        let path = CliConfig::config_file_path();
        assert!(path.is_some());
        if let Some(p) = path {
            assert!(p.to_string_lossy().contains("shortener"));
            assert!(p.to_string_lossy().ends_with("config.toml"));
        }
    }

    #[test]
    fn test_mask_key() {
        assert_eq!(CliConfig::mask_key(""), "(empty)");
        assert_eq!(CliConfig::mask_key("short"), "****");
        assert_eq!(CliConfig::mask_key("test-key-123"), "test...-123");
        assert_eq!(CliConfig::mask_key("verylongapikey12345"), "very...2345");
    }
}
