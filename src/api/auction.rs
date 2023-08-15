use actix_web::web::{scope, Json};
use actix_web::{get, post, web, HttpResponse, Scope};
use cdrs_tokio::query::QueryValues;
use cdrs_tokio::types::value::Value;
use validator::Validate;

use crate::core::error::Error;
use crate::core::orm::filter::{Filter, Operator};
use crate::core::orm::query_builder::{QueryBuilder, QueryType};
use crate::core::orm::session::CassandraSession;
use crate::core::pagination::PaginatedResponse;
use crate::models::trade::{CreateTrade, Trade, TRADE_ALL_COLUMNS, TRADE_TABLE};

pub fn get_auction_router() -> Scope {
    scope("/api/v1/auction/trades")
        .service(list_trades)
        .service(create_trade)
}

#[get("")]
async fn list_trades(db: web::Data<CassandraSession>) -> Result<HttpResponse, Error> {
    let mut filter_values: Vec<Value> = Vec::new();
    filter_values.push(false.into());

    // TODO: Add pagination options
    // TODO: Add custom filtering by the item_id / item_name, price range

    let query = QueryBuilder::new(&TRADE_TABLE)
        .query_type(QueryType::Select)
        .columns(&TRADE_ALL_COLUMNS)
        .allow_filtering(true)
        .filter_by(Filter::new("is_deleted", Operator::Eq))
        .build();
    let query_values = QueryValues::SimpleValues(filter_values);

    let objects = query
        .get_paginated_entries::<Trade>(&db, &query_values)
        .await?;

    let paginated_response = PaginatedResponse::new(1, 1, objects);
    Ok(HttpResponse::Ok().json(paginated_response))
}

#[post("")]
async fn create_trade(
    data: Json<CreateTrade>,
    db: web::Data<CassandraSession>,
) -> Result<HttpResponse, Error> {
    data.validate()?;

    let trade = Trade::from(data.into_inner());
    let query = QueryBuilder::new(&TRADE_TABLE)
        .query_type(QueryType::Insert)
        .columns(&TRADE_ALL_COLUMNS)
        .build();
    let query_values = trade.into_query_values();
    query.insert(&db, &query_values).await;

    Ok(HttpResponse::Ok().finish())
}
