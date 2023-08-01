use actix_web::{
    error,
    http::{header::ContentType, StatusCode},
    HttpResponse,
};
use derive_more::{Display, Error};
use validator::{ValidationErrors};

#[derive(Debug, Display, Error)]
pub enum Error {
    #[display(fmt = "{{\"detail\": \"{0}\"}}", message)]
    ValidationError { message: String, errors: ValidationErrors },
}

impl error::ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::json())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            Error::ValidationError {..} => StatusCode::BAD_REQUEST,
        }
    }
}