use actix_web::{post, HttpResponse, Scope};
use actix_web::web::{Json, scope};
use validator::Validate;

use crate::core::error::Error;
use crate::core::orm::query_builder::{QueryBuilder, QueryType};
use crate::models::trade::{TRADE_TABLE, TRADE_ALL_COLUMNS, CreateTrade};

pub fn get_auction_router() -> Scope {
    scope("/api/v1/auction")
        .service(create_trade)
}

#[post("/create")]
async fn create_trade(data: Json<CreateTrade>) -> Result<HttpResponse, Error> {
    data.validate()?;

    let create_trade = data.into_inner();
    let query = QueryBuilder::new(&TRADE_TABLE)
        .query_type(QueryType::Insert)
        .columns(&TRADE_ALL_COLUMNS)
        .build();

    // read data back and give it back
    
    Ok(HttpResponse::Ok().finish())
}
