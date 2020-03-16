//!

use crate::{DeltaError, Deltoid, DeltaResult};
use crate::convert::{FromDelta, IntoDelta};
use serde::{Deserialize, Serialize};
use std::ops::Range;


impl<T> Deltoid for Range<T>
where T: Clone + PartialEq + Deltoid + std::fmt::Debug
    + Serialize
    + for<'de> Deserialize<'de>
    + IntoDelta
    + FromDelta
{
    type Delta = RangeDelta<T>;

    fn apply_delta(&self, delta: &Self::Delta) -> DeltaResult<Self> {
        match &delta.0 {
            Some(range) => Ok(range.start.clone() .. range.end.clone()),
            None        => Ok(self.start.clone() ..  self.end.clone()),
        }
    }

    fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
        Ok(RangeDelta(if self == rhs {
            None
        } else {
            Some(rhs.clone())
        }))
    }
}

#[derive(Clone, Debug, PartialEq, Hash)]
#[derive(serde_derive::Deserialize, serde_derive::Serialize)]
pub struct RangeDelta<T>(#[doc(hidden)]pub Option<Range<T>>);

impl<T> IntoDelta for Range<T>
where T: Clone + PartialEq + Deltoid + std::fmt::Debug
    + for<'de> serde::Deserialize<'de>
    + serde::Serialize
    + IntoDelta
    + FromDelta
{
    fn into_delta(self) -> DeltaResult<<Self as Deltoid>::Delta> {
        Ok(RangeDelta(Some(self)))
    }
}

impl<T> FromDelta for Range<T>
where T: Clone + PartialEq + Deltoid + std::fmt::Debug
    + for<'de> serde::Deserialize<'de>
    + serde::Serialize
    + IntoDelta
    + FromDelta
{
    fn from_delta(delta: <Self as Deltoid>::Delta) -> DeltaResult<Self> {
        Ok(delta.0.ok_or(DeltaError::ExpectedValue)?)
    }
}
