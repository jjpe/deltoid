//!

use crate::{Apply, Core, Delta, DeltaResult, FromDelta, IntoDelta};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::mem::{self, MaybeUninit};

impl<T, const LEN: usize> Core for [T; LEN]
where T: Clone + Debug + PartialEq + Core
    + for<'de> Deserialize<'de>
    + Serialize
{
    type Delta = ArrayDelta<T, LEN>;
}

impl<T, const LEN: usize> Apply for [T; LEN]
where T: Apply + FromDelta
    + for<'de> Deserialize<'de>
    + Serialize
{
    fn apply(&self, delta: Self::Delta) -> DeltaResult<Self> {
        let mut new: [MaybeUninit<T>; LEN] =
            unsafe { MaybeUninit::uninit().assume_init() };
        let initialized: Vec<usize> = delta.0.iter()
            .map(|edit| edit.index)
            .collect();
        // NOTE: initialize the delta `new[index]` cells:
        for Edit { delta: d, index } in delta.0 {
            new[index] = MaybeUninit::new(self[index].apply(d)?);
        }
        // NOTE: initialize the non-delta `new[index]` cells:
        for index in 0 .. LEN {
            if initialized.contains(&index) { continue }
            // NOTE: offset `new[index]` not yet initialized:
            let elt = unsafe { &mut *new[index].as_mut_ptr() };
            *elt = self[index].clone();
        }
        Ok(unsafe { array_assume_init(new) })
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
    + Default
{
    fn from_delta(delta: <Self as Core>::Delta) -> DeltaResult<Self> {
        let mut new: [MaybeUninit<T>; LEN] =
            unsafe { MaybeUninit::uninit().assume_init() };
        let initialized: Vec<usize> = delta.0.iter()
            .map(|edit| edit.index)
            .collect();
        // NOTE: initialize the delta `new[index]` cells:
        for Edit { delta: d, index } in delta.0 {
            new[index] = MaybeUninit::new(<T>::from_delta(d)?);
        }
        // NOTE: initialize the non-delta `new[index]` cells:
        for index in 0 .. LEN {
            if initialized.contains(&index) { continue }
            // NOTE: offset `new[index]` not yet initialized:
            let elt = unsafe { &mut *new[index].as_mut_ptr() };
            *elt = T::default();
        }
        Ok(unsafe { array_assume_init(new) })
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
    delta: <T as Core>::Delta,
    index: usize,
}

#[inline(never)]
unsafe fn array_assume_init<T, const N: usize>(
    array: [MaybeUninit<T>; N]
) -> [T; N] {
    mem::transmute_copy(&array)
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

}
