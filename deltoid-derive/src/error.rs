//! Defines error infrastructure.

use serde_derive::{Deserialize, Serialize};

#[allow(unused)]
macro_rules! ensure {
    ($predicate:expr) => {
        if $predicate {
            DeriveResult::Ok(())
        } else {
            use $crate::error::{DeriveError, DeriveResult};
            DeriveResult::Err(DeriveError::FailedToEnsure {
                predicate: stringify!($predicate),
                file: file!(),
                line: line!(),
                column: column!(),
            })
        }
    };
}


#[allow(unused)]
macro_rules! bug_detected {
    () => {
        Err($crate::error::DeriveError::BugDetected {
            file: file!(),
            line: line!(),
            column: column!(),
        })
    };
}

pub type DeriveResult<T> = Result<T, DeriveError>;


#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum DeriveError {
    BugDetected {
        file: &'static str,
        line: u32,
        column: u32
    },
    ExpectedNamedField,
    ExpectedPositionalField,
    FailedToEnsure {
        predicate: &'static str,
        file: &'static str,
        line: u32,
        column: u32
    },
    UnsupportedUnion,
    UnknownDeltaAttribute {
        found: String
    },

    // Add more error variants here
}

impl std::fmt::Display for DeriveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BugDetected { file, line, column } => write!(
                f,
                "deltoid-derive: bug detected at {}:{}:{}",
                file, line, column
            ),
            Self::ExpectedNamedField =>
                write!(f, "deltoid-derive: expected a named field"),
            Self::ExpectedPositionalField =>
                write!(f, "deltoid-derive: expected a positional field"),
            Self::FailedToEnsure { predicate, file, line, column } => write!(
                f,
                "deltoid-derive: failed to ensure `{}` at {}:{}:{}",
                predicate, file, line, column
            ),
            Self::UnsupportedUnion =>
                write!(f, "deltoid-derive: unions are not supported"),
            Self::UnknownDeltaAttribute { found } => write!(
                f,
                "deltoid-derive: unknown #[delta(..)] argument \
                 `{}`; expected `ignore_field`",
                found
            ),
        }
    }
}
