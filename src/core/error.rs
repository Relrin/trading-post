use cdrs_tokio::error::Error as CdrsError;
use derive_more::Display;
use tonic::{Code, Status};
use tonic_types::{ErrorDetails, StatusExt};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Display)]
pub enum Error {
    #[display(fmt = "Validation error for the `{0}` field: {1}", field, message)]
    ValidationError {
        field: String,
        message: String,
    },
    CassandraError(String),
}

impl Error {
    fn code(&self) -> Code {
        match self {
            Error::ValidationError { .. } => Code::InvalidArgument,
            Error::CassandraError(_) => Code::Internal,
        }
    }

    fn message(&self) -> String {
        self.code().description().to_string()
    }

    fn details(&self) -> ErrorDetails {
        let mut details = ErrorDetails::new();

        match self {
            Error::ValidationError { field, message } => {
                details.add_bad_request_violation(field, message);
            }
            _ => {}
        };

        details
    }
}

impl From<CdrsError> for Error {
    fn from(_: CdrsError) -> Self {
        Error::CassandraError("Internal error".to_string())
    }
}

impl From<Error> for Status {
    fn from(err: Error) -> Self {
        let code = err.code();
        let message = err.message();
        let details = err.details();

        Status::with_error_details(code, message, details)
    }
}
