//!

use crate::*;


impl<T> Deltoid for Option<T>
where T: Deltoid + PartialEq + Clone + std::fmt::Debug
    + for<'de> serde::Deserialize<'de>
    + serde::Serialize
    + IntoDelta
    + FromDelta
{
    type Delta = OptionDelta<T>;

    fn apply_delta(&self, delta: &Self::Delta) -> DeltaResult<Self> {
        if let Self::None = self {
            if let Self::Delta::None = delta {
                return Ok(Self::None);
            }
        }
        if let Self::Some(t) = self {
            if let Self::Delta::Some(delta_t) = delta {
                return Ok(Self::Some(
                    match delta_t.as_ref() {
                        None => t.clone(),
                        Some(d) => t.apply_delta(d)?,
                    },
                ));
            }
        }
        if let Self::Delta::None = delta {
            return Ok(Self::None);
        }
        if let Self::Delta::Some(delta_t) = delta {
            return Ok(Self::Some(
                match delta_t.as_ref() {
                    Some(d) => <T>::from_delta(d.clone())?,
                    None => return Err(DeltaError::ExpectedValue)?,
                },
            ));
        }
        crate::bug_detected!()
    }

    fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
        if let Self::None = self {
            if let Self::None = rhs {
                return Ok(Self::Delta::None);
            }
        }
        if let Self::Some(lhs_0) = self {
            if let Self::Some(rhs_0) = rhs {
                let delta_0: Option<<T as Deltoid>::Delta> = if lhs_0 != rhs_0 {
                    Some(lhs_0.delta(&rhs_0)?)
                } else {
                    None
                };
                return Ok(Self::Delta::Some(delta_0));
            }
        }
        if let Self::None = rhs {
            return Ok(Self::Delta::None);
        }
        if let Self::Some(t) = rhs {
            return Ok(Self::Delta::Some(
                Some(t.clone().into_delta()?),
            ));
        }
        crate::bug_detected!()
    }
}


#[derive(Clone, Debug, PartialEq)]
#[derive(serde_derive::Deserialize, serde_derive::Serialize)]
pub enum OptionDelta<T: Deltoid> {
    None,
    Some(Option<<T as Deltoid>::Delta>),
}


impl<T> IntoDelta for Option<T>
where T: Deltoid + IntoDelta
    + for<'de> serde::Deserialize<'de>
    + serde::Serialize
    + IntoDelta
    + FromDelta
{
    fn into_delta(self) -> DeltaResult<<Self as Deltoid>::Delta> {
        Ok(match self {
            Self::None => OptionDelta::None,
            Self::Some(field0) => OptionDelta::Some(
                Some(field0.into_delta()?)
            ),
        })
    }
}

impl<T> FromDelta for Option<T>
where T: Deltoid + FromDelta
    + for<'de> serde::Deserialize<'de>
    + serde::Serialize
    + IntoDelta
    + FromDelta
{
    fn from_delta(delta: <Self as Deltoid>::Delta) -> DeltaResult<Self> {
        Ok(match delta {
            Self::Delta::None => Self::None,
            Self::Delta::Some(field0) => Self::Some(
                <T>::from_delta(field0.ok_or(DeltaError::ExpectedValue)?)?,
            ),
        })
    }
}
