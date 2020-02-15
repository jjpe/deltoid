//!

mod error;

pub use crate::error::{DeltaError, DeltaResult};


/// Definitions for delta operations.
pub trait DeltaOps: Sized + PartialEq {
    type Delta;

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
pub enum Change<T: DeltaOps> {
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
                type Delta = Self;

                fn apply_delta(&self, delta: &Self::Delta) -> DeltaResult<Self> {
                    Ok(*delta) // use the Copy trait
                }

                fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
                    Ok(*rhs)// use the Copy trait
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
    type Delta = Self;

    fn apply_delta(&self, delta: &Self::Delta) -> DeltaResult<Self> {
        Ok(delta.clone()) // TODO: improve space efficiency
    }

    fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
        Ok(rhs.clone()) // TODO: improve space efficiency
    }
}


impl<T> DeltaOps for Vec<T>
where T: Clone + PartialEq + DeltaOps {
    // TODO This impl is actually more suited to a `Stack`-like type in terms
    // of efficiency. However, in terms of soundness it should work fine.

    type Delta = Vec<Change<T>>;

    fn apply_delta(&self, delta: &Self::Delta) -> DeltaResult<Self> {
        let mut new: Self = self.clone();
        for change in delta.iter() { match change {
            Change::ScalarEdit(_) => return bug_detected!()?,
            Change::IndexedEdit { index, item } => {
                ensure_lt![*index, self.len()]?;
                new[*index] = item.clone();
                // TODO: Use deltas on the items themselves, as well
            },
            Change::Remove { count } =>  for _ in 0 .. *count {
                new.pop().ok_or(DeltaError::ExpectedValue)?;
            },
            Change::Add(value) =>  new.push(value.clone()),
        }}
        Ok(new)
    }

    fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
        let (lhs_len, rhs_len) = (self.len(), rhs.len());
        let max_len = usize::max(lhs_len, rhs_len);
        let mut changes: Vec<Change<T>> = vec![];
        for index in 0 .. max_len { match (self.get(index), rhs.get(index)) {
            (None, Some(rhs)) => changes.push(Change::Add(rhs.clone())),
            (Some(_), None) => match changes.last_mut() {
                Some(Change::Remove { ref mut count }) =>  *count += 1,
                _ =>  changes.push(Change::Remove { count: 1 }),
            },
            (Some(lhs), Some(rhs)) if lhs == rhs => {/* only record changes */},
            (Some(_), Some(rhs)) => {
                // TODO: Use deltas on the items themselves, as well
                changes.push(Change::IndexedEdit { index, item: rhs.clone() });
            },
            _ => return bug_detected!(),
        }}
        Ok(changes)
    }
}





impl<T0> DeltaOps for (T0,)
where T0: DeltaOps + Clone + PartialEq {
    type Delta = (<T0 as DeltaOps>::Delta,);

    fn apply_delta(&self, delta: &Self::Delta) -> DeltaResult<Self> {
        let field0: T0 = self.0.apply_delta(&delta.0)?;
        Ok((field0,))
    }

    fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
        let delta0: <T0 as DeltaOps>::Delta = DeltaOps::delta(&self.0, &rhs.0)?;
        Ok((delta0,))
    }
}

impl<T0, T1> DeltaOps for (T0, T1)
where T0: DeltaOps + Clone + PartialEq,
      T1: DeltaOps + Clone + PartialEq {
    type Delta = (<T0 as DeltaOps>::Delta, <T1 as DeltaOps>::Delta);

    fn apply_delta(&self, delta: &Self::Delta) -> DeltaResult<Self> {
        let field0: T0 = self.0.apply_delta(&delta.0)?;
        let field1: T1 = self.1.apply_delta(&delta.1)?;
        Ok((field0, field1))
    }

    fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
        let delta0: <T0 as DeltaOps>::Delta = DeltaOps::delta(&self.0, &rhs.0)?;
        let delta1: <T1 as DeltaOps>::Delta = DeltaOps::delta(&self.1, &rhs.1)?;
        Ok((delta0, delta1))
    }
}

impl<T0, T1, T2> DeltaOps for (T0, T1, T2)
where T0: DeltaOps + Clone + PartialEq,
      T1: DeltaOps + Clone + PartialEq,
      T2: DeltaOps + Clone + PartialEq, {
    type Delta = (
        <T0 as DeltaOps>::Delta,
        <T1 as DeltaOps>::Delta,
        <T2 as DeltaOps>::Delta,
    );

    fn apply_delta(&self, delta: &Self::Delta) -> DeltaResult<Self> {
        let field0: T0 = self.0.apply_delta(&delta.0)?;
        let field1: T1 = self.1.apply_delta(&delta.1)?;
        let field2: T2 = self.2.apply_delta(&delta.2)?;
        Ok((field0, field1, field2))
    }

    fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
        let delta0: <T0 as DeltaOps>::Delta = DeltaOps::delta(&self.0, &rhs.0)?;
        let delta1: <T1 as DeltaOps>::Delta = DeltaOps::delta(&self.1, &rhs.1)?;
        let delta2: <T2 as DeltaOps>::Delta = DeltaOps::delta(&self.2, &rhs.2)?;
        Ok((delta0, delta1, delta2))
    }
}

impl<T0, T1, T2, T3> DeltaOps for (T0, T1, T2, T3)
where T0: DeltaOps + Clone + PartialEq,
      T1: DeltaOps + Clone + PartialEq,
      T2: DeltaOps + Clone + PartialEq,
      T3: DeltaOps + Clone + PartialEq, {
    type Delta = (
        <T0 as DeltaOps>::Delta,
        <T1 as DeltaOps>::Delta,
        <T2 as DeltaOps>::Delta,
        <T3 as DeltaOps>::Delta,
    );

    fn apply_delta(&self, delta: &Self::Delta) -> DeltaResult<Self> {
        let field0: T0 = self.0.apply_delta(&delta.0)?;
        let field1: T1 = self.1.apply_delta(&delta.1)?;
        let field2: T2 = self.2.apply_delta(&delta.2)?;
        let field3: T3 = self.3.apply_delta(&delta.3)?;
        Ok((field0, field1, field2, field3))
    }

    fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
        let delta0: <T0 as DeltaOps>::Delta = DeltaOps::delta(&self.0, &rhs.0)?;
        let delta1: <T1 as DeltaOps>::Delta = DeltaOps::delta(&self.1, &rhs.1)?;
        let delta2: <T2 as DeltaOps>::Delta = DeltaOps::delta(&self.2, &rhs.2)?;
        let delta3: <T3 as DeltaOps>::Delta = DeltaOps::delta(&self.3, &rhs.3)?;
        Ok((delta0, delta1, delta2, delta3))
    }
}




// #[cfg(test)]
// mod tests {
//     #[test]
//     fn it_works() {
//         assert_eq!(2 + 2, 4);
//     }
// }
