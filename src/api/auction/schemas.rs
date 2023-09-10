use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Deserialize)]
pub struct TradeDetail {
    pub(crate) id: Uuid,
}

#[derive(Debug, Validate, Deserialize)]
pub struct TradeBid {
    pub(crate) user_id: Uuid,
    pub(crate) username: String,
    #[validate(range(min = 0))]
    pub(crate) amount: i64,
}

#[derive(Debug, Validate, Deserialize)]
pub struct TradeBuyout {
    pub(crate) user_id: Uuid,
    pub(crate) username: String,
    #[validate(range(min = 0))]
    pub(crate) amount: i64,
}
