use cdrs_tokio::query::QueryValues;
use cdrs_tokio::query_values;
use cdrs_tokio_helpers_derive::{IntoCdrsValue, TryFromRow};
use chrono::{DateTime, Days, Utc};
use lazy_static::lazy_static;
use serde::Serialize;
use std::str::FromStr;
use std::time::Duration;
use uuid::Uuid;

use crate::proto::{CreateTradeRequest, Trade as TradeDetail};

lazy_static! {
    pub static ref TRADE_TABLE: &'static str = "trading_post.trade";
    pub static ref TRADE_ALL_COLUMNS: &'static [&'static str] = &[
        "id",
        "item_id",
        "item_name",
        "bid_price",
        "buyout_price",
        "created_by",
        "created_by_username",
        "created_at",
        "bought_by",
        "bought_by_username",
        "expired_at",
        "is_deleted",
    ];
    pub static ref EMPTY_UUID: Uuid =
        Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap();
}

#[derive(Serialize, IntoCdrsValue, TryFromRow, Debug)]
pub struct Trade {
    id: Uuid,
    item_id: Uuid,
    item_name: String,
    bid_price: i64,
    buyout_price: i64,
    created_by: Uuid,
    created_by_username: String,
    created_at: DateTime<Utc>,
    bought_by: Uuid,
    bought_by_username: String,
    expired_at: DateTime<Utc>,
    is_deleted: bool,
}

impl Trade {
    pub fn item_id(&self) -> Uuid {
        self.item_id
    }

    pub fn created_by(&self) -> Uuid {
        self.created_by
    }

    pub fn bid_price(&self) -> i64 {
        self.bid_price
    }

    pub fn buyout_price(&self) -> i64 {
        self.buyout_price
    }

    pub fn bought_by(&self) -> Uuid {
        self.bought_by
    }

    pub fn into_query_values(self) -> QueryValues {
        query_values!(
            "id" => self.id,
            "item_id" => self.item_id,
            "item_name" => self.item_name,
            "bid_price" => self.bid_price,
            "buyout_price" => self.buyout_price,
            "created_by" => self.created_by,
            "created_by_username" => self.created_by_username,
            "created_at" => self.created_at,
            "bought_by" => self.bought_by,
            "bought_by_username" => self.bought_by_username,
            "expired_at" => self.expired_at,
            "is_deleted" => self.is_deleted
        )
    }
}

impl From<CreateTradeRequest> for Trade {
    fn from(request: CreateTradeRequest) -> Self {
        let created_at = Utc::now();
        let expired_at = match request.expire_in {
            // Set a date before the created_at date to indicate that expiration wasn't set
            0 => created_at - Days::new(1),
            _ => created_at + Duration::from_secs(request.expire_in as u64),
        };

        Self {
            id: Uuid::new_v4(),
            item_id: Uuid::from_str(&request.item_id).unwrap(),
            item_name: request.item_name,
            bid_price: request.bid_price,
            buyout_price: request.buyout_price,
            created_by: Uuid::from_str(&request.created_by).unwrap(),
            created_by_username: request.created_by_username,
            created_at,
            bought_by: *EMPTY_UUID,
            bought_by_username: String::new(),
            expired_at,
            is_deleted: false,
        }
    }
}

impl From<&Trade> for TradeDetail {
    fn from(instance: &Trade) -> Self {
        let created_at = instance.created_at.timestamp();
        let expired_at = match instance.expired_at > instance.created_at {
            true => Some(instance.expired_at.timestamp()),
            false => None,
        };

        Self {
            id: instance.id.to_string(),
            item_id: instance.item_id.to_string(),
            item_name: instance.item_name.clone(),
            bid_price: instance.bid_price,
            buyout_price: instance.buyout_price,
            created_at,
            created_by: instance.created_by.to_string(),
            created_by_username: instance.created_by_username.clone(),
            bought_by: instance.bought_by.to_string(),
            bought_by_username: instance.bought_by_username.to_string(),
            expired_at,
        }
    }
}
