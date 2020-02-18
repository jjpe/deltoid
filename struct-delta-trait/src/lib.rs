//!

pub mod error;

pub use crate::error::{DeltaError, DeltaResult};
use std::borrow::{Borrow, Cow, ToOwned};
use std::convert::{TryFrom, TryInto};
use std::marker::PhantomData;


/// Definitions for delta operations.
pub trait DeltaOps: Sized + PartialEq {
    type Delta: PartialEq + Clone + std::fmt::Debug
        + TryFrom<Self, Error = DeltaError>
        + TryInto<Self, Error = DeltaError>;

    /// Calculate a new instance of `Self` based on `self` and
    /// `delta` i.e. calculate `self --[delta]--> other`.
    ///                                           ^^^^^
    fn apply_delta(&self, delta: &Self::Delta) -> DeltaResult<Self>;

    /// Calculate `self --[delta]--> other`.
    ///                    ^^^^^
    fn delta(&self, other: &Self) -> DeltaResult<Self::Delta>;

    /// Calculate `other --[delta]--> self`.
    ///                     ^^^^^
    fn inverse_delta(&self, other: &Self) -> DeltaResult<Self::Delta> {
        other.delta(self)
    }
}


#[derive(Clone, Debug, PartialEq, Hash)]
pub enum Delta<T: DeltaOps + std::fmt::Debug> {
    /// Edit a value
    ScalarEdit(T),
    /// Edit a value at a given `index`.
    IndexedEdit {
        /// The location of the edit
        index: usize,
        /// The new value of the item
        item: T,
    },
    /// Remove `count` elements from the end of the Vec.
    Remove { count: usize },
    /// Add a value.
    Add(T),
}



macro_rules! impl_delta_trait_for_primitive_types {
    ( $($type:ty),* $(,)? ) => {
        $(
            impl DeltaOps for $type {
                type Delta = Delta<Self>;

                fn apply_delta(&self, delta: &Self::Delta) -> DeltaResult<Self> {
                    match delta {
                        Delta::ScalarEdit(val) => Ok(*val),
                        _ => bug_detected!(),
                    }
                }

                fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
                    Ok(Delta::ScalarEdit(*rhs))
                }
            }

            impl TryFrom<$type> for Delta<$type>
            where $type: Clone + PartialEq + DeltaOps + std::fmt::Debug {
                type Error = DeltaError;
                fn try_from(thing: $type) -> Result<Self, Self::Error> {
                    Ok(Delta::ScalarEdit(thing))
                }
            }

            impl TryFrom<Delta<$type>> for $type
            where $type: Clone + PartialEq + DeltaOps + std::fmt::Debug {
                type Error = DeltaError;
                fn try_from(delta: Delta<$type>) -> Result<Self, Self::Error> {
                    match delta {
                        Delta::ScalarEdit(item) => Ok(item),
                        _ => bug_detected!()
                    }
                }
            }

        )*
    };
}

impl_delta_trait_for_primitive_types! {
    i8, i16, i32, i64, i128, isize,
    u8, u16, u32, u64, u128, usize,
    f32, f64, bool, char, (),
    // TODO:
    // Can a delta be applied to a value of:
    //   + an array type i.e. [T, N]?
    //   + a slice type  i.e. &[T]  and  &str?
    //   + a shared-ownership type e.g. Rc and Arc?
}


impl DeltaOps for String {
    type Delta = Delta<Self>;

    fn apply_delta(&self, delta: &Self::Delta) -> DeltaResult<Self> {
        match delta {  // TODO: Improve space efficiency
            Delta::ScalarEdit(item) => Ok(item.clone()),
            _ => bug_detected!()
        }
    }

    fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
        Delta::try_from(rhs.clone())
    }
}

impl TryFrom<String> for Delta<String>
where String: Clone + PartialEq + DeltaOps + std::fmt::Debug {
    type Error = DeltaError;
    fn try_from(thing: String) -> Result<Self, Self::Error> {
        Ok(Delta::ScalarEdit(thing)) // TODO: improve space efficiency
    }
}

