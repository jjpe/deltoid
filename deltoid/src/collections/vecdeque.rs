//! A newtype wrapping [`VecDeque`] that provides extra functionality in
//! the form of delta support, de/serialization, partial equality and more.
//!
//! [`VecDeque`]: https://doc.rust-lang.org/std/collections/struct.VecDeque.html

use crate::{Apply, Core, Delta, DeltaError, DeltaResult, FromDelta, IntoDelta};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fmt::Debug;



impl<T> Core for VecDeque<T>
where T: Clone + Debug + PartialEq + Core
    + for<'de> Deserialize<'de>
    + Serialize,
{
    type Delta = VecDequeDelta<T>;
}

impl<T> Apply for VecDeque<T>
where T: Clone + Debug + PartialEq + Apply + FromDelta
    + for<'de> Deserialize<'de>
    + Serialize,
{
    fn apply(&self, delta: Self::Delta) -> DeltaResult<Self> {
        let mut new: Self = self.clone();
        for change in delta.into_iter() { match change {
            EltDelta::Edit { index, item } => {
                // NOTE: validated against `new` so malformed/out-of-order deltas
                //       yield DeltaError instead of panicking:
                ensure_lt![index, new.len()]?;
                let updated = new[index].apply(item)?;
                new[index] = updated;
            },
            EltDelta::Add(delta) =>  new.push_back(<T>::from_delta(delta)?),
            EltDelta::Remove { count } =>  for _ in 0 .. count {
                new.pop_back().ok_or_else(|| ExpectedValue!("VecDequeDelta<T>"))?;
            },
        }}
        Ok(new)
    }
}

impl<T> Delta for VecDeque<T>
where T: Clone + Debug + PartialEq + Delta + IntoDelta
    + for<'de> Deserialize<'de>
    + Serialize,
{
    fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
        let (lhs_len, rhs_len) = (self.len(), rhs.len());
        let max_len = usize::max(lhs_len, rhs_len);
        let mut changes: VecDeque<EltDelta<T>> = VecDeque::new();
        for index in 0 .. max_len { match (self.get(index), rhs.get(index)) {
            (None, None) => return bug_detected!(),
            (Some(l), Some(r)) if l == r => {/*NOP*/},
            (Some(l), Some(r)) =>
                changes.push_back(EltDelta::Edit { index, item: l.delta(r)? }),
            (None, Some(r)) =>
                changes.push_back(EltDelta::Add(r.clone().into_delta()?)),
            (Some(_),   None) => match changes.back_mut() {
                Some(EltDelta::Remove { count }) => *count += 1,
                _ => changes.push_back(EltDelta::Remove { count: 1 }),
            },
        }}
        Ok(VecDequeDelta(changes))
    }
}

impl<T> FromDelta for VecDeque<T>
where T: Clone + Debug + PartialEq + FromDelta
    + for<'de> Deserialize<'de>
    + Serialize
{
    fn from_delta(delta: Self::Delta) -> DeltaResult<Self> {
        let mut changes: VecDeque<T> = VecDeque::new();
        for (index, element) in delta.0.into_iter().enumerate() {
            match element {
                EltDelta::Add(elt) => changes.push_back(<T>::from_delta(elt)?),
                _ => return Err(DeltaError::IllegalDelta { index })?,
            }
        }
        Ok(changes)
    }
}

impl<T> IntoDelta for VecDeque<T>
where T: Clone + Debug + PartialEq + IntoDelta
    + for<'de> Deserialize<'de>
    + Serialize
{
    fn into_delta(self) -> DeltaResult<Self::Delta> {
        let mut changes: VecDeque<EltDelta<T>> = VecDeque::new();
        for elt in self {
            changes.push_back(EltDelta::Add(elt.into_delta()?));
        }
        Ok(VecDequeDelta(changes))
    }
}




#[derive(Clone, PartialEq)]
#[derive(serde_derive::Deserialize, serde_derive::Serialize)]
pub struct VecDequeDelta<T: Core>(#[doc(hidden)] pub VecDeque<EltDelta<T>>);

impl<T: Core> VecDequeDelta<T> {
    pub fn iter<'d>(&'d self) -> impl Iterator<Item = &'d EltDelta<T>> + 'd {
        self.0.iter()
    }

    pub fn into_iter<'d>(self) -> impl Iterator<Item = EltDelta<T>> + 'd
    where
        Self: 'd
    {
        self.0.into_iter()
    }

    pub fn len(&self) -> usize { self.0.len() }
}

impl<T> std::fmt::Debug for VecDequeDelta<T>
where T: std::fmt::Debug + Core {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "VecDequeDelta ")?;
        f.debug_list().entries(self.0.iter()).finish()
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






