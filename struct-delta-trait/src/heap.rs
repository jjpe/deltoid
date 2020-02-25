//!

use crate::{DeltaOps, DeltaResult};
use crate::convert::{FromDelta, IntoDelta};
// use serde::{Deserialize, Serialize};
// use std::convert::{TryFrom, TryInto};
use std::rc::{Rc};
use std::sync::{
    Arc,
    // RwLock, RwLockReadGuard, RwLockWriteGuard
};


impl<T> DeltaOps for Rc<T>
where T: DeltaOps + PartialEq + Clone + std::fmt::Debug
    + for<'de> serde::Deserialize<'de>
    + serde::Serialize
{
    type Delta = RcDelta<T>;

    fn apply_delta(&self, delta: &Self::Delta) -> DeltaResult<Self> {
        let lhs: &T = self.as_ref();
        let rhs: &<T as DeltaOps>::Delta = &delta.0;
        lhs.apply_delta(rhs).map(Rc::new)
    }

    fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
        let lhs: &T = self.as_ref();
        let rhs: &T = rhs.as_ref();
        lhs.delta(rhs).map(RcDelta)
    }
}


#[derive(Clone, Debug, PartialEq)]
#[derive(serde_derive::Deserialize, serde_derive::Serialize)]
pub struct RcDelta<T: DeltaOps>(<T as DeltaOps>::Delta);


impl<T> IntoDelta for Rc<T>
where T: DeltaOps + IntoDelta
    + for<'de> serde::Deserialize<'de>
    + serde::Serialize
// where Rc<T>: Clone + PartialEq + DeltaOps + std::fmt::Debug
{
    fn into_delta(self) -> DeltaResult<<Self as DeltaOps>::Delta> {
        self.as_ref().clone().into_delta().map(RcDelta)
    }
}

impl<T> FromDelta for Rc<T>
where T: DeltaOps + FromDelta
    + for<'de> serde::Deserialize<'de>
    + serde::Serialize
// where Rc<T>: Clone + PartialEq + DeltaOps + std::fmt::Debug
{
    fn from_delta(delta: <Self as DeltaOps>::Delta) -> DeltaResult<Self> {
        <T>::from_delta(delta.0).map(Rc::new)
    }
}








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
        lhs.delta(rhs).map(ArcDelta)
    }
}


#[derive(Clone, Debug, PartialEq)]
#[derive(serde_derive::Deserialize, serde_derive::Serialize)]
pub struct ArcDelta<T: DeltaOps>(<T as DeltaOps>::Delta);

impl<T> IntoDelta for Arc<T>
where T: DeltaOps + IntoDelta
    + for<'de> serde::Deserialize<'de>
    + serde::Serialize
// where Arc<T>: Clone + PartialEq + DeltaOps + std::fmt::Debug
{
    fn into_delta(self) -> DeltaResult<<Self as DeltaOps>::Delta> {
        self.as_ref().clone().into_delta().map(ArcDelta)
    }
}

impl<T> FromDelta for Arc<T>
where T: DeltaOps + FromDelta
    + for<'de> serde::Deserialize<'de>
    + serde::Serialize
// where Arc<T>: Clone + PartialEq + DeltaOps + std::fmt::Debug
{
    fn from_delta(delta: <Self as DeltaOps>::Delta) -> DeltaResult<Self> {
        <T>::from_delta(delta.0).map(Arc::new)
    }
}






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
        lhs.delta(rhs).map(BoxDelta)
    }
}


#[derive(Clone, Debug, PartialEq)]
#[derive(serde_derive::Deserialize, serde_derive::Serialize)]
pub struct BoxDelta<T: DeltaOps>(<T as DeltaOps>::Delta);


impl<T> IntoDelta for Box<T>
where T: DeltaOps + IntoDelta
    + for<'de> serde::Deserialize<'de>
    + serde::Serialize
// where Box<T>: Clone + PartialEq + DeltaOps + std::fmt::Debug
{
    fn into_delta(self) -> DeltaResult<<Self as DeltaOps>::Delta> {
        self.as_ref().clone().into_delta().map(BoxDelta)
    }
}

impl<T> FromDelta for Box<T>
where T: DeltaOps + FromDelta
    + for<'de> serde::Deserialize<'de>
    + serde::Serialize
// where Box<T>: Clone + PartialEq + DeltaOps + std::fmt::Debug
{
    fn from_delta(delta: <Self as DeltaOps>::Delta) -> DeltaResult<Self> {
        <T>::from_delta(delta.0).map(Box::new)
    }
}
