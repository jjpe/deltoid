//!

use crate::{DeltaError, DeltaOps, DeltaResult};
use crate::convert::{FromDelta, IntoDelta};

impl DeltaOps for String { // TODO: Improve space efficiency
    type Delta = StringDelta;

    fn apply_delta(&self, delta: &Self::Delta) -> DeltaResult<Self> {
        Self::from_delta(delta.clone())
    }

    fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
        rhs.clone().into_delta()
    }
}

#[derive(Clone, Debug, PartialEq, Hash)]
#[derive(serde_derive::Deserialize, serde_derive::Serialize)]
pub struct StringDelta(Option<String>);

impl IntoDelta for String {
    fn into_delta(self) -> DeltaResult<<Self as DeltaOps>::Delta> {
        Ok(StringDelta(Some(self)))
    }
}

impl FromDelta for String {
    fn from_delta(delta: <Self as DeltaOps>::Delta) -> DeltaResult<Self> {
        delta.0.ok_or(DeltaError::ExpectedValue)
    }
}
