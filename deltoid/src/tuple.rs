//!

use crate::{Deltoid, DeltaResult};


impl<T0> Deltoid for (T0,)
where T0: Deltoid + Clone + PartialEq {
    type Delta = (
        Option<<T0 as Deltoid>::Delta>,
    );

    fn apply_delta(&self, delta: &Self::Delta) -> DeltaResult<Self> {
        let field0: T0 = self.0.apply_delta(
            delta.0.as_ref().ok_or_else(|| ExpectedValue!(
                "Option<<T0 as Deltoid>::Delta>"
            ))?
        )?;
        Ok((field0,))
    }

    fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
        let delta0: <T0 as Deltoid>::Delta = Deltoid::delta(&self.0, &rhs.0)?;
        Ok((
            if self.0 == rhs.0 { None } else { Some(delta0) },
        ))
    }
}

impl<T0, T1> Deltoid for (T0, T1)
where T0: Deltoid + Clone + PartialEq,
      T1: Deltoid + Clone + PartialEq {
    type Delta = (
        Option<<T0 as Deltoid>::Delta>,
        Option<<T1 as Deltoid>::Delta>
    );

    fn apply_delta(&self, delta: &Self::Delta) -> DeltaResult<Self> {
        let field0: T0 = self.0.apply_delta(
            delta.0.as_ref().ok_or_else(|| ExpectedValue!(
                "Option<<T0 as Deltoid>::Delta>"
            ))?
        )?;
        let field1: T1 = self.1.apply_delta(
            delta.1.as_ref().ok_or_else(|| ExpectedValue!(
                "Option<<T1 as Deltoid>::Delta>"
            ))?
        )?;
        Ok((field0, field1))
    }

    fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
        let delta0: <T0 as Deltoid>::Delta = Deltoid::delta(&self.0, &rhs.0)?;
        let delta1: <T1 as Deltoid>::Delta = Deltoid::delta(&self.1, &rhs.1)?;
        Ok((
            if self.0 == rhs.0 { None } else { Some(delta0) },
            if self.1 == rhs.1 { None } else { Some(delta1) },
        ))
    }
}

impl<T0, T1, T2> Deltoid for (T0, T1, T2)
where T0: Deltoid + Clone + PartialEq,
      T1: Deltoid + Clone + PartialEq,
      T2: Deltoid + Clone + PartialEq, {
    type Delta = (
        Option<<T0 as Deltoid>::Delta>,
        Option<<T1 as Deltoid>::Delta>,
        Option<<T2 as Deltoid>::Delta>
    );

    fn apply_delta(&self, delta: &Self::Delta) -> DeltaResult<Self> {
        let field0: T0 = self.0.apply_delta(
            delta.0.as_ref().ok_or_else(|| ExpectedValue!(
                "Option<<T0 as Deltoid>::Delta>"
            ))?
        )?;
        let field1: T1 = self.1.apply_delta(
            delta.1.as_ref().ok_or_else(|| ExpectedValue!(
                "Option<<T1 as Deltoid>::Delta>"
            ))?
        )?;
        let field2: T2 = self.2.apply_delta(
            delta.2.as_ref().ok_or_else(|| ExpectedValue!(
                "Option<<T2 as Deltoid>::Delta>"
            ))?
        )?;
        Ok((field0, field1, field2))
    }

    fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
        let delta0: <T0 as Deltoid>::Delta = Deltoid::delta(&self.0, &rhs.0)?;
        let delta1: <T1 as Deltoid>::Delta = Deltoid::delta(&self.1, &rhs.1)?;
        let delta2: <T2 as Deltoid>::Delta = Deltoid::delta(&self.2, &rhs.2)?;
        Ok((
            if self.0 == rhs.0 { None } else { Some(delta0) },
            if self.1 == rhs.1 { None } else { Some(delta1) },
            if self.2 == rhs.2 { None } else { Some(delta2) },
        ))
    }
}

impl<T0, T1, T2, T3> Deltoid for (T0, T1, T2, T3)
where T0: Deltoid + Clone + PartialEq,
      T1: Deltoid + Clone + PartialEq,
      T2: Deltoid + Clone + PartialEq,
      T3: Deltoid + Clone + PartialEq, {
    type Delta = (
        Option<<T0 as Deltoid>::Delta>,
        Option<<T1 as Deltoid>::Delta>,
        Option<<T2 as Deltoid>::Delta>,
        Option<<T3 as Deltoid>::Delta>
    );

    fn apply_delta(&self, delta: &Self::Delta) -> DeltaResult<Self> {
        let field0: T0 = self.0.apply_delta(
            delta.0.as_ref().ok_or_else(|| ExpectedValue!(
                "Option<<T0 as Deltoid>::Delta>"
            ))?
        )?;
        let field1: T1 = self.1.apply_delta(
            delta.1.as_ref().ok_or_else(|| ExpectedValue!(
                "Option<<T1 as Deltoid>::Delta>"
            ))?
        )?;
        let field2: T2 = self.2.apply_delta(
            delta.2.as_ref().ok_or_else(|| ExpectedValue!(
                "Option<<T2 as Deltoid>::Delta>"
            ))?
        )?;
        let field3: T3 = self.3.apply_delta(
            delta.3.as_ref().ok_or_else(|| ExpectedValue!(
                "Option<<T3 as Deltoid>::Delta>"
            ))?
        )?;
        Ok((field0, field1, field2, field3))
    }

    fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
        let delta0: <T0 as Deltoid>::Delta = Deltoid::delta(&self.0, &rhs.0)?;
        let delta1: <T1 as Deltoid>::Delta = Deltoid::delta(&self.1, &rhs.1)?;
        let delta2: <T2 as Deltoid>::Delta = Deltoid::delta(&self.2, &rhs.2)?;
        let delta3: <T3 as Deltoid>::Delta = Deltoid::delta(&self.3, &rhs.3)?;
        Ok((
            if self.0 == rhs.0 { None } else { Some(delta0) },
            if self.1 == rhs.1 { None } else { Some(delta1) },
            if self.2 == rhs.2 { None } else { Some(delta2) },
            if self.3 == rhs.3 { None } else { Some(delta3) },
        ))
    }
}
