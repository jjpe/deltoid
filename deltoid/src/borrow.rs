//!

use crate::{DeltaError, DeltaResult, Deltoid};
use crate::convert::{FromDelta, IntoDelta};
use serde::{Deserialize, Serialize};
use std::borrow::{Borrow, Cow, ToOwned};
use std::marker::PhantomData;


impl<'a, B> Deltoid for Cow<'a, B>
where B: Clone + std::fmt::Debug + PartialEq + Deltoid + ToOwned
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
    fn into_delta(self) -> DeltaResult<<Self as Deltoid>::Delta> {
        Ok(CowDelta {
            inner: Some((self.borrow() as &B).clone().into_delta()?),
            _phantom: PhantomData,
        })
    }
}

impl<'a, B> FromDelta for Cow<'a, B>
where B: FromDelta + Serialize + for<'de> Deserialize<'de> {
    fn from_delta(delta: <Self as Deltoid>::Delta) -> DeltaResult<Self> {
        let delta: <B as Deltoid>::Delta = delta.inner
            .ok_or(DeltaError::ExpectedValue)?;
        B::from_delta(delta)
            .map(|b: B| b.to_owned())
            .map(Cow::Owned)
    }
}


#[derive(Clone, Debug, PartialEq)]
#[derive(serde_derive::Deserialize, serde_derive::Serialize)]
pub struct CowDelta<'a, B: Deltoid + Clone> {
    #[doc(hidden)]pub inner: Option<<B as Deltoid>::Delta>,
    #[doc(hidden)]pub _phantom: PhantomData<&'a B>
}