impl TryFrom<&str> for Delta<String>
where String: Clone + PartialEq + DeltaOps + std::fmt::Debug {
    type Error = DeltaError;
    fn try_from(thing: &str) -> Result<Self, Self::Error> {
        thing.to_string().try_into()
    }
}

impl TryFrom<Delta<String>> for String
where String: Clone + PartialEq + DeltaOps + std::fmt::Debug {
    type Error = DeltaError;
    fn try_from(delta: Delta<String>) -> Result<Self, Self::Error> {
        match delta {  // TODO: Improve space efficiency
            Delta::ScalarEdit(item) => Ok(item),
            _ => bug_detected!()
        }
    }
}




impl<T> DeltaOps for Vec<T>
where T: Clone + PartialEq + DeltaOps + std::fmt::Debug {
    // TODO This impl is actually more suited to a `Stack`-like type in terms
    // of efficiency. However, in terms of soundness it should work fine.

    type Delta = VecDelta<T>;

    fn apply_delta(&self, delta: &Self::Delta) -> DeltaResult<Self> {
        let mut new: Self = self.clone();
        for change in delta.iter() { match change {
            Delta::ScalarEdit(_) => return bug_detected!()?,
            Delta::IndexedEdit { index, item } => {
                ensure_lt![*index, self.len()]?;
                new[*index] = item.clone();
                // TODO: Use deltas on the items themselves, as well
            },
            Delta::Remove { count } =>  for _ in 0 .. *count {
                new.pop().ok_or(DeltaError::ExpectedValue)?;
            },
            Delta::Add(value) =>  new.push(value.clone()),
        }}
        Ok(new)
    }

    fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
        let (lhs_len, rhs_len) = (self.len(), rhs.len());
        let max_len = usize::max(lhs_len, rhs_len);
        let mut changes: Vec<Delta<T>> = vec![];
        for index in 0 .. max_len { match (self.get(index), rhs.get(index)) {
            (None, Some(rhs)) => changes.push(Delta::Add(rhs.clone())),
            (Some(_), None) => match changes.last_mut() {
                Some(Delta::Remove { ref mut count }) =>  *count += 1,
                _ =>  changes.push(Delta::Remove { count: 1 }),
            },
            (Some(lhs), Some(rhs)) if lhs == rhs => {/* only record changes */},
            (Some(_), Some(rhs)) => {
                // TODO: Use deltas on the items themselves, as well
                changes.push(Delta::IndexedEdit { index, item: rhs.clone() });
            },
            _ => return bug_detected!(),
        }}
        Ok(VecDelta(changes))
    }
}

impl<T> TryFrom<VecDelta<T>> for Vec<T>
where T: Clone + PartialEq + DeltaOps + std::fmt::Debug {
    type Error = DeltaError;
    fn try_from(deltas: VecDelta<T>) -> Result<Self, Self::Error> {
        let mut vec: Vec<T> = vec![];
        let mut index = 0;
        for delta in deltas.into_iter() {
            match delta {
                Delta::Add(item) => vec.push(item),
                _ => return Err(DeltaError::IllegalDelta { index })?,
            }
            index += 1;
        }
        Ok(vec)
    }
}


#[derive(Clone, Debug, PartialEq)]
pub struct VecDelta<T>(Vec<Delta<T>>)
where T: Clone + PartialEq + DeltaOps + std::fmt::Debug;

impl<T> VecDelta<T>
where T: Clone + PartialEq + DeltaOps + std::fmt::Debug {
    pub fn iter(&self) -> impl Iterator<Item = &Delta<T>> {
        self.0.iter()
    }

    pub fn into_iter(self) -> impl IntoIterator<Item = Delta<T>> {
        self.0.into_iter()
    }
}

impl<T> TryFrom<Vec<T>> for VecDelta<T>
where T: Clone + PartialEq + DeltaOps + std::fmt::Debug {
    type Error = DeltaError;
    fn try_from(thing: Vec<T>) -> Result<Self, Self::Error> {
        Ok(VecDelta(thing.into_iter().map(Delta::Add).collect()))
    }
}





// impl<T0> DeltaOps for (T0,)
// where T0: DeltaOps + Clone + PartialEq {
//     type Delta = (
//         <T0 as DeltaOps>::Delta,
//     );

