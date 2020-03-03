//!

use crate::{DeltaOps, DeltaResult};


impl<T0> DeltaOps for (T0,)
where T0: DeltaOps + Clone + PartialEq {
    type Delta = (
        <T0 as DeltaOps>::Delta,
    );

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
    type Delta = (
        <T0 as DeltaOps>::Delta,
        <T1 as DeltaOps>::Delta
    );

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
