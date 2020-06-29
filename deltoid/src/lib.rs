//!

// TODO:
// Can a delta be applied to a value of:
//   + an array type i.e. [T, N]?             (Probably yes)
//   + a slice type  e.g. &[T]  and  &str?    (Very unlikely for borrowed types)

#[macro_use] pub mod error;
#[macro_use] pub mod snapshot;
pub mod borrow;
pub mod boxed;
pub mod collections;
pub mod convert;
pub mod option;
pub mod range;
pub mod result;
pub mod rc;
pub mod string;
pub mod sync;
pub mod tuple;
pub mod vec;


pub use crate::borrow::CowDelta;
pub use crate::boxed::*;
pub use crate::collections::*;
pub use crate::convert::{FromDelta, IntoDelta};
pub use crate::error::{DeltaError, DeltaResult};
pub use crate::option::{OptionDelta};
pub use crate::range::RangeDelta;
pub use crate::rc::*;
pub use crate::string::StringDelta;
pub use crate::sync::*;
pub use crate::tuple::*;
pub use crate::vec::{EltDelta, VecDelta};
use serde::{Deserialize, Serialize};




/// Definitions for delta operations.
pub trait Deltoid: Sized + PartialEq + Clone + std::fmt::Debug {
    type Delta: Clone + std::fmt::Debug + PartialEq
        + Serialize
        + for<'de> Deserialize<'de>;

    /// Calculate a new instance of `Self` based on `self` and
    /// `delta` i.e. calculate `self --[delta]--> other`.
    ///                                           ^^^^^
    fn apply_delta(&self, delta: &Self::Delta) -> DeltaResult<Self>;

    /// Calculate `self --[delta]--> other`.
    ///                    ^^^^^
    fn delta(&self, other: &Self) -> DeltaResult<Self::Delta>;
}


macro_rules! impl_delta_trait_for_primitive_types {
    ( $($type:ty => $delta:ident $(: $($traits:ident),+)?);* $(;)? ) => {
        $(
            impl Deltoid for $type {
                type Delta = $delta;

                fn apply_delta(&self, delta: &Self::Delta) -> DeltaResult<Self> {
                    Self::from_delta(delta.clone())
                }

                fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
                    rhs.clone().into_delta()
                }
            }

            $( #[derive( $($traits),+ )] )?
            #[derive(serde_derive::Deserialize, serde_derive::Serialize)]
            pub struct $delta(#[doc(hidden)] pub Option<$type>);

            impl IntoDelta for $type {
                fn into_delta(self) -> DeltaResult<<Self as Deltoid>::Delta> {
                    Ok($delta(Some(self)))
                }
            }

            impl FromDelta for $type {
                fn from_delta(delta: <Self as Deltoid>::Delta) -> DeltaResult<Self> {
                    delta.0.ok_or_else(|| ExpectedValue!(stringify!($delta)))
                }
            }
        )*
    };
}

impl_delta_trait_for_primitive_types! {
    i8    => I8Delta:    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash;
    i16   => I16Delta:   Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash;
    i32   => I32Delta:   Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash;
    i64   => I64Delta:   Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash;
    i128  => I128Delt:   Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash;
    isize => IsizeDelta: Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash;

    u8    => U8Delta:    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash;
    u16   => U16Delta:   Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash;
    u32   => U32Delta:   Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash;
    u64   => U64Delta:   Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash;
    u128  => U128Delta:  Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash;
    usize => UsizeDelta: Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash;

    f32   => F32Delta:   Clone, Copy, Debug, PartialEq,     PartialOrd           ;
    f64   => F64Delta:   Clone, Copy, Debug, PartialEq,     PartialOrd           ;
    bool  => BoolDelta:  Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash;
    char  => CharDelta:  Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash;
    ()    => UnitDelta:  Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash;
}
