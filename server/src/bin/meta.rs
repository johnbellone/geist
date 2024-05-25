// SPDX-License-Identifier: Apache-2.0

use geist_sdk::pb::{
    info_service_server::InfoServiceServer, 
};

use geist_server::meta::{
    info::InfoServer,
};

use clap::Parser;
use dotenvy::dotenv;
use std::error::Error;
use tonic::transport::Server;

#[derive(Parser, Debug)]
#[command(name = "meta-server")]
#[command(bin_name = "meta-server")]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
struct Args {
    #[clap(short = 'a', long = "address", default_value = "[::1]:50051")]
    address: std::net::SocketAddr,

    #[arg(long, default_value_t = false)]
    debug: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let args: Args = Args::parse();

    // TODO: Add health service and reporter for production environment (non-debug).
    // TODO: Add reflect service for non-production environment (debug).

    let svc1 = InfoServiceServer::new(InfoServer::default());

    Server::builder()
        .trace_fn(|_| tracing::info_span!("geist-meta-server"))
        .add_service(svc1)
        .serve(args.address)
        .await?;

    Ok(())
}