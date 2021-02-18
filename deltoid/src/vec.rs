//!

use crate::{Apply, Core, Delta, DeltaError, DeltaResult, FromDelta, IntoDelta};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;


// TODO While these impls should work fine in terms of soundness, it
//      is actually more suited to a `Stack`-like type in terms of
//      efficiency.  So, change the scheme to be more efficient,
//      possibly using dynamic programming techniques to do so.

impl<T> Core for Vec<T>
where T: Clone + Debug + PartialEq + Core
    + for<'de> Deserialize<'de>
    + Serialize
{
    type Delta = VecDelta<T>;
}

impl<T> Apply for Vec<T>
where T: Clone + Debug + PartialEq + Apply + FromDelta
    + for<'de> Deserialize<'de>
    + Serialize
{
    fn apply(&self, delta: Self::Delta) -> DeltaResult<Self> {
        let mut new: Self = self.clone();
        for change in delta.into_iter() { match change {
            EltDelta::Edit { index, item } => {
                // NOTE: If self.len() == 0, the Edit should have been an Add:
                ensure_gt![self.len(), 0]?;
                // NOTE: Ensure index is not out of bounds:
                ensure_lt![index, self.len()]?;
                new[index] = self[index].apply(item)?;
            },
            EltDelta::Add(delta) =>  new.push(<T>::from_delta(delta)?),
            EltDelta::Remove { count } =>  for _ in 0 .. count {
                new.pop().ok_or_else(|| ExpectedValue!("VecDelta<T>"))?;
            },
        }}
        Ok(new)
    }
}

impl<T> Delta for Vec<T>
where T: Clone + Debug + PartialEq + Delta + IntoDelta
    + for<'de> Deserialize<'de>
    + Serialize
{
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
        Ok(VecDelta(changes))
    }
}

impl<T> FromDelta for Vec<T>
where T: Clone + Debug + PartialEq + FromDelta
    + for<'de> Deserialize<'de>
    + Serialize
{
    fn from_delta(delta: <Self as Core>::Delta) -> DeltaResult<Self> {
        let mut vec: Vec<T> = vec![];
        for (index, element) in delta.0.into_iter().enumerate() {
            match element {
                EltDelta::Add(elt) => vec.push(<T>::from_delta(elt)?),
                _ => return Err(DeltaError::IllegalDelta { index })?,
            }
        }
        Ok(vec)
    }
}

impl<T> IntoDelta for Vec<T>
where T: Clone + Debug + PartialEq + IntoDelta
    + for<'de> Deserialize<'de>
    + Serialize
{
    fn into_delta(self) -> DeltaResult<<Self as Core>::Delta> {
        let mut changes: Vec<EltDelta<T>> = vec![];
        for elt in self {
            changes.push(EltDelta::Add(elt.into_delta()?));
        }
        Ok(VecDelta(changes))
    }
}



#[derive(Clone, PartialEq)]
#[derive(serde_derive::Deserialize, serde_derive::Serialize)]
pub enum EltDelta<T: Core> {
    /// Edit a value at a given `index`.
    Edit {
        /// The location of the edit
        index: usize,
        /// The new value of the item
        item: <T as Core>::Delta,
    },
    /// Remove `count` elements from the end of the Vec.
    Remove { count: usize },
    /// Add a value.
    Add(<T as Core>::Delta),
}

impl<T: Core> std::fmt::Debug for EltDelta<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match &self {
            Self::Edit { index, item } => f.debug_struct("Edit")
                .field("index", index)
                .field("item", item)
                .finish(),
            Self::Remove { count } => f.debug_struct("Remove")
                .field("count", count)
                .finish(),
            Self::Add(delta) => write!(f, "Add({:#?})", delta),
        }
    }
}


#[derive(Clone, PartialEq)]
#[derive(serde_derive::Deserialize, serde_derive::Serialize)]
pub struct VecDelta<T: Core>(#[doc(hidden)] pub Vec<EltDelta<T>>);

impl<T: Core> VecDelta<T> {
    pub fn iter<'d>(&'d self) -> Box<dyn Iterator<Item = &EltDelta<T>> + 'd> {
        Box::new(self.0.iter())
    }

    pub fn into_iter<'d>(self) -> Box<dyn Iterator<Item = EltDelta<T>> + 'd>
    where Self: 'd {
        Box::new(self.0.into_iter())
    }

    pub fn len(&self) -> usize { self.0.len() }
}

