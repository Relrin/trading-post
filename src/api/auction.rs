use actix_web::{post, HttpResponse, Scope};
use actix_web::web::{Json, scope};
use validator::Validate;

use crate::core::error::Error;
use crate::models::trade::CreateTrade;

pub fn get_auction_router() -> Scope {
    scope("/api/v1/auction")
        .service(create_trade)
}

#[post("/create")]
async fn create_trade(data: Json<CreateTrade>) -> Result<HttpResponse, Error> {
    data.validate()?;

    Ok(HttpResponse::Ok().finish())
}
