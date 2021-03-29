//! Core definitions

use crate::error::DeltaResult;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

/// Defines an associated Delta type. This is used by the other core traits
/// to agree on a common Delta definition for each implementing type.
pub trait Core {
    type Delta: Sized + Clone + Debug + PartialEq
        + for<'de> Deserialize<'de>
        + Serialize;
}

pub trait Apply: Core + Clone + Debug + PartialEq {
    /// Calculate a new instance of `Self` based on `self` and `delta`
    /// i.e. calculate `self --[delta]--> other`.
    ///                                   ^^^^^
    fn apply(&self, delta: Self::Delta) -> DeltaResult<Self>;
}

pub trait Delta: Core + Clone + Debug + PartialEq {
    /// Calculate `self --[delta]--> other`.
    ///                    ^^^^^
    fn delta(&self, other: &Self) -> DeltaResult<Self::Delta>;
}

/// Conversion from type *Delta to type *
pub trait FromDelta: Core + Sized {
    /// Convert `Self::Delta` to `Self`.
    fn from_delta(delta: Self::Delta) -> DeltaResult<Self>;
}

/// Conversion from type * to type *Delta
pub trait IntoDelta: Core {
    /// Convert `Self` to `Self::Delta`.
    fn into_delta(self) -> DeltaResult<Self::Delta>;
}


macro_rules! impl_delta_trait_for_primitive_types {
    ( $($type:ty => $delta:ident $(: $($traits:ident),+)?);* $(;)? ) => {
        $(
            $( #[derive( $($traits),+ )] )?
            #[derive(serde_derive::Deserialize, serde_derive::Serialize)]
            pub struct $delta(#[doc(hidden)] pub Option<$type>);

            impl Core for $type {
                type Delta = $delta;
            }

            impl Apply for $type {
                #[inline(always)]
                fn apply(&self, delta: Self::Delta) -> DeltaResult<Self> {
                    Self::from_delta(delta)
                }
            }

            impl Delta for $type {
                #[inline(always)]
                fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
                    rhs.clone().into_delta()
                }
            }

            impl FromDelta for $type {
                #[inline(always)]
                fn from_delta(delta: Self::Delta) -> DeltaResult<Self> {
                    // TODO: perhaps turn the option to choose either returning
                    //       an error or a `Default` value into a policy?
                    // match delta.0 {
                    //     Some(value) => Ok(value),
                    //     None => Ok(Self::default()),
                    // }
                    delta.0.ok_or_else(|| ExpectedValue!(stringify!($delta)))
                }
            }

            impl IntoDelta for $type {
                #[inline(always)]
                fn into_delta(self) -> DeltaResult<Self::Delta> {
                    Ok($delta(Some(self)))
                }
            }

            impl std::fmt::Debug for $delta {
                fn fmt(&self, f: &mut std::fmt::Formatter)
                       -> Result<(), std::fmt::Error>
                {
                    match self.0 {
                        None =>
                            write!(f, "{}(None)", stringify!($delta)),
                        Some(prim) =>
                            write!(f, "{}({:#?})", stringify!($delta), prim),
                    }
                }
            }
        )*
    };
}

impl_delta_trait_for_primitive_types! {
    i8    => I8Delta:    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash;
    i16   => I16Delta:   Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash;
    i32   => I32Delta:   Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash;
    i64   => I64Delta:   Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash;
    i128  => I128Delta:  Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash;
    isize => IsizeDelta: Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash;

    u8    => U8Delta:    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash;
    u16   => U16Delta:   Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash;
    u32   => U32Delta:   Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash;
    u64   => U64Delta:   Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash;
    u128  => U128Delta:  Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash;
    usize => UsizeDelta: Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash;

    f32   => F32Delta:   Clone, Copy, PartialEq,     PartialOrd           ;
    f64   => F64Delta:   Clone, Copy, PartialEq,     PartialOrd           ;
    bool  => BoolDelta:  Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash;
    char  => CharDelta:  Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash;
    ()    => UnitDelta:  Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash;
}
