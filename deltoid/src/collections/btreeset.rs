//! A newtype wrapping [`BTreeSet`] that provides extra functionality in
//! the form of delta support, de/serialization, partial equality and more.
//!
//! [`BTreeSet`]: https://doc.rust-lang.org/std/collections/struct.BTreeSet.html

use crate::{Deltoid, DeltaResult, FromDelta, IntoDelta};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt::Debug;


impl<T> Deltoid for BTreeSet<T>
where T: Deltoid + PartialEq + Clone + Debug + Ord
    + for<'de> Deserialize<'de> + Serialize
    + FromDelta + IntoDelta,
{
    type Delta = BTreeSetDelta<T>;

    fn apply_delta(&self, delta: &Self::Delta) -> DeltaResult<Self> {
        match delta.0 {
            None => Ok(self.clone()),
            Some(ref entry_deltas) => {
                let mut new: Self = self.clone();
                for entry_delta in entry_deltas { match entry_delta {
                    EntryDelta::Add { item } => {
                        new.insert(<T>::from_delta(item.clone())?);
                    },
                    EntryDelta::Remove { item } => {
                        new.remove(&(<T>::from_delta(item.clone())?));
                    },
                }}
                Ok(new)
            },
        }
    }

    fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
        Ok(BTreeSetDelta(if self == rhs {
            None
        } else {
            let mut entry_deltas: Vec<EntryDelta<T>> = vec![];
            for addition in rhs.difference(&self) {
                let addition = addition.clone().into_delta()?;
                entry_deltas.push(EntryDelta::Add { item: addition });
            }
            for removal in self.difference(&rhs) {
                let removal = removal.clone().into_delta()?;
                entry_deltas.push(EntryDelta::Remove { item: removal });
            }
            Some(entry_deltas)
        }))
    }
}



#[derive(Clone, Debug, PartialEq)]
#[derive(serde_derive::Deserialize, serde_derive::Serialize)]
pub struct BTreeSetDelta<T>(
    #[doc(hidden)] pub Option<Vec<EntryDelta<T>>>,
) where T: Deltoid + FromDelta + IntoDelta;

impl<T> BTreeSetDelta<T>
where T: Deltoid + PartialEq + Clone + Debug + Ord
    + for<'de> Deserialize<'de> + Serialize
    + FromDelta + IntoDelta
{
    pub fn iter<'d>(&'d self) -> Box<dyn Iterator<Item = &EntryDelta<T>> + 'd> {
        match &self.0 {
            Some(deltas) => Box::new(deltas.iter()),
            None => Box::new(std::iter::empty()),
        }
    }

    pub fn into_iter<'d>(self) -> Box<dyn Iterator<Item = EntryDelta<T>> + 'd>
    where Self: 'd {
        match self.0 {
            Some(delta) => Box::new(delta.into_iter()),
            None => Box::new(std::iter::empty()),
        }
    }

    pub fn len(&self) -> usize {
        match &self.0 {
            Some(deltas) => deltas.len(),
            None => 0,
        }
    }
}


#[derive(Clone, Debug, PartialEq)]
#[derive(serde_derive::Deserialize, serde_derive::Serialize)]
pub enum EntryDelta<T: Deltoid> {
    Add { item: <T as Deltoid>::Delta },
    Remove { item: <T as Deltoid>::Delta },
}


impl<T> IntoDelta for BTreeSet<T>
where T: Deltoid + PartialEq + Clone + Debug + Ord
    + for<'de> Deserialize<'de> + Serialize
    + FromDelta + IntoDelta
{
    fn into_delta(self) -> DeltaResult<<Self as Deltoid>::Delta> {
        Ok(BTreeSetDelta(if self.is_empty() {
            None
        } else {
            let mut changes: Vec<EntryDelta<T>> = vec![];
            for item in self {
                changes.push(EntryDelta::Add { item: item.into_delta()? });
            }
            Some(changes)
        }))
    }
}

impl<T> FromDelta for std::collections::BTreeSet<T>
where T: Deltoid + Clone + std::fmt::Debug + Ord
    + for<'de> Deserialize<'de> + Serialize
    + FromDelta + IntoDelta
{
    fn from_delta(delta: <Self as Deltoid>::Delta) -> DeltaResult<Self> {
        let mut map = Self::new();
        if let Some(delta_entries) = delta.0 {
            for entry in delta_entries { match entry {
                EntryDelta::Add { item } => {
                    map.insert(<T>::from_delta(item)?);
                },
                EntryDelta::Remove { item } => {
                    let item = <T>::from_delta(item)?;
                    map.remove(&item);
                },
            }}
        }
        Ok(map)
    }
}



#[allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    macro_rules! set {
        ($($val:expr),* $(,)?) => {{ #[allow(redundant_semicolons)] {
            let mut set = BTreeSet::new();
            $( set.insert($val); )* ;
            set
        }}}
    }

    #[test]
    fn calculate_delta_for_BTreeSet() -> DeltaResult<()> {
        let v0: BTreeSet<String> = set! {
            "bar".into(),
            "foo".into(),
            "floozie".into(),
            "quux".into(),
        };
        let v1: BTreeSet<String> = set! {
            "bar".into(),
            "baz".into(),
            "foo".into(),
            "quux".into(),
        };
        let delta0 = v0.delta(&v1)?;
        println!("delta0: {:#?}", delta0);
        let expected = BTreeSetDelta(Some(vec![
            EntryDelta::Add { item: "baz".to_string().into_delta()? },
            EntryDelta::Remove { item: "floozie".to_string().into_delta()? },
        ]));
        assert_eq!(delta0, expected, "{:#?}\n    !=\n{:#?}", delta0, expected);
        let v2 = v0.apply_delta(&delta0)?;
        println!("v2: {:#?}", v2);
        assert_eq!(v1, v2);

        let delta1 = v1.delta(&v0)?;
        println!("delta1: {:#?}", delta1);
        assert_eq!(delta1, BTreeSetDelta(Some(vec![
            EntryDelta::Add { item: "floozie".to_string().into_delta()? },
            EntryDelta::Remove { item: "baz".to_string().into_delta()? },
        ])));
        let v3 = v1.apply_delta(&delta1)?;
        println!("v3: {:#?}", v3);
        assert_eq!(v0, v3);

        Ok(())
    }

    #[test]
    fn apply_delta_to_BTreeSet() -> DeltaResult<()> {
        let v0: BTreeSet<String> = set! {
            "bar".into(),
            "foo".into(),
            "floozie".into(),
            "quux".into(),
        };
        let delta = BTreeSetDelta(Some(vec![
            EntryDelta::Add  { item: "baz".to_string().into_delta()? },
            EntryDelta::Remove { item: "floozie".to_string().into_delta()? },
        ]));
        let v1 = v0.apply_delta(&delta)?;
        let expected: BTreeSet<String> = set! {
            "bar".into(),
            "baz".into(),
            "foo".into(),
            "quux".into(),
        };
        assert_eq!(expected, v1);
        Ok(())
    }

}
