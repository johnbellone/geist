// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use geist_sdk::{Environment, LogLevel};
use std::net::SocketAddr;

#[derive(Debug, Clone, Parser)]
#[command(name = "geist-server", version)]
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
}

impl AppConfig {
    /// Load configuration from environment variables and command-line arguments
    pub fn load() -> Result<Self, clap::Error> {
        AppConfig::try_parse()
    }

    /// Validate configuration and return any validation errors
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.grpc_address == self.http_address {
            errors.push("GRPC_ADDRESS and HTTP_ADDRESS cannot be the same".to_string());
        }

        if self.grpc_address == self.metrics_address {
            errors.push("GRPC_ADDRESS and METRICS_ADDRESS cannot be the same".to_string());
        }

        if self.http_address == self.metrics_address {
            errors.push("HTTP_ADDRESS and METRICS_ADDRESS cannot be the same".to_string());
        }

        if self.db_pool_size == 0 {
            errors.push("DB_POOL_SIZE must be greater than 0".to_string());
        }

        if self.db_timeout_secs == 0 {
            errors.push("DB_TIMEOUT_SECS must be greater than 0".to_string());
        }

        if self.server_timeout_secs == 0 {
            errors.push("SERVER_TIMEOUT_SECS must be greater than 0".to_string());
        }

        match self.environment {
            Environment::Production => {}
            Environment::Preview => {}
            Environment::Staging => {}
            Environment::Development => {}
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn db_timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.db_timeout_secs)
    }

    pub fn server_timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.server_timeout_secs)
    }

    pub fn is_production(&self) -> bool {
        matches!(self.environment, Environment::Production)
    }

    pub fn is_development(&self) -> bool {
        matches!(self.environment, Environment::Development)
    }

    pub fn is_preview(&self) -> bool {
        matches!(self.environment, Environment::Preview)
    }

    pub fn is_staging(&self) -> bool {
        matches!(self.environment, Environment::Staging)
    }

    pub fn effective_log_level(&self) -> tracing::Level {
        if self.debug {
            tracing::Level::DEBUG
        } else {
            self.log_level.into()
        }
    }
}
