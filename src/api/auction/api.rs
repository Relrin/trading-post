use cdrs_tokio::query_values;
//use chrono::Utc;
use tonic::{Request, Response, Status};
use uuid::Uuid;

use crate::api::auction::filters::{
    ItemBidPriceRangeFilter, ItemBuyoutPriceRangeFilter, ItemNameFilter,
};
use crate::core::error::Error;
use crate::core::orm::filter::{CustomFilter, Filter, IntoCustomFilter, Operator};
use crate::core::orm::query_builder::{QueryBuilder, QueryType};
use crate::core::orm::session::CassandraSession;
use crate::core::pagination::PaginationParams;
use crate::core::validation::Validate;
use crate::models::trade::{Trade, EMPTY_UUID, TRADE_ALL_COLUMNS, TRADE_TABLE};
use crate::proto::{
    auction_server::Auction, BidRequest, BidResponse, BuyoutRequest, BuyoutResponse,
    CancelTradeRequest, CancelTradeResponse, CreateTradeRequest, CreateTradeResponse,
    ListTradesRequest, ListTradesResponse, Trade as TradeDetail,
};

pub struct AuctionServiceImpl {
    db: CassandraSession,
}

impl AuctionServiceImpl {
    pub fn new(db: CassandraSession) -> Self {
        Self { db }
    }
}

#[tonic::async_trait]
impl Auction for AuctionServiceImpl {
    async fn list_trades(
        &self,
        request: Request<ListTradesRequest>,
    ) -> Result<Response<ListTradesResponse>, Status> {
        let params = request.into_inner();
        let filter_params = params.filter_params.unwrap_or_default();

        let item_name_filter = ItemNameFilter::new(&filter_params).into_custom_filter();
        let item_bid_price_filter =
            ItemBidPriceRangeFilter::new(&filter_params).into_custom_filter();
        let item_buyout_price_filter =
            ItemBuyoutPriceRangeFilter::new(&filter_params).into_custom_filter();

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

        let pagination_params = PaginationParams::new(params.page, params.page_size);
        let trades = query
            .get_paginated_entries::<Trade>(&self.db, &pagination_params)
            .await?;

        Ok(Response::new(ListTradesResponse {
            page: pagination_params.page,
            page_size: pagination_params.page_size,
            trades: trades
                .iter()
                .map(|trade| TradeDetail::from(trade))
                .collect(),
        }))
    }

    async fn create_trade(
        &self,
        request: Request<CreateTradeRequest>,
    ) -> Result<Response<CreateTradeResponse>, Status> {
        request.validate()?;

        let trade = Trade::from(request.into_inner());
        let query = QueryBuilder::new(&TRADE_TABLE)
            .query_type(QueryType::Insert)
            .columns(&TRADE_ALL_COLUMNS)
            .build();
        let query_values = trade.into_query_values();
        query.insert(&self.db, &query_values).await;

        Ok(Response::new(CreateTradeResponse {}))
    }

    async fn bid(&self, request: Request<BidRequest>) -> Result<Response<BidResponse>, Status> {
        request.validate()?;
        let data = request.get_ref();
        let trade_id = Uuid::parse_str(&data.id).expect("parse valid uuid from request");
        let user_id = Uuid::parse_str(&data.user_id).expect("parse valid uuid from request");

        let read_query = QueryBuilder::new(&TRADE_TABLE)
            .query_type(QueryType::Select)
            .columns(&TRADE_ALL_COLUMNS)
            .limit(1)
            .filter_by(Filter::new("id", Operator::Eq, Some(trade_id.into())))
            .filter_by(Filter::new("is_deleted", Operator::Eq, Some(false.into())))
            .allow_filtering(true)
            .build();
        let trade = read_query.get_instance::<Trade>(&self.db).await?;

        if data.amount <= trade.bid_price() {
            return Err(Status::from(Error::ValidationError {
                field: "amount".to_string(),
                message: "The bid can't be less that the current price.".to_string(),
            }));
        }

        if data.amount >= trade.buyout_price() {
            return Err(Status::from(Error::ValidationError {
                field: "amount".to_string(),
                message: "The bid can't be greater that the buyout price.".to_string(),
            }));
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
            "bought_by" => user_id,
            "bought_by_username" => data.username.to_owned()
        );
        update_query
            .update(&self.db, &update_query_values)
            .await
            .map_err(|_| {
                Error::CassandraError(String::from(
                    "The item expired or was bought by other player.",
                ))
            })?;

        // TODO: Return currency to the latest bidder

        Ok(Response::new(BidResponse {}))
    }

    async fn buyout(
        &self,
        request: Request<BuyoutRequest>,
    ) -> Result<Response<BuyoutResponse>, Status> {
        todo!()
    }

