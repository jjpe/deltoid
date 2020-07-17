//! A newtype wrapping [`BTreeSet`] that provides extra functionality in
//! the form of delta support, de/serialization, partial equality and more.
//!
//! [`BTreeSet`]: https://doc.rust-lang.org/std/collections/struct.BTreeSet.html

use crate::{Apply, Core, Delta, DeltaResult, FromDelta, IntoDelta};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt::Debug;


impl<T> Core for BTreeSet<T>
where T: Clone + Debug + PartialEq + Ord + Core
    + for<'de> Deserialize<'de>
    + Serialize,
{
    type Delta = BTreeSetDelta<T>;
}

impl<T> Apply for BTreeSet<T>
where T: Clone + Debug + PartialEq + Ord + Apply + FromDelta
    + for<'de> Deserialize<'de>
    + Serialize,
{
    fn apply(&self, delta: Self::Delta) -> DeltaResult<Self> {
        match delta.0 {
            None => Ok(self.clone()),
            Some(entry_deltas) => {
                let mut new: Self = self.clone();
                for entry_delta in entry_deltas { match entry_delta {
                    EntryDelta::Add { item } => {
                        new.insert(<T>::from_delta(item)?);
                    },
                    EntryDelta::Remove { item } => {
                        new.remove(&(<T>::from_delta(item)?));
                    },
                }}
                Ok(new)
            },
        }
    }
}

impl<T> Delta for BTreeSet<T>
where T: Clone + Debug + PartialEq + Ord + Delta + IntoDelta
    + for<'de> Deserialize<'de>
    + Serialize,
{
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

impl<T> FromDelta for BTreeSet<T>
where T: Clone + Debug + PartialEq + Ord + FromDelta
    + for<'de> Deserialize<'de>
    + Serialize,
{
    fn from_delta(delta: Self::Delta) -> DeltaResult<Self> {
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

impl<T> IntoDelta for BTreeSet<T>
where T: Clone + Debug + PartialEq + Ord + IntoDelta
    + for<'de> Deserialize<'de>
    + Serialize,
{
    fn into_delta(self) -> DeltaResult<Self::Delta> {
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




#[derive(Clone, PartialEq)]
#[derive(serde_derive::Deserialize, serde_derive::Serialize)]
pub struct BTreeSetDelta<T: Core>(
    #[doc(hidden)] pub Option<Vec<EntryDelta<T>>>,
);

impl<T> BTreeSetDelta<T>
where T: Clone + Debug + PartialEq + Ord + Core
    + for<'de> Deserialize<'de>
    + Serialize,
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

impl<T> std::fmt::Debug for BTreeSetDelta<T>
where T: std::fmt::Debug + Core {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "BTreeSetDelta ")?;
        let mut buf = f.debug_list();
        if let Some(d) = &self.0 {
            buf.entries(d.iter());
        } else {
            buf.entries(std::iter::empty::<Vec<EntryDelta<T>>>());
        }
        buf.finish()
    }
}



#[derive(Clone, PartialEq)]
#[derive(serde_derive::Deserialize, serde_derive::Serialize)]
pub enum EntryDelta<T: Core> {
    Add { item: <T as Core>::Delta },
    Remove { item: <T as Core>::Delta },
}

impl<T> std::fmt::Debug for EntryDelta<T>
where T: std::fmt::Debug + Core {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match &self {
            Self::Add { item } => f.debug_struct("Add")
                .field("item", item)
                .finish(),
            Self::Remove { item } => f.debug_struct("Remove")
                .field("item", item)
                .finish(),
        }
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
    fn BTreeSet__delta__same_values() -> DeltaResult<()> {
        let set0: BTreeSet<String> = set! {
            "bar".into(),
            "foo".into(),
            "floozie".into(),
            "quux".into(),
        };
        let set1: BTreeSet<String> = set! {
            "bar".into(),
            "foo".into(),
            "floozie".into(),
            "quux".into(),
        };
        let delta = set0.delta(&set1)?;
        let expected = BTreeSetDelta(None);
        assert_eq!(delta, expected);
        let set2 = set0.apply(delta)?;
        assert_eq!(set0, set2);
        assert_eq!(set1, set2);

        let delta = set1.delta(&set0)?;
        assert_eq!(delta, BTreeSetDelta(None));
        let set3 = set1.apply(delta)?;
        assert_eq!(set0, set3);
        assert_eq!(set1, set3);

        Ok(())
    }

    #[test]
    fn BTreeSet__delta__different_values() -> DeltaResult<()> {
        let set0: BTreeSet<String> = set! {
            "bar".into(),
            "foo".into(),
            "floozie".into(),
            "quux".into(),
        };
        let set1: BTreeSet<String> = set! {
            "bar".into(),
            "baz".into(),
            "foo".into(),
            "quux".into(),
        };
        let delta0 = set0.delta(&set1)?;
        let expected = BTreeSetDelta(Some(vec![
            EntryDelta::Add { item: "baz".to_string().into_delta()? },
            EntryDelta::Remove { item: "floozie".to_string().into_delta()? },
        ]));
        assert_eq!(delta0, expected);
        let set2 = set0.apply(delta0)?;
        assert_eq!(set1, set2);

        let delta1 = set1.delta(&set0)?;
        assert_eq!(delta1, BTreeSetDelta(Some(vec![
            EntryDelta::Add { item: "floozie".to_string().into_delta()? },
            EntryDelta::Remove { item: "baz".to_string().into_delta()? },
        ])));
        let set3 = set1.apply(delta1)?;
        assert_eq!(set0, set3);

        Ok(())
    }

    #[test]
    fn BTreeSet__apply__same_values() -> DeltaResult<()> {
        let set0: BTreeSet<String> = set! {
            "bar".into(),
            "foo".into(),
            "floozie".into(),
            "quux".into(),
        };
        let set1: BTreeSet<String> = set! {
            "bar".into(),
            "foo".into(),
            "floozie".into(),
            "quux".into(),
        };
        let delta = set0.delta(&set1)?;
        assert_eq!(delta, BTreeSetDelta(None));
        let set2 = set0.apply(delta)?;
        assert_eq!(set1, set2);
        Ok(())
    }

    #[test]
    fn BTreeSet__apply__different_values() -> DeltaResult<()> {
        let set0: BTreeSet<String> = set! {
            "bar".into(),
            "foo".into(),
            "floozie".into(),
            "quux".into(),
        };
        let set1: BTreeSet<String> = set! {
            "bar".into(),
            "baz".into(),
            "foo".into(),
            "quux".into(),
        };
        let delta = set0.delta(&set1)?;
        assert_eq!(delta, BTreeSetDelta(Some(vec![
            EntryDelta::Add { item: "baz".to_string().into_delta()? },
            EntryDelta::Remove { item: "floozie".to_string().into_delta()? },
        ])));
        let set2 = set0.apply(delta)?;
        assert_eq!(set1, set2);
        Ok(())
    }
}
