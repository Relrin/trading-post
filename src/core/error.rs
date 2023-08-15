use actix_web::{
    error,
    http::{header::ContentType, StatusCode},
    HttpResponse,
};
use cdrs_tokio::error::Error as CdrsError;
use derive_more::{Display, Error};
use serde_json::{json, Value};
use validator::ValidationErrors;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Display, Error)]
pub enum Error {
    #[display(fmt = "{{\"detail\": \"{0}\", \"errors\": {1}}}", message, errors)]
    ValidationError { message: String, errors: Value },
    #[display(fmt = "{{\"detail\": \"{0}\"}}", message)]
    CassandraError { message: String },
}

impl error::ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match *self {
            Error::ValidationError { .. } => StatusCode::BAD_REQUEST,
            Error::CassandraError { .. } => StatusCode::BAD_REQUEST,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .body(self.to_string())
    }
}

impl From<ValidationErrors> for Error {
    fn from(value: ValidationErrors) -> Self {
        Error::ValidationError {
            message: String::from("Validation error"),
            errors: json!(value.errors()),
        }
    }
}

impl From<CdrsError> for Error {
    fn from(value: CdrsError) -> Self {
        println!("{:?}", value);

        Error::CassandraError {
            message: String::from("Internal error"),
        }
    }
}
