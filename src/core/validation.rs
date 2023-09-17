use crate::core::error::Error;

pub trait Validate {
    fn validate(&self) -> Result<(), Error>;
}
