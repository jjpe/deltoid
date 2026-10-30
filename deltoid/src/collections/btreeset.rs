//! A newtype wrapping [`BTreeSet`] that provides extra functionality in
//! the form of delta support, de/serialization, partial equality and more.
//!
//! [`BTreeSet`]: https://doc.rust-lang.org/std/collections/struct.BTreeSet.html

use crate::{Apply, Core, Delta, DeltaResult, FromDelta, IntoDelta};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt::Debug;


impl<T> Core for BTreeSet<T>
where T: Clone + Debug + PartialEq + Ord
    + for<'de> Deserialize<'de>
    + Serialize,
{
    type Delta = BTreeSetDelta<T>;
}

impl<T> Apply for BTreeSet<T>
where T: Clone + Debug + PartialEq + Ord
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

impl<T> Delta for BTreeSet<T>
where T: Clone + Debug + PartialEq + Ord
    + for<'de> Deserialize<'de>
    + Serialize,
{
    fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
        // NOTE: set membership is identity; entries carry raw items.
        //       equal sets naturally yield an empty diff.
        let mut entry_deltas: Vec<EntryDelta<T>> = vec![];
        for addition in rhs.difference(&self) {
            entry_deltas.push(EntryDelta::Add { item: addition.clone() });
        }
        for removal in self.difference(&rhs) {
            entry_deltas.push(EntryDelta::Remove { item: removal.clone() });
        }
        Ok(BTreeSetDelta(entry_deltas))
    }
}

impl<T> FromDelta for BTreeSet<T>
where T: Clone + Debug + PartialEq + Ord
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

impl<T> IntoDelta for BTreeSet<T>
where T: Clone + Debug + PartialEq + Ord
    + for<'de> Deserialize<'de>
    + Serialize,
{
    fn into_delta(self) -> DeltaResult<Self::Delta> {
        let mut changes: Vec<EntryDelta<T>> = vec![];
        for item in self {
            changes.push(EntryDelta::Add { item });
        }
        Ok(BTreeSetDelta(changes))
    }
}



#[derive(Clone, PartialEq)]
#[derive(serde_derive::Deserialize, serde_derive::Serialize)]
pub struct BTreeSetDelta<T>(
    #[doc(hidden)] pub Vec<EntryDelta<T>>,
);

impl<T> BTreeSetDelta<T> {
    pub fn iter<'d>(&'d self) -> impl Iterator<Item = &'d EntryDelta<T>> + 'd {
        self.0.iter()
    }

    pub fn into_iter<'d>(self) -> impl Iterator<Item = EntryDelta<T>> + 'd
    where
        Self: 'd
    {
        self.0.into_iter()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl<T> std::fmt::Debug for BTreeSetDelta<T>
where T: std::fmt::Debug {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "BTreeSetDelta ")?;
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
        let expected = BTreeSetDelta(vec![]);
        assert_eq!(delta, expected);
        let set2 = set0.apply(delta)?;
        assert_eq!(set0, set2);
        assert_eq!(set1, set2);

        let delta = set1.delta(&set0)?;
        assert_eq!(delta, BTreeSetDelta(vec![]));
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
        let expected = BTreeSetDelta(vec![
            EntryDelta::Add { item: "baz".to_string() },
            EntryDelta::Remove { item: "floozie".to_string() },
        ]);
        assert_eq!(delta0, expected);
        let set2 = set0.apply(delta0)?;
        assert_eq!(set1, set2);

        let delta1 = set1.delta(&set0)?;
        assert_eq!(delta1, BTreeSetDelta(vec![
            EntryDelta::Add { item: "floozie".to_string() },
            EntryDelta::Remove { item: "baz".to_string() },
        ]));
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
        assert_eq!(delta, BTreeSetDelta(vec![]));
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
        assert_eq!(delta, BTreeSetDelta(vec![
            EntryDelta::Add { item: "baz".to_string() },
            EntryDelta::Remove { item: "floozie".to_string() },
        ]));
        let set2 = set0.apply(delta)?;
        assert_eq!(set1, set2);
        Ok(())
    }

    /// Element type that does *not* implement `Core`: set deltas only
    /// require Clone/Debug/PartialEq/Ord/serde on the item.
    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord,
             serde_derive::Serialize, serde_derive::Deserialize)]
    struct NotDeltoid(u8);

    #[test]
    fn BTreeSet__roundtrip__non_core_items() -> DeltaResult<()> {
        let set0: BTreeSet<NotDeltoid> = set! { NotDeltoid(1), NotDeltoid(2) };
        let set1: BTreeSet<NotDeltoid> = set! { NotDeltoid(2), NotDeltoid(3) };
        let delta = set0.delta(&set1)?;
        assert_eq!(delta, BTreeSetDelta(vec![
            EntryDelta::Add { item: NotDeltoid(3) },
            EntryDelta::Remove { item: NotDeltoid(1) },
        ]));
        assert_eq!(set0.apply(delta)?, set1);
        Ok(())
    }
}
