// SPDX-License-Identifier: Apache-2.0

use geist_server::{
    config::AppConfig,
    meta::{FeedServer, GroupServer, UserServer},
    tracing_metrics_layer,
};

use geist_sdk::geist::meta::v1alpha::{
    feed_service_server::FeedServiceServer, group_service_server::GroupServiceServer,
    user_service_server::UserServiceServer,
};

use dotenvy::dotenv;
use metrics_exporter_prometheus::PrometheusBuilder;
use std::error::Error;
use std::net::SocketAddr;
use tonic::transport::Server;
use tracing_subscriber::prelude::*;

fn setup_logging(config: &AppConfig) {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        format!("geist_server={}", config.effective_log_level())
            .parse()
            .expect("Failed to parse log level")
    });

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_target(true))
        .with(tracing_metrics_layer())
        .with(env_filter)
        .init();

    tracing::info!(
        environment = %config.environment,
        log_level = %config.effective_log_level(),
        debug = config.debug,
        "Starting Geist server"
    );
}

pub fn setup_metrics(addr: SocketAddr) -> anyhow::Result<()> {
    let builder = PrometheusBuilder::new();
    builder
        .with_http_listener(addr)
        .install()
        .map_err(|e| anyhow::anyhow!("Failed to install metrics exporter: {}", e))?;

    tracing::info!("Metrics exporter listening on {}", addr);
    Ok(())
}

fn print_configuration(config: &AppConfig) {
    tracing::info!(
        grpc_address = %config.grpc_address,
        http_address = %config.http_address,
        metrics_address = %config.metrics_address,
        db_pool_size = config.db_pool_size,
        db_timeout_secs = config.db_timeout_secs,
        server_timeout_secs = config.server_timeout_secs,
        enable_reflection = config.enable_reflection,
        "Server configuration"
    );
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Load environment variables from .env file if it exists
    dotenv().ok();

    // Load the service configuration from args or environment variables.
    let config = match AppConfig::load() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Configuration error: {}", e);
            std::process::exit(1);
        }
    };

    // Validate the configuration that was loaded.
    if let Err(validation_errors) = config.validate() {
        eprintln!("Configuration validation failed:");
        for error in validation_errors {
            eprintln!("  - {}", error);
        }
        std::process::exit(2);
    }

    // Setup logging
    setup_logging(&config);

    // Print configuration (with sensitive data redacted)
    print_configuration(&config);

    if let Err(e) = setup_metrics(config.metrics_address) {
        tracing::error!("Failed to set up metrics: {}", e);
        std::process::exit(3);
    }

    let svc1 = UserServiceServer::new(UserServer::default());
    let svc2 = FeedServiceServer::new(FeedServer::default());
    let svc3 = GroupServiceServer::new(GroupServer::default());

    Server::builder()
        .trace_fn(|_| tracing::info_span!("geist-server"))
        .add_service(svc1)
        .add_service(svc2)
        .add_service(svc3)
        .serve(config.grpc_address)
        .await?;

    Ok(())
}
