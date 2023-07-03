mod api;

#[actix_web::main]
async fn main() {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let host = "0.0.0.0";
    let port = 8000;

    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Logger::default().log_target("http_log"))
    })
    .bind((host, port))?
    .run()
    .await
}
