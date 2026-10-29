//! A newtype wrapping [`HashSet`] that provides extra functionality in
//! the form of delta support, de/serialization, partial equality and more.
//!
//! [`HashSet`]: https://doc.rust-lang.org/std/collections/struct.HashSet.html

use crate::{Apply, Core, Delta, DeltaResult, FromDelta, IntoDelta};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;


impl<T> Core for HashSet<T>
where T: Clone + Debug + PartialEq + Eq + Hash + Ord
    + for<'de> Deserialize<'de>
    + Serialize,
{
    type Delta = HashSetDelta<T>;
}

impl<T> Apply for HashSet<T>
where T: Clone + Debug + PartialEq + Eq + Hash + Ord
    + for<'de> Deserialize<'de>
    + Serialize,
{
    fn apply(&self, delta: Self::Delta) -> DeltaResult<Self> {
        let mut new: Self = self.clone();
        for entry_delta in delta.0 { match entry_delta {
            EntryDelta::Add { item } => {
                new.insert(item);
            },
            EntryDelta::Remove { item } => {
                new.remove(&item);
            },
        }}
        Ok(new)
    }
}

impl<T> Delta for HashSet<T>
where T: Clone + Debug + PartialEq + Eq + Hash + Ord
    + for<'de> Deserialize<'de>
    + Serialize,
{
    fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
        // NOTE: set membership is identity; entries carry raw items.
        //       equal sets naturally yield an empty diff.
        //       items are sorted so deltas are deterministic.
        let mut entry_deltas: Vec<EntryDelta<T>> = vec![];
        let mut additions: Vec<&T> = rhs.difference(&self).collect();
        additions.sort();
        for addition in additions {
            entry_deltas.push(EntryDelta::Add { item: addition.clone() });
        }
        let mut removals: Vec<&T> = self.difference(&rhs).collect();
        removals.sort();
        for removal in removals {
            entry_deltas.push(EntryDelta::Remove { item: removal.clone() });
        }
        Ok(HashSetDelta(entry_deltas))
    }
}

impl<T> FromDelta for HashSet<T>
where T: Clone + Debug + PartialEq + Eq + Hash + Ord
    + for<'de> Deserialize<'de>
    + Serialize,
{
    fn from_delta(delta: Self::Delta) -> DeltaResult<Self> {
        let mut set = Self::new();
        for entry in delta.0 { match entry {
            EntryDelta::Add { item } => {
                set.insert(item);
            },
            // A base-free conversion cannot remove; no-op.
            EntryDelta::Remove { .. } => {},
        }}
        Ok(set)
    }
}

impl<T> IntoDelta for HashSet<T>
where T: Clone + Debug + PartialEq + Eq + Hash + Ord
    + for<'de> Deserialize<'de>
    + Serialize,
{
    fn into_delta(self) -> DeltaResult<Self::Delta> {
        let mut items: Vec<T> = self.into_iter().collect();
        items.sort();
        let mut changes: Vec<EntryDelta<T>> = vec![];
        for item in items {
            changes.push(EntryDelta::Add { item });
        }
        Ok(HashSetDelta(changes))
    }
}



#[derive(Clone, PartialEq)]
#[derive(serde_derive::Deserialize, serde_derive::Serialize)]
pub struct HashSetDelta<T>(
    #[doc(hidden)] pub Vec<EntryDelta<T>>,
);

impl<T> HashSetDelta<T> {
    pub fn iter<'d>(&'d self) -> Box<dyn Iterator<Item = &EntryDelta<T>> + 'd> {
        Box::new(self.0.iter())
    }

    pub fn into_iter<'d>(self) -> Box<dyn Iterator<Item = EntryDelta<T>> + 'd>
    where Self: 'd {
        Box::new(self.0.into_iter())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl<T> std::fmt::Debug for HashSetDelta<T>
where T: std::fmt::Debug {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "HashSetDelta ")?;
        f.debug_list().entries(self.0.iter()).finish()
    }
}



