//! A newtype wrapping [`BtreeMap`] that provides extra functionality in
//! the form of delta support, de/serialization, partial equality and more.
//!
//! [`BtreeMap`]: https://doc.rust-lang.org/std/collections/struct.BTreeMap.html

use crate::{DeltaError, Deltoid, DeltaResult, FromDelta, IntoDelta};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt::{Debug};


impl<K, V> Deltoid for std::collections::BTreeMap<K, V>
where K: Deltoid + PartialEq + Clone + Debug + Ord
    + for<'de> Deserialize<'de>
    + Serialize
    + FromDelta
    + IntoDelta,
      V: Deltoid
    + for<'de> Deserialize<'de>
    + Serialize
    + FromDelta
    + IntoDelta
{
    type Delta = BTreeMapDelta<K, V>;

    fn apply_delta(&self, delta: &Self::Delta) -> DeltaResult<Self> {
        let mut new: Self = self.clone();
        for change in delta.iter() { match change {
            EntryDelta::Edit { key, value } => {
                let place: &mut V = &mut *new.get_mut(key)
                    .ok_or(DeltaError::ExpectedValue)?;
                *place = <V>::from_delta(value.clone())?;
            },
            EntryDelta::Add { key, value } => {
                new.insert(key.clone(), <V>::from_delta(value.clone())?);
            },
            EntryDelta::Remove { key } =>  { new.remove(key); },
        }}
        Ok(new)
    }

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
            let delta: <V as Deltoid>::Delta = lhs_val.delta(rhs_val)?;
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


#[derive(
    Clone, Debug, PartialEq, serde_derive::Deserialize, serde_derive::Serialize
)]
pub struct BTreeMapDelta<K, V>(
    #[doc(hidden)]
    pub Option<Vec<EntryDelta<K, V>>>,
) where V: Deltoid + FromDelta + IntoDelta;

impl<K, V> BTreeMapDelta<K, V>
where K: Deltoid + PartialEq + Clone + Debug + Ord
    + for<'de> Deserialize<'de>
    + Serialize
    + FromDelta
    + IntoDelta,
      V: Deltoid
    + for<'de> Deserialize<'de>
    + Serialize
    + FromDelta
    + IntoDelta
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


#[derive(
    Clone, Debug, PartialEq, serde_derive::Deserialize, serde_derive::Serialize
)]
pub enum EntryDelta<K, V: Deltoid> {
    /// Edit a `value` of a given `key`
    Edit { key: K, value: <V as Deltoid>::Delta },
    /// Add a given `key` and `value` entry.
    Add { key: K, value: <V as Deltoid>::Delta },
    /// Remove the entry with a given `key` from the map.
    Remove { key: K },
}


impl<K, V> IntoDelta for std::collections::BTreeMap<K, V>
where K: Deltoid + PartialEq + Clone + Debug + Ord
    + for<'de> Deserialize<'de>
    + Serialize
    + FromDelta
    + IntoDelta,
      V: Deltoid
    + for<'de> Deserialize<'de>
    + Serialize
    + FromDelta
    + IntoDelta
{
    fn into_delta(self) -> DeltaResult<<Self as Deltoid>::Delta> {
        let mut changes: Vec<EntryDelta<K, V>> = vec![];
        for (key, val) in self {
            changes.push(EntryDelta::Add { key: key, value: val.into_delta()? });
        }
        Ok(BTreeMapDelta(if !changes.is_empty() {
            Some(changes)
        } else {
            None
        }))
    }
}

impl<K, V> FromDelta for std::collections::BTreeMap<K, V>
where K: Deltoid + Clone + std::fmt::Debug + Ord
    + for<'de> Deserialize<'de>
    + Serialize
    + FromDelta
    + IntoDelta,
      V: Deltoid
    + for<'de> Deserialize<'de>
    + Serialize
    + FromDelta
    + IntoDelta
{
    fn from_delta(delta: <Self as Deltoid>::Delta) -> DeltaResult<Self> {
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
    fn calculate_delta_for_BTreeMap() -> DeltaResult<()> {
        let v0: BTreeMap<String, usize> = map! {
            "bar".into()     => 300usize,
            "foo".into()     =>  42usize,
            "floozie".into() =>  0usize,
            "quux".into()    => 16000usize,
        };
        let v1: BTreeMap<String, usize> = map! {
            "bar".into()  =>   350usize,
            "baz".into()  =>  9000usize,
            "foo".into()  =>    42usize,
            "quux".into() => 16000usize,
        };
        let delta0 = v0.delta(&v1)?;
        println!("delta0: {:#?}", delta0);
        let expected = BTreeMapDelta(Some(vec![
            EntryDelta::Edit { key: "bar".into(),  value:   350usize.into_delta()? },
            EntryDelta::Add  { key: "baz".into(),  value:  9000usize.into_delta()? },
            EntryDelta::Remove { key: "floozie".into() },
        ]));
        assert_eq!(delta0, expected, "{:#?}\n    !=\n{:#?}", delta0, expected);
        let v2 = v0.apply_delta(&delta0)?;
        println!("v2: {:#?}", v2);
        assert_eq!(v1, v2);

        let delta1 = v1.delta(&v0)?;
        println!("delta1: {:#?}", delta1);
        assert_eq!(delta1, BTreeMapDelta(Some(vec![
            EntryDelta::Edit { key: "bar".into(),     value: 300usize.into_delta()? },
            EntryDelta::Add  { key: "floozie".into(), value:   0usize.into_delta()? },
            EntryDelta::Remove { key: "baz".into() },
        ])));
        let v3 = v1.apply_delta(&delta1)?;
        println!("v3: {:#?}", v3);
        assert_eq!(v0, v3);

        Ok(())
    }

    #[test]
    fn apply_delta_to_BTreeMap() -> DeltaResult<()> {
        let v0: BTreeMap<String, usize> = map! {
            "bar".into()     => 300usize,
            "foo".into()     =>  42usize,
            "floozie".into() =>  0usize,
            "quux".into()    => 16000usize,
        };
        let delta = BTreeMapDelta(Some(vec![
            EntryDelta::Edit { key: "bar".into(),  value:   350usize.into_delta()? },
            EntryDelta::Add  { key: "baz".into(),  value:  9000usize.into_delta()? },
            EntryDelta::Remove { key: "floozie".into() },
        ]));
        let v1 = v0.apply_delta(&delta)?;
        let expected: BTreeMap<String, usize> = map! {
            "bar".into()  =>   350usize,
            "baz".into()  =>  9000usize,
            "foo".into()  =>    42usize,
            "quux".into() => 16000usize,
        };
        assert_eq!(expected, v1);
        Ok(())
    }

}
