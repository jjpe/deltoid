//! A DeltaOps impl for [`Box`] that provides extra functionality in
//! the form of delta support, de/serialization, partial equality and more.
//!
//! [`Box`]: https://doc.rust-lang.org/std/boxed/struct.Box.html

use crate::{DeltaOps, DeltaResult};
use crate::convert::{FromDelta, IntoDelta};


impl<T> DeltaOps for Box<T>
where T: DeltaOps + PartialEq + Clone + std::fmt::Debug
    + for<'de> serde::Deserialize<'de>
    + serde::Serialize
{
    type Delta = BoxDelta<T>;

    fn apply_delta(&self, delta: &Self::Delta) -> DeltaResult<Self> {
        let lhs: &T = self.as_ref();
        let rhs: &<T as DeltaOps>::Delta = &delta.0;
        lhs.apply_delta(rhs).map(Box::new)
    }

    fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
        let lhs: &T = self.as_ref();
        let rhs: &T = rhs.as_ref();
        lhs.delta(rhs).map(Box::new).map(BoxDelta)
    }
}


#[derive(Clone, Debug, PartialEq)]
#[derive(serde_derive::Deserialize, serde_derive::Serialize)]
pub struct BoxDelta<T: DeltaOps>(Box<<T as DeltaOps>::Delta>);


impl<T> IntoDelta for Box<T>
where T: DeltaOps + IntoDelta
    + for<'de> serde::Deserialize<'de>
    + serde::Serialize
{
    fn into_delta(self) -> DeltaResult<<Self as DeltaOps>::Delta> {
        self.as_ref().clone().into_delta().map(Box::new).map(BoxDelta)
    }
}

impl<T> FromDelta for Box<T>
where T: DeltaOps + FromDelta
    + for<'de> serde::Deserialize<'de>
    + serde::Serialize
{
    fn from_delta(delta: <Self as DeltaOps>::Delta) -> DeltaResult<Self> {
        <T>::from_delta(*delta.0).map(Box::new)
    }
}
