//!

use crate::{DeltaError, Deltoid, DeltaResult};
use crate::convert::{FromDelta, IntoDelta};
use serde::{Deserialize, Serialize};


impl<T> Deltoid for Vec<T>
where T: Clone + PartialEq + Deltoid + std::fmt::Debug
        + Serialize
        + for<'de> Deserialize<'de>
        + IntoDelta
        + FromDelta
{
    // TODO While this impl should work fine in terms of soundness, it
    //      is actually more suited to a `Stack`-like type in terms of
    //      efficiency.  So, change the scheme to be more efficient,
    //      possibly using dynamic programming techniques to do so.

    type Delta = VecDelta<T>;

    fn apply_delta(&self, delta: &Self::Delta) -> DeltaResult<Self> {
        let mut new: Self = self.clone();
        for change in delta.iter() { match change {
            EltDelta::Edit { index, item } => {
                // NOTE: If self.len() == 0, the Edit should have been an Add:
                ensure_gt![self.len(), 0]?;
                // NOTE: Ensure index is not out of bounds:
                ensure_lt![*index, self.len()]?;
                new[*index] = self[*index].apply_delta(item)?;
            },
            EltDelta::Add(delta) =>  new.push(<T>::from_delta(delta.clone())?),
            EltDelta::Remove { count } =>  for _ in 0 .. *count {
                new.pop().ok_or(DeltaError::ExpectedValue)?;
            },
        }}
        Ok(new)
    }

    fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
        let (lhs_len, rhs_len) = (self.len(), rhs.len());
        let max_len = usize::max(lhs_len, rhs_len);
        let mut changes: Vec<EltDelta<T>> = vec![];
        for index in 0 .. max_len { match (self.get(index), rhs.get(index)) {
            (None, None) => return bug_detected!(),
            (Some(lhs), Some(rhs)) if lhs == rhs => {/*NOP*/},
            (Some(lhs), Some(rhs)) =>
                changes.push(EltDelta::Edit { index, item: lhs.delta(rhs)? }),
            (None, Some(rhs)) =>
                changes.push(EltDelta::Add(rhs.clone().into_delta()?)),
            (Some(_),   None) => match changes.last_mut() {
                Some(EltDelta::Remove { ref mut count }) => *count += 1,
                _ => changes.push(EltDelta::Remove { count: 1 }),
            },
        }}
        Ok(VecDelta(if !changes.is_empty() {
            Some(changes)
        } else {
            None
        }))
    }
}


impl<T> IntoDelta for Vec<T>
where T: Clone + PartialEq + Deltoid + std::fmt::Debug
        + for<'de> serde::Deserialize<'de>
        + serde::Serialize
        + IntoDelta
        + FromDelta
{
    fn into_delta(self) -> DeltaResult<<Self as Deltoid>::Delta> {
        let mut changes: Vec<EltDelta<T>> = vec![];
        for elt in self {
            changes.push(EltDelta::Add(elt.into_delta()?));
        }
        Ok(VecDelta(if !changes.is_empty() {
            Some(changes)
        } else {
            None
        }))
    }
}

impl<T> FromDelta for Vec<T>
where T: Clone + PartialEq + Deltoid + std::fmt::Debug
        + for<'de> serde::Deserialize<'de>
        + serde::Serialize
        + IntoDelta
        + FromDelta
{
    fn from_delta(delta: <Self as Deltoid>::Delta) -> DeltaResult<Self> {
        let mut vec: Vec<T> = vec![];
        if let Some(delta) = delta.0 {
            for (index, element) in delta.into_iter().enumerate() {
                match element {
                    EltDelta::Add(elt) => vec.push(<T>::from_delta(elt)?),
                    _ => return Err(DeltaError::IllegalDelta { index })?,
                }
            }
        }
        Ok(vec)
     }
}

#[derive(Clone, Debug, PartialEq)]
#[derive(serde_derive::Deserialize, serde_derive::Serialize)]
pub enum EltDelta<T: Deltoid> {
    /// Edit a value at a given `index`.
    Edit {
        /// The location of the edit
        index: usize,
        /// The new value of the item
        item: <T as Deltoid>::Delta,
    },
    /// Remove `count` elements from the end of the Vec.
    Remove { count: usize },
    /// Add a value.
    Add(<T as Deltoid>::Delta),
}

#[derive(Clone, Debug, PartialEq)]
#[derive(serde_derive::Deserialize, serde_derive::Serialize)]
pub struct VecDelta<T: Deltoid>(#[doc(hidden)] pub Option<Vec<EltDelta<T>>>);

impl<T: Deltoid> VecDelta<T> {
    pub fn iter<'d>(&'d self) -> Box<dyn Iterator<Item = &EltDelta<T>> + 'd> {
        match &self.0 {
            Some(vec) => Box::new(vec.iter()),
            None => Box::new(std::iter::empty()),
        }

    }

    pub fn into_iter<'d>(self) -> Box<dyn Iterator<Item = EltDelta<T>> + 'd>
    where Self: 'd {
        match self.0 {
            Some(vec) => Box::new(vec.into_iter()),
            None => Box::new(std::iter::empty()),
        }
    }

    pub fn len(&self) -> usize {
        match &self.0 {
            Some(vec) => vec.len(),
            None => 0,
        }
    }
}





#[allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculate_delta_for_Vec() -> DeltaResult<()> {
        let v0: Vec<i32> = vec![1, 3, 10, 30];
        let v1: Vec<i32> = vec![1, 3, 10, 49, 30, 500];
        let delta0 = v0.delta(&v1)?;
        println!("delta0: {:#?}", delta0);
        assert_eq!(delta0, VecDelta(Some(vec![
            EltDelta::Edit { index: 3, item:  49.into_delta()?, },
            EltDelta::Add(30.into_delta()?),
            EltDelta::Add(500.into_delta()?),
        ])));
        let v2 = v0.apply_delta(&delta0)?;
        println!("v2: {:#?}", v2);
        assert_eq!(v1, v2);

        let delta1 = v1.delta(&v0)?;
        println!("delta1: {:#?}", delta1);
        assert_eq!(delta1, VecDelta(Some(vec![
            EltDelta::Edit { index: 3, item: 30.into_delta()?, },
            EltDelta::Remove  { count: 2, },
        ])));
        let v3 = v1.apply_delta(&delta1)?;
        println!("v3: {:#?}", v3);
        assert_eq!(v0, v3);

        let v0 = vec![1, 3, 10, 49, 30, 500];
        let v1 = vec![1, 3, 10, 30, 500, 49];
        let delta0 = v0.delta(&v1)?;
        println!("delta0: {:#?}", delta0);
        assert_eq!(delta0, VecDelta(Some(vec![
            EltDelta::Edit { index: 3, item:  30i32.into_delta()?, },
            EltDelta::Edit { index: 4, item: 500i32.into_delta()?, },
            EltDelta::Edit { index: 5, item:  49i32.into_delta()?, },
        ])));
        let v2 = v0.apply_delta(&delta0)?;
        println!("v2: {:#?}", v2);
        assert_eq!(v1, v2);

        Ok(())
    }

    #[test]
    fn apply_delta_to_Vec() -> DeltaResult<()> {
        let v0 = vec![1,3,10,30, 30];
        let delta = VecDelta(Some(vec![
            EltDelta::Edit { index: 3, item:  49i32.into_delta()?, },
            EltDelta::Add(500i32.into_delta()?),
        ]));
        let v1 = v0.apply_delta(&delta)?;
        let expected = vec![1,3,10,49, 30, 500];
        assert_eq!(expected, v1);
        Ok(())
    }
}
