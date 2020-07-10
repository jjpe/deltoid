//!

use crate::{Apply, Core, Delta, DeltaResult, FromDelta, IntoDelta};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::mem::{self, MaybeUninit};


macro_rules! cast {
    ($expr:expr) => {{
        let ptr = &$expr as *const MaybeUninit<T> as *const T;
        unsafe { ptr.read() }
    }};
}


impl<T> Core for [T; 0]
where T: Clone + Debug + PartialEq + Core
    + for<'de> Deserialize<'de>
    + Serialize
{
    type Delta = Array0Delta<T>;
}

impl<T> Apply for [T; 0]
where T: Apply + FromDelta
    + for<'de> Deserialize<'de>
    + Serialize
{
    fn apply(&self, delta: Self::Delta) -> DeltaResult<Self> {
        let mut new: [MaybeUninit<T>; 0] =
            unsafe { MaybeUninit::uninit().assume_init() };
        let initialized: Vec<usize> = delta.0.iter()
            .map(|edit| edit.index)
            .collect();
        // NOTE: initialize the delta `new[index]` cells:
        for Edit { delta, index } in delta.0 {
            new[index] = MaybeUninit::new(self[index].apply(delta)?);
        }
        // NOTE: initialize the non-delta `new[index]` cells:
        for index in 0 .. 0 {
            if initialized.contains(&index) { continue }
            // NOTE: offset `new[index]` not yet initialized:
            let elt = unsafe { &mut *new[index].as_mut_ptr() };
            *elt = self[index].clone();
        }
        let array: [T; 0] = [
        ];
        mem::forget(new);
        Ok(array)
    }
}

impl<T> Delta for [T; 0]
where T: Delta + IntoDelta
    + for<'de> Deserialize<'de>
    + Serialize
{
    fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
        let mut delta = Array0Delta(Vec::with_capacity(0));
        for i in 0 .. 0 {
            if self[i] == rhs[i] { continue }
            delta.0.push(Edit {
                delta: self[i].delta(&rhs[i])?,
                index: i,
            });
        }
        Ok(delta)
    }
}

impl<T> FromDelta for [T; 0]
where T: Clone + Debug + PartialEq + FromDelta
    + for<'de> Deserialize<'de>
    + Serialize
    + Default
{
    fn from_delta(delta: <Self as Core>::Delta) -> DeltaResult<Self> {
        let mut new: [MaybeUninit<T>; 0] =
            unsafe { MaybeUninit::uninit().assume_init() };
        let initialized: Vec<usize> = delta.0.iter()
            .map(|edit| edit.index)
            .collect();
        // NOTE: initialize the delta `new[index]` cells:
        for Edit { delta: d, index } in delta.0 {
            new[index] = MaybeUninit::new(<T>::from_delta(d)?);
        }
        // NOTE: initialize the non-delta `new[index]` cells:
        for index in 0 .. 0 {
            if initialized.contains(&index) { continue }
            // NOTE: offset `new[index]` not yet initialized:
            let elt = unsafe { &mut *new[index].as_mut_ptr() };
            *elt = T::default();
        }
        let array: [T; 0] = [
        ];
        mem::forget(new);
        Ok(array)
    }
}

