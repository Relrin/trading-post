mod api;
mod cli;
mod core;
mod models;
mod multiplex_service;

use axum::{routing::get, Router};
use log::info;
use structopt::StructOpt;

use crate::api::auction::api::AuctionServiceImpl;
use crate::api::k8s::healthcheck;
use crate::cli::CliOptions;
use crate::core::orm::session::create_cassandra_session;
use crate::multiplex_service::MultiplexService;

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

    // build the rest service
    let rest = Router::new().route("/health", get(healthcheck));

    // build the grpc service
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(proto::AUCTION_DESCRIPTOR_SET)
        .build()
        .unwrap();
    let grpc = tonic::transport::Server::builder()
        .add_service(reflection_service)
        .add_service(proto::auction_server::AuctionServer::new(
            AuctionServiceImpl::new(cassandra_session),
        ))
        .into_service();

    // combine them into one service
    let service = MultiplexService::new(rest, grpc);

    info!("Listening {0}:{1}...", opts.host, opts.port);
    let addr = format!("{}:{}", opts.host, opts.port);
    let socket = &addr.parse().unwrap();
    hyper::Server::bind(socket)
        .serve(tower::make::Shared::new(service))
        .await
        .unwrap();

    Ok(())
}
