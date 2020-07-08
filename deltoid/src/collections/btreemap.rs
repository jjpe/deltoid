//! A newtype wrapping [`BtreeMap`] that provides extra functionality in
//! the form of delta support, de/serialization, partial equality and more.
//!
//! [`BtreeMap`]: https://doc.rust-lang.org/std/collections/struct.BTreeMap.html

use crate::{Apply, Core, Delta, DeltaError, DeltaResult, FromDelta, IntoDelta};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, BTreeMap};
use std::fmt::Debug;


impl<K, V> Core for BTreeMap<K, V>
where K: Clone + Debug + PartialEq + Ord + Core
    + for<'de> Deserialize<'de>
    + Serialize,
      V: Clone + Debug + PartialEq + Ord + Core
    + for<'de> Deserialize<'de>
    + Serialize,
{
    type Delta = BTreeMapDelta<K, V>;
}

impl<K, V> Apply for BTreeMap<K, V>
where K: Clone + Debug + PartialEq + Ord + Apply
    + for<'de> Deserialize<'de>
    + Serialize,
      V: Clone + Debug + PartialEq + Ord + Apply + FromDelta
    + for<'de> Deserialize<'de>
    + Serialize,
{
    fn apply(&self, delta: Self::Delta) -> DeltaResult<Self> {
        let mut new: Self = self.clone();
        for change in delta.into_iter() { match change {
            EntryDelta::Edit { key, value } => {
                let place: &mut V = &mut *new.get_mut(&key)
                    .ok_or_else(|| ExpectedValue!("BTreeMapDelta<K, V>"))?;
                *place = <V>::from_delta(value)?;
            },
            EntryDelta::Add { key, value } => {
                new.insert(key, <V>::from_delta(value)?);
            },
            EntryDelta::Remove { key } =>  { new.remove(&key); },
        }}
        Ok(new)
    }
}

impl<K, V> Delta for BTreeMap<K, V>
where K: Clone + Debug + PartialEq + Ord + Delta
    + for<'de> Deserialize<'de>
    + Serialize,
      V: Clone + Debug + PartialEq + Ord + Delta + IntoDelta
    + for<'de> Deserialize<'de>
    + Serialize,
{
    fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
        let lkeys: BTreeSet<&K> = self.keys().collect();
        let rkeys: BTreeSet<&K> =  rhs.keys().collect();
        let edited_keys = lkeys.intersection(&rkeys)
            .filter(|key| self[key] != rhs[key]);
        let removed_keys = lkeys.difference(&rkeys);
        let added_keys = rkeys.difference(&lkeys);
        let mut changes: Vec<EntryDelta<K, V>> = vec![];
        for key in edited_keys {
            let (lhs_val, rhs_val): (&V, &V) = (&self[key], &rhs[key]);
            let delta: <V as Core>::Delta = lhs_val.delta(rhs_val)?;
            changes.push(EntryDelta::Edit { key: (*key).clone(), value: delta });
        }
        for key in added_keys {
            changes.push(EntryDelta::Add {
                key: (*key).clone(),
                value: rhs[key].clone().into_delta()?,
            });
        }
        for key in removed_keys {
            changes.push(EntryDelta::Remove { key: (*key).clone() });
        }
        Ok(BTreeMapDelta(if !changes.is_empty() {
            Some(changes)
        } else {
            None
        }))
    }
}

impl<K, V> FromDelta for BTreeMap<K, V>
where K: Clone + Debug + PartialEq + Ord + FromDelta
    + for<'de> Deserialize<'de>
    + Serialize,
      V: Clone + Debug + PartialEq + Ord + FromDelta
    + for<'de> Deserialize<'de>
    + Serialize,
{
    fn from_delta(delta: Self::Delta) -> DeltaResult<Self> {
        let mut map: Self = Self::new();
        if let Some(delta) = delta.0 {
            for (index, element) in delta.into_iter().enumerate() {
                match element {
                    EntryDelta::Add { key, value } =>
                        map.insert(key, <V>::from_delta(value)?),
                    _ => return Err(DeltaError::IllegalDelta { index })?,
                };
            }
        }
        Ok(map)
    }
}

impl<K, V> IntoDelta for BTreeMap<K, V>
where K: Clone + Debug + PartialEq + Ord + IntoDelta
    + for<'de> Deserialize<'de>
    + Serialize,
      V: Clone + Debug + PartialEq + Ord + IntoDelta
    + for<'de> Deserialize<'de>
    + Serialize,
{
    fn into_delta(self) -> DeltaResult<Self::Delta> {
        let mut changes: Vec<EntryDelta<K, V>> = vec![];
        for (key, val) in self {
            changes.push(EntryDelta::Add { key, value: val.into_delta()? });
        }
        Ok(BTreeMapDelta(if !changes.is_empty() {
            Some(changes)
        } else {
            None
        }))
    }
}




#[derive(Clone, Debug, PartialEq)]
#[derive(serde_derive::Deserialize, serde_derive::Serialize)]
pub struct BTreeMapDelta<K: Core, V: Core>(
    #[doc(hidden)] pub Option<Vec<EntryDelta<K, V>>>,
);

