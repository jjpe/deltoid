//!

// TODO:
// Can a delta be applied to a value of:
//   + an array type i.e. [T, N]?
//   + a slice type  i.e. &[T]  and  &str?
//   + a shared-ownership type e.g. Rc and Arc?


pub mod borrow;
pub mod convert;
#[macro_use] pub mod error;
pub mod heap;
pub mod option;
pub mod range;
pub mod string;
pub mod tuple;
pub mod vec;


pub use crate::borrow::CowDelta;
pub use crate::convert::{FromDelta, IntoDelta};
pub use crate::error::{DeltaError, DeltaResult};
pub use crate::heap::{ArcDelta, BoxDelta, RcDelta};
pub use crate::option::{OptionDelta};
pub use crate::range::RangeDelta;
pub use crate::string::StringDelta;
pub use crate::tuple::*;
pub use crate::vec::{EltDelta, VecDelta};
use serde::{Deserialize, Serialize};


/// Definitions for delta operations.
pub trait DeltaOps: Sized + PartialEq + Clone + std::fmt::Debug {
    type Delta: PartialEq + Clone + std::fmt::Debug
        + Serialize
        + for<'de> Deserialize<'de>;

    /// Calculate a new instance of `Self` based on `self` and
    /// `delta` i.e. calculate `self --[delta]--> other`.
    ///                                           ^^^^^
    fn apply_delta(&self, delta: &Self::Delta) -> DeltaResult<Self>;

    /// Calculate `self --[delta]--> other`.
    ///                    ^^^^^
    fn delta(&self, other: &Self) -> DeltaResult<Self::Delta>;

    /// Calculate `other --[delta]--> self`.
    ///                     ^^^^^
    fn inverse_delta(&self, other: &Self) -> DeltaResult<Self::Delta> {
        other.delta(self)
    }
}


macro_rules! impl_delta_trait_for_primitive_types {
    ( $($type:ty => $delta:ident),* $(,)? ) => {
        $(
            impl DeltaOps for $type {
                type Delta = $delta;

                fn apply_delta(&self, delta: &Self::Delta) -> DeltaResult<Self> {
                    Self::from_delta(delta.clone())
                }

                fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
                    rhs.clone().into_delta()
                }
            }

            #[derive(Clone, Debug, PartialEq)]
            #[derive(serde_derive::Deserialize, serde_derive::Serialize, )]
            pub struct $delta(Option<$type>);

            impl IntoDelta for $type {
                fn into_delta(self) -> DeltaResult<<Self as DeltaOps>::Delta> {
                    Ok($delta(Some(self)))
                }
            }

            impl FromDelta for $type {
                fn from_delta(delta: <Self as DeltaOps>::Delta) -> DeltaResult<Self> {
                    delta.0.ok_or(DeltaError::ExpectedValue)
                }
            }
        )*
    };
}

impl_delta_trait_for_primitive_types! {
    i8    => I8Delta,
    i16   => I16Delta,
    i32   => I32Delta,
    i64   => I64Delta,
    i128  => I128Delta,
    isize => IsizeDelta,

    u8    => U8Delta,
    u16   => U16Delta,
    u32   => U32Delta,
    u64   => U64Delta,
    u128  => U128Delta,
    usize => UsizeDelta,

    f32  => F32Delta,
    f64  => F64Delta,
    bool => BoolDelta,
    char => CharDelta,
    ()   => UnitDelta,
}



// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn foo() -> DeltaResult<()> {
//         Ok(())
//     }

// }
