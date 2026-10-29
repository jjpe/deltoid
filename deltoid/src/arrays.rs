//!

use crate::{Apply, Core, Delta, DeltaError, DeltaResult, FromDelta, IntoDelta};
use serde::{Deserialize, Serialize};
use std::convert::TryInto;
use std::fmt::Debug;

impl<T, const LEN: usize> Core for [T; LEN]
where T: Clone + Debug + PartialEq + Core
    + for<'de> Deserialize<'de>
    + Serialize
{
    type Delta = ArrayDelta<T, LEN>;
}

impl<T, const LEN: usize> Apply for [T; LEN]
where T: Apply
    + for<'de> Deserialize<'de>
    + Serialize
{
    fn apply(&self, delta: Self::Delta) -> DeltaResult<Self> {
        let mut new: Vec<T> = self.to_vec();
        for (position, Edit { delta: d, index }) in delta.0.into_iter().enumerate() {
            if index >= LEN { return Err(DeltaError::IllegalDelta { index: position }); }
            let updated = new[index].apply(d)?;
            new[index] = updated;
        }
        match new.try_into() { Ok(array) => Ok(array), Err(_) => bug_detected!() }
    }
}

impl<T, const LEN: usize> Delta for [T; LEN]
where T: Delta + IntoDelta
    + for<'de> Deserialize<'de>
    + Serialize
{
    fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
        let mut delta = ArrayDelta(Vec::with_capacity(LEN));
        for i in 0 .. LEN {
            if self[i] == rhs[i] { continue }
            delta.0.push(Edit {
                delta: self[i].delta(&rhs[i])?,
                index: i,
            });
        }
        Ok(delta)
    }
}

impl<T, const LEN: usize> FromDelta for [T; LEN]
where T: Clone + Debug + PartialEq + FromDelta
    + for<'de> Deserialize<'de>
    + Serialize
{
    fn from_delta(delta: <Self as Core>::Delta) -> DeltaResult<Self> {
        let mut slots: Vec<Option<T>> = vec![None; LEN];
        for (position, Edit { delta: d, index }) in delta.0.into_iter().enumerate() {
            if index >= LEN || slots[index].is_some() {
                return Err(DeltaError::IllegalDelta { index: position });
            }
            slots[index] = Some(<T>::from_delta(d)?);
        }
        let mut new: Vec<T> = Vec::with_capacity(LEN);
        for slot in slots {
            match slot {
                Some(value) => new.push(value),
                None => return Err(ExpectedValue!("ArrayDelta<T, LEN>")),
            }
        }
        match new.try_into() { Ok(array) => Ok(array), Err(_) => bug_detected!() }
    }
}

impl<T, const LEN: usize> IntoDelta for [T; LEN]
where T: Clone + Debug + PartialEq + IntoDelta
    + for<'de> Deserialize<'de>
    + Serialize
{
    fn into_delta(self) -> DeltaResult<<Self as Core>::Delta> {
        let mut delta = ArrayDelta(Vec::with_capacity(LEN));
        for index in 0 .. LEN {
            delta.0.push(Edit {
                delta: self[index].clone().into_delta()?,
                index,
            });
        }
        Ok(delta)
    }
}

#[derive(Clone, Debug, PartialEq)]
#[derive(serde_derive::Deserialize, serde_derive::Serialize)]
pub struct ArrayDelta<T: Core, const LEN: usize> (
    #[doc(hidden)] pub Vec<Edit<T>>
);

#[derive(Clone, Debug, PartialEq)]
#[derive(serde_derive::Deserialize, serde_derive::Serialize)]
pub struct Edit<T: Core> {
    #[doc(hidden)] pub delta: <T as Core>::Delta,
    #[doc(hidden)] pub index: usize,
}

