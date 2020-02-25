//! This module defines 2 traits which are used to convert between a type
//! `T: DeltaOps` and its corresponding Delta type, `<T as DeltaOps>::Delta`.

use crate::{DeltaOps, DeltaResult};


/// Convert `self` into its corresponding delta type.
pub trait IntoDelta: Sized + DeltaOps {
    /// Performs the conversion from `Self` to `<Self as DeltaOps>::Delta`.
    fn into_delta(self) -> DeltaResult<<Self as DeltaOps>::Delta>;
}

pub trait FromDelta: Sized + DeltaOps {
    /// Performs the conversion from `<Self as DeltaOps>::Delta` to `Self`.
    fn from_delta(delta: <Self as DeltaOps>::Delta) -> DeltaResult<Self>;
}
