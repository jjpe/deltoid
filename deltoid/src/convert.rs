//! This module defines 2 traits which are used to convert between a type
//! `T: Deltoid` and its corresponding Delta type, `<T as Deltoid>::Delta`.

use crate::{Deltoid, DeltaResult};


/// Convert `self` into its corresponding delta type.
pub trait IntoDelta: Sized + Deltoid {
    /// Performs the conversion from `Self` to `<Self as Deltoid>::Delta`.
    fn into_delta(self) -> DeltaResult<<Self as Deltoid>::Delta>;
}

pub trait FromDelta: Sized + Deltoid {
    /// Performs the conversion from `<Self as Deltoid>::Delta` to `Self`.
    fn from_delta(delta: <Self as Deltoid>::Delta) -> DeltaResult<Self>;
}
