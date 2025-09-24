// SPDX-License-Identifier: Apache-2.0

use color_eyre::Result;
use dotenvy::dotenv;
use geist_client::config::AppConfig;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    color_eyre::install()?;

    // Load the configuration from args and/or environment variables.
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

    tracing::info!(
        log_level = %config.effective_log_level(),
        debug = config.debug,
        "Starting Geist client"
    );

    Ok(())
}
