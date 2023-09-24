use tonic::Request;
use uuid::Uuid;

use crate::core::error::Error;
use crate::core::validation::Validate;
use crate::proto::{BidRequest, BuyoutRequest, CancelTradeRequest, CreateTradeRequest};

impl Validate for Request<CreateTradeRequest> {
    fn validate(&self) -> Result<(), Error> {
        let data = self.get_ref();

        if Uuid::try_parse(&data.item_id).is_err() {
            return Err(Error::ValidationError {
                field: "item_id".to_string(),
                message: format!("{0} is not a valid UUID.", &data.item_id),
            });
        };

        if data.item_name.is_empty() {
            return Err(Error::ValidationError {
                field: "item_name".to_string(),
                message: "This field can't be empty.".to_string(),
            });
        }

        if Uuid::try_parse(&data.created_by).is_err() {
            return Err(Error::ValidationError {
                field: "created_by".to_string(),
                message: format!("{0} is not a valid UUID.", &data.item_id),
            });
        };

        if data.created_by_username.is_empty() {
            return Err(Error::ValidationError {
                field: "item_name".to_string(),
                message: "This field can't be empty.".to_string(),
            });
        }

        if data.bid_price <= 0 {
            return Err(Error::ValidationError {
                field: "bid_price".to_string(),
                message: "The item must have an initial price.".to_string(),
            });
        }

        if data.buyout_price > 0 && data.bid_price > data.buyout_price {
            return Err(Error::ValidationError {
                field: "buyout_price".to_string(),
                message: "The buyout price must be greater than the bid price".to_string(),
            });
        }

        if data.expire_in < 0 {
            return Err(Error::ValidationError {
                field: "expire_in".to_string(),
                message: "The expire duration must be zero or a positive value.".to_string(),
            });
        }

        Ok(())
    }
}

impl Validate for Request<BidRequest> {
    fn validate(&self) -> Result<(), Error> {
        let data = self.get_ref();

        if Uuid::try_parse(&data.id).is_err() {
            return Err(Error::ValidationError {
                field: "id".to_string(),
                message: format!("{0} is not a valid UUID.", &data.id),
            });
        }

        if Uuid::try_parse(&data.user_id).is_err() {
            return Err(Error::ValidationError {
                field: "user_id".to_string(),
                message: format!("{0} is not a valid UUID.", &data.user_id),
            });
        }

        if data.username.is_empty() {
            return Err(Error::ValidationError {
                field: "username".to_string(),
                message: "This field can't be empty.".to_string(),
            });
        }

        if data.amount <= 0 {
            return Err(Error::ValidationError {
                field: "amount".to_string(),
                message: "The amount must be a positive value.".to_string(),
            });
        }

        Ok(())
    }
}

impl Validate for Request<BuyoutRequest> {
    fn validate(&self) -> Result<(), Error> {
        let data = self.get_ref();

        if Uuid::try_parse(&data.id).is_err() {
            return Err(Error::ValidationError {
                field: "id".to_string(),
                message: format!("{0} is not a valid UUID.", &data.id),
            });
        }

        if Uuid::try_parse(&data.user_id).is_err() {
            return Err(Error::ValidationError {
                field: "user_id".to_string(),
                message: format!("{0} is not a valid UUID.", &data.user_id),
            });
        }

        if data.username.is_empty() {
            return Err(Error::ValidationError {
                field: "username".to_string(),
                message: "This field can't be empty.".to_string(),
            });
        }

        if data.amount <= 0 {
            return Err(Error::ValidationError {
                field: "amount".to_string(),
                message: "The amount must be a positive value.".to_string(),
            });
        }

        Ok(())
    }
}

impl Validate for Request<CancelTradeRequest> {
    fn validate(&self) -> Result<(), Error> {
        let data = self.get_ref();

        if Uuid::try_parse(&data.id).is_err() {
            return Err(Error::ValidationError {
                field: "id".to_string(),
                message: format!("{0} is not a valid UUID.", &data.id),
            });
        }

        if Uuid::try_parse(&data.user_id).is_err() {
            return Err(Error::ValidationError {
                field: "user_id".to_string(),
                message: format!("{0} is not a valid UUID.", &data.user_id),
            });
        }

        Ok(())
    }
}
