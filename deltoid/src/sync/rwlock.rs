//! A newtype wrapping [`RwLock`] that provides extra functionality in
//! the form of delta support, de/serialization, partial equality and more.
//!
//! [`RwLock`]: https://doc.rust-lang.org/std/sync/struct.RwLock.html

use crate::{DeltaError, Deltoid, DeltaResult, FromDelta, IntoDelta};
use serde::{Serialize, Serializer, Deserialize, Deserializer};
use serde::de::{Visitor};
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;
pub use std::sync::{LockResult, RwLockReadGuard, RwLockWriteGuard};


#[derive(Debug, Default)]
pub struct RwLock<T>(std::sync::RwLock<T>);

impl<T> RwLock<T> {
    pub fn new(thing: T) -> Self { Self(std::sync::RwLock::new(thing)) }

    pub fn into_inner(self) -> LockResult<T> { self.0.into_inner() }

    pub fn try_read(&self) -> DeltaResult<RwLockReadGuard<T>> {
        self.0.try_read().map_err(DeltaError::from)
    }

    pub fn try_write(&self) -> DeltaResult<RwLockWriteGuard<T>> {
        self.0.try_write().map_err(DeltaError::from)
    }
}

impl<T: Clone> Clone for RwLock<T> {
    fn clone(&self) -> Self {
        let value: &T = &*self.try_read().unwrap();
        Self::new(value.clone())
    }
}

impl<T: Hash> Hash for RwLock<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.try_read().unwrap().hash(state)
    }
}

impl<T: PartialEq> PartialEq for RwLock<T> {
    fn eq(&self, rhs: &Self) -> bool {
        let lhs: &T = &*self.0.try_read().unwrap();
        let rhs: &T = &*rhs.0.try_read().unwrap();
        lhs.eq(rhs)
    }
}

impl<T: Eq> Eq for RwLock<T> { }

impl<T: PartialOrd> PartialOrd for RwLock<T> {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        let lhs: &T = &*self.0.try_read().unwrap();
        let rhs: &T = &*rhs.0.try_read().unwrap();
        lhs.partial_cmp(rhs)
    }
}

impl<T: Ord> Ord for RwLock<T> {
    fn cmp(&self, rhs: &Self) -> Ordering {
        let lhs: &T = &*self.0.try_read().unwrap();
        let rhs: &T = &*rhs.0.try_read().unwrap();
        lhs.cmp(rhs)
    }
}


impl<T: Serialize> Serialize for RwLock<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let value: &T = &self.0.try_read().unwrap(/*TODO*/);
        serializer.serialize_newtype_struct("RwLock", value)
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for RwLock<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de> {
        struct RwLockVisitor<V>(PhantomData<V>);

        impl<'de, V: Deserialize<'de>> Visitor<'de> for RwLockVisitor<V> {
            type Value = RwLock<V>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct RwLock<T>")
            }

            fn visit_newtype_struct<D: Deserializer<'de>>(
                self,
                deserializer: D
            ) -> Result<Self::Value, D::Error> {
                Deserialize::deserialize(deserializer).map(RwLock::new)
            }
        }

        deserializer.deserialize_newtype_struct(
            "RwLock",
            RwLockVisitor(PhantomData)
        )
    }
}



impl<T> Deltoid for RwLock<T>
where T: Deltoid
    + for<'de> Deserialize<'de>
    + Serialize
{
    type Delta = RwLockDelta<T>;

    fn apply_delta(&self, delta: &Self::Delta) -> DeltaResult<Self> {
        let lhs: &T = &*self.0.try_read().unwrap(/*TODO*/);
        match &delta.0 {
            Some(delta) => lhs.apply_delta(&delta).map(Self::new),
            None => Ok(Self::new(lhs.clone())),
        }
    }

    fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
        let lhs: &T = &*self.0.try_read().unwrap(/*TODO*/);
        let rhs: &T = &*rhs.0.try_read().unwrap(/*TODO*/);
        lhs.delta(rhs).map(Some).map(RwLockDelta)
    }
}

#[derive(Clone, Debug, PartialEq)]
#[derive(serde_derive::Deserialize, serde_derive::Serialize)]
pub struct RwLockDelta<T: Deltoid>(
    #[doc(hidden)]
    pub Option<<T as Deltoid>::Delta>
);

impl<T> IntoDelta for RwLock<T>
where T: Deltoid + Clone + std::fmt::Debug
    + for<'de> Deserialize<'de>
    + Serialize
    + IntoDelta
{
    fn into_delta(self) -> DeltaResult<<Self as Deltoid>::Delta> {
        let value: &T = &*self.0.try_read().unwrap(/*TODO*/);
        value.clone().into_delta().map(Some).map(RwLockDelta)
    }
}

impl<T> FromDelta for RwLock<T>
where T: Deltoid + Clone + std::fmt::Debug
    + for<'de> Deserialize<'de>
    + Serialize
    + FromDelta
{
    fn from_delta(delta: <Self as Deltoid>::Delta) -> DeltaResult<Self> {
        let inner_delta = delta.0.ok_or_else(|| ExpectedValue!("RwLockDelta<T>"))?;
        <T>::from_delta(inner_delta).map(Self::new)
    }
}
