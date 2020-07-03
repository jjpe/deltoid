//!

use crate::{Apply, Core, Delta, DeltaResult, FromDelta, IntoDelta};
use std::fmt::Debug;
use serde::{Deserialize, Serialize};

impl<T> Core for Option<T>
where T: Clone + Debug + PartialEq + Core
    + for<'de> Deserialize<'de>
    + Serialize
{
    type Delta = OptionDelta<T>;
}

impl<T> Apply for Option<T>
where T: Apply + FromDelta
    + for<'de> Deserialize<'de>
    + Serialize
{
    fn apply(&self, delta: Self::Delta) -> DeltaResult<Self> {
        Ok(match (&self, delta) {
            (None,    Self::Delta::None)    => None,
            (Some(_), Self::Delta::None)    => self.clone(),
            (None,    Self::Delta::Some(ref d)) => Some(
                <T>::from_delta(d.clone(/*TODO: rm clone for more efficiency*/))?
            ),
            (Some(t), Self::Delta::Some(ref d)) =>
                Some(t.apply(d.clone(/*TODO: rm clone for more efficiency*/))?),
        })
    }
}

impl<T> Delta for Option<T>
where T: Delta + IntoDelta
    + for<'de> Deserialize<'de>
    + Serialize
{
    fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
        Ok(match (self, rhs) {
            (Some(lhs), Some(rhs)) => Self::Delta::Some(lhs.delta(&rhs)?),
            (None,      Some(rhs)) => Self::Delta::Some(rhs.clone().into_delta()?),
            (Some(_),   None)      => Self::Delta::None,
            (None,      None)      => Self::Delta::None,
        })
    }
}

impl<T> FromDelta for Option<T>
where T: Clone + Debug + PartialEq + FromDelta
    + for<'de> Deserialize<'de>
    + Serialize
{
    fn from_delta(delta: <Self as Core>::Delta) -> DeltaResult<Self> {
        Ok(match delta {
            Self::Delta::None => None,
            Self::Delta::Some(delta) => Some(<T>::from_delta(delta)?),
        })
    }
}

impl<T> IntoDelta for Option<T>
where T: Clone + Debug + PartialEq + IntoDelta
    + for<'de> Deserialize<'de>
    + Serialize
{
    fn into_delta(self) -> DeltaResult<<Self as Core>::Delta> {
        Ok(match self {
            Self::None => OptionDelta::None,
            Self::Some(t) => OptionDelta::Some(t.into_delta()?),
        })
    }
}



#[derive(Clone, Debug, PartialEq)]
#[derive(serde_derive::Deserialize, serde_derive::Serialize)]
pub enum OptionDelta<T: Core> {
    None,
    Some(<T as Core>::Delta),
}