impl<K, V> BTreeMapDelta<K, V>
where K: Clone + Debug + PartialEq + Ord + Core
    + for<'de> Deserialize<'de>
    + Serialize,
      V: Core
    + for<'de> Deserialize<'de>
    + Serialize,
{
    pub fn iter<'d>(&'d self) -> Box<dyn Iterator<Item = &EntryDelta<K, V>> + 'd> {
        match &self.0 {
            Some(delta) => Box::new(delta.iter()),
            None => Box::new(std::iter::empty()),
        }
    }

    pub fn into_iter<'d>(self) -> Box<dyn Iterator<Item = EntryDelta<K, V>> + 'd>
    where Self: 'd {
        match self.0 {
            Some(delta) => Box::new(delta.into_iter()),
            None => Box::new(std::iter::empty()),
        }
    }

    pub fn len(&self) -> usize {
        match &self.0 {
            Some(delta) => delta.len(),
            None => 0,
        }
    }
}


#[derive(Clone, Debug, PartialEq)]
#[derive(serde_derive::Deserialize, serde_derive::Serialize)]
pub enum EntryDelta<K, V: Core> {
    /// Edit a `value` of a given `key`
    Edit { key: K, value: <V as Core>::Delta },
    /// Add a given `key` and `value` entry.
    Add { key: K, value: <V as Core>::Delta },
    /// Remove the entry with a given `key` from the map.
    Remove { key: K },
}





#[allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    macro_rules! map {
        ($($key:expr => $val:expr),* $(,)?) => {{
            let mut map = BTreeMap::new();
            $( map.insert($key, $val); )*
                map
        }}
    }

    #[test]
    fn BTreeMap__delta__same_values() -> DeltaResult<()> {
        let map0: BTreeMap<String, usize> = map! {
            "bar".into()     => 300usize,
            "foo".into()     =>  42usize,
            "floozie".into() =>  0usize,
            "quux".into()    => 16000usize,
        };
        let map1: BTreeMap<String, usize> = map! {
            "bar".into()     => 300usize,
            "foo".into()     =>  42usize,
            "floozie".into() =>  0usize,
            "quux".into()    => 16000usize,
        };
        let delta = map0.delta(&map1)?;
        let expected = BTreeMapDelta(None);
        assert_eq!(delta, expected);
        let map2 = map0.apply(delta)?;
        assert_eq!(map0, map2);
        assert_eq!(map1, map2);

        let delta = map1.delta(&map0)?;
        assert_eq!(delta, BTreeMapDelta(None));
        let map3 = map1.apply(delta)?;
        assert_eq!(map0, map3);
        assert_eq!(map1, map3);

        Ok(())
    }

    #[test]
    fn BTreeMap__delta__different_values() -> DeltaResult<()> {
        let map0: BTreeMap<String, usize> = map! {
            "bar".into()     => 300usize,
            "foo".into()     =>  42usize,
            "floozie".into() =>  0usize,
            "quux".into()    => 16000usize,
        };
        let map1: BTreeMap<String, usize> = map! {
            "bar".into()  =>   350usize,
            "baz".into()  =>  9000usize,
            "foo".into()  =>    42usize,
            "quux".into() => 16000usize,
        };
        let delta0 = map0.delta(&map1)?;
        let expected = BTreeMapDelta(Some(vec![
            EntryDelta::Edit { key: "bar".into(),  value:   350usize.into_delta()? },
            EntryDelta::Add  { key: "baz".into(),  value:  9000usize.into_delta()? },
            EntryDelta::Remove { key: "floozie".into() },
        ]));
        assert_eq!(delta0, expected);
        let map2 = map0.apply(delta0)?;
        assert_eq!(map1, map2);

        let delta1 = map1.delta(&map0)?;
        assert_eq!(delta1, BTreeMapDelta(Some(vec![
            EntryDelta::Edit { key: "bar".into(),     value: 300usize.into_delta()? },
            EntryDelta::Add  { key: "floozie".into(), value:   0usize.into_delta()? },
            EntryDelta::Remove { key: "baz".into() },
        ])));
        let map3 = map1.apply(delta1)?;
        assert_eq!(map0, map3);

        Ok(())
    }

    #[test]
    fn BTreeMap__apply__same_values() -> DeltaResult<()> {
        let map0: BTreeMap<String, usize> = map! {
            "bar".into()     => 300usize,
            "foo".into()     =>  42usize,
            "floozie".into() =>  0usize,
            "quux".into()    => 16000usize,
        };
        let map1: BTreeMap<String, usize> = map! {
            "bar".into()     => 300usize,
            "foo".into()     =>  42usize,
            "floozie".into() =>  0usize,
            "quux".into()    => 16000usize,
        };
        let delta = map0.delta(&map1)?;
        assert_eq!(delta, BTreeMapDelta(None));
        let map2 = map0.apply(delta)?;
        assert_eq!(map1, map2);
        Ok(())
    }

    #[test]
    fn BTreeMap__apply__different_values() -> DeltaResult<()> {
        let map0: BTreeMap<String, usize> = map! {
            "bar".into()     => 300usize,
            "foo".into()     =>  42usize,
            "floozie".into() =>  0usize,
            "quux".into()    => 16000usize,
        };
        let map1: BTreeMap<String, usize> = map! {
            "bar".into()  =>   350usize,
            "baz".into()  =>  9000usize,
            "foo".into()  =>    42usize,
            "quux".into() => 16000usize,
        };
        let delta = map0.delta(&map1)?;
        assert_eq!(delta, BTreeMapDelta(Some(vec![
            EntryDelta::Edit { key: "bar".into(),  value:   350usize.into_delta()? },
            EntryDelta::Add  { key: "baz".into(),  value:  9000usize.into_delta()? },
            EntryDelta::Remove { key: "floozie".into() },
        ])));
        let map2 = map0.apply(delta)?;
        assert_eq!(map1, map2);
        Ok(())
    }

}
