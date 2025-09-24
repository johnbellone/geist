// SPDX-License-Identifier: Apache-2.0

use clap::Parser;
use geist_sdk::LogLevel;
use std::net::SocketAddr;

#[derive(Debug, Clone, Parser)]
#[command(name = "geist-client", version)]
pub struct AppConfig {
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

    /// gRPC server address
    #[arg(
        long,
        env = "GRPC_ADDRESS",
        default_value = "rpc.geist.services:50051",
        help = "gRPC server address"
    )]
    pub grpc_address: SocketAddr,

    /// HTTP server address
    #[arg(
        long,
        env = "HTTP_ADDRESS",
        default_value = "rpc.geist.services:443",
        help = "HTTP server address"
    )]
    pub http_address: SocketAddr,
}

impl AppConfig {
    /// Load configuration from environment variables and command-line arguments.
    pub fn load() -> Result<Self, clap::Error> {
        AppConfig::try_parse()
    }

    /// Validate configuration and return any errors.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.grpc_address == self.http_address {
            errors.push("GRPC_ADDRESS and HTTP_ADDRESS cannot be the same".to_string());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Retrieve the effective log level based on the debug flag.
    pub fn effective_log_level(&self) -> LogLevel {
        if self.debug {
            LogLevel::Debug
        } else {
            self.log_level
        }
    }
}
