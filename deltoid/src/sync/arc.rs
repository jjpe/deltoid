//! A Deltoid impl for [`Arc`] that provides extra functionality in
//! the form of delta support, de/serialization, partial equality and more.
//!
//! [`Arc`]: https://doc.rust-lang.org/std/sync/struct.Arc.html

use crate::{Deltoid, DeltaResult};
use crate::convert::{FromDelta, IntoDelta};
use std::sync::{Arc};


impl<T> Deltoid for Arc<T>
where T: Deltoid + PartialEq + Clone + std::fmt::Debug
    + for<'de> serde::Deserialize<'de>
    + serde::Serialize
{
    type Delta = ArcDelta<T>;

    fn apply_delta(&self, delta: &Self::Delta) -> DeltaResult<Self> {
        let lhs: &T = self.as_ref();
        match &delta.0 {
            None => Ok(self.clone()),
            Some(delta) => lhs.apply_delta(delta).map(Arc::new),
        }
    }

    fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
        let lhs: &T = self.as_ref();
        let rhs: &T = rhs.as_ref();
        Ok(ArcDelta(if lhs == rhs {
            None
        } else {
            Some(Box::new(lhs.delta(rhs)?))
        }))
    }
}


#[derive(Clone, Debug, PartialEq)]
#[derive(serde_derive::Deserialize, serde_derive::Serialize)]
pub struct ArcDelta<T: Deltoid>(
    #[doc(hidden)] pub Option<Box<<T as Deltoid>::Delta>>
);

impl<T> IntoDelta for Arc<T>
where T: Deltoid + IntoDelta
    + for<'de> serde::Deserialize<'de>
    + serde::Serialize
{
    fn into_delta(self) -> DeltaResult<<Self as Deltoid>::Delta> {
        let thing: T = self.as_ref().clone();
        thing.into_delta().map(Box::new).map(Some).map(ArcDelta)
    }
}

impl<T> FromDelta for Arc<T>
where T: Deltoid + FromDelta + Default
    + for<'de> serde::Deserialize<'de>
    + serde::Serialize
{
    fn from_delta(delta: <Self as Deltoid>::Delta) -> DeltaResult<Self> {
        match delta.0 {
            None => Ok(Self::default()),
            Some(delta) => <T>::from_delta(*delta).map(Arc::new),
        }
    }
}
