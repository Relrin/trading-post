use actix_web::{post, HttpResponse, Scope};
use actix_web::web::scope;
use actix_web_validator::{Json};

use crate::models::trade::CreateTrade;

pub fn get_auction_router() -> Scope {
    scope("/api/v1/auction")
        .service(create_trade)
}

#[post("/create")]
async fn create_trade(data: Json<CreateTrade>) -> HttpResponse {
    HttpResponse::Ok().body("OK")
}
