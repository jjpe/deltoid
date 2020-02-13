//!

use serde_json::{Error as JsonError};

pub type DeltaResult<T> = Result<T, DeltaError>;

#[derive(Debug)]
pub enum DeltaError {
    JsonError(JsonError),
    ExpectedValue,
}

impl From<JsonError> for DeltaError {
    fn from(err: JsonError) -> Self { Self::JsonError(err) }
}
