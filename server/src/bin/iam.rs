// SPDX-License-Identifier: Apache-2.0

use geist_sdk::pb::{
    user_service_server::UserServiceServer, 
    role_service_server::RoleServiceServer, 
};
use geist_server::iam::{
    user::UserServer,
    role::RoleServer,
};

use clap::Parser;
use dotenvy::dotenv;
use std::error::Error;
use tonic::transport::Server;

#[derive(Parser, Debug)]
#[command(name = "iam-server")]
#[command(bin_name = "iam-server")]
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

    let svc1 = UserServiceServer::new(UserServer::default());
    let svc2 = RoleServiceServer::new(RoleServer::default());

    Server::builder()
        .trace_fn(|_| tracing::info_span!("geist-iam-server"))
        .add_service(svc1)
        .add_service(svc2)
        .serve(args.address)
        .await?;

    Ok(())
}