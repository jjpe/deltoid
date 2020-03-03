//! A newtype wrapping [`HashMap`] that provides extra functionality in
//! the form of delta support, de/serialization, partial equality and more.
//!
//! [`HashMap`]: https://doc.rust-lang.org/std/collections/struct.HashMap.html
use crate::{DeltaError, DeltaOps, DeltaResult, FromDelta, IntoDelta};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt::{Debug};


impl<K, V> DeltaOps for std::collections::HashMap<K, V>
where K: DeltaOps + PartialEq + Eq + Clone + Debug + Ord + std::hash::Hash
    + for<'de> Deserialize<'de>
    + Serialize
    + FromDelta
    + IntoDelta,
      V: DeltaOps
    + for<'de> Deserialize<'de>
    + Serialize
    + FromDelta
    + IntoDelta
{
    type Delta = HashMapDelta<K, V>;

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
        let lkeys: HashSet<&K> = self.keys().collect();
        let rkeys: HashSet<&K> =  rhs.keys().collect();
        let edited_keys = lkeys.intersection(&rkeys)
            .filter(|key| self[key] != rhs[key]);
        let removed_keys = lkeys.difference(&rkeys);
        let added_keys = rkeys.difference(&lkeys);
        let mut changes: Vec<EntryDelta<K, V>> = vec![];
        for key in edited_keys {
            let (lhs_val, rhs_val): (&V, &V) = (&self[key], &rhs[key]);
            let delta: <V as DeltaOps>::Delta = lhs_val.delta(rhs_val)?;
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


#[derive(
    Clone, Debug, PartialEq, serde_derive::Deserialize, serde_derive::Serialize
)]
pub struct HashMapDelta<K, V>(
    #[doc(hidden)]
    pub Vec<EntryDelta<K, V>>,
) where V: DeltaOps + FromDelta + IntoDelta;

impl<K, V> HashMapDelta<K, V>
where K: DeltaOps + PartialEq + Clone + Debug + Ord
    + for<'de> Deserialize<'de>
    + Serialize
    + FromDelta
    + IntoDelta,
      V: DeltaOps
    + for<'de> Deserialize<'de>
    + Serialize
    + FromDelta
    + IntoDelta
{
    pub fn iter(&self) -> impl Iterator<Item = &EntryDelta<K, V>> {
        self.0.iter()
    }

    pub fn into_iter(self) -> impl Iterator<Item = EntryDelta<K, V>> {
        self.0.into_iter()
    }

    pub fn len(&self) -> usize { self.0.len() }
}


#[derive(
    Clone, Debug, PartialEq, serde_derive::Deserialize, serde_derive::Serialize
)]
pub enum EntryDelta<K, V: DeltaOps> {
    /// Edit a `value` of a given `key`
    Edit { key: K, value: <V as DeltaOps>::Delta },
    /// Add a given `key` and `value` entry.
    Add { key: K, value: <V as DeltaOps>::Delta },
    /// Remove the entry with a given `key` from the map.
    Remove { key: K },
}


impl<K, V> IntoDelta for std::collections::HashMap<K, V>
where K: DeltaOps + PartialEq + Eq + Clone + Debug + Ord + std::hash::Hash
    + for<'de> Deserialize<'de>
    + Serialize
    + FromDelta
    + IntoDelta,
      V: DeltaOps
    + for<'de> Deserialize<'de>
    + Serialize
    + FromDelta
    + IntoDelta
{
    fn into_delta(self) -> DeltaResult<<Self as DeltaOps>::Delta> {
        let mut vec: Vec<EntryDelta<K, V>> = vec![];
        for (key, val) in self {
            vec.push(EntryDelta::Add { key: key, value: val.into_delta()? });
        }
        Ok(HashMapDelta(vec))
    }
}

impl<K, V> FromDelta for std::collections::HashMap<K, V>
where K: DeltaOps + PartialEq + Eq + Clone + Debug + Ord + std::hash::Hash
    + for<'de> Deserialize<'de>
    + Serialize
    + FromDelta
    + IntoDelta,
      V: DeltaOps
    + for<'de> Deserialize<'de>
    + Serialize
    + FromDelta
    + IntoDelta
{
    fn from_delta(delta: <Self as DeltaOps>::Delta) -> DeltaResult<Self> {
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
    fn calculate_delta_for_hashmap() -> DeltaResult<()> {
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
        let expected = HashMapDelta(vec![
            EntryDelta::Edit { key: "bar".into(),  value:   350usize.into_delta()? },
            EntryDelta::Add  { key: "baz".into(),  value:  9000usize.into_delta()? },
            EntryDelta::Remove { key: "floozie".into() },
        ]);
        assert_eq!(delta0, expected, "{:#?}\n    !=\n{:#?}", delta0, expected);
        let v2 = v0.apply_delta(&delta0)?;
        println!("v2: {:#?}", v2);
        assert_eq!(v1, v2);

        let delta1 = v1.delta(&v0)?;
        println!("delta1: {:#?}", delta1);
        assert_eq!(delta1, HashMapDelta(vec![
            EntryDelta::Edit { key: "bar".into(),     value: 300usize.into_delta()? },
            EntryDelta::Add  { key: "floozie".into(), value:   0usize.into_delta()? },
            EntryDelta::Remove { key: "baz".into() },
        ]));
        let v3 = v1.apply_delta(&delta1)?;
        println!("v3: {:#?}", v3);
        assert_eq!(v0, v3);

        Ok(())
    }

    #[test]
    fn apply_delta_to_hashmap() -> DeltaResult<()> {
        let v0: HashMap<String, usize> = map! {
            "bar".into()     => 300usize,
            "foo".into()     =>  42usize,
            "floozie".into() =>  0usize,
            "quux".into()    => 16000usize,
        };
        let delta = HashMapDelta(vec![
            EntryDelta::Edit { key: "bar".into(),  value:   350usize.into_delta()? },
            EntryDelta::Add  { key: "baz".into(),  value:  9000usize.into_delta()? },
            EntryDelta::Remove { key: "floozie".into() },
        ]);
        let v1 = v0.apply_delta(&delta)?;
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




// #![allow(unused)]

// use std::borrow::Borrow;
// use std::collections::hash_map::*;
// use std::fmt::{self, Debug};
// use std::hash::{BuildHasher, Hash};
// use std::ops::Index;

// #[derive(Clone)]
// pub struct HashMap<K, V, S = RandomState>(std::collections::HashMap<K, V, S>);

// impl<K: Hash + Eq, V> HashMap<K, V, RandomState> {
//     pub fn new() -> Self { Self(std::collections::HashMap::new()) }

//     pub fn with_capacity(capacity: usize) -> Self {
//         Self(std::collections::HashMap::with_capacity(capacity))
//     }
// }

// impl<K, V, S> HashMap<K, V, S> {
//     pub fn capacity(&self) -> usize { self.0.capacity() }

//     pub fn keys(&self) -> Keys<'_, K, V> { self.0.keys() }

//     pub fn values(&self) -> Values<'_, K, V> { self.0.values() }

//     pub fn values_mut(&mut self) -> ValuesMut<'_, K, V> { self.0.values_mut() }

//     pub fn iter(&self) -> Iter<'_, K, V> { self.0.iter() }

//     pub fn iter_mut(&mut self) -> IterMut<'_, K, V> { self.0.iter_mut() }

//     pub fn len(&self) -> usize { self.0.len() }

//     pub fn is_empty(&self) -> bool { self.0.is_empty() }

//     pub fn drain(&mut self) -> Drain<'_, K, V> { self.0.drain() }

//     pub fn clear(&mut self) { self.0.clear() }
// }

// impl<K, V, S> PartialEq for HashMap<K, V, S>
// where K: Eq + Hash,
//       V: PartialEq,
//       S: BuildHasher,
// {
//     fn eq(&self, other: &Self) -> bool { self.0.eq(&other.0) }
// }

// impl<K, V, S> Eq for HashMap<K, V, S>
// where K: Eq + Hash,
//       V: Eq,
//       S: BuildHasher
// {}


// impl<K, V, S> Debug for HashMap<K, V, S>
// where K: Eq + Hash + Debug,
//       V: Debug,
//       S: BuildHasher,
// {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         self.0.fmt(f)
//     }
// }

// impl<K, V, S> Default for HashMap<K, V, S>
// where K: Eq + Hash,
//       S: BuildHasher + Default,
// {
//     #[inline]
//     fn default() -> Self { Self(std::collections::HashMap::default()) }
// }

// impl<K, Q: ?Sized, V, S> Index<&Q> for HashMap<K, V, S>
// where K: Eq + Hash + Borrow<Q>,
//       Q: Eq + Hash,
//       S: BuildHasher,
// {
//     type Output = V;

//     /// Returns a reference to the value corresponding to the supplied key.
//     ///
//     /// # Panics
//     ///
//     /// Panics if the key is not present in the `HashMap`.
//     #[inline]
//     fn index(&self, key: &Q) -> &V {
//         self.get(key).expect("no entry found for key")
//     }
// }



// impl<K, V, S> HashMap<K, V, S>
// where K: Eq + Hash,
//       S: BuildHasher,
// {
//     pub fn with_hasher(hash_builder: S) -> Self {
//         Self(std::collections::HashMap::with_hasher(hash_builder))
//     }

//     pub fn with_capacity_and_hasher(
//         capacity: usize,
//         hash_builder: S
//     ) -> HashMap<K, V, S> {
//         Self(std::collections::HashMap::with_capacity_and_hasher(
//             capacity,
//             hash_builder
//         ))
//     }

//     pub fn hasher(&self) -> &S { self.0.hasher() }

//     pub fn reserve(&mut self, additional: usize) { self.0.reserve(additional) }

//     pub fn entry(&mut self, key: K) -> Entry<'_, K, V> { self.0.entry(key) }

//     pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<&V>
//     where K: Borrow<Q>,
//           Q: Hash + Eq,
//     {
//         self.0.get(key)
//     }

//     pub fn get_key_value<Q: ?Sized>(&self, key: &Q) -> Option<(&K, &V)>
//     where
//         K: Borrow<Q>,
//         Q: Hash + Eq,
//     {
//         self.0.get_key_value(key)
//     }

//     pub fn contains_key<Q: ?Sized>(&self, key: &Q) -> bool
//     where K: Borrow<Q>,
//           Q: Hash + Eq,
//     {
//         self.0.contains_key(key)
//     }

//     pub fn get_mut<Q: ?Sized>(&mut self, key: &Q) -> Option<&mut V>
//     where K: Borrow<Q>,
//           Q: Hash + Eq,
//     {
//         self.0.get_mut(key)
//     }

//     pub fn insert(&mut self, key: K, value: V) -> Option<V> {
//         self.0.insert(key, value)
//     }

//     pub fn remove<Q: ?Sized>(&mut self, key: &Q) -> Option<V>
//     where K: Borrow<Q>,
//           Q: Hash + Eq,
//     {
//         self.0.remove(key)
//     }

//     pub fn remove_entry<Q: ?Sized>(&mut self, key: &Q) -> Option<(K, V)>
//     where
//         K: Borrow<Q>,
//         Q: Hash + Eq,
//     {
//         self.0.remove_entry(key)
//     }

//     pub fn retain<F>(&mut self, f: F)
//     where F: FnMut(&K, &mut V) -> bool {
//         self.0.retain(f)
//     }
// }
