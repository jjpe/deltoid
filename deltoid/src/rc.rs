//! A Deltoid impl for [`Rc`] that provides extra functionality in
//! the form of delta support, de/serialization, partial equality and more.
//!
//! [`Rc`]: https://doc.rust-lang.org/std/rc/struct.Rc.html

use crate::{Deltoid, DeltaResult};
use crate::convert::{FromDelta, IntoDelta};
use std::rc::{Rc};


impl<T> Deltoid for Rc<T>
where T: Deltoid + PartialEq + Clone + std::fmt::Debug
    + for<'de> serde::Deserialize<'de>
    + serde::Serialize
{
    type Delta = RcDelta<T>;

    fn apply_delta(&self, delta: &Self::Delta) -> DeltaResult<Self> {
        let lhs: &T = self.as_ref();
        let rhs: &<T as Deltoid>::Delta = &delta.0;
        lhs.apply_delta(rhs).map(Rc::new)
    }

    fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
        let lhs: &T = self.as_ref();
        let rhs: &T = rhs.as_ref();
        lhs.delta(rhs).map(Rc::new).map(RcDelta)
    }
}


#[derive(Clone, Debug, PartialEq)]
#[derive(serde_derive::Deserialize, serde_derive::Serialize)]
pub struct RcDelta<T: Deltoid>(Rc<<T as Deltoid>::Delta>);


impl<T> IntoDelta for Rc<T>
where T: Deltoid + IntoDelta
    + for<'de> serde::Deserialize<'de>
    + serde::Serialize
{
    fn into_delta(self) -> DeltaResult<<Self as Deltoid>::Delta> {
        self.as_ref().clone().into_delta().map(Rc::new).map(RcDelta)
    }
}

impl<T> FromDelta for Rc<T>
where T: Deltoid + FromDelta
    + for<'de> serde::Deserialize<'de>
    + serde::Serialize
{
    fn from_delta(delta: <Self as Deltoid>::Delta) -> DeltaResult<Self> {
        let unboxed = Rc::try_unwrap(delta.0).unwrap(/*TODO*/);
        <T>::from_delta(unboxed).map(Rc::new)
    }
}