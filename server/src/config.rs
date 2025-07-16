// SPDX-License-Identifier: Apache-2.0

use clap::{Parser, ValueEnum};
use std::net::SocketAddr;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl From<LogLevel> for tracing::Level {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Error => tracing::Level::ERROR,
            LogLevel::Warn => tracing::Level::WARN,
            LogLevel::Info => tracing::Level::INFO,
            LogLevel::Debug => tracing::Level::DEBUG,
            LogLevel::Trace => tracing::Level::TRACE,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Environment {
    Development,
    Staging,
    Production,
}

impl FromStr for Environment {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "development" | "dev" => Ok(Environment::Development),
            "staging" | "stage" => Ok(Environment::Staging),
            "production" | "prod" => Ok(Environment::Production),
            _ => Err(format!("Unknown environment: {}", s)),
        }
    }
}

impl std::fmt::Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Environment::Development => write!(f, "development"),
            Environment::Staging => write!(f, "staging"),
            Environment::Production => write!(f, "production"),
        }
    }
}

#[derive(Debug, Clone, Parser)]
#[command(
    name = "geist-server",
    version,
    long_about = "A production-ready server with gRPC and HTTP APIs."
)]
pub struct AppConfig {
    /// Environment (development, staging, production)
    #[arg(
        long,
        env = "APP_ENV",
        default_value = "development",
        value_enum,
        help = "Application environment"
    )]
    pub environment: Environment,

    /// gRPC server address
    #[arg(
        long,
        env = "GRPC_ADDRESS",
        default_value = "127.0.0.1:50051",
        help = "gRPC server listen address"
    )]
    pub grpc_address: SocketAddr,

    /// HTTP server address
    #[arg(
        long,
        env = "HTTP_ADDRESS",
        default_value = "127.0.0.1:8080",
        help = "HTTP server listen address"
    )]
    pub http_address: SocketAddr,

    /// Metrics server address
    #[arg(
        long,
        env = "METRICS_ADDRESS",
        default_value = "127.0.0.1:9090",
        help = "Metrics server listen address"
    )]
    pub metrics_address: SocketAddr,

    /// Log level
    #[arg(
        long,
        env = "LOG_LEVEL",
        default_value = "info",
        value_enum,
        help = "Logging level"
    )]
    pub log_level: LogLevel,

    /// Enable debug mode
    #[arg(
        long,
        env = "ENABLE_DEBUG",
        default_value = "false",
        help = "Enable debug mode with additional logging and features"
    )]
    pub debug: bool,

    /// Database connection pool size
    #[arg(
        long,
        env = "DB_POOL_SIZE",
        default_value = "5",
        help = "Database connection pool size"
    )]
    pub db_pool_size: u32,

    /// Database connection timeout (seconds)
    #[arg(
        long,
        env = "DB_TIMEOUT_SECS",
        default_value = "30",
        help = "Database connection timeout in seconds"
    )]
    pub db_timeout_secs: u64,

    /// Server timeout (seconds)
    #[arg(
        long,
        env = "SERVER_TIMEOUT_SECS",
        default_value = "30",
        help = "Server request timeout in seconds"
    )]
    pub server_timeout_secs: u64,

    /// Enable gRPC reflection service (for development)
    #[arg(
        long,
        env = "ENABLE_REFLECTION",
        default_value = "false",
        help = "Enable gRPC reflection service (development only)"
    )]
    pub enable_reflection: bool,
}

impl AppConfig {
    /// Load configuration from environment variables and command-line arguments
    pub fn load() -> Result<Self, clap::Error> {
        AppConfig::try_parse()
    }

    /// Validate configuration and return any validation errors
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Validate addresses are not the same
        if self.grpc_address == self.http_address {
            errors.push("GRPC_ADDRESS and HTTP_ADDRESS cannot be the same".to_string());
        }

        if self.grpc_address == self.metrics_address {
            errors.push("GRPC_ADDRESS and METRICS_ADDRESS cannot be the same".to_string());
        }

        if self.http_address == self.metrics_address {
            errors.push("HTTP_ADDRESS and METRICS_ADDRESS cannot be the same".to_string());
        }

        // Validate pool size
        if self.db_pool_size == 0 {
            errors.push("DB_POOL_SIZE must be greater than 0".to_string());
        }

        // Validate timeouts
        if self.db_timeout_secs == 0 {
            errors.push("DB_TIMEOUT_SECS must be greater than 0".to_string());
        }

        if self.server_timeout_secs == 0 {
            errors.push("SERVER_TIMEOUT_SECS must be greater than 0".to_string());
        }

        // Environment-specific validations
        match self.environment {
            Environment::Production => {
                if self.debug {
                    errors.push("Debug mode should not be enabled in production".to_string());
                }
                if self.enable_reflection {
                    errors.push("gRPC reflection should not be enabled in production".to_string());
                }
                if self.grpc_address.ip().is_loopback() {
                    errors
                        .push("gRPC server should not bind to loopback in production".to_string());
                }
                if self.http_address.ip().is_loopback() {
                    errors
                        .push("HTTP server should not bind to loopback in production".to_string());
                }
            }
            Environment::Staging => {
                if self.enable_reflection {
                    errors.push("gRPC reflection should not be enabled in staging".to_string());
                }
            }
            Environment::Development => {
                // Development allows all configurations
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Get database connection timeout as Duration
    pub fn db_timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.db_timeout_secs)
    }

    /// Get server timeout as Duration
    pub fn server_timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.server_timeout_secs)
    }

    /// Check if running in production
    pub fn is_production(&self) -> bool {
        matches!(self.environment, Environment::Production)
    }

    /// Check if running in development
    pub fn is_development(&self) -> bool {
        matches!(self.environment, Environment::Development)
    }

    /// Get effective log level based on environment and debug flag
    pub fn effective_log_level(&self) -> tracing::Level {
        if self.debug {
            tracing::Level::DEBUG
        } else {
            self.log_level.into()
        }
    }
}