impl<T> IntoDelta for [T; 0]
where T: Clone + Debug + PartialEq + IntoDelta
    + for<'de> Deserialize<'de>
    + Serialize
{
    fn into_delta(self) -> DeltaResult<<Self as Core>::Delta> {
        let mut delta = Array0Delta(Vec::with_capacity(0));
        for index in 0 .. 0 {
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
pub struct Array0Delta<T: Core> (#[doc(hidden)] pub Vec<Edit<T>>);



impl<T> Core for [T; 1]
where T: Clone + Debug + PartialEq + Core
    + for<'de> Deserialize<'de>
    + Serialize
{
    type Delta = Array1Delta<T>;
}

impl<T> Apply for [T; 1]
where T: Apply + FromDelta
    + for<'de> Deserialize<'de>
    + Serialize
{
    fn apply(&self, delta: Self::Delta) -> DeltaResult<Self> {
        let mut new: [MaybeUninit<T>; 1] =
            unsafe { MaybeUninit::uninit().assume_init() };
        let initialized: Vec<usize> = delta.0.iter()
            .map(|edit| edit.index)
            .collect();
        // NOTE: initialize the delta `new[index]` cells:
        for Edit { delta, index } in delta.0 {
            new[index] = MaybeUninit::new(self[index].apply(delta)?);
        }
        // NOTE: initialize the non-delta `new[index]` cells:
        for index in 0 .. 1 {
            if initialized.contains(&index) { continue }
            // NOTE: offset `new[index]` not yet initialized:
            let elt = unsafe { &mut *new[index].as_mut_ptr() };
            *elt = self[index].clone();
        }
        let array: [T; 1] = [
            cast!(new[0]),
        ];
        mem::forget(new);
        Ok(array)
    }
}

impl<T> Delta for [T; 1]
where T: Delta + IntoDelta
    + for<'de> Deserialize<'de>
    + Serialize
{
    fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
        let mut delta = Array1Delta(Vec::with_capacity(1));
        for i in 0 .. 1 {
            if self[i] == rhs[i] { continue }
            delta.0.push(Edit {
                delta: self[i].delta(&rhs[i])?,
                index: i,
            });
        }
        Ok(delta)
    }
}

impl<T> FromDelta for [T; 1]
where T: Clone + Debug + PartialEq + FromDelta
    + for<'de> Deserialize<'de>
    + Serialize
    + Default
{
    fn from_delta(delta: <Self as Core>::Delta) -> DeltaResult<Self> {
        let mut new: [MaybeUninit<T>; 1] =
            unsafe { MaybeUninit::uninit().assume_init() };
        let initialized: Vec<usize> = delta.0.iter()
            .map(|edit| edit.index)
            .collect();
        // NOTE: initialize the delta `new[index]` cells:
        for Edit { delta: d, index } in delta.0 {
            new[index] = MaybeUninit::new(<T>::from_delta(d)?);
        }
        // NOTE: initialize the non-delta `new[index]` cells:
        for index in 0 .. 1 {
            if initialized.contains(&index) { continue }
            // NOTE: offset `new[index]` not yet initialized:
            let elt = unsafe { &mut *new[index].as_mut_ptr() };
            *elt = T::default();
        }
        let array: [T; 1] = [
            cast!(new[0]),
        ];
        mem::forget(new);
        Ok(array)
    }
}

impl<T> IntoDelta for [T; 1]
where T: Clone + Debug + PartialEq + IntoDelta
    + for<'de> Deserialize<'de>
    + Serialize
{
    fn into_delta(self) -> DeltaResult<<Self as Core>::Delta> {
        let mut delta = Array1Delta(Vec::with_capacity(1));
        for index in 0 .. 1 {
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
pub struct Array1Delta<T: Core> (#[doc(hidden)] pub Vec<Edit<T>>);



impl<T> Core for [T; 2]
where T: Clone + Debug + PartialEq + Core
    + for<'de> Deserialize<'de>
    + Serialize
{
    type Delta = Array2Delta<T>;
}

impl<T> Apply for [T; 2]
where T: Apply + FromDelta
    + for<'de> Deserialize<'de>
    + Serialize
{
    fn apply(&self, delta: Self::Delta) -> DeltaResult<Self> {
        let mut new: [MaybeUninit<T>; 2] =
            unsafe { MaybeUninit::uninit().assume_init() };
        let initialized: Vec<usize> = delta.0.iter()
            .map(|edit| edit.index)
            .collect();
        // NOTE: initialize the delta `new[index]` cells:
        for Edit { delta, index } in delta.0 {
            new[index] = MaybeUninit::new(self[index].apply(delta)?);
        }
        // NOTE: initialize the non-delta `new[index]` cells:
        for index in 0 .. 2 {
            if initialized.contains(&index) { continue }
            // NOTE: offset `new[index]` not yet initialized:
            let elt = unsafe { &mut *new[index].as_mut_ptr() };
            *elt = self[index].clone();
        }
        let array: [T; 2] = [
            cast!(new[0]),
            cast!(new[1]),
        ];
        mem::forget(new);
        Ok(array)
    }
}

impl<T> Delta for [T; 2]
where T: Delta + IntoDelta
    + for<'de> Deserialize<'de>
    + Serialize
{
    fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
        let mut delta = Array2Delta(Vec::with_capacity(2));
        for i in 0 .. 2 {
            if self[i] == rhs[i] { continue }
            delta.0.push(Edit {
                delta: self[i].delta(&rhs[i])?,
                index: i,
            });
        }
        Ok(delta)
    }
}

impl<T> FromDelta for [T; 2]
where T: Clone + Debug + PartialEq + FromDelta
    + for<'de> Deserialize<'de>
    + Serialize
    + Default
{
    fn from_delta(delta: <Self as Core>::Delta) -> DeltaResult<Self> {
        let mut new: [MaybeUninit<T>; 2] =
            unsafe { MaybeUninit::uninit().assume_init() };
        let initialized: Vec<usize> = delta.0.iter()
            .map(|edit| edit.index)
            .collect();
        // NOTE: initialize the delta `new[index]` cells:
        for Edit { delta: d, index } in delta.0 {
            new[index] = MaybeUninit::new(<T>::from_delta(d)?);
        }
        // NOTE: initialize the non-delta `new[index]` cells:
        for index in 0 .. 2 {
            if initialized.contains(&index) { continue }
            // NOTE: offset `new[index]` not yet initialized:
            let elt = unsafe { &mut *new[index].as_mut_ptr() };
            *elt = T::default();
        }
        let array: [T; 2] = [
            cast!(new[0]),
            cast!(new[1]),
        ];
        mem::forget(new);
        Ok(array)
    }
}

impl<T> IntoDelta for [T; 2]
where T: Clone + Debug + PartialEq + IntoDelta
    + for<'de> Deserialize<'de>
    + Serialize
{
    fn into_delta(self) -> DeltaResult<<Self as Core>::Delta> {
        let mut delta = Array2Delta(Vec::with_capacity(2));
        for index in 0 .. 2 {
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
pub struct Array2Delta<T: Core> (#[doc(hidden)] pub Vec<Edit<T>>);



impl<T> Core for [T; 3]
where T: Clone + Debug + PartialEq + Core
    + for<'de> Deserialize<'de>
    + Serialize
{
    type Delta = Array3Delta<T>;
}

impl<T> Apply for [T; 3]
where T: Apply + FromDelta
    + for<'de> Deserialize<'de>
    + Serialize
{
    fn apply(&self, delta: Self::Delta) -> DeltaResult<Self> {
        let mut new: [MaybeUninit<T>; 3] =
            unsafe { MaybeUninit::uninit().assume_init() };
        let initialized: Vec<usize> = delta.0.iter()
            .map(|edit| edit.index)
            .collect();
        // NOTE: initialize the delta `new[index]` cells:
        for Edit { delta, index } in delta.0 {
            new[index] = MaybeUninit::new(self[index].apply(delta)?);
        }
        // NOTE: initialize the non-delta `new[index]` cells:
        for index in 0 .. 3 {
            if initialized.contains(&index) { continue }
            // NOTE: offset `new[index]` not yet initialized:
            let elt = unsafe { &mut *new[index].as_mut_ptr() };
            *elt = self[index].clone();
        }
        let array: [T; 3] = [
            cast!(new[0]),
            cast!(new[1]),
            cast!(new[2]),
        ];
        mem::forget(new);
        Ok(array)
    }
}

impl<T> Delta for [T; 3]
where T: Delta + IntoDelta
    + for<'de> Deserialize<'de>
    + Serialize
{
    fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
        let mut delta = Array3Delta(Vec::with_capacity(3));
        for i in 0 .. 3 {
            if self[i] == rhs[i] { continue }
            delta.0.push(Edit {
                delta: self[i].delta(&rhs[i])?,
                index: i,
            });
        }
        Ok(delta)
    }
}

impl<T> FromDelta for [T; 3]
where T: Clone + Debug + PartialEq + FromDelta
    + for<'de> Deserialize<'de>
    + Serialize
    + Default
{
    fn from_delta(delta: <Self as Core>::Delta) -> DeltaResult<Self> {
        let mut new: [MaybeUninit<T>; 3] =
            unsafe { MaybeUninit::uninit().assume_init() };
        let initialized: Vec<usize> = delta.0.iter()
            .map(|edit| edit.index)
            .collect();
        // NOTE: initialize the delta `new[index]` cells:
        for Edit { delta: d, index } in delta.0 {
            new[index] = MaybeUninit::new(<T>::from_delta(d)?);
        }
        // NOTE: initialize the non-delta `new[index]` cells:
        for index in 0 .. 3 {
            if initialized.contains(&index) { continue }
            // NOTE: offset `new[index]` not yet initialized:
            let elt = unsafe { &mut *new[index].as_mut_ptr() };
            *elt = T::default();
        }
        let array: [T; 3] = [
            cast!(new[0]),
            cast!(new[1]),
            cast!(new[2]),
        ];
        mem::forget(new);
        Ok(array)
    }
}

impl<T> IntoDelta for [T; 3]
where T: Clone + Debug + PartialEq + IntoDelta
    + for<'de> Deserialize<'de>
    + Serialize
{
    fn into_delta(self) -> DeltaResult<<Self as Core>::Delta> {
        let mut delta = Array3Delta(Vec::with_capacity(3));
        for index in 0 .. 3 {
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
pub struct Array3Delta<T: Core> (#[doc(hidden)] pub Vec<Edit<T>>);



impl<T> Core for [T; 4]
where T: Clone + Debug + PartialEq + Core
    + for<'de> Deserialize<'de>
    + Serialize
{
    type Delta = Array4Delta<T>;
}

impl<T> Apply for [T; 4]
where T: Apply + FromDelta
    + for<'de> Deserialize<'de>
    + Serialize
{
    fn apply(&self, delta: Self::Delta) -> DeltaResult<Self> {
        let mut new: [MaybeUninit<T>; 4] =
            unsafe { MaybeUninit::uninit().assume_init() };
        let initialized: Vec<usize> = delta.0.iter()
            .map(|edit| edit.index)
            .collect();
        // NOTE: initialize the delta `new[index]` cells:
        for Edit { delta, index } in delta.0 {
            new[index] = MaybeUninit::new(self[index].apply(delta)?);
        }
        // NOTE: initialize the non-delta `new[index]` cells:
        for index in 0 .. 4 {
            if initialized.contains(&index) { continue }
            // NOTE: offset `new[index]` not yet initialized:
            let elt = unsafe { &mut *new[index].as_mut_ptr() };
            *elt = self[index].clone();
        }
        let array: [T; 4] = [
            cast!(new[0]),
            cast!(new[1]),
            cast!(new[2]),
            cast!(new[3]),
        ];
        mem::forget(new);
        Ok(array)
    }
}

impl<T> Delta for [T; 4]
where T: Delta + IntoDelta
    + for<'de> Deserialize<'de>
    + Serialize
{
    fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
        let mut delta = Array4Delta(Vec::with_capacity(4));
        for i in 0 .. 4 {
            if self[i] == rhs[i] { continue }
            delta.0.push(Edit {
                delta: self[i].delta(&rhs[i])?,
                index: i,
            });
        }
        Ok(delta)
    }
}

impl<T> FromDelta for [T; 4]
where T: Clone + Debug + PartialEq + FromDelta
    + for<'de> Deserialize<'de>
    + Serialize
    + Default
{
    fn from_delta(delta: <Self as Core>::Delta) -> DeltaResult<Self> {
        let mut new: [MaybeUninit<T>; 4] =
            unsafe { MaybeUninit::uninit().assume_init() };
        let initialized: Vec<usize> = delta.0.iter()
            .map(|edit| edit.index)
            .collect();
        // NOTE: initialize the delta `new[index]` cells:
        for Edit { delta: d, index } in delta.0 {
            new[index] = MaybeUninit::new(<T>::from_delta(d)?);
        }
        // NOTE: initialize the non-delta `new[index]` cells:
        for index in 0 .. 4 {
            if initialized.contains(&index) { continue }
            // NOTE: offset `new[index]` not yet initialized:
            let elt = unsafe { &mut *new[index].as_mut_ptr() };
            *elt = T::default();
        }
        let array: [T; 4] = [
            cast!(new[0]),
            cast!(new[1]),
            cast!(new[2]),
            cast!(new[3]),
        ];
        mem::forget(new);
        Ok(array)
    }
}

impl<T> IntoDelta for [T; 4]
where T: Clone + Debug + PartialEq + IntoDelta
    + for<'de> Deserialize<'de>
    + Serialize
{
    fn into_delta(self) -> DeltaResult<<Self as Core>::Delta> {
        let mut delta = Array4Delta(Vec::with_capacity(4));
        for index in 0 .. 4 {
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
pub struct Array4Delta<T: Core> (#[doc(hidden)] pub Vec<Edit<T>>);




impl<T> Core for [T; 5]
where T: Clone + Debug + PartialEq + Core
    + for<'de> Deserialize<'de>
    + Serialize
{
    type Delta = Array5Delta<T>;
}

impl<T> Apply for [T; 5]
where T: Apply + FromDelta
    + for<'de> Deserialize<'de>
    + Serialize
{
    fn apply(&self, delta: Self::Delta) -> DeltaResult<Self> {
        let mut new: [MaybeUninit<T>; 5] =
            unsafe { MaybeUninit::uninit().assume_init() };
        let initialized: Vec<usize> = delta.0.iter()
            .map(|edit| edit.index)
            .collect();
        // NOTE: initialize the delta `new[index]` cells:
        for Edit { delta, index } in delta.0 {
            new[index] = MaybeUninit::new(self[index].apply(delta)?);
        }
        // NOTE: initialize the non-delta `new[index]` cells:
        for index in 0 .. 5 {
            if initialized.contains(&index) { continue }
            // NOTE: offset `new[index]` not yet initialized:
            let elt = unsafe { &mut *new[index].as_mut_ptr() };
            *elt = self[index].clone();
        }
        let array: [T; 5] = [
            cast!(new[0]),
            cast!(new[1]),
            cast!(new[2]),
            cast!(new[3]),
            cast!(new[4]),
        ];
        mem::forget(new);
        Ok(array)
    }
}

impl<T> Delta for [T; 5]
where T: Delta + IntoDelta
    + for<'de> Deserialize<'de>
    + Serialize
{
    fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
        let mut delta = Array5Delta(Vec::with_capacity(5));
        for i in 0 .. 5 {
            if self[i] == rhs[i] { continue }
            delta.0.push(Edit {
                delta: self[i].delta(&rhs[i])?,
                index: i,
            });
        }
        Ok(delta)
    }
}

impl<T> FromDelta for [T; 5]
where T: Clone + Debug + PartialEq + FromDelta
    + for<'de> Deserialize<'de>
    + Serialize
    + Default
{
    fn from_delta(delta: <Self as Core>::Delta) -> DeltaResult<Self> {
        let mut new: [MaybeUninit<T>; 5] =
            unsafe { MaybeUninit::uninit().assume_init() };
        let initialized: Vec<usize> = delta.0.iter()
            .map(|edit| edit.index)
            .collect();
        // NOTE: initialize the delta `new[index]` cells:
        for Edit { delta: d, index } in delta.0 {
            new[index] = MaybeUninit::new(<T>::from_delta(d)?);
        }
        // NOTE: initialize the non-delta `new[index]` cells:
        for index in 0 .. 5 {
            if initialized.contains(&index) { continue }
            // NOTE: offset `new[index]` not yet initialized:
            let elt = unsafe { &mut *new[index].as_mut_ptr() };
            *elt = T::default();
        }
        let array: [T; 5] = [
            cast!(new[0]),
            cast!(new[1]),
            cast!(new[2]),
            cast!(new[3]),
            cast!(new[4]),
        ];
        mem::forget(new);
        Ok(array)
    }
}

impl<T> IntoDelta for [T; 5]
where T: Clone + Debug + PartialEq + IntoDelta
    + for<'de> Deserialize<'de>
    + Serialize
{
    fn into_delta(self) -> DeltaResult<<Self as Core>::Delta> {
        let mut delta = Array5Delta(Vec::with_capacity(5));
        for index in 0 .. 5 {
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
pub struct Array5Delta<T: Core> (#[doc(hidden)] pub Vec<Edit<T>>);






#[derive(Clone, Debug, PartialEq)]
#[derive(serde_derive::Deserialize, serde_derive::Serialize)]
pub struct Edit<T: Core> {
    delta: <T as Core>::Delta,
    index: usize,
}




#[allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use serde_json;
    use super::*;

    #[test]
    fn Array0_delta__same_values() -> DeltaResult<()> {
        let array0: [u16; 0] = [];
        let array1: [u16; 0] = [];
        let delta: <[u16; 0] as Core>::Delta = array0.delta(&array1)?;
        let json_string = serde_json::to_string(&delta)
            .expect("Could not serialize to json");
        println!("json_string: \"{}\"", json_string);
        assert_eq!(json_string, "[]");
        let delta1: <[u16; 0] as Core>::Delta = serde_json::from_str(
            &json_string
        ).expect("Could not deserialize from json");
        assert_eq!(delta, delta1);
        Ok(())
    }

    #[test]
    fn Array0_apply__same_values() -> DeltaResult<()> {
        let array0: [u16; 0] = [];
        let array1: [u16; 0] = [];
        let delta: <[u16; 0] as Core>::Delta = array0.delta(&array1)?;
        let array2 = array0.apply(delta)?;
        assert_eq!(array1, array2);
        Ok(())
    }



    #[test]
    fn Array1_delta__same_values() -> DeltaResult<()> {
        let array0: [u16; 1] = [42];
        let array1: [u16; 1] = [42];
        let delta: <[u16; 1] as Core>::Delta = array0.delta(&array1)?;
        let json_string = serde_json::to_string(&delta)
            .expect("Could not serialize to json");
        println!("json_string: \"{}\"", json_string);
        assert_eq!(json_string, "[]");
        let delta1: <[u16; 1] as Core>::Delta = serde_json::from_str(
            &json_string
        ).expect("Could not deserialize from json");
        assert_eq!(delta, delta1);
        Ok(())
    }

    #[test]
    fn Array1_delta__different_values() -> DeltaResult<()> {
        let array0: [u16; 1] = [10];
        let array1: [u16; 1] = [42];
        let delta: <[u16; 1] as Core>::Delta = array0.delta(&array1)?;
        let json_string = serde_json::to_string(&delta)
            .expect("Could not serialize to json");
        println!("json_string: \"{}\"", json_string);
        assert_eq!(json_string, "[{\"delta\":42,\"index\":0}]");
        let delta1: <[u16; 1] as Core>::Delta = serde_json::from_str(
            &json_string
        ).expect("Could not deserialize from json");
        assert_eq!(delta, delta1);
        Ok(())
    }

    #[test]
    fn Array1_apply__same_values() -> DeltaResult<()> {
        let array0: [u16; 1] = [42];
        let array1: [u16; 1] = [42];
        let delta: <[u16; 1] as Core>::Delta = array0.delta(&array1)?;
        let array2 = array0.apply(delta)?;
        assert_eq!(array1, array2);
        Ok(())
    }

    #[test]
    fn Array1_apply__different_values() -> DeltaResult<()> {
        let array0: [u16; 1] = [10];
        let array1: [u16; 1] = [42];
        let delta: <[u16; 1] as Core>::Delta = array0.delta(&array1)?;
        let array2 = array0.apply(delta)?;
        assert_eq!(array1, array2);
        Ok(())
    }



    #[test]
    fn Array2_delta__same_values() -> DeltaResult<()> {
        let array0: [u16; 2] = [42, 300];
        let array1: [u16; 2] = [42, 300];
        let delta: <[u16; 2] as Core>::Delta = array0.delta(&array1)?;
        let json_string = serde_json::to_string(&delta)
            .expect("Could not serialize to json");
        println!("json_string: \"{}\"", json_string);
        assert_eq!(json_string, "[]");
        let delta1: <[u16; 2] as Core>::Delta = serde_json::from_str(
            &json_string
        ).expect("Could not deserialize from json");
        assert_eq!(delta, delta1);
        Ok(())
    }

    #[test]
    fn Array2_delta__different_values() -> DeltaResult<()> {
        let array0: [u16; 2] = [10,  20];
        let array1: [u16; 2] = [42, 300];
        let delta: <[u16; 2] as Core>::Delta = array0.delta(&array1)?;
        let json_string = serde_json::to_string_pretty(&delta)
            .expect("Could not serialize to json");
        println!("json_string: \"{}\"", json_string);
        assert_eq!(json_string, "[
  {
    \"delta\": 42,
    \"index\": 0
  },
  {
    \"delta\": 300,
    \"index\": 1
  }
]");
        let delta1: <[u16; 2] as Core>::Delta = serde_json::from_str(
            &json_string
        ).expect("Could not deserialize from json");
        assert_eq!(delta, delta1);
        Ok(())
    }

    #[test]
    fn Array2_apply__same_values() -> DeltaResult<()> {
        let array0: [u16; 2] = [42, 300];
        let array1: [u16; 2] = [42, 300];
        let delta: <[u16; 2] as Core>::Delta = array0.delta(&array1)?;
        let array2 = array0.apply(delta)?;
        assert_eq!(array1, array2);
        Ok(())
    }

    #[test]
    fn Array2_apply__different_values() -> DeltaResult<()> {
        let array0: [u16; 2] = [10,  20];
        let array1: [u16; 2] = [42, 300];
        let delta: <[u16; 2] as Core>::Delta = array0.delta(&array1)?;
        let array2 = array0.apply(delta)?;
        assert_eq!(array1, array2);
        Ok(())
    }



    #[test]
    fn Array3_delta__same_values() -> DeltaResult<()> {
        let array0: [u16; 3] = [42, 300, 9000];
        let array1: [u16; 3] = [42, 300, 9000];
        let delta: <[u16; 3] as Core>::Delta = array0.delta(&array1)?;
        let json_string = serde_json::to_string(&delta)
            .expect("Could not serialize to json");
        println!("json_string: \"{}\"", json_string);
        assert_eq!(json_string, "[]");
        let delta1: <[u16; 3] as Core>::Delta = serde_json::from_str(
            &json_string
        ).expect("Could not deserialize from json");
        assert_eq!(delta, delta1);
        Ok(())
    }

    #[test]
    fn Array3_delta__different_values() -> DeltaResult<()> {
        let array0: [u16; 3] = [10,  20, 9000];
        let array1: [u16; 3] = [42, 300, 89];
        let delta: <[u16; 3] as Core>::Delta = array0.delta(&array1)?;
        let json_string = serde_json::to_string_pretty(&delta)
            .expect("Could not serialize to json");
        println!("json_string: \"{}\"", json_string);
        assert_eq!(json_string, "[
  {
    \"delta\": 42,
    \"index\": 0
  },
  {
    \"delta\": 300,
    \"index\": 1
  },
  {
    \"delta\": 89,
    \"index\": 2
  }
]");
        let delta1: <[u16; 3] as Core>::Delta = serde_json::from_str(
            &json_string
        ).expect("Could not deserialize from json");
        assert_eq!(delta, delta1);
        Ok(())
    }

    #[test]
    fn Array3_apply__same_values() -> DeltaResult<()> {
        let array0: [u16; 3] = [42, 300, 9000];
        let array1: [u16; 3] = [42, 300, 9000];
        let delta: <[u16; 3] as Core>::Delta = array0.delta(&array1)?;
        let array2 = array0.apply(delta)?;
        assert_eq!(array1, array2);
        Ok(())
    }

    #[test]
    fn Array3_apply__different_values() -> DeltaResult<()> {
        let array0: [u16; 3] = [10,  20, 9000];
        let array1: [u16; 3] = [42, 300, 89];
        let delta: <[u16; 3] as Core>::Delta = array0.delta(&array1)?;
        let array2 = array0.apply(delta)?;
        assert_eq!(array1, array2);
        Ok(())
    }



    #[test]
    fn Array4_delta__same_values() -> DeltaResult<()> {
        let array0: [u16; 4] = [42, 300, 9000, 10];
        let array1: [u16; 4] = [42, 300, 9000, 10];
        let delta: <[u16; 4] as Core>::Delta = array0.delta(&array1)?;
        let json_string = serde_json::to_string(&delta)
            .expect("Could not serialize to json");
        println!("json_string: \"{}\"", json_string);
        assert_eq!(json_string, "[]");
        let delta1: <[u16; 4] as Core>::Delta = serde_json::from_str(
            &json_string
        ).expect("Could not deserialize from json");
        assert_eq!(delta, delta1);
        Ok(())
    }

    #[test]
    fn Array4_delta__different_values() -> DeltaResult<()> {
        let array0: [u16; 4] = [10,  20, 9000, 28];
        let array1: [u16; 4] = [42, 300, 89, 1];
        let delta: <[u16; 4] as Core>::Delta = array0.delta(&array1)?;
        let json_string = serde_json::to_string_pretty(&delta)
            .expect("Could not serialize to json");
        println!("json_string: \"{}\"", json_string);
        assert_eq!(json_string, "[
  {
    \"delta\": 42,
    \"index\": 0
  },
  {
    \"delta\": 300,
    \"index\": 1
  },
  {
    \"delta\": 89,
    \"index\": 2
  },
  {
    \"delta\": 1,
    \"index\": 3
  }
]");
        let delta1: <[u16; 4] as Core>::Delta = serde_json::from_str(
            &json_string
        ).expect("Could not deserialize from json");
        assert_eq!(delta, delta1);
        Ok(())
    }

    #[test]
    fn Array4_apply__same_values() -> DeltaResult<()> {
        let array0: [u16; 4] = [42, 300, 9000, 10];
        let array1: [u16; 4] = [42, 300, 9000, 10];
        let delta: <[u16; 4] as Core>::Delta = array0.delta(&array1)?;
        let array2 = array0.apply(delta)?;
        assert_eq!(array1, array2);
        Ok(())
    }

    #[test]
    fn Array4_apply__different_values() -> DeltaResult<()> {
        let array0: [u16; 4] = [10,  20, 9000, 28];
        let array1: [u16; 4] = [42, 300, 89, 1];
        let delta: <[u16; 4] as Core>::Delta = array0.delta(&array1)?;
        let array2 = array0.apply(delta)?;
        assert_eq!(array1, array2);
        Ok(())
    }



    #[test]
    fn Array5_delta__same_values() -> DeltaResult<()> {
        let array0: [u16; 5] = [42, 300, 9000, 10, 20];
        let array1: [u16; 5] = [42, 300, 9000, 10, 20];
        let delta: <[u16; 5] as Core>::Delta = array0.delta(&array1)?;
        let json_string = serde_json::to_string(&delta)
            .expect("Could not serialize to json");
        println!("json_string: \"{}\"", json_string);
        assert_eq!(json_string, "[]");
        let delta1: <[u16; 5] as Core>::Delta = serde_json::from_str(
            &json_string
        ).expect("Could not deserialize from json");
        assert_eq!(delta, delta1);
        Ok(())
    }

    #[test]
    fn Array5_delta__different_values() -> DeltaResult<()> {
        let array0: [u16; 5] = [10,  20, 9000, 28,  17];
        let array1: [u16; 5] = [42, 300,   89,  1, 456];
        let delta: <[u16; 5] as Core>::Delta = array0.delta(&array1)?;
        let json_string = serde_json:: to_string_pretty(&delta)
            .expect("Could not serialize to json");
        println!("json_string: \"{}\"", json_string);
        assert_eq!(json_string, "[
  {
    \"delta\": 42,
    \"index\": 0
  },
  {
    \"delta\": 300,
    \"index\": 1
  },
  {
    \"delta\": 89,
    \"index\": 2
  },
  {
    \"delta\": 1,
    \"index\": 3
  },
  {
    \"delta\": 456,
    \"index\": 4
  }
]");
        let delta1: <[u16; 5] as Core>::Delta = serde_json::from_str(
            &json_string
        ).expect("Could not deserialize from json");
        assert_eq!(delta, delta1);
        Ok(())
    }

    #[test]
    fn Array5_apply__same_values() -> DeltaResult<()> {
        let array0: [u16; 5] = [42, 300, 9000, 10, 20];
        let array1: [u16; 5] = [42, 300, 9000, 10, 20];
        let delta: <[u16; 5] as Core>::Delta = array0.delta(&array1)?;
        let array2 = array0.apply(delta)?;
        assert_eq!(array1, array2);
        Ok(())
    }

    #[test]
    fn Array5_apply__different_values() -> DeltaResult<()> {
        let array0: [u16; 5] = [10,  20, 9000, 28,  17];
        let array1: [u16; 5] = [42, 300,   89,  1, 456];
        let delta: <[u16; 5] as Core>::Delta = array0.delta(&array1)?;
        let array2 = array0.apply(delta)?;
        assert_eq!(array1, array2);
        Ok(())
    }

}
