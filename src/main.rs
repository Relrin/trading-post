mod api;
mod cli;
mod core;
mod models;

use actix_web::{App, web, HttpServer, middleware};
use structopt::StructOpt;

use crate::api::auction::get_auction_router;
use crate::api::k8s::get_k8s_router;
use crate::cli::CliOptions;
use crate::core::orm::session::CassandraSession;

const MAX_JSON_SIZE: usize = 4096;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let opts = CliOptions::from_args();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let cassandra_session = CassandraSession::new(&opts).await;

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default().log_target("http_log"))
            .app_data(web::JsonConfig::default().limit(MAX_JSON_SIZE))
            .app_data(cassandra_session.clone())
            .service(get_auction_router())
            .service(get_k8s_router())
    })
    .bind((opts.host, opts.port))?
    .run()
    .await
}
