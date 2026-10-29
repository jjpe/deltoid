//! A newtype wrapping [`HashMap`] that provides extra functionality in
//! the form of delta support, de/serialization, partial equality and more.
//!
//! [`HashMap`]: https://doc.rust-lang.org/std/collections/struct.HashMap.html

use crate::{Apply, Core, Delta, DeltaError, DeltaResult, FromDelta, IntoDelta};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;


impl<K, V> Core for HashMap<K, V>
where K: Clone + Debug + PartialEq + Ord + Hash + Core
    + for<'de> Deserialize<'de>
    + Serialize,
      V: Clone + Debug + PartialEq + Core
    + for<'de> Deserialize<'de>
    + Serialize,
{
    type Delta = HashMapDelta<K, V>;
}

impl<K, V> Apply for HashMap<K, V>
where K: Clone + Debug + PartialEq + Ord + Hash + Apply
    + for<'de> Deserialize<'de>
    + Serialize,
      V: Clone + Debug + PartialEq + Apply + FromDelta
    + for<'de> Deserialize<'de>
    + Serialize,
{
    fn apply(&self, delta: Self::Delta) -> DeltaResult<Self> {
        let mut new: Self = self.clone();
        for change in delta.into_iter() { match change {
            EntryDelta::Edit { key, value } => {
                let place: &mut V = &mut *new.get_mut(&key)
                    .ok_or_else(|| ExpectedValue!("HashMapDelta<K, V>"))?;
                let updated = place.apply(value)?;
                *place = updated;
            },
            EntryDelta::Add { key, value } => {
                new.insert(key, <V>::from_delta(value)?);
            },
            EntryDelta::Remove { key } =>  { new.remove(&key); },
        }}
        Ok(new)
    }
}

impl<K, V> Delta for HashMap<K, V>
where K: Clone + Debug + PartialEq + Ord + Hash + Delta
    + for<'de> Deserialize<'de>
    + Serialize,
      V: Clone + Debug + PartialEq + Delta + IntoDelta
    + for<'de> Deserialize<'de>
    + Serialize,
{
    fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
        let mut edited_keys: Vec<&K> = self.keys()
            .filter(|k| rhs.contains_key(*k) && self[*k] != rhs[*k])
            .collect();
        let mut added_keys: Vec<&K> =
            rhs.keys().filter(|k| !self.contains_key(*k)).collect();
        let mut removed_keys: Vec<&K> =
            self.keys().filter(|k| !rhs.contains_key(*k)).collect();
        edited_keys.sort();
        added_keys.sort();
        removed_keys.sort();
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
        Ok(HashMapDelta(changes))
    }
}

impl<K, V> FromDelta for HashMap<K, V>
where K: Clone + Debug + PartialEq + Ord + Hash + FromDelta
    + for<'de> Deserialize<'de>
    + Serialize,
      V: Clone + Debug + PartialEq + FromDelta
    + for<'de> Deserialize<'de>
    + Serialize,
{
    fn from_delta(delta: Self::Delta) -> DeltaResult<Self> {
        let mut map: Self = Self::new();
        for (index, element) in delta.0.into_iter().enumerate() {
            match element {
                EntryDelta::Add { key, value } =>
                    map.insert(key, <V>::from_delta(value)?),
                _ => return Err(DeltaError::IllegalDelta { index })?,
            };
        }
        Ok(map)
    }
}

impl<K, V> IntoDelta for HashMap<K, V>
where K: Clone + Debug + PartialEq + Ord + Hash + IntoDelta
    + for<'de> Deserialize<'de>
    + Serialize,
      V: Clone + Debug + PartialEq + IntoDelta
    + for<'de> Deserialize<'de>
    + Serialize,
{
    fn into_delta(self) -> DeltaResult<Self::Delta> {
        let mut entries: Vec<(K, V)> = self.into_iter().collect();
        entries.sort_by(|a, b| a.0.cmp(&b.0));
        let mut changes: Vec<EntryDelta<K, V>> = vec![];
        for (key, val) in entries {
            changes.push(EntryDelta::Add { key, value: val.into_delta()? });
        }
        Ok(HashMapDelta(changes))
    }
}


#[derive(Clone, PartialEq)]
#[derive(serde_derive::Deserialize, serde_derive::Serialize)]
pub struct HashMapDelta<K: Core, V: Core>(#[doc(hidden)] pub Vec<EntryDelta<K, V>>);

impl<K: Core, V: Core> HashMapDelta<K, V>
{
    pub fn iter<'d>(&'d self) -> Box<dyn Iterator<Item = &EntryDelta<K, V>> + 'd> {
        Box::new(self.0.iter())
    }

    pub fn into_iter<'d>(self) -> Box<dyn Iterator<Item = EntryDelta<K, V>> + 'd>
    where Self: 'd {
        Box::new(self.0.into_iter())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl<K, V> std::fmt::Debug for HashMapDelta<K, V>
where K: std::fmt::Debug + Core,
      V: std::fmt::Debug + Core
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "HashMapDelta ")?;
        f.debug_list().entries(self.0.iter()).finish()
    }
}


#[derive(Clone, PartialEq)]
#[derive(serde_derive::Deserialize, serde_derive::Serialize)]
pub enum EntryDelta<K, V: Core> {
    /// Edit a `value` of a given `key`
    Edit { key: K, value: <V as Core>::Delta },
    /// Add a given `key` and `value` entry.
    Add { key: K, value: <V as Core>::Delta },
    /// Remove the entry with a given `key` from the map.
    Remove { key: K },
}