//     fn apply_delta(&self, delta: &Self::Delta) -> DeltaResult<Self> {
//         let field0: T0 = self.0.apply_delta(&delta.0)?;
//         Ok((field0,))
//     }

//     fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
//         let delta0: <T0 as DeltaOps>::Delta = DeltaOps::delta(&self.0, &rhs.0)?;
//         Ok((delta0,))
//     }
// }

// impl<T0, T1> DeltaOps for (T0, T1)
// where T0: DeltaOps + Clone + PartialEq,
//       T1: DeltaOps + Clone + PartialEq {
//     type Delta = (
//         <T0 as DeltaOps>::Delta,
//         <T1 as DeltaOps>::Delta
//     );

//     fn apply_delta(&self, delta: &Self::Delta) -> DeltaResult<Self> {
//         let field0: T0 = self.0.apply_delta(&delta.0)?;
//         let field1: T1 = self.1.apply_delta(&delta.1)?;
//         Ok((field0, field1))
//     }

//     fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
//         let delta0: <T0 as DeltaOps>::Delta = DeltaOps::delta(&self.0, &rhs.0)?;
//         let delta1: <T1 as DeltaOps>::Delta = DeltaOps::delta(&self.1, &rhs.1)?;
//         Ok((delta0, delta1))
//     }
// }

// impl<T0, T1, T2> DeltaOps for (T0, T1, T2)
// where T0: DeltaOps + Clone + PartialEq,
//       T1: DeltaOps + Clone + PartialEq,
//       T2: DeltaOps + Clone + PartialEq, {
//     type Delta = (
//         <T0 as DeltaOps>::Delta,
//         <T1 as DeltaOps>::Delta,
//         <T2 as DeltaOps>::Delta,
//     );

//     fn apply_delta(&self, delta: &Self::Delta) -> DeltaResult<Self> {
//         let field0: T0 = self.0.apply_delta(&delta.0)?;
//         let field1: T1 = self.1.apply_delta(&delta.1)?;
//         let field2: T2 = self.2.apply_delta(&delta.2)?;
//         Ok((field0, field1, field2))
//     }

//     fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
//         let delta0: <T0 as DeltaOps>::Delta = DeltaOps::delta(&self.0, &rhs.0)?;
//         let delta1: <T1 as DeltaOps>::Delta = DeltaOps::delta(&self.1, &rhs.1)?;
//         let delta2: <T2 as DeltaOps>::Delta = DeltaOps::delta(&self.2, &rhs.2)?;
//         Ok((delta0, delta1, delta2))
//     }
// }

// impl<T0, T1, T2, T3> DeltaOps for (T0, T1, T2, T3)
// where T0: DeltaOps + Clone + PartialEq,
//       T1: DeltaOps + Clone + PartialEq,
//       T2: DeltaOps + Clone + PartialEq,
//       T3: DeltaOps + Clone + PartialEq, {
//     type Delta = (
//         <T0 as DeltaOps>::Delta,
//         <T1 as DeltaOps>::Delta,
//         <T2 as DeltaOps>::Delta,
//         <T3 as DeltaOps>::Delta,
//     );

//     fn apply_delta(&self, delta: &Self::Delta) -> DeltaResult<Self> {
//         let field0: T0 = self.0.apply_delta(&delta.0)?;
//         let field1: T1 = self.1.apply_delta(&delta.1)?;
//         let field2: T2 = self.2.apply_delta(&delta.2)?;
//         let field3: T3 = self.3.apply_delta(&delta.3)?;
//         Ok((field0, field1, field2, field3))
//     }

//     fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
//         let delta0: <T0 as DeltaOps>::Delta = DeltaOps::delta(&self.0, &rhs.0)?;
//         let delta1: <T1 as DeltaOps>::Delta = DeltaOps::delta(&self.1, &rhs.1)?;
//         let delta2: <T2 as DeltaOps>::Delta = DeltaOps::delta(&self.2, &rhs.2)?;
//         let delta3: <T3 as DeltaOps>::Delta = DeltaOps::delta(&self.3, &rhs.3)?;
//         Ok((delta0, delta1, delta2, delta3))
//     }
// }



