//! A newtype wrapping [`HashMap`] that provides extra functionality in
//! the form of delta support, de/serialization, partial equality and more.
//!
//! [`HashMap`]: https://doc.rust-lang.org/std/collections/struct.HashMap.html

use crate::{Apply, Core, Delta, DeltaError, DeltaResult, FromDelta, IntoDelta};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;


impl<K, V> Core for HashMap<K, V>
where K: Clone + Debug + PartialEq + Ord + Hash + Core
    + for<'de> Deserialize<'de>
    + Serialize,
      V: Clone + Debug + PartialEq + Ord + Core
    + for<'de> Deserialize<'de>
    + Serialize,
{
    type Delta = HashMapDelta<K, V>;
}

impl<K, V> Apply for HashMap<K, V>
where K: Clone + Debug + PartialEq + Ord + Hash + Apply
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
                    .ok_or_else(|| ExpectedValue!("HashMapDelta<K, V>"))?;
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

impl<K, V> Delta for HashMap<K, V>
where K: Clone + Debug + PartialEq + Ord + Hash + Delta
    + for<'de> Deserialize<'de>
    + Serialize,
      V: Clone + Debug + PartialEq + Ord + Delta + IntoDelta
    + for<'de> Deserialize<'de>
    + Serialize,
{
    fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
        let lkeys: HashSet<&K> = self.keys().collect();
        let rkeys: HashSet<&K> =  rhs.keys().collect();
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
        Ok(HashMapDelta(if !changes.is_empty() {
            Some(changes)
        } else {
            None
        }))
    }
}

impl<K, V> FromDelta for HashMap<K, V>
where K: Clone + Debug + PartialEq + Ord + Hash + FromDelta
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

impl<K, V> IntoDelta for HashMap<K, V>
where K: Clone + Debug + PartialEq + Ord + Hash + IntoDelta
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
        Ok(HashMapDelta(if !changes.is_empty() {
            Some(changes)
        } else {
            None
        }))
    }
}


#[derive(Clone, Debug, PartialEq)]
#[derive(serde_derive::Deserialize, serde_derive::Serialize)]
pub struct HashMapDelta<K, V: Core>(
    #[doc(hidden)]
    pub Option<Vec<EntryDelta<K, V>>>,
);

impl<K, V> HashMapDelta<K, V>
where K: Clone + Debug + PartialEq + Ord + Hash + Core
    + for<'de> Deserialize<'de>
    + Serialize,
      V: Clone + Debug + PartialEq + Ord + Core
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
    use std::collections::HashMap;

    macro_rules! map {
        ($($key:expr => $val:expr),* $(,)?) => {{
            let mut map = HashMap::new();
            $( map.insert($key, $val); )*
                map
        }}
    }

    #[test]
    fn calculate_delta_for_HashMap() -> DeltaResult<()> {
        let v0: HashMap<String, usize> = map! {
            "bar".into()     => 300usize,
            "foo".into()     =>  42usize,
            "floozie".into() =>  0usize,
            "quux".into()    => 16000usize,
        };
        let v1: HashMap<String, usize> = map! {
            "bar".into()  =>   350usize,
            "baz".into()  =>  9000usize,
            "foo".into()  =>    42usize,
            "quux".into() => 16000usize,
        };
        let delta0 = v0.delta(&v1)?;
        println!("delta0: {:#?}", delta0);
        let expected = HashMapDelta(Some(vec![
            EntryDelta::Edit { key: "bar".into(),  value:   350usize.into_delta()? },
            EntryDelta::Add  { key: "baz".into(),  value:  9000usize.into_delta()? },
            EntryDelta::Remove { key: "floozie".into() },
        ]));
        assert_eq!(delta0, expected, "{:#?}\n    !=\n{:#?}", delta0, expected);
        let v2 = v0.apply(delta0)?;
        println!("v2: {:#?}", v2);
        assert_eq!(v1, v2);

        let delta1 = v1.delta(&v0)?;
        println!("delta1: {:#?}", delta1);
        assert_eq!(delta1, HashMapDelta(Some(vec![
            EntryDelta::Edit { key: "bar".into(),     value: 300usize.into_delta()? },
            EntryDelta::Add  { key: "floozie".into(), value:   0usize.into_delta()? },
            EntryDelta::Remove { key: "baz".into() },
        ])));
        let v3 = v1.apply(delta1)?;
        println!("v3: {:#?}", v3);
        assert_eq!(v0, v3);

        Ok(())
    }

    #[test]
    fn apply_delta_to_HashMap() -> DeltaResult<()> {
        let v0: HashMap<String, usize> = map! {
            "bar".into()     => 300usize,
            "foo".into()     =>  42usize,
            "floozie".into() =>  0usize,
            "quux".into()    => 16000usize,
        };
        let delta = HashMapDelta(Some(vec![
            EntryDelta::Edit { key: "bar".into(),  value:   350usize.into_delta()? },
            EntryDelta::Add  { key: "baz".into(),  value:  9000usize.into_delta()? },
            EntryDelta::Remove { key: "floozie".into() },
        ]));
        let v1 = v0.apply(delta)?;
        let expected: HashMap<String, usize> = map! {
            "bar".into()  =>   350usize,
            "baz".into()  =>  9000usize,
            "foo".into()  =>    42usize,
            "quux".into() => 16000usize,
        };
        assert_eq!(expected, v1);
        Ok(())
    }

}
