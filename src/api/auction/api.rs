use actix_web::web::{scope, Json};
use actix_web::{get, post, put, web, HttpResponse, Scope};
use actix_web_validator::Query;
use cdrs_tokio::query::QueryValues;
use cdrs_tokio::query_values;
use cdrs_tokio::types::value::Value;
use serde_json::json;
use validator::Validate;

use crate::api::auction::filters::{
    FilterParams, ItemBidPriceRangeFilter, ItemBuyoutPriceRangeFilter, ItemNameFilter,
};
use crate::api::auction::schemas::{TradeBid, TradeDetail};
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
        .service(bid_trade)
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
    let item_bid_price_filter = ItemBidPriceRangeFilter::new(&filters).into_custom_filter();
    let item_buyout_price_filter = ItemBuyoutPriceRangeFilter::new(&filters).into_custom_filter();

    let backend_filters: Vec<&CustomFilter> = vec![
        &item_name_filter,
        &item_bid_price_filter,
        &item_buyout_price_filter,
    ]
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

#[put("/{id}/bid")]
async fn bid_trade(
    detail: web::Path<TradeDetail>,
    data: Json<TradeBid>,
    db: web::Data<CassandraSession>,
) -> Result<HttpResponse, Error> {
    data.validate()?;
    let trade_id = detail.into_inner().id;

    let mut filter_values: Vec<Value> = Vec::new();
    filter_values.push(false.into());
    filter_values.push(trade_id.into());

    let read_query = QueryBuilder::new(&TRADE_TABLE)
        .query_type(QueryType::Select)
        .columns(&TRADE_ALL_COLUMNS)
        .limit(1)
        .filter_by(Filter::new("id", Operator::Eq))
        .filter_by(Filter::new("is_deleted", Operator::Eq))
        .allow_filtering(true)
        .build();
    let read_query_values = QueryValues::SimpleValues(filter_values);
    let trade = read_query
        .get_instance::<Trade>(&db, &read_query_values)
        .await?;

    if data.amount <= trade.bid_price() {
        return Err(Error::ValidationError {
            message: String::from("Validation error"),
            errors: json!({"amount": "The bid can't be less that the current price."}),
        });
    }

    if data.amount >= trade.buyout_price() {
        return Err(Error::ValidationError {
            message: String::from("Validation error"),
            errors: json!({"amount": "The bid can't be greater that the buyout price."}),
        });
    }

    let update_query = QueryBuilder::new(&TRADE_TABLE)
        .query_type(QueryType::Update)
        .columns(&["bid_price", "bought_by", "bought_by_username"])
        .filter_by(Filter::new("id", Operator::Eq))
        .filter_by(Filter::new("item_id", Operator::Eq))
        .filter_by(Filter::new("created_by", Operator::Eq))
        .build();
    let update_query_values = query_values!(
        "id" => trade_id,
        "item_id" => trade.item_id(),
        "created_by" => trade.created_by(),
        "bid_price" => data.amount,
        "bought_by" => data.user_id,
        "bought_by_username" => data.username.to_owned()
    );
    update_query
        .update(&db, &update_query_values)
        .await
        .map_err(|_| Error::CassandraError {
            message: String::from("Object was not found or doesn't exist."),
        })?;

    Ok(HttpResponse::Ok().finish())
}
