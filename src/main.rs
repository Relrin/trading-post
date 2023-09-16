mod api;
mod cli;
mod multiplex_service;
//mod core;
//mod models;

use std::net::SocketAddr;

use axum::{routing::get, Router};
use structopt::StructOpt;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::api::auction::api::AuctionServiceImpl;
use crate::api::k8s::healthcheck;
use crate::cli::CliOptions;
use crate::multiplex_service::MultiplexService;
use crate::proto::auction_server::AuctionServer;

mod proto {
    tonic::include_proto!("auction");
    pub(crate) const AUCTION_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("auction_descriptor");
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let opts = CliOptions::from_args();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example_rest_grpc_multiplex=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // build the rest service
    let rest = Router::new().route("/health", get(healthcheck));

    // build the grpc service
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(proto::AUCTION_DESCRIPTOR_SET)
        .build()
        .unwrap();
    let grpc = tonic::transport::Server::builder()
        .add_service(reflection_service)
        .add_service(AuctionServer::new(AuctionServiceImpl::default()))
        .into_service();

    // combine them into one service
    let service = MultiplexService::new(rest, grpc);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    hyper::Server::bind(&addr)
        .serve(tower::make::Shared::new(service))
        .await
        .unwrap();

    Ok(())
}
