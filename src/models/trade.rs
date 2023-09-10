use cdrs_tokio::query::QueryValues;
use cdrs_tokio::query_values;
use cdrs_tokio_helpers_derive::{IntoCdrsValue, TryFromRow};
use chrono::{DateTime, Days, Duration, Utc};
use go_parse_duration::parse_duration;
use lazy_static::lazy_static;
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize};
use uuid::Uuid;
use validator::{Validate, ValidationError};

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
    static ref EMPTY_UUID: Uuid = Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap();
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
    #[serde(skip)]
    is_deleted: bool,
}

#[derive(Debug, PartialEq, Deserialize, Validate)]
#[validate(schema(function = "validate_create_trade", skip_on_field_errors = false))]
pub struct CreateTrade {
    item_id: Uuid,
    #[validate(length(min = 1))]
    item_name: String,
    #[validate(range(min = 0))]
    bid_price: i64,
    buyout_price: Option<i64>,
    created_by: Uuid,
    #[validate(length(min = 1))]
    created_by_username: String,
    #[serde(deserialize_with = "parse_expire_in")]
    expire_in: Option<Duration>,
}

#[derive(Deserialize, Debug, Validate)]
pub struct TradeOperation {
    #[validate(range(min = 1))]
    price: i64,
}

fn validate_create_trade(instance: &CreateTrade) -> Result<(), ValidationError> {
    if let Some(buyout_price) = instance.buyout_price {
        if buyout_price > 0 && instance.bid_price > buyout_price {
            return Err(ValidationError::new(
                "The bid can't be greater than the buyout price",
            ));
        }
    }

    Ok(())
}

fn parse_expire_in<'de, D>(deserializer: D) -> Result<Option<Duration>, D::Error>
where
    D: Deserializer<'de>,
{
    let input = String::deserialize(deserializer)?;
    match parse_duration(&input) {
        Ok(duration_ns) => Ok(Some(Duration::nanoseconds(duration_ns))),
        Err(_) => Err(Error::custom("Duration format is invalid")),
    }
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

impl From<CreateTrade> for Trade {
    fn from(instance: CreateTrade) -> Self {
        let buyout_price = match instance.buyout_price {
            Some(value) => value,
            None => 0,
        };

        let created_at = Utc::now();
        let expired_at = match instance.expire_in {
            Some(duration) => Utc::now() + duration,
            // Set a date before the created_at date to indicate that expiration wasn't set
            None => created_at - Days::new(1),
        };

        Trade {
            id: Uuid::new_v4(),
            item_id: instance.item_id,
            item_name: instance.item_name,
            bid_price: instance.bid_price,
            buyout_price,
            created_by: instance.created_by,
            created_by_username: instance.created_by_username.clone(),
            created_at,
            bought_by: *EMPTY_UUID,
            bought_by_username: String::new(),
            expired_at,
            is_deleted: false,
        }
    }
}
