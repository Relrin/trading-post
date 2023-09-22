use tonic::Request;
use uuid::Uuid;

use crate::core::error::Error;
use crate::core::validation::Validate;
use crate::proto::CreateTradeRequest;

impl Validate for Request<CreateTradeRequest> {
    fn validate(&self) -> Result<(), Error> {
        let data = self.get_ref();

        if Uuid::try_parse(&data.item_id).is_err() {
            let message = format!("{0} is not a valid UUID.", &data.item_id);
            return Err(Error::ValidationError {
                field: "item_id".to_string(),
                message,
            });
        };

        Ok(())
    }
}
