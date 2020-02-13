//!

mod error;

pub use crate::error::{DeltaError, DeltaResult};

pub trait Delta where Self: Sized {
    type Delta;

    /// Calculate a new instance of `Self` based on `self` and
    /// `delta` i.e. calculate `self --[delta]--> other`.
    ///                                           ^^^^^
    fn apply_delta(&self, delta: &Self::Delta) -> DeltaResult<Self>;

    /// Calculate `self --[delta]--> other`.
    ///                    ^^^^^
    fn delta(&self, other: &Self) -> DeltaResult<Self::Delta>;

    /// Calculate `self <--[delta]-- other`.
    fn inverse_delta(&self, other: &Self) -> DeltaResult<Self::Delta> {
        other.delta(self)
    }
}

macro_rules! impl_delta_trait_for_primitive_types {
    ( $($type:ty),* $(,)? ) => {
        $(
            impl Delta for $type {
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
    // TODO: - array (i.e. [T, N])
    //       - slice (i.e. &[T]) (can a Delta even be applied to
    //                          a value of this non-owned type?)
    //       - &str  (can a Delta even be applied to
    //              a value of this non-owned type?)
}

impl Delta for String {
    type Delta = Self;

    fn apply_delta(&self, delta: &Self::Delta) -> DeltaResult<Self> {
        Ok(delta.clone()) // TODO: improve space efficiency
    }

    fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
        Ok(rhs.clone()) // TODO: improve space efficiency
    }
}

impl<T> Delta for Vec<T>
where T: Clone {
    type Delta = Self;

    fn apply_delta(&self, delta: &Self::Delta) -> DeltaResult<Self> {
        Ok((*delta).clone()) // TODO: improve space efficiency
    }

    fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
        Ok(rhs.clone()) // TODO: improve space efficiency
    }
}


impl<T0> Delta for (T0,)
where T0: Delta + Clone + PartialEq {
    type Delta = (<T0 as Delta>::Delta,);

    fn apply_delta(&self, delta: &Self::Delta) -> DeltaResult<Self> {
        let field0: T0 = self.0.apply_delta(&delta.0)?;
        Ok((field0,))
    }

    fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
        let delta0: <T0 as Delta>::Delta = Delta::delta(&self.0, &rhs.0)?;
        Ok((delta0,))
    }
}

impl<T0, T1> Delta for (T0, T1)
where T0: Delta + Clone + PartialEq,
      T1: Delta + Clone + PartialEq {
    type Delta = (<T0 as Delta>::Delta, <T1 as Delta>::Delta);

    fn apply_delta(&self, delta: &Self::Delta) -> DeltaResult<Self> {
        let field0: T0 = self.0.apply_delta(&delta.0)?;
        let field1: T1 = self.1.apply_delta(&delta.1)?;
        Ok((field0, field1))
    }

    fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
        let delta0: <T0 as Delta>::Delta = Delta::delta(&self.0, &rhs.0)?;
        let delta1: <T1 as Delta>::Delta = Delta::delta(&self.1, &rhs.1)?;
        Ok((delta0, delta1))
    }
}

impl<T0, T1, T2> Delta for (T0, T1, T2)
where T0: Delta + Clone + PartialEq,
      T1: Delta + Clone + PartialEq,
      T2: Delta + Clone + PartialEq, {
    type Delta = (
        <T0 as Delta>::Delta,
        <T1 as Delta>::Delta,
        <T2 as Delta>::Delta,
    );

    fn apply_delta(&self, delta: &Self::Delta) -> DeltaResult<Self> {
        let field0: T0 = self.0.apply_delta(&delta.0)?;
        let field1: T1 = self.1.apply_delta(&delta.1)?;
        let field2: T2 = self.2.apply_delta(&delta.2)?;
        Ok((field0, field1, field2))
    }

    fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
        let delta0: <T0 as Delta>::Delta = Delta::delta(&self.0, &rhs.0)?;
        let delta1: <T1 as Delta>::Delta = Delta::delta(&self.1, &rhs.1)?;
        let delta2: <T2 as Delta>::Delta = Delta::delta(&self.2, &rhs.2)?;
        Ok((delta0, delta1, delta2))
    }
}

impl<T0, T1, T2, T3> Delta for (T0, T1, T2, T3)
where T0: Delta + Clone + PartialEq,
      T1: Delta + Clone + PartialEq,
      T2: Delta + Clone + PartialEq,
      T3: Delta + Clone + PartialEq, {
    type Delta = (
        <T0 as Delta>::Delta,
        <T1 as Delta>::Delta,
        <T2 as Delta>::Delta,
        <T3 as Delta>::Delta,
    );

    fn apply_delta(&self, delta: &Self::Delta) -> DeltaResult<Self> {
        let field0: T0 = self.0.apply_delta(&delta.0)?;
        let field1: T1 = self.1.apply_delta(&delta.1)?;
        let field2: T2 = self.2.apply_delta(&delta.2)?;
        let field3: T3 = self.3.apply_delta(&delta.3)?;
        Ok((field0, field1, field2, field3))
    }

    fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
        let delta0: <T0 as Delta>::Delta = Delta::delta(&self.0, &rhs.0)?;
        let delta1: <T1 as Delta>::Delta = Delta::delta(&self.1, &rhs.1)?;
        let delta2: <T2 as Delta>::Delta = Delta::delta(&self.2, &rhs.2)?;
        let delta3: <T3 as Delta>::Delta = Delta::delta(&self.3, &rhs.3)?;
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
