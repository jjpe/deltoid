//!


pub type DeltaResult<T> = Result<T, DeltaError>;

#[derive(Debug)]
pub enum DeltaError {
    ExpectedValue,
}
