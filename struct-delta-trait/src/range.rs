//!

use crate::{DeltaOps, DeltaResult};
use crate::convert::{FromDelta, IntoDelta};
use serde::{Deserialize, Serialize};
use std::ops::Range;


impl<T> DeltaOps for Range<T>
where T: Clone + PartialEq + DeltaOps + std::fmt::Debug
    + Serialize
    + for<'de> Deserialize<'de>
    + IntoDelta
    + FromDelta
{
    type Delta = RangeDelta<T>;

    fn apply_delta(&self, delta: &Self::Delta) -> DeltaResult<Self> {
        let (_lhs_start, rhs_start) = (self.start.clone(), delta.0.start.clone());
        let (_lhs_end,   rhs_end) =   (self.end.clone(),   delta.0.end.clone());
        Ok(rhs_start .. rhs_end)
    }

    fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
        let (_lhs_start, rhs_start) = (self.start.clone(), rhs.start.clone());
        let (_lhs_end,   rhs_end) =   (self.end.clone(),   rhs.end.clone());
        Ok(RangeDelta(rhs_start .. rhs_end))
    }
}

#[derive(Clone, Debug, PartialEq, Hash)]
#[derive(serde_derive::Deserialize, serde_derive::Serialize)]
pub struct RangeDelta<T>(Range<T>);

impl<T> IntoDelta for Range<T>
where T: Clone + PartialEq + DeltaOps + std::fmt::Debug
    + for<'de> serde::Deserialize<'de>
    + serde::Serialize
    + IntoDelta
    + FromDelta
{
    fn into_delta(self) -> DeltaResult<<Self as DeltaOps>::Delta> {
        Ok(RangeDelta(self))
    }
}

impl<T> FromDelta for Range<T>
where T: Clone + PartialEq + DeltaOps + std::fmt::Debug
    + for<'de> serde::Deserialize<'de>
    + serde::Serialize
    + IntoDelta
    + FromDelta
{
    fn from_delta(delta: <Self as DeltaOps>::Delta) -> DeltaResult<Self> {
        Ok(delta.0)
    }
}
