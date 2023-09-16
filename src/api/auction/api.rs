use actix_web::web::{scope, Json};
use actix_web::{delete, get, post, put, web, HttpResponse, Scope};
use actix_web_validator::Query;
use cdrs_tokio::query_values;
use chrono::Utc;
use serde_json::json;
use validator::Validate;

use crate::api::auction::filters::{
    FilterParams, ItemBidPriceRangeFilter, ItemBuyoutPriceRangeFilter, ItemNameFilter,
};
use crate::api::auction::schemas::{TradeBid, TradeBuyout, TradeDelete, TradeDetail};
use crate::core::error::Error;
use crate::core::orm::filter::{CustomFilter, Filter, IntoCustomFilter, Operator};
use crate::core::orm::query_builder::{QueryBuilder, QueryType};
use crate::core::orm::session::CassandraSession;
use crate::core::pagination::{PaginatedResponse, PaginationParams};
use crate::models::trade::{CreateTrade, Trade, EMPTY_UUID, TRADE_ALL_COLUMNS, TRADE_TABLE};

pub fn get_auction_router() -> Scope {
    scope("/api/v1/auction/trades")
        .service(list_trades)
        .service(create_trade)
        .service(bid_trade)
        .service(buyout_trade)
        .service(delete_trade)
}

#[get("")]
async fn list_trades(
    db: web::Data<CassandraSession>,
    pagination: Query<PaginationParams>,
    filters: Query<FilterParams>,
) -> Result<HttpResponse, Error> {
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
        .filter_by(Filter::new("is_deleted", Operator::Eq, Some(false.into())))
        .custom_filters(&backend_filters)
        .build();

    let objects = query
        .get_paginated_entries::<Trade>(&db, &pagination)
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

    let read_query = QueryBuilder::new(&TRADE_TABLE)
        .query_type(QueryType::Select)
        .columns(&TRADE_ALL_COLUMNS)
        .limit(1)
        .filter_by(Filter::new("id", Operator::Eq, Some(trade_id.into())))
        .filter_by(Filter::new("is_deleted", Operator::Eq, Some(false.into())))
        .allow_filtering(true)
        .build();
    let trade = read_query.get_instance::<Trade>(&db).await?;

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
        .filter_by(Filter::new("id", Operator::Eq, Some(trade_id.into())))
        .filter_by(Filter::new(
            "item_id",
            Operator::Eq,
            Some(trade.item_id().into()),
        ))
        .filter_by(Filter::new(
            "created_by",
            Operator::Eq,
            Some(trade.created_by().into()),
        ))
        .build();
    let update_query_values = query_values!(
        "bid_price" => data.amount,
        "bought_by" => data.user_id,
        "bought_by_username" => data.username.to_owned()
    );
    update_query
        .update(&db, &update_query_values)
        .await
        .map_err(|_| Error::CassandraError {
            message: String::from("The item expired or was bought by other player."),
        })?;

    // TODO: Return currency to the latest bidder

    Ok(HttpResponse::Ok().finish())
}

#[put("/{id}/buyout")]
async fn buyout_trade(
    detail: web::Path<TradeDetail>,
    data: Json<TradeBuyout>,
    db: web::Data<CassandraSession>,
) -> Result<HttpResponse, Error> {
    data.validate()?;
    let trade_id = detail.into_inner().id;

    let read_query = QueryBuilder::new(&TRADE_TABLE)
        .query_type(QueryType::Select)
        .columns(&TRADE_ALL_COLUMNS)
        .limit(1)
        .filter_by(Filter::new("id", Operator::Eq, Some(trade_id.into())))
        .filter_by(Filter::new("is_deleted", Operator::Eq, Some(false.into())))
        .allow_filtering(true)
        .build();
    let trade = read_query.get_instance::<Trade>(&db).await?;

    if data.amount != trade.buyout_price() {
        return Err(Error::ValidationError {
            message: String::from("Validation error"),
            errors: json!({"amount": "The amount of currency must correspond to the buyout price."}),
        });
    }

    let update_query = QueryBuilder::new(&TRADE_TABLE)
        .query_type(QueryType::Update)
        .columns(&[
            "bought_by",
            "bought_by_username",
            "is_deleted",
            "expired_at",
        ])
        .filter_by(Filter::new("id", Operator::Eq, Some(trade_id.into())))
        .filter_by(Filter::new(
            "item_id",
            Operator::Eq,
            Some(trade.item_id().into()),
        ))
        .filter_by(Filter::new(
            "created_by",
            Operator::Eq,
            Some(trade.created_by().into()),
        ))
        .build();
    let update_query_values = query_values!(
        "bought_by" => data.user_id,
        "bought_by_username" => data.username.to_owned(),
        "is_deleted" => true,
        "expired_at" => Utc::now()
    );
    update_query
        .update(&db, &update_query_values)
        .await
        .map_err(|_| Error::CassandraError {
            message: String::from("The item expired or was bought by other player."),
        })?;

    // TODO: Return currency to the latest bidder
    // TODO: Add the item to buyer's inventory

    Ok(HttpResponse::Ok().finish())
}

#[delete("/{id}")]
async fn delete_trade(
    detail: web::Path<TradeDetail>,
    data: Json<TradeDelete>,
    db: web::Data<CassandraSession>,
) -> Result<HttpResponse, Error> {
    let trade_id = detail.into_inner().id;

    let read_query = QueryBuilder::new(&TRADE_TABLE)
        .query_type(QueryType::Select)
        .columns(&TRADE_ALL_COLUMNS)
        .limit(1)
        .filter_by(Filter::new("id", Operator::Eq, Some(trade_id.into())))
        .filter_by(Filter::new("is_deleted", Operator::Eq, Some(false.into())))
        .allow_filtering(true)
        .build();
    let trade = read_query.get_instance::<Trade>(&db).await?;

    if trade.bought_by() != *EMPTY_UUID {
        return Err(Error::ValidationError {
            message: String::from("Validation error"),
            errors: json!({"amount": "The trade can't be deleted when someone did a bid."}),
        });
    }

    if data.user_id != trade.created_by() {
        return Err(Error::ValidationError {
            message: String::from("Validation error"),
            errors: json!({"amount": "Only the owner can delete the trade."}),
        });
    }

    let delete_query = QueryBuilder::new(&TRADE_TABLE)
        .query_type(QueryType::Update)
        .columns(&["is_deleted"])
        .filter_by(Filter::new("id", Operator::Eq, Some(trade_id.into())))
        .filter_by(Filter::new(
            "item_id",
            Operator::Eq,
            Some(trade.item_id().into()),
        ))
        .filter_by(Filter::new(
            "created_by",
            Operator::Eq,
            Some(trade.created_by().into()),
        ))
        .build();
    let delete_query_values = query_values!(
        "is_deleted" => true,
        "expired_at" => Utc::now()
    );
    delete_query
        .update(&db, &delete_query_values)
        .await
        .map_err(|_| Error::CassandraError {
            message: String::from("The trade was not found."),
        })?;

    // TODO: Return an item to an inventory

    Ok(HttpResponse::Ok().finish())
}