#[allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use serde_json;
    use super::*;

    #[test]
    fn array_of_len_0__delta() -> DeltaResult<()> {
        let array0: [u16; 0] = [];
        let array1: [u16; 0] = [];
        let delta: <[u16; 0] as Core>::Delta = array0.delta(&array1)?;
        let json: String = serde_json::to_string(&delta)
            .expect("Could not serialize to json");
        assert_eq!(json, "[]");
        let delta1: <[u16; 0] as Core>::Delta = serde_json::from_str(&json)
            .expect("Could not deserialize from json");
        assert_eq!(delta, delta1);
        Ok(())
    }

    #[test]
    fn array_of_len_0__apply() -> DeltaResult<()> {
        let array0: [u16; 0] = [];
        let array1: [u16; 0] = [];
        let delta: <[u16; 0] as Core>::Delta = array0.delta(&array1)?;
        let array2 = array0.apply(delta)?;
        assert_eq!(array1, array2);
        Ok(())
    }

    #[test]
    fn array_of_len_1__delta__same_values() -> DeltaResult<()> {
        let array0: [u16; 1] = [42];
        let array1: [u16; 1] = [42];
        let delta: <[u16; 1] as Core>::Delta = array0.delta(&array1)?;
        let json: String = serde_json::to_string(&delta)
            .expect("Could not serialize to json");
        assert_eq!(json, "[]");
        let delta1: <[u16; 1] as Core>::Delta = serde_json::from_str(&json)
            .expect("Could not deserialize from json");
        assert_eq!(delta, delta1);
        Ok(())
    }

    #[test]
    fn array_of_len_1__delta__different_values() -> DeltaResult<()> {
        let array0: [u16; 1] = [10];
        let array1: [u16; 1] = [42];
        let delta: <[u16; 1] as Core>::Delta = array0.delta(&array1)?;
        let json: String = serde_json::to_string(&delta)
            .expect("Could not serialize to json");
        assert_eq!(json, "[{\"delta\":42,\"index\":0}]");
        let delta1: <[u16; 1] as Core>::Delta = serde_json::from_str(&json)
            .expect("Could not deserialize from json");
        assert_eq!(delta, delta1);
        Ok(())
    }

    #[test]
    fn array_of_len_1__apply__same_values() -> DeltaResult<()> {
        let array0: [u16; 1] = [42];
        let array1: [u16; 1] = [42];
        let delta: <[u16; 1] as Core>::Delta = array0.delta(&array1)?;
        let array2 = array0.apply(delta)?;
        assert_eq!(array1, array2);
        Ok(())
    }

    #[test]
    fn array_of_len_1__apply__different_values() -> DeltaResult<()> {
        let array0: [u16; 1] = [10];
        let array1: [u16; 1] = [42];
        let delta: <[u16; 1] as Core>::Delta = array0.delta(&array1)?;
        let array2 = array0.apply(delta)?;
        assert_eq!(array1, array2);
        Ok(())
    }

    const N: usize = 2;

    #[test]
    fn array_of_len_N__delta__same_values() -> DeltaResult<()> {
        let array0: [u16; N] = [42, 300];
        let array1: [u16; N] = [42, 300];
        let delta: <[u16; N] as Core>::Delta = array0.delta(&array1)?;
        let json: String = serde_json::to_string(&delta)
            .expect("Could not serialize to json");
        assert_eq!(json, "[]");
        let delta1: <[u16; N] as Core>::Delta = serde_json::from_str(&json)
            .expect("Could not deserialize from json");
        assert_eq!(delta, delta1);
        Ok(())
    }

    #[test]
    fn array_of_len_N__delta__different_values() -> DeltaResult<()> {
        let array0: [u16; N] = [10,  20];
        let array1: [u16; N] = [42, 300];
        let delta: <[u16; N] as Core>::Delta = array0.delta(&array1)?;
        let json: String = serde_json::to_string_pretty(&delta)
            .expect("Could not serialize to json");
        assert_eq!(json, "[
  {
    \"delta\": 42,
    \"index\": 0
  },
  {
    \"delta\": 300,
    \"index\": 1
  }
]");
        let delta1: <[u16; N] as Core>::Delta = serde_json::from_str(&json)
            .expect("Could not deserialize from json");
        assert_eq!(delta, delta1);
        Ok(())
    }

    #[test]
    fn array_of_len_N__apply__same_values() -> DeltaResult<()> {
        let array0: [u16; N] = [42, 300];
        let array1: [u16; N] = [42, 300];
        let delta: <[u16; N] as Core>::Delta = array0.delta(&array1)?;
        let array2 = array0.apply(delta)?;
        assert_eq!(array1, array2);
        Ok(())
    }

    #[test]
    fn array_of_len_N__apply__different_values() -> DeltaResult<()> {
        let array0: [u16; N] = [10,  20];
        let array1: [u16; N] = [42, 300];
        let delta: <[u16; N] as Core>::Delta = array0.delta(&array1)?;
        let array2 = array0.apply(delta)?;
        assert_eq!(array1, array2);
        Ok(())
    }

    #[test]
    fn array__apply__out_of_bounds_index_errors() {
        let a: [u16; 1] = [10];
        let d: <[u16; 1] as Core>::Delta =
            serde_json::from_str("[{\"delta\":42,\"index\":99}]").unwrap();
        assert!(matches!(a.apply(d), Err(_))); // was: panic
    }

    #[test]
    fn array__from_delta__sparse_or_duplicate_errors() {
        let sparse: <[u16; 2] as Core>::Delta =
            serde_json::from_str("[{\"delta\":42,\"index\":0}]").unwrap();
        assert!(matches!(<[u16; 2]>::from_delta(sparse), Err(_))); // was: fabricates Default
        let dup: <[u16; 1] as Core>::Delta =
            serde_json::from_str("[{\"delta\":1,\"index\":0},{\"delta\":2,\"index\":0}]").unwrap();
        assert!(matches!(<[u16; 1]>::from_delta(dup), Err(_)));
    }

}
