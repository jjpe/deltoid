//! A DeltaOps impl for [`Arc`] that provides extra functionality in
//! the form of delta support, de/serialization, partial equality and more.
//!
//! [`Arc`]: https://doc.rust-lang.org/std/sync/struct.Arc.html

use crate::{DeltaOps, DeltaResult};
use crate::convert::{FromDelta, IntoDelta};
use std::sync::{Arc};


impl<T> DeltaOps for Arc<T>
where T: DeltaOps + PartialEq + Clone + std::fmt::Debug
    + for<'de> serde::Deserialize<'de>
    + serde::Serialize
{
    type Delta = ArcDelta<T>;

    fn apply_delta(&self, delta: &Self::Delta) -> DeltaResult<Self> {
        let lhs: &T = self.as_ref();
        let rhs: &<T as DeltaOps>::Delta = &delta.0;
        lhs.apply_delta(rhs).map(Arc::new)
    }

    fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
        let lhs: &T = self.as_ref();
        let rhs: &T = rhs.as_ref();
        lhs.delta(rhs).map(Arc::new).map(ArcDelta)
    }
}


#[derive(Clone, Debug, PartialEq)]
#[derive(serde_derive::Deserialize, serde_derive::Serialize)]
pub struct ArcDelta<T: DeltaOps>(Arc<<T as DeltaOps>::Delta>);

impl<T> IntoDelta for Arc<T>
where T: DeltaOps + IntoDelta
    + for<'de> serde::Deserialize<'de>
    + serde::Serialize
{
    fn into_delta(self) -> DeltaResult<<Self as DeltaOps>::Delta> {
        self.as_ref().clone().into_delta().map(Arc::new).map(ArcDelta)
    }
}

impl<T> FromDelta for Arc<T>
where T: DeltaOps + FromDelta
    + for<'de> serde::Deserialize<'de>
    + serde::Serialize
{
    fn from_delta(delta: <Self as DeltaOps>::Delta) -> DeltaResult<Self> {
        let unboxed = Arc::try_unwrap(delta.0).unwrap(/*TODO*/);
        <T>::from_delta(unboxed).map(Arc::new)
    }
}