#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;

    macro_rules! vecdeque {
        ($($item:expr),* $(,)?) => {{
            #[allow(unused_mut)]
            let mut v = VecDeque::new();
            $( v.push_back($item); )*
            v
        }}
    }

    #[allow(non_snake_case)]
    #[test]
    fn VecDeque__delta__same_values() -> DeltaResult<()> {
        let vecdeque0: VecDeque<String> = vecdeque![
            "bar".into(),
            "foo".into(),
            "floozie".into(),
            "quux".into(),
        ];
        let vecdeque1: VecDeque<String> = vecdeque![
            "bar".into(),
            "foo".into(),
            "floozie".into(),
            "quux".into(),
        ];
        let delta = vecdeque0.delta(&vecdeque1)?;
        let expected = VecDequeDelta(vecdeque![]);
        assert_eq!(delta, expected);
        let vecdeque2 = vecdeque0.apply(delta)?;
        assert_eq!(vecdeque0, vecdeque2);
        assert_eq!(vecdeque1, vecdeque2);

        let delta = vecdeque1.delta(&vecdeque0)?;
        assert_eq!(delta, VecDequeDelta(vecdeque![]));
        let vecdeque3 = vecdeque1.apply(delta)?;
        assert_eq!(vecdeque0, vecdeque3);
        assert_eq!(vecdeque1, vecdeque3);

        Ok(())
    }

    #[allow(non_snake_case)]
    #[test]
    fn VecDeque__delta__different_values() -> DeltaResult<()> {
        let vecdeque0: VecDeque<String> = vecdeque![
            "bar".into(),
            "foo".into(),
            "floozie".into(),
            "quux".into(),
        ];
        let vecdeque1: VecDeque<String> = vecdeque![
            "bar".into(),
            "baz".into(),
            "foo".into(),
            "quux".into(),
            "corge".into(),
        ];
        let delta0 = vecdeque0.delta(&vecdeque1)?;
        let expected = VecDequeDelta(vecdeque![ // TODO
            EltDelta::Edit { index: 1, item: "baz".to_string().into_delta()? },
            EltDelta::Edit { index: 2, item: "foo".to_string().into_delta()? },
            EltDelta::Add("corge".to_string().into_delta()?),
        ]);
        assert_eq!(delta0, expected);
        let vecdeque2 = vecdeque0.apply(delta0)?;
        assert_eq!(vecdeque1, vecdeque2);

        let delta1 = vecdeque1.delta(&vecdeque0)?;
        assert_eq!(delta1, VecDequeDelta(vecdeque![
            EltDelta::Edit { index: 1, item: "foo".to_string().into_delta()? },
            EltDelta::Edit { index: 2, item: "floozie".to_string().into_delta()? },
            EltDelta::Remove { count: 1 },
        ]));
        let vecdeque3 = vecdeque1.apply(delta1)?;
        assert_eq!(vecdeque0, vecdeque3);

        Ok(())
    }

    #[allow(non_snake_case)]
    #[test]
    fn VecDeque__apply__same_values() -> DeltaResult<()> {
        let vecdeque0: VecDeque<String> = vecdeque![
            "bar".into(),
            "foo".into(),
            "floozie".into(),
            "quux".into(),
        ];
        let vecdeque1: VecDeque<String> = vecdeque![
            "bar".into(),
            "foo".into(),
            "floozie".into(),
            "quux".into(),
        ];
        let delta = vecdeque0.delta(&vecdeque1)?;
        assert_eq!(delta, VecDequeDelta(vecdeque![]));
        let vecdeque2 = vecdeque0.apply(delta)?;
        assert_eq!(vecdeque1, vecdeque2);
        Ok(())
    }

    #[allow(non_snake_case)]
    #[test]
    fn VecDeque__apply__different_values() -> DeltaResult<()> {
        let vecdeque0: VecDeque<String> = vecdeque![
            "bar".into(),
            "foo".into(),
            "floozie".into(),
            "quux".into(),
        ];
        let vecdeque1: VecDeque<String> = vecdeque![
            "bar".into(),
            "baz".into(),
            "foo".into(),
            "quux".into(),
        ];
        let delta = vecdeque0.delta(&vecdeque1)?;
        assert_eq!(delta, VecDequeDelta(vecdeque![
            EltDelta::Edit { index: 1, item: "baz".to_string().into_delta()? },
            EltDelta::Edit { index: 2, item: "foo".to_string().into_delta()? },
        ]));
        let vecdeque2 = vecdeque0.apply(delta)?;
        assert_eq!(vecdeque1, vecdeque2);
        Ok(())
    }

    #[allow(non_snake_case)]
    #[test]
    fn VecDeque__delta__removal_before_any_change() -> DeltaResult<()> {
        let d0: VecDeque<String> = vecdeque!["a".into(), "b".into()];
        let d1: VecDeque<String> = vecdeque![];
        let delta = d0.delta(&d1)?; // was: `changes.len() - 1` underflow panic in debug
        assert_eq!(delta, VecDequeDelta(vecdeque![EltDelta::Remove { count: 2 }]));
        assert_eq!(d0.apply(delta)?, d1);
        Ok(())
    }

    #[allow(non_snake_case)]
    #[test]
    fn VecDeque__roundtrip__no_ord_elements() -> DeltaResult<()> {
        // f64: !Ord; compiles only because the VecDeque impls no longer
        // require Ord (Step 6c).
        let d0: VecDeque<f64> = vecdeque![1.0, 2.0, 3.0];
        let d1: VecDeque<f64> = vecdeque![1.0, 5.0];
        let delta = d0.delta(&d1)?;
        assert_eq!(d0.apply(delta)?, d1);
        let back = d1.delta(&d0)?;
        assert_eq!(d1.apply(back)?, d0);
        Ok(())
    }

}
