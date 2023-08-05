use chrono::{DateTime, Days, Utc, Duration};
use go_parse_duration::{parse_duration};
use lazy_static::lazy_static;
use serde::{Serialize, Deserialize, Deserializer};
use serde::de::Error;
use validator::{Validate, ValidationError};
use uuid::{Uuid};

lazy_static! {
    static ref EMPTY_UUID: Uuid = Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap();
}

#[derive(Serialize, Debug)]
pub struct Trade {
    id: Uuid,
    item_id: Uuid,
    item_name: String,
    bid_price: i64,
    buyout_price: i64,
    created_by: Uuid,
    created_by_username: String,
    created_at: DateTime<Utc>,
    buyer_by: Uuid,
    buyer_username: String,
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
            return Err(ValidationError::new("The bid can't be greater than the buyout price"))
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
            id:  Uuid::new_v4(),
            item_id: instance.item_id,
            item_name: instance.item_name,
            bid_price: instance.bid_price,
            buyout_price,
            created_by: instance.created_by,
            created_by_username: instance.created_by_username.clone(),
            created_at,
            buyer_by: *EMPTY_UUID,
            buyer_username: String::new(),
            expired_at,
            is_deleted: false,
        }
    }
}