impl<'a, B> DeltaOps for Cow<'a, B>
where B: ToOwned + PartialEq + DeltaOps + Clone + std::fmt::Debug {
    type Delta = CowDelta<'a, B>;
    fn apply_delta(&self, delta: &Self::Delta) -> DeltaResult<Self> {
        let lhs: &B = self.borrow();
        if let Some(delta) = delta.inner.as_ref() {
            let rhs: &<B as DeltaOps>::Delta = delta;
            lhs.apply_delta(rhs)
                .map(|new| new.to_owned())
                .map(Cow::Owned)
        } else {
            Ok(self.clone())
        }
    }

    fn delta(&self, other: &Self) -> DeltaResult<Self::Delta> {
        let (lhs, rhs): (&B, &B) = (self.borrow(), other.borrow());
        Ok(CowDelta {
            inner: Some(lhs.delta(rhs)?),
            _phantom: PhantomData,
        })
    }
}

impl<'a, B> TryFrom<CowDelta<'a, B>> for Cow<'a, B>
where B: Clone + std::fmt::Debug + DeltaOps {
    type Error = DeltaError;
    fn try_from(delta: CowDelta<'a, B>) -> Result<Self, Self::Error> {
        delta.inner
            .ok_or(DeltaError::ExpectedValue)?
            .try_into()
            .map(Cow::Owned)
    }
}



#[derive(Clone, Debug, PartialEq)]
pub struct CowDelta<'a, B: DeltaOps + Clone> {
    inner: Option<<B as DeltaOps>::Delta>,
    _phantom: PhantomData<&'a B>
}

impl<'a, B> TryFrom<Cow<'a, B>> for CowDelta<'a, B>
where B: Clone + std::fmt::Debug + DeltaOps {
    type Error = DeltaError;
    fn try_from(thing: Cow<'a, B>) -> Result<Self, Self::Error> {
        let borrowed: &B = thing.borrow();
        Ok(CowDelta {
            inner: Some(borrowed.to_owned().try_into()?),
            _phantom: PhantomData
        })
    }
}





#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculate_delta_for_vec() -> DeltaResult<()> {
        let v0 = vec![1, 3, 10, 30];
        let v1 = vec![1, 3, 10, 49, 30, 500];
        let delta0 = v0.delta(&v1)?;
        println!("delta0: {:#?}", delta0);
        assert_eq!(delta0, VecDelta(vec![
            Delta::IndexedEdit { index: 3, item:  49, },
            Delta::Add(30),
            Delta::Add(500),
        ]));
        let v2 = v0.apply_delta(&delta0)?;
        println!("v2: {:#?}", v2);
        assert_eq!(v1, v2);

        let delta1 = v1.delta(&v0)?;
        println!("delta1: {:#?}", delta1);
        assert_eq!(delta1, VecDelta(vec![
            Delta::IndexedEdit { index: 3, item: 30, },
            Delta::Remove  { count: 2, },
        ]));
        let v3 = v1.apply_delta(&delta1)?;
        println!("v3: {:#?}", v3);
        assert_eq!(v0, v3);

        let v0 = vec![1, 3, 10, 49, 30, 500];
        let v1 = vec![1, 3, 10, 30, 500, 49];
        let delta0 = v0.delta(&v1)?;
        println!("delta0: {:#?}", delta0);
        assert_eq!(delta0, VecDelta(vec![
            Delta::IndexedEdit { index: 3, item:  30, },
            Delta::IndexedEdit { index: 4, item: 500, },
            Delta::IndexedEdit { index: 5, item:  49, },
        ]));
        let v2 = v0.apply_delta(&delta0)?;
        println!("v2: {:#?}", v2);
        assert_eq!(v1, v2);

        Ok(())
    }

    #[test]
    fn apply_delta_to_vec() -> DeltaResult<()> {
        let v0 = vec![1,3,10,30, 30];
        let delta = VecDelta(vec![
            Delta::IndexedEdit { index: 3, item:  49, },
            Delta::Add(500),
        ]);
        let v1 = v0.apply_delta(&delta)?;
        let expected = vec![1,3,10,49, 30, 500];
        assert_eq!(expected, v1);
        Ok(())
    }
}
