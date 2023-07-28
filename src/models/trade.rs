use chrono::{DateTime, Days, Utc};
use serde::{Serialize, Deserialize};
use validator::{Validate, ValidationError};
use uuid::{Uuid};

#[derive(Serialize, Deserialize, Debug, Validate)]
#[validate(schema(function = "validate_trade", skip_on_field_errors = false))]
pub struct Trade {
    #[serde(default = "init_id")]
    id: Uuid,
    item_id: Uuid,
    #[validate(length(min = 1))]
    item_name: String,
    #[validate(range(min = 0))]
    bid_price: i64,
    #[validate(range(min = 0))]
    buyout_price: i64,
    created_by: Uuid,
    #[validate(length(min = 1))]
    created_by_username: String,
    #[serde(skip_deserializing, default="init_created_at")]
    created_at: DateTime<Utc>,
    #[serde(skip_deserializing, default="init_expired_at")]
    expired_at: DateTime<Utc>,
    #[serde(skip)]
    is_deleted: bool,
}

#[derive(Deserialize, Debug)]
pub struct TradeOperation {
    price: i64,
}

fn validate_trade(instance: &Trade) -> Result<(), ValidationError> {
    if let Some(buyout_price) = instance.buyout_price {
        if buyout_price > 0 && instance.bid_price > buyout_price {
            return Err(ValidationError::new("The bid can't be greater a buyout price"))
        }
    }

    Ok(())
}

fn init_id() -> Uuid {
    Uuid::new_v4()
}

fn init_created_at() -> DateTime<Utc> {
    Utc::now()
}

// Set a date before the created_at date to indicate that expiration wasn't set
fn init_expired_at() -> DateTime<Utc> {
    Utc::now() - Days::new(1)
}
