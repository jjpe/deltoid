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

    // Add more error variants here
}
