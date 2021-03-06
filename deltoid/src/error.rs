//!

use crate::{Core, Apply, Delta, FromDelta, IntoDelta};
use serde_derive::{Deserialize, Serialize};
use std::sync::TryLockError;


#[macro_export]
macro_rules! ensure_eq {
    ($left:expr, $right:expr) => {{
        let (lstr, rstr) = (stringify!($left), stringify!($right));
        ensure!(
            $left == $right,
            "violation: {} == {}\n{} == {}\n{} == {}",
            lstr, rstr,
            lstr, $left,
            rstr, $right
        )
    }};
}

#[macro_export]
macro_rules! ensure_ne {
    ($left:expr, $right:expr) => {{
        let (lstr, rstr) = (stringify!($left), stringify!($right));
        ensure!(
            $left != $right,
            "violation: {} != {}\n{} == {}\n{} == {}",
            lstr, rstr,
            lstr, $left,
            rstr, $right
        )
    }};
}

#[macro_export]
macro_rules! ensure_gt {
    ($left:expr, $right:expr) => {{
        let (lstr, rstr) = (stringify!($left), stringify!($right));
        ensure!(
            $left > $right,
            "violation: {} > {}\n{} == {}\n{} == {}",
            lstr, rstr,
            lstr, $left,
            rstr, $right
        )
    }};
}

#[macro_export]
macro_rules! ensure_lt {
    ($left:expr, $right:expr) => {{
        let (lstr, rstr) = (stringify!($left), stringify!($right));
        ensure!(
            $left < $right,
            "violation: {} < {}\n{} == {}\n{} == {}",
            lstr, rstr,
            lstr, $left,
            rstr, $right
        )
    }};
}

#[macro_export]
macro_rules! ensure_ge {
    ($left:expr, $right:expr) => {{
        let (lstr, rstr) = (stringify!($left), stringify!($right));
        ensure!(
            $left >= $right,
            "violation: {} >= {}\n{} == {}\n{} == {}",
            lstr, rstr,
            lstr, $left,
            rstr, $right
        )
    }};
}

#[macro_export]
macro_rules! ensure_le {
    ($left:expr, $right:expr) => {{
        let (lstr, rstr) = (stringify!($left), stringify!($right));
        ensure!(
            $left <= $right,
            "violation: {} <= {}\n{} == {}\n{} == {}",
            lstr, rstr,
            lstr, $left,
            rstr, $right
        )
    }};
}

#[macro_export]
macro_rules! ensure {
    ($predicate:expr $(, $fmt:expr $(, $args:expr)*)? ) => {
        if $predicate {
            $crate::error::DeltaResult::Ok(())
        } else {
            Err($crate::error::DeltaError::FailedToEnsure {
                predicate: stringify!($predicate).to_string(),
                msg: {
                    #[allow(unused)] let mut msg = String::new();
                    $(  msg = format!($fmt $(, $args)*);  )?
                    msg
                },
                file: file!().to_string(),
                line: line!(),
                column: column!(),
            })
        }
    };
}

#[macro_export]
macro_rules! bug_detected {
    ($($fmt:expr $(, $args:expr)*)?) => {
        Err($crate::error::DeltaError::BugDetected {
            msg: { #[allow(redundant_semicolons)] {
                #[allow(unused)] let mut msg = String::new();
                $(  msg = format!($fmt $(, $args)*);  )? ;
                msg
            }},
            file: file!().to_string(),
            line: line!(),
            column: column!(),
        })
    };
}

#[macro_export]
macro_rules! ExpectedValue {
    ($name:expr) => {
        $crate::error::DeltaError::ExpectedValue {
            type_name: $name.to_string(),
            file: file!().to_string(),
            line: line!(),
            column: column!(),
        }
    };
}



pub type DeltaResult<T> = Result<T, DeltaError>;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[derive(Deserialize, Serialize)]
pub enum DeltaError {
    BugDetected {
        msg: String,
        file: String,
        line: u32,
        column: u32
    },
    ExpectedValue {
        type_name: String,
        file: String,
        line: u32,
        column: u32
    },
    FailedToEnsure {
        predicate: String,
        msg: String,
        file: String,
        line: u32,
        column: u32,
    },
    FailedToApplyDelta { reason: String },
    FailedToConvertFromDelta { reason: String },
    IllegalDelta { index: usize },
    RwLockAccessWouldBlock,
    RwLockPoisoned(String)
}

impl<T> From<TryLockError<T>> for DeltaError {
    fn from(err: TryLockError<T>) -> DeltaError {
        match err {
            TryLockError::WouldBlock =>
                DeltaError::RwLockAccessWouldBlock,
            TryLockError::Poisoned(psn_err) =>
                DeltaError::RwLockPoisoned(format!("{}", psn_err)),
        }
    }
}


#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct DeltaErrorDelta(Option<DeltaError>);

impl Core for DeltaError {
    type Delta = DeltaErrorDelta;
}

impl Apply for DeltaError {
    fn apply(&self, delta: Self::Delta) -> DeltaResult<Self> {
        match delta {
            DeltaErrorDelta(Some(derr)) => Ok(derr),
            DeltaErrorDelta(None) => Ok(self.clone()),
        }
    }
}

impl Delta for DeltaError {
    fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
        Ok(DeltaErrorDelta(Some(rhs.clone())))
    }
}

impl FromDelta for DeltaError {
    fn from_delta(delta: Self::Delta) -> DeltaResult<Self> {
        match delta {
            DeltaErrorDelta(Some(derr)) => Ok(derr),
            DeltaErrorDelta(None) => Err(DeltaError::FailedToConvertFromDelta {
                reason: format!("Got no delta to convert from"),
            })
        }
    }
}

impl IntoDelta for DeltaError {
    fn into_delta(self) -> DeltaResult<Self::Delta> {
        Ok(DeltaErrorDelta(Some(self)))
    }
}
