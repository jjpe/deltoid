//!

use crate::{Apply, Core, Delta, DeltaResult, FromDelta, IntoDelta};


impl<T0> Core for (T0,)
where T0: Core
{
    type Delta = (
        Option<<T0 as Core>::Delta>,
    );
}

impl<T0> Apply for (T0,)
where T0: Apply,
{
    fn apply(&self, delta: Self::Delta) -> DeltaResult<Self> {
        let field0: T0 = self.0.apply(
            delta.0.ok_or_else(|| ExpectedValue!("Option<<T0 as Core>::Delta>"))?
        )?;
        Ok((field0,))
    }
}

impl<T0> Delta for (T0,)
where T0: Delta,
{
    fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
        let delta0: <T0 as Core>::Delta = Delta::delta(&self.0, &rhs.0)?;
        Ok((
            if self.0 == rhs.0 { None } else { Some(delta0) },
        ))
    }
}

impl<T0> FromDelta for (T0,)
where T0: FromDelta,
{
    fn from_delta(delta: Self::Delta) -> DeltaResult<Self> {
        Ok((
            delta.0.map(<T0>::from_delta)
                .ok_or_else(|| ExpectedValue!("Option<<T0 as Core>::Delta>"))??,
        ))
    }
}

impl<T0> IntoDelta for (T0,)
where T0: IntoDelta,
{
    fn into_delta(self) -> DeltaResult<Self::Delta> {
        Ok((
            Some(self.0.into_delta()?),
        ))
    }
}




impl<T0, T1> Core for (T0, T1)
where T0: Core,
      T1: Core,
{
    type Delta = (
        Option<<T0 as Core>::Delta>,
        Option<<T1 as Core>::Delta>
    );
}

impl<T0, T1> Apply for (T0, T1)
where T0: Apply,
      T1: Apply,
{
    fn apply(&self, delta: Self::Delta) -> DeltaResult<Self> {
        let field0: T0 = self.0.apply(
            delta.0.ok_or_else(|| ExpectedValue!("Option<<T0 as Core>::Delta>"))?
        )?;
        let field1: T1 = self.1.apply(
            delta.1.ok_or_else(|| ExpectedValue!("Option<<T1 as Core>::Delta>"))?
        )?;
        Ok((field0, field1))
    }
}

impl<T0, T1> Delta for (T0, T1)
where T0: Delta,
      T1: Delta,
{
    fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
        let delta0: <T0 as Core>::Delta = Delta::delta(&self.0, &rhs.0)?;
        let delta1: <T1 as Core>::Delta = Delta::delta(&self.1, &rhs.1)?;
        Ok((
            if self.0 == rhs.0 { None } else { Some(delta0) },
            if self.1 == rhs.1 { None } else { Some(delta1) },
        ))
    }
}

impl<T0, T1> FromDelta for (T0, T1)
where T0: FromDelta,
      T1: FromDelta,
{
    fn from_delta(delta: Self::Delta) -> DeltaResult<Self> {
        Ok((
            delta.0.map(<T0>::from_delta)
                .ok_or_else(|| ExpectedValue!("Option<<T0 as Core>::Delta>"))??,
            delta.1.map(<T1>::from_delta)
                .ok_or_else(|| ExpectedValue!("Option<<T1 as Core>::Delta>"))??,
        ))
    }
}

impl<T0, T1> IntoDelta for (T0, T1)
where T0: IntoDelta,
      T1: IntoDelta,
{
    fn into_delta(self) -> DeltaResult<Self::Delta> {
        Ok((
            Some(self.0.into_delta()?),
            Some(self.1.into_delta()?),
        ))
    }
}




impl<T0, T1, T2> Core for (T0, T1, T2)
where T0: Core,
      T1: Core,
      T2: Core,
{
    type Delta = (
        Option<<T0 as Core>::Delta>,
        Option<<T1 as Core>::Delta>,
        Option<<T2 as Core>::Delta>
    );
}

impl<T0, T1, T2> Apply for (T0, T1, T2)
where T0: Apply,
      T1: Apply,
      T2: Apply,
{
    fn apply(&self, delta: Self::Delta) -> DeltaResult<Self> {
        let field0: T0 = self.0.apply(
            delta.0.ok_or_else(|| ExpectedValue!("Option<<T0 as Core>::Delta>"))?
        )?;
        let field1: T1 = self.1.apply(
            delta.1.ok_or_else(|| ExpectedValue!("Option<<T1 as Core>::Delta>"))?
        )?;
        let field2: T2 = self.2.apply(
            delta.2.ok_or_else(|| ExpectedValue!("Option<<T2 as Core>::Delta>"))?
        )?;
        Ok((field0, field1, field2))
    }
}

impl<T0, T1, T2> Delta for (T0, T1, T2)
where T0: Delta,
      T1: Delta,
      T2: Delta,
{
    fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
        let delta0: <T0 as Core>::Delta = Delta::delta(&self.0, &rhs.0)?;
        let delta1: <T1 as Core>::Delta = Delta::delta(&self.1, &rhs.1)?;
        let delta2: <T2 as Core>::Delta = Delta::delta(&self.2, &rhs.2)?;
        Ok((
            if self.0 == rhs.0 { None } else { Some(delta0) },
            if self.1 == rhs.1 { None } else { Some(delta1) },
            if self.2 == rhs.2 { None } else { Some(delta2) },
        ))
    }
}

