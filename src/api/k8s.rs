use actix_web::{get, HttpResponse, Scope};
use actix_web::web::scope;

pub fn get_k8s_router() -> Scope {
    scope("/api/v1")
        .service(healthcheck)
}

#[get("/health")]
async fn healthcheck() -> HttpResponse {
    HttpResponse::Ok().body("OK")
}