//!

use crate::{DeltaError, DeltaOps, DeltaResult};
use crate::convert::{FromDelta, IntoDelta};
use serde::{Deserialize, Serialize};
use std::borrow::{Borrow, Cow, ToOwned};
use std::marker::PhantomData;


impl<'a, B> DeltaOps for Cow<'a, B>
where B: ToOwned + PartialEq + DeltaOps + Clone + std::fmt::Debug
        + Serialize
        + for<'de> Deserialize<'de>,
      <B as ToOwned>::Owned: std::fmt::Debug
{
    type Delta = CowDelta<'a, B>;

    fn apply_delta(&self, delta: &Self::Delta) -> DeltaResult<Self> {
        let lhs: &B = self.borrow();
        if let Some(delta) = delta.inner.as_ref() {
            lhs.apply_delta(delta)
                .map(|new| new.to_owned())
                .map(Cow::Owned)
        } else {
            Ok(self.clone())
        }
    }

    fn delta(&self, other: &Self) -> DeltaResult<Self::Delta> {
        let (lhs, rhs): (&B, &B) = (self.borrow(), other.borrow());
        Ok(CowDelta {
            inner: Some(lhs.delta(rhs)?),
            _phantom: PhantomData,
        })
    }
}



impl<'a, B> IntoDelta for Cow<'a, B>
where B: IntoDelta + Serialize + for<'de> Deserialize<'de> {
    fn into_delta(self) -> DeltaResult<<Self as DeltaOps>::Delta> {
        Ok(CowDelta {
            inner: Some((self.borrow() as &B).clone().into_delta()?),
            _phantom: PhantomData,
        })
    }
}

impl<'a, B> FromDelta for Cow<'a, B>
where B: FromDelta + Serialize + for<'de> Deserialize<'de> {
    fn from_delta(delta: <Self as DeltaOps>::Delta) -> DeltaResult<Self> {
        let delta: <B as DeltaOps>::Delta = delta.inner
            .ok_or(DeltaError::ExpectedValue)?;
        B::from_delta(delta)
            .map(|b: B| b.to_owned())
            .map(Cow::Owned)
    }
}


#[derive(Clone, Debug, PartialEq)]
#[derive(serde_derive::Deserialize, serde_derive::Serialize)]
pub struct CowDelta<'a, B: DeltaOps + Clone> {
    inner: Option<<B as DeltaOps>::Delta>,
    _phantom: PhantomData<&'a B>
}