impl<T0, T1, T2> FromDelta for (T0, T1, T2)
where T0: FromDelta,
      T1: FromDelta,
      T2: FromDelta,
{
    fn from_delta(delta: Self::Delta) -> DeltaResult<Self> {
        Ok((
            delta.0.map(<T0>::from_delta)
                .ok_or_else(|| ExpectedValue!("Option<<T0 as Core>::Delta>"))??,
            delta.1.map(<T1>::from_delta)
                .ok_or_else(|| ExpectedValue!("Option<<T1 as Core>::Delta>"))??,
            delta.2.map(<T2>::from_delta)
                .ok_or_else(|| ExpectedValue!("Option<<T2 as Core>::Delta>"))??,
        ))
    }
}

impl<T0, T1, T2> IntoDelta for (T0, T1, T2)
where T0: IntoDelta,
      T1: IntoDelta,
      T2: IntoDelta,
{
    fn into_delta(self) -> DeltaResult<Self::Delta> {
        Ok((
            Some(self.0.into_delta()?),
            Some(self.1.into_delta()?),
            Some(self.2.into_delta()?),
        ))
    }
}




impl<T0, T1, T2, T3> Core for (T0, T1, T2, T3)
where T0: Core,
      T1: Core,
      T2: Core,
      T3: Core,
{
    type Delta = (
        Option<<T0 as Core>::Delta>,
        Option<<T1 as Core>::Delta>,
        Option<<T2 as Core>::Delta>,
        Option<<T3 as Core>::Delta>
    );
}

impl<T0, T1, T2, T3> Apply for (T0, T1, T2, T3)
where T0: Apply,
      T1: Apply,
      T2: Apply,
      T3: Apply,
{
    fn apply(&self, delta: Self::Delta) -> DeltaResult<Self> {
        let field0: T0 = self.0.apply(
            delta.0.ok_or_else(|| ExpectedValue!("Option<<T0 as Core>::Delta>"))?
        )?;
        let field1: T1 = self.1.apply(
            delta.1.ok_or_else(|| ExpectedValue!("Option<<T1 as Core>::Delta>"))?
        )?;
        let field2: T2 = self.2.apply(
            delta.2.ok_or_else(|| ExpectedValue!("Option<<T2 as Core>::Delta>"))?
        )?;
        let field3: T3 = self.3.apply(
            delta.3.ok_or_else(|| ExpectedValue!("Option<<T3 as Core>::Delta>"))?
        )?;
        Ok((field0, field1, field2, field3))
    }
}

impl<T0, T1, T2, T3> Delta for (T0, T1, T2, T3)
where T0: Delta,
      T1: Delta,
      T2: Delta,
      T3: Delta,
{
    fn delta(&self, rhs: &Self) -> DeltaResult<Self::Delta> {
        let delta0: <T0 as Core>::Delta = Delta::delta(&self.0, &rhs.0)?;
        let delta1: <T1 as Core>::Delta = Delta::delta(&self.1, &rhs.1)?;
        let delta2: <T2 as Core>::Delta = Delta::delta(&self.2, &rhs.2)?;
        let delta3: <T3 as Core>::Delta = Delta::delta(&self.3, &rhs.3)?;
        Ok((
            if self.0 == rhs.0 { None } else { Some(delta0) },
            if self.1 == rhs.1 { None } else { Some(delta1) },
            if self.2 == rhs.2 { None } else { Some(delta2) },
            if self.3 == rhs.3 { None } else { Some(delta3) },
        ))
    }
}

impl<T0, T1, T2, T3> FromDelta for (T0, T1, T2, T3)
where T0: FromDelta,
      T1: FromDelta,
      T2: FromDelta,
      T3: FromDelta,
{
    fn from_delta(delta: Self::Delta) -> DeltaResult<Self> {
        Ok((
            delta.0.map(<T0>::from_delta)
                .ok_or_else(|| ExpectedValue!("Option<<T0 as Core>::Delta>"))??,
            delta.1.map(<T1>::from_delta)
                .ok_or_else(|| ExpectedValue!("Option<<T1 as Core>::Delta>"))??,
            delta.2.map(<T2>::from_delta)
                .ok_or_else(|| ExpectedValue!("Option<<T2 as Core>::Delta>"))??,
            delta.3.map(<T3>::from_delta)
                .ok_or_else(|| ExpectedValue!("Option<<T3 as Core>::Delta>"))??,
        ))
    }
}

impl<T0, T1, T2, T3> IntoDelta for (T0, T1, T2, T3)
where T0: IntoDelta,
      T1: IntoDelta,
      T2: IntoDelta,
      T3: IntoDelta,
{
    fn into_delta(self) -> DeltaResult<Self::Delta> {
        Ok((
            Some(self.0.into_delta()?),
            Some(self.1.into_delta()?),
            Some(self.2.into_delta()?),
            Some(self.3.into_delta()?),
        ))
    }
}
