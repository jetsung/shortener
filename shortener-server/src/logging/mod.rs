use serde::{Deserialize, Serialize};
use tracing::Level;
use tracing_subscriber::{
    EnvFilter, Layer,
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
};

/// Logging configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    #[serde(default = "default_level")]
    pub level: String,

    /// Log format (json, pretty, compact)
    #[serde(default = "default_format")]
    pub format: LogFormat,

    /// Whether to include timestamps
    #[serde(default = "default_true")]
    pub with_timestamp: bool,

    /// Whether to include target (module path)
    #[serde(default = "default_true")]
    pub with_target: bool,

    /// Whether to include thread IDs
    #[serde(default = "default_false")]
    pub with_thread_ids: bool,

    /// Whether to include thread names
    #[serde(default = "default_false")]
    pub with_thread_names: bool,

    /// Whether to include file and line number
    #[serde(default = "default_false")]
    pub with_file: bool,

    /// Whether to include line number
    #[serde(default = "default_false")]
    pub with_line_number: bool,

    /// Whether to use ANSI colors (for terminal output)
    #[serde(default = "default_true")]
    pub with_ansi: bool,
}

/// Log format enum
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    /// JSON format for structured logging
    Json,
    /// Pretty format for human-readable output
    Pretty,
    /// Compact format for minimal output
    Compact,
}

fn default_level() -> String {
    "info".to_string()
}

fn default_format() -> LogFormat {
    LogFormat::Pretty
}

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_level(),
            format: default_format(),
            with_timestamp: true,
            with_target: true,
            with_thread_ids: false,
            with_thread_names: false,
            with_file: false,
            with_line_number: false,
            with_ansi: true,
        }
    }
}

impl LoggingConfig {
    /// Initialize the logging system with this configuration
    pub fn init(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Build the environment filter
        let env_filter = self.build_env_filter()?;

        // Build the subscriber based on format
        match self.format {
            LogFormat::Json => {
                if self.with_timestamp {
                    let fmt_layer = fmt::layer()
                        .json()
                        .with_target(self.with_target)
                        .with_thread_ids(self.with_thread_ids)
                        .with_thread_names(self.with_thread_names)
                        .with_file(self.with_file)
                        .with_line_number(self.with_line_number)
                        .with_span_events(FmtSpan::CLOSE)
                        .with_filter(env_filter);

                    tracing_subscriber::registry().with(fmt_layer).init();
                } else {
                    let fmt_layer = fmt::layer()
                        .json()
                        .with_target(self.with_target)
                        .with_thread_ids(self.with_thread_ids)
                        .with_thread_names(self.with_thread_names)
                        .with_file(self.with_file)
                        .with_line_number(self.with_line_number)
                        .with_span_events(FmtSpan::CLOSE)
                        .without_time()
                        .with_filter(env_filter);

                    tracing_subscriber::registry().with(fmt_layer).init();
                }
            }
            LogFormat::Pretty => {
                if self.with_timestamp {
                    let fmt_layer = fmt::layer()
                        .pretty()
                        .with_target(self.with_target)
                        .with_thread_ids(self.with_thread_ids)
                        .with_thread_names(self.with_thread_names)
                        .with_file(self.with_file)
                        .with_line_number(self.with_line_number)
                        .with_ansi(self.with_ansi)
                        .with_filter(env_filter);

                    tracing_subscriber::registry().with(fmt_layer).init();
                } else {
                    let fmt_layer = fmt::layer()
                        .pretty()
                        .with_target(self.with_target)
                        .with_thread_ids(self.with_thread_ids)
                        .with_thread_names(self.with_thread_names)
                        .with_file(self.with_file)
                        .with_line_number(self.with_line_number)
                        .with_ansi(self.with_ansi)
                        .without_time()
                        .with_filter(env_filter);

                    tracing_subscriber::registry().with(fmt_layer).init();
                }
            }
            LogFormat::Compact => {
                if self.with_timestamp {
                    let fmt_layer = fmt::layer()
                        .compact()
                        .with_target(self.with_target)
                        .with_thread_ids(self.with_thread_ids)
                        .with_thread_names(self.with_thread_names)
                        .with_file(self.with_file)
                        .with_line_number(self.with_line_number)
                        .with_ansi(self.with_ansi)
                        .with_filter(env_filter);

                    tracing_subscriber::registry().with(fmt_layer).init();
                } else {
                    let fmt_layer = fmt::layer()
                        .compact()
                        .with_target(self.with_target)
                        .with_thread_ids(self.with_thread_ids)
                        .with_thread_names(self.with_thread_names)
                        .with_file(self.with_file)
                        .with_line_number(self.with_line_number)
                        .with_ansi(self.with_ansi)
                        .without_time()
                        .with_filter(env_filter);

                    tracing_subscriber::registry().with(fmt_layer).init();
                }
            }
        }

        Ok(())
    }

