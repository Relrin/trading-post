mod api;
mod cli;
mod core;
mod models;
mod multiplex_service;

use std::net::SocketAddr;

use axum::{routing::get, Router};
use structopt::StructOpt;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::api::auction::api::AuctionServiceImpl;
use crate::api::k8s::healthcheck;
use crate::cli::CliOptions;
use crate::core::orm::session::create_cassandra_session;
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

    let cassandra_session = create_cassandra_session(&opts).await;

    // initialize tracing
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_env("APP_LOG"))
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
        .add_service(AuctionServer::new(AuctionServiceImpl::new(
            cassandra_session,
        )))
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