#[derive(Clone, PartialEq)]
#[derive(serde_derive::Deserialize, serde_derive::Serialize)]
pub enum EntryDelta<T> {
    Add { item: T },
    Remove { item: T },
}

impl<T> std::fmt::Debug for EntryDelta<T>
where T: std::fmt::Debug {
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
    use std::collections::HashSet;

    macro_rules! set {
        ($($val:expr),* $(,)?) => {{ #[allow(redundant_semicolons)] {
            let mut set = HashSet::new();
            $( set.insert($val); )* ;
            set
        }}}
    }

    #[test]
    fn HashSet__delta__same_values() -> DeltaResult<()> {
        let set0: HashSet<String> = set! {
            "bar".into(),
            "foo".into(),
            "floozie".into(),
            "quux".into(),
        };
        let set1: HashSet<String> = set! {
            "bar".into(),
            "foo".into(),
            "floozie".into(),
            "quux".into(),
        };
        let delta = set0.delta(&set1)?;
        let expected = HashSetDelta(vec![]);
        assert_eq!(delta, expected);
        let set2 = set0.apply(delta)?;
        assert_eq!(set0, set2);
        assert_eq!(set1, set2);

        let delta = set1.delta(&set0)?;
        assert_eq!(delta, HashSetDelta(vec![]));
        let set3 = set1.apply(delta)?;
        assert_eq!(set0, set3);
        assert_eq!(set1, set3);

        Ok(())
    }

    #[test]
    fn HashSet__delta__different_values() -> DeltaResult<()> {
        let set0: HashSet<String> = set! {
            "bar".into(),
            "foo".into(),
            "floozie".into(),
            "quux".into(),
        };
        let set1: HashSet<String> = set! {
            "bar".into(),
            "baz".into(),
            "foo".into(),
            "quux".into(),
        };
        let delta0 = set0.delta(&set1)?;
        let expected = HashSetDelta(vec![
            EntryDelta::Add { item: "baz".to_string() },
            EntryDelta::Remove { item: "floozie".to_string() },
        ]);
        assert_eq!(delta0, expected);
        let set2 = set0.apply(delta0)?;
        assert_eq!(set1, set2);

        let delta1 = set1.delta(&set0)?;
        assert_eq!(delta1, HashSetDelta(vec![
            EntryDelta::Add { item: "floozie".to_string() },
            EntryDelta::Remove { item: "baz".to_string() },
        ]));
        let set3 = set1.apply(delta1)?;
        assert_eq!(set0, set3);

        Ok(())
    }

    #[test]
    fn HashSet__apply__same_values() -> DeltaResult<()> {
        let set0: HashSet<String> = set! {
            "bar".into(),
            "foo".into(),
            "floozie".into(),
            "quux".into(),
        };
        let set1: HashSet<String> = set! {
            "bar".into(),
            "foo".into(),
            "floozie".into(),
            "quux".into(),
        };
        let delta = set0.delta(&set1)?;
        assert_eq!(delta, HashSetDelta(vec![]));
        let set2 = set0.apply(delta)?;
        assert_eq!(set1, set2);
        Ok(())
    }

    #[test]
    fn HashSet__apply__different_values() -> DeltaResult<()> {
        let set0: HashSet<String> = set! {
            "bar".into(),
            "foo".into(),
            "floozie".into(),
            "quux".into(),
        };
        let set1: HashSet<String> = set! {
            "bar".into(),
            "baz".into(),
            "foo".into(),
            "quux".into(),
        };
        let delta = set0.delta(&set1)?;
        assert_eq!(delta, HashSetDelta(vec![
            EntryDelta::Add { item: "baz".to_string() },
            EntryDelta::Remove { item: "floozie".to_string() },
        ]));
        let set2 = set0.apply(delta)?;
        assert_eq!(set1, set2);
        Ok(())
    }
}
