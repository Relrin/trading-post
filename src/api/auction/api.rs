use actix_web::web::{scope, Json};
use actix_web::{get, post, web, HttpResponse, Scope};
use actix_web_validator::Query;
use cdrs_tokio::query::QueryValues;
use cdrs_tokio::types::value::Value;
use validator::Validate;

use crate::api::auction::filters::{FilterParams, ItemNameFilter};
use crate::core::error::Error;
use crate::core::orm::filter::{CustomFilter, Filter, IntoCustomFilter, Operator};
use crate::core::orm::query_builder::{QueryBuilder, QueryType};
use crate::core::orm::session::CassandraSession;
use crate::core::pagination::{PaginatedResponse, PaginationParams};
use crate::models::trade::{CreateTrade, Trade, TRADE_ALL_COLUMNS, TRADE_TABLE};

pub fn get_auction_router() -> Scope {
    scope("/api/v1/auction/trades")
        .service(list_trades)
        .service(create_trade)
}

#[get("")]
async fn list_trades(
    db: web::Data<CassandraSession>,
    pagination: Query<PaginationParams>,
    filters: Query<FilterParams>,
) -> Result<HttpResponse, Error> {
    let mut filter_values: Vec<Value> = Vec::new();
    filter_values.push(false.into());

    let item_name_filter = ItemNameFilter::new(&filters).into_custom_filter();
    let backend_filters: Vec<&CustomFilter> = vec![&item_name_filter]
        .iter()
        .filter(|f| f.is_some())
        .map(|f| f.as_ref().unwrap())
        .collect();

    let query = QueryBuilder::new(&TRADE_TABLE)
        .query_type(QueryType::Select)
        .columns(&TRADE_ALL_COLUMNS)
        .allow_filtering(true)
        .filter_by(Filter::new("is_deleted", Operator::Eq))
        .custom_filters(&backend_filters)
        .build();
    println!("{:?}", query);
    let query_values = QueryValues::SimpleValues(filter_values);

    let objects = query
        .get_paginated_entries::<Trade>(&db, &query_values, &pagination)
        .await?;

    let paginated_response = PaginatedResponse::new(pagination.page, pagination.page_size, objects);
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
