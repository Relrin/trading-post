//mod api;
mod cli;
//mod core;
//mod models;

//use actix_web::{middleware, web, App, HttpServer};
use structopt::StructOpt;

//use crate::api::auction::api::get_auction_router;
//use crate::api::k8s::get_k8s_router;
use crate::cli::CliOptions;
//use crate::core::error::transform_actix_web_validator_error;
//use crate::core::orm::session::create_cassandra_session;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let opts = CliOptions::from_args();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // let cassandra_session = create_cassandra_session(&opts).await;
    //
    // HttpServer::new(move || {
    //     App::new()
    //         .wrap(middleware::Logger::default().log_target("http_log"))
    //         .app_data(web::JsonConfig::default().limit(MAX_JSON_SIZE))
    //         .app_data(web::Data::new(cassandra_session.clone()))
    //         .app_data(
    //             actix_web_validator::QueryConfig::default()
    //                 .error_handler(|err, req| transform_actix_web_validator_error(err, req)),
    //         )
    //         .service(get_auction_router())
    //         .service(get_k8s_router())
    // })
    // .bind((opts.host, opts.port))?
    // .run()
    // .await

    Ok(())
}
