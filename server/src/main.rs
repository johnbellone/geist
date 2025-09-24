// SPDX-License-Identifier: Apache-2.0

use geist_server::{
    config::AppConfig,
    meta::{FeedServer, GroupServer, UserServer},
    tracing_metrics_layer,
};

use geist_sdk::pb::meta::v1alpha::{
    feed_service_server::FeedServiceServer, group_service_server::GroupServiceServer,
    user_service_server::UserServiceServer,
};

use dotenvy::dotenv;
use metrics_exporter_prometheus::PrometheusBuilder;
use std::error::Error;
use tonic::transport::Server;
use tracing_subscriber::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    color_eyre::install()?;

    // Load the service configuration from args or environment variables.
    let config = match AppConfig::load() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Configuration error: {}", e);
            std::process::exit(1);
        }
    };

    // Validate the configuration that was loaded.
    if let Err(e) = config.validate() {
        eprintln!("Configuration validation failed:");
        for error in e {
            eprintln!("  - {}", error);
        }
        std::process::exit(2);
    }

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

    let builder = PrometheusBuilder::new();
    builder
        .with_http_listener(config.metrics_address)
        .install()
        .map_err(|e| anyhow::anyhow!("Failed to install metrics exporter: {}", e))?;

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
