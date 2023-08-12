use actix_web::{post, HttpResponse, Scope, web};
use actix_web::web::{Json, scope};
use validator::Validate;

use crate::core::error::Error;
use crate::core::orm::query_builder::{QueryBuilder, QueryType};
use crate::core::orm::session::CassandraSession;
use crate::models::trade::{TRADE_TABLE, TRADE_ALL_COLUMNS, CreateTrade, Trade};

pub fn get_auction_router() -> Scope {
    scope("/api/v1/auction")
        .service(create_trade)
}

#[post("/create")]
async fn create_trade(data: Json<CreateTrade>, db: web::Data<CassandraSession>) -> Result<HttpResponse, Error> {
    data.validate()?;

    let trade = Trade::from(data.into_inner());
    let query = QueryBuilder::new(&TRADE_TABLE)
        .query_type(QueryType::Insert)
        .columns(&TRADE_ALL_COLUMNS)
        .build();
    let query_values = trade.into_query_values();
    query.insert(&db, &query_values).await;

    // read data back and give it back
    
    Ok(HttpResponse::Ok().finish())
}