impl<T: Core> std::fmt::Debug for VecDelta<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "VecDelta ")?;
        f.debug_list().entries(self.0.iter()).finish()
    }
}





#[cfg(test)]
mod tests {
    use super::*;

    #[allow(non_snake_case)]
    #[test]
    fn Vec__delta__same_values() -> DeltaResult<()> {
        let vec0: Vec<i32> = vec![1, 3, 10, 30];
        let vec1: Vec<i32> = vec![1, 3, 10, 30];
        assert_eq!(vec0, vec1);

        let delta = vec0.delta(&vec1)?;
        assert_eq!(delta, VecDelta(vec![]));
        let vec2 = vec0.apply(delta)?;
        assert_eq!(vec1, vec2);

        let delta = vec1.delta(&vec0)?;
        assert_eq!(delta, VecDelta(vec![]));
        let vec3 = vec1.apply(delta)?;
        assert_eq!(vec0, vec3);

        Ok(())
    }

    #[allow(non_snake_case)]
    #[test]
    fn Vec__delta__different_values__same_length() -> DeltaResult<()> {
        let vec0 = vec![1, 3, 10, 49, 30, 500];
        let vec1 = vec![1, 3, 10, 30, 500, 49];

        let delta0 = vec0.delta(&vec1)?;
        assert_eq!(delta0, VecDelta(vec![
            EltDelta::Edit { index: 3, item:  30i32.into_delta()?, },
            EltDelta::Edit { index: 4, item: 500i32.into_delta()?, },
            EltDelta::Edit { index: 5, item:  49i32.into_delta()?, },
        ]));
        let vec2 = vec0.apply(delta0)?;
        assert_eq!(vec1, vec2);

        let delta = vec1.delta(&vec0)?;
        assert_eq!(delta, VecDelta(vec![
            EltDelta::Edit { index: 3, item:  49i32.into_delta()?, },
            EltDelta::Edit { index: 4, item:  30i32.into_delta()?, },
            EltDelta::Edit { index: 5, item: 500i32.into_delta()?, },
        ]));
        let vec3 = vec1.apply(delta)?;
        assert_eq!(vec0, vec3);

        Ok(())
    }

    #[allow(non_snake_case)]
    #[test]
    fn Vec__delta__different_values__different_length() -> DeltaResult<()> {
        let vec0: Vec<i32> = vec![1, 3, 10, 30];
        let vec1: Vec<i32> = vec![1, 3, 10, 49, 30, 500];

        let delta0 = vec0.delta(&vec1)?;
        assert_eq!(delta0, VecDelta(vec![
            EltDelta::Edit { index: 3, item:  49.into_delta()?, },
            EltDelta::Add(30.into_delta()?),
            EltDelta::Add(500.into_delta()?),
        ]));
        let vec2 = vec0.apply(delta0)?;
        assert_eq!(vec1, vec2);

        let delta = vec1.delta(&vec0)?;
        assert_eq!(delta, VecDelta(vec![
            EltDelta::Edit { index: 3, item: 30.into_delta()?, },
            EltDelta::Remove  { count: 2, },
        ]));
        let vec3 = vec1.apply(delta)?;
        assert_eq!(vec0, vec3);

        Ok(())
    }

    #[allow(non_snake_case)]
    #[test]
    fn Vec__apply__different_values__same_length() -> DeltaResult<()> {
        let vec0 = vec![1, 3, 10, 30, 30];
        let vec1 = vec![1, 3, 10, 30, 40];
        let delta = vec0.delta(&vec1)?;
        let vec2 = vec0.apply(delta)?;
        assert_eq!(vec1, vec2);
        Ok(())
    }

    #[allow(non_snake_case)]
    #[test]
    fn Vec__apply__different_values__different_length() -> DeltaResult<()> {
        let vec0 = vec![1, 3, 10, 30, 30];
        let vec1 = vec![1, 3, 10, 30, 30, 40];
        let delta = vec0.delta(&vec1)?;
        let vec2 = vec0.apply(delta)?;
        assert_eq!(vec1, vec2);
        Ok(())
    }

}