    /// Build the environment filter from the log level
    fn build_env_filter(&self) -> Result<EnvFilter, Box<dyn std::error::Error>> {
        // Try to get filter from environment variable first
        if let Ok(filter) = EnvFilter::try_from_default_env() {
            return Ok(filter);
        }

        // Otherwise, use the configured level
        let level = self.parse_level()?;
        let filter = EnvFilter::new(format!("shortener_server={}", level))
            .add_directive(format!("tower_http={}", level).parse()?)
            .add_directive(format!("axum={}", level).parse()?);

        Ok(filter)
    }

    /// Parse the log level string into a tracing Level
    fn parse_level(&self) -> Result<Level, Box<dyn std::error::Error>> {
        match self.level.to_lowercase().as_str() {
            "trace" => Ok(Level::TRACE),
            "debug" => Ok(Level::DEBUG),
            "info" => Ok(Level::INFO),
            "warn" | "warning" => Ok(Level::WARN),
            "error" => Ok(Level::ERROR),
            _ => Err(format!("Invalid log level: {}", self.level).into()),
        }
    }

    /// Validate the logging configuration
    pub fn validate(&self) -> Result<(), String> {
        // Validate log level
        match self.level.to_lowercase().as_str() {
            "trace" | "debug" | "info" | "warn" | "warning" | "error" => Ok(()),
            _ => Err(format!("Invalid log level: {}", self.level)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_logging_config() {
        let config = LoggingConfig::default();
        assert_eq!(config.level, "info");
        assert_eq!(config.format, LogFormat::Pretty);
        assert!(config.with_timestamp);
        assert!(config.with_target);
        assert!(!config.with_thread_ids);
        assert!(!config.with_thread_names);
        assert!(!config.with_file);
        assert!(!config.with_line_number);
        assert!(config.with_ansi);
    }

    #[test]
    fn test_parse_level() {
        let config = LoggingConfig {
            level: "debug".to_string(),
            ..Default::default()
        };
        assert_eq!(config.parse_level().unwrap(), Level::DEBUG);

        let config = LoggingConfig {
            level: "INFO".to_string(),
            ..Default::default()
        };
        assert_eq!(config.parse_level().unwrap(), Level::INFO);

        let config = LoggingConfig {
            level: "warn".to_string(),
            ..Default::default()
        };
        assert_eq!(config.parse_level().unwrap(), Level::WARN);

        let config = LoggingConfig {
            level: "warning".to_string(),
            ..Default::default()
        };
        assert_eq!(config.parse_level().unwrap(), Level::WARN);

        let config = LoggingConfig {
            level: "error".to_string(),
            ..Default::default()
        };
        assert_eq!(config.parse_level().unwrap(), Level::ERROR);

        let config = LoggingConfig {
            level: "trace".to_string(),
            ..Default::default()
        };
        assert_eq!(config.parse_level().unwrap(), Level::TRACE);
    }

    #[test]
    fn test_parse_invalid_level() {
        let config = LoggingConfig {
            level: "invalid".to_string(),
            ..Default::default()
        };
        assert!(config.parse_level().is_err());
    }

    #[test]
    fn test_validate_valid_levels() {
        let levels = vec!["trace", "debug", "info", "warn", "warning", "error"];
        for level in levels {
            let config = LoggingConfig {
                level: level.to_string(),
                ..Default::default()
            };
            assert!(config.validate().is_ok());
        }
    }

    #[test]
    fn test_validate_invalid_level() {
        let config = LoggingConfig {
            level: "invalid".to_string(),
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_log_format_serialization() {
        let json_format = LogFormat::Json;
        let pretty_format = LogFormat::Pretty;
        let compact_format = LogFormat::Compact;

        assert_eq!(json_format, LogFormat::Json);
        assert_eq!(pretty_format, LogFormat::Pretty);
        assert_eq!(compact_format, LogFormat::Compact);
    }
}
