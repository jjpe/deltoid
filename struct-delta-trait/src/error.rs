//!


pub type DeltaResult<T> = Result<T, DeltaError>;

#[derive(Clone, Debug)]
pub enum DeltaError {
    ExpectedValue,
    FailedToEnsure {
        predicate: &'static str,
        msg: String,
        file: &'static str,
        line: u32,
        column: u32
    },
}
