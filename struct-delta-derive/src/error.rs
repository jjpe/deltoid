//! Defines error infrastructure.

// macro_rules! ensure {
//     ($predicate:expr) => {
//         if $predicate {
//             DeriveResult::Ok(())
//         } else {
//             use $crate::error::{DeriveError, DeriveResult};
//             DeriveResult::Err(DeriveError::FailedToEnsure {
//                 predicate: stringify!($predicate),
//                 file: file!(),
//                 line: line!(),
//                 column: column!(),
//             })
//         }
//     };
// }


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


#[derive(Clone, Copy, Debug)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub enum DeriveError {
    BugDetected {
        file: &'static str,
        line: u32,
        column: u32
    },
    FailedToEnsure {
        predicate: &'static str,
        file: &'static str,
        line: u32,
        column: u32
    },

    // Add more error variants here
}