    async fn cancel_trade(
        &self,
        request: Request<CancelTradeRequest>,
    ) -> Result<Response<CancelTradeResponse>, Status> {
        todo!()
    }
}

// #[put("/{id}/buyout")]
// async fn buyout_trade(
//     detail: web::Path<TradeDetail>,
//     data: Json<TradeBuyout>,
//     db: web::Data<CassandraSession>,
// ) -> Result<HttpResponse, Error> {
//     data.validate()?;
//     let trade_id = detail.into_inner().id;
//
//     let read_query = QueryBuilder::new(&TRADE_TABLE)
//         .query_type(QueryType::Select)
//         .columns(&TRADE_ALL_COLUMNS)
//         .limit(1)
//         .filter_by(Filter::new("id", Operator::Eq, Some(trade_id.into())))
//         .filter_by(Filter::new("is_deleted", Operator::Eq, Some(false.into())))
//         .allow_filtering(true)
//         .build();
//     let trade = read_query.get_instance::<Trade>(&db).await?;
//
//     if data.amount != trade.buyout_price() {
//         return Err(Error::ValidationError {
//             message: String::from("Validation error"),
//             errors: json!({"amount": "The amount of currency must correspond to the buyout price."}),
//         });
//     }
//
//     let update_query = QueryBuilder::new(&TRADE_TABLE)
//         .query_type(QueryType::Update)
//         .columns(&[
//             "bought_by",
//             "bought_by_username",
//             "is_deleted",
//             "expired_at",
//         ])
//         .filter_by(Filter::new("id", Operator::Eq, Some(trade_id.into())))
//         .filter_by(Filter::new(
//             "item_id",
//             Operator::Eq,
//             Some(trade.item_id().into()),
//         ))
//         .filter_by(Filter::new(
//             "created_by",
//             Operator::Eq,
//             Some(trade.created_by().into()),
//         ))
//         .build();
//     let update_query_values = query_values!(
//         "bought_by" => data.user_id,
//         "bought_by_username" => data.username.to_owned(),
//         "is_deleted" => true,
//         "expired_at" => Utc::now()
//     );
//     update_query
//         .update(&db, &update_query_values)
//         .await
//         .map_err(|_| Error::CassandraError {
//             message: String::from("The item expired or was bought by other player."),
//         })?;
//
//     // TODO: Return currency to the latest bidder
//     // TODO: Add the item to buyer's inventory
//
//     Ok(HttpResponse::Ok().finish())
// }
//
// #[delete("/{id}")]
// async fn delete_trade(
//     detail: web::Path<TradeDetail>,
//     data: Json<TradeDelete>,
//     db: web::Data<CassandraSession>,
// ) -> Result<HttpResponse, Error> {
//     let trade_id = detail.into_inner().id;
//
//     let read_query = QueryBuilder::new(&TRADE_TABLE)
//         .query_type(QueryType::Select)
//         .columns(&TRADE_ALL_COLUMNS)
//         .limit(1)
//         .filter_by(Filter::new("id", Operator::Eq, Some(trade_id.into())))
//         .filter_by(Filter::new("is_deleted", Operator::Eq, Some(false.into())))
//         .allow_filtering(true)
//         .build();
//     let trade = read_query.get_instance::<Trade>(&db).await?;
//
//     if trade.bought_by() != *EMPTY_UUID {
//         return Err(Error::ValidationError {
//             message: String::from("Validation error"),
//             errors: json!({"amount": "The trade can't be deleted when someone did a bid."}),
//         });
//     }
//
//     if data.user_id != trade.created_by() {
//         return Err(Error::ValidationError {
//             message: String::from("Validation error"),
//             errors: json!({"amount": "Only the owner can delete the trade."}),
//         });
//     }
//
//     let delete_query = QueryBuilder::new(&TRADE_TABLE)
//         .query_type(QueryType::Update)
//         .columns(&["is_deleted"])
//         .filter_by(Filter::new("id", Operator::Eq, Some(trade_id.into())))
//         .filter_by(Filter::new(
//             "item_id",
//             Operator::Eq,
//             Some(trade.item_id().into()),
//         ))
//         .filter_by(Filter::new(
//             "created_by",
//             Operator::Eq,
//             Some(trade.created_by().into()),
//         ))
//         .build();
//     let delete_query_values = query_values!(
//         "is_deleted" => true,
//         "expired_at" => Utc::now()
//     );
//     delete_query
//         .update(&db, &delete_query_values)
//         .await
//         .map_err(|_| Error::CassandraError {
//             message: String::from("The trade was not found."),
//         })?;
//
//     // TODO: Return an item to an inventory
//
//     Ok(HttpResponse::Ok().finish())
// }