impl<K, V> std::fmt::Debug for EntryDelta<K, V>
where K: std::fmt::Debug,
      V: std::fmt::Debug + Core
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match &self {
            Self::Edit { key, value } => f.debug_struct("Edit")
                .field("key", key)
                .field("value", value)
                .finish(),
            Self::Add { key, value } => f.debug_struct("Add")
                .field("key", key)
                .field("value", value)
                .finish(),
            Self::Remove { key } => f.debug_struct("Remove")
                .field("key", key)
                .finish(),
        }
    }
}



#[allow(non_snake_case)]
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    macro_rules! map {
        ($($key:expr => $val:expr),* $(,)?) => {{
            let mut map = HashMap::new();
            $( map.insert($key, $val); )*
                map
        }}
    }

    #[test]
    fn HashMap__delta__same_values() -> DeltaResult<()> {
        let map0: HashMap<String, usize> = map! {
            "bar".into()     => 300usize,
            "foo".into()     =>  42usize,
            "floozie".into() =>  0usize,
            "quux".into()    => 16000usize,
        };
        let map1: HashMap<String, usize> = map! {
            "bar".into()     => 300usize,
            "foo".into()     =>  42usize,
            "floozie".into() =>  0usize,
            "quux".into()    => 16000usize,
        };
        let delta = map0.delta(&map1)?;
        let expected = HashMapDelta(vec![]);
        assert_eq!(delta, expected);
        let map2 = map0.apply(delta)?;
        assert_eq!(map0, map2);
        assert_eq!(map1, map2);

        let delta = map1.delta(&map0)?;
        assert_eq!(delta, HashMapDelta(vec![]));
        let map3 = map1.apply(delta)?;
        assert_eq!(map0, map3);
        assert_eq!(map1, map3);

        Ok(())
    }

    #[test]
    fn HashMap__delta__different_values() -> DeltaResult<()> {
        let map0: HashMap<String, usize> = map! {
            "bar".into()     => 300usize,
            "foo".into()     =>  42usize,
            "floozie".into() =>  0usize,
            "quux".into()    => 16000usize,
        };
        let map1: HashMap<String, usize> = map! {
            "bar".into()  =>   350usize,
            "baz".into()  =>  9000usize,
            "foo".into()  =>    42usize,
            "quux".into() => 16000usize,
        };
        let delta0 = map0.delta(&map1)?;
        let expected = HashMapDelta(vec![
            EntryDelta::Edit { key: "bar".into(),  value:   350usize.into_delta()? },
            EntryDelta::Add  { key: "baz".into(),  value:  9000usize.into_delta()? },
            EntryDelta::Remove { key: "floozie".into() },
        ]);
        assert_eq!(delta0, expected);
        let map2 = map0.apply(delta0)?;
        assert_eq!(map1, map2);

        let delta1 = map1.delta(&map0)?;
        assert_eq!(delta1, HashMapDelta(vec![
            EntryDelta::Edit { key: "bar".into(),     value: 300usize.into_delta()? },
            EntryDelta::Add  { key: "floozie".into(), value:   0usize.into_delta()? },
            EntryDelta::Remove { key: "baz".into() },
        ]));
        let map3 = map1.apply(delta1)?;
        assert_eq!(map0, map3);

        Ok(())
    }

    #[test]
    fn HashMap__apply__same_values() -> DeltaResult<()> {
        let map0: HashMap<String, usize> = map! {
            "bar".into()     => 300usize,
            "foo".into()     =>  42usize,
            "floozie".into() =>  0usize,
            "quux".into()    => 16000usize,
        };
        let map1: HashMap<String, usize> = map! {
            "bar".into()     => 300usize,
            "foo".into()     =>  42usize,
            "floozie".into() =>  0usize,
            "quux".into()    => 16000usize,
        };
        let delta = map0.delta(&map1)?;
        assert_eq!(delta, HashMapDelta(vec![]));
        let map2 = map0.apply(delta)?;
        assert_eq!(map1, map2);
        Ok(())
    }

    #[test]
    fn HashMap__apply__different_values() -> DeltaResult<()> {
        let map0: HashMap<String, usize> = map! {
            "bar".into()     => 300usize,
            "foo".into()     =>  42usize,
            "floozie".into() =>  0usize,
            "quux".into()    => 16000usize,
        };
        let map1: HashMap<String, usize> = map! {
            "bar".into()  =>   350usize,
            "baz".into()  =>  9000usize,
            "foo".into()  =>    42usize,
            "quux".into() => 16000usize,
        };
        let delta = map0.delta(&map1)?;
        assert_eq!(delta, HashMapDelta(vec![
            EntryDelta::Edit { key: "bar".into(),  value:   350usize.into_delta()? },
            EntryDelta::Add  { key: "baz".into(),  value:  9000usize.into_delta()? },
            EntryDelta::Remove { key: "floozie".into() },
        ]));
        let map2 = map0.apply(delta)?;
        assert_eq!(map1, map2);
        Ok(())
    }

    #[test]
    fn HashMap__roundtrip__composite_values() -> DeltaResult<()> {
        let map0: HashMap<String, Vec<usize>> = map! { "a".into() => vec![1] };
        let map1: HashMap<String, Vec<usize>> = map! { "a".into() => vec![1, 2] };
        let delta = map0.delta(&map1)?;
        assert_eq!(map0.apply(delta)?, map1); // was: {"a": [2]} — silent data loss
        let map2: HashMap<String, Vec<usize>> = map! { "a".into() => vec![1, 2, 3] };
        let map3: HashMap<String, Vec<usize>> = map! { "a".into() => vec![1, 2, 4] };
        assert_eq!(map2.apply(map2.delta(&map3)?)?, map3); // was: Err(IllegalDelta)
        Ok(())
    }
}
