mod api;
mod cli;

use actix_web::{App, web, HttpServer, middleware};
use structopt::StructOpt;

use crate::api::k8s::get_k8s_router;
use crate::cli::CliOptions;

const MAX_JSON_SIZE: usize = 4096;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let cli = CliOptions::from_args();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default().log_target("http_log"))
            .app_data(web::JsonConfig::default().limit(MAX_JSON_SIZE))
            .service(get_k8s_router())
    })
    .bind((cli.host, cli.port))?
    .run()
    .await
}
