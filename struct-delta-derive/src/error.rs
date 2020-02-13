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


pub type DeriveResult<T> = Result<T, DeriveError>;


#[derive(Clone, Copy, Debug)]
#[derive(serde_derive::Serialize, serde_derive::Deserialize)]
pub enum DeriveError {
    FailedToEnsure {
        predicate: &'static str,
        file: &'static str,
        line: u32,
        column: u32
    },

    // Add more error variants here
}
