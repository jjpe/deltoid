//!

use chrono::prelude::{DateTime, Utc};
use crate::{Apply, Core, Delta, DeltaError, DeltaResult};
use crate::snapshot::SnapshotCtx;
use crate::snapshot::delta::{DeltaSnapshot, DeltaSnapshots};
use serde_derive::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

#[cfg(feature = "snapshot")]
#[inline(always)]
pub fn full_snapshot<S, E, C>(
    ctx: &mut C,
    origin: String,
    msg: Option<String>,
    state: S,
) -> Result<(), E>
where
    S: Delta + Apply + Default,
    E: From<DeltaError>,
    C: SnapshotCtx<S, History = FullSnapshots<S>>
{
    let history: &mut <C as SnapshotCtx<S>>::History = ctx.history();
    let timestamp: DateTime<Utc> = Utc::now();
    history.add_snapshot(FullSnapshot { timestamp, origin, msg, state });
    Ok(())
}



#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct FullSnapshots<T: Core>(pub(crate) Vec<FullSnapshot<T>>);

impl<T: Apply + Delta + Default> Default for FullSnapshots<T> {
    fn default() -> Self {
        Self(vec![ FullSnapshot::default() ])
    }
}

impl<T: Apply + Delta + Default> FullSnapshots<T> {
    #[inline(always)]
    pub fn current(&self) -> &FullSnapshot<T> {
        self.0.last().unwrap(/*safe b/c there's always at least 1 snapshot*/)
    }

    #[inline(always)]
    pub fn clear(&mut self) { self.0 = vec![ FullSnapshot::default() ]; }

    #[inline(always)]
    pub fn len(&self) -> usize { self.0.len() }

    #[inline(always)]
    pub fn is_empty(&self) -> bool { self.0.is_empty() }

    pub fn push_snapshot(
        &mut self,
        origin: String,
        msg: Option<String>,
        state: T
    ) -> DeltaResult<()> {
        let timestamp = Utc::now();
        self.add_snapshot(FullSnapshot { timestamp, origin, msg, state });
        Ok(())
    }

    #[inline(always)]
    pub fn add_snapshot(&mut self, snapshot: FullSnapshot<T>) {
        self.0.push(snapshot);
    }

    #[inline(always)]
    pub fn snapshot_ref(&self, idx: usize) -> DeltaResult<&FullSnapshot<T>> {
        self.0.get(idx).ok_or_else(|| ExpectedValue!("FullSnapshot<T>"))
    }

    pub fn to_delta_snapshots(mut self) -> DeltaResult<DeltaSnapshots<T>> {
        let initial = FullSnapshot::default();
        let mut deltas: Vec<DeltaSnapshot<T>> = vec![];
        for (sidx, snapshot) in self.0.iter().enumerate() {
            let old: &T =
                if sidx == 0 { &initial.state  }
                else { &self.0[sidx - 1].state };
            let new: &T = &snapshot.state;
            deltas.push(DeltaSnapshot {
                timestamp: snapshot.timestamp.clone(),
                origin:    snapshot.origin.clone(),
                msg:       snapshot.msg.clone(),
                delta:     old.delta(new)?,
            });
        }
        let base = initial.state.clone();
        Ok(DeltaSnapshots {
            base,
            snapshots: deltas,
            current: self.0.pop().unwrap_or(initial),
        })
    }

    #[inline(always)]
    pub fn into_iter(self) -> impl Iterator<Item = FullSnapshot<T>> {
        self.0.into_iter()
    }

    #[inline(always)]
    pub fn iter(&self) -> impl Iterator<Item = &FullSnapshot<T>> {
        self.0.iter()
    }
}

impl<T: Core + Hash> Hash for FullSnapshots<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}




#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FullSnapshot<T: Core> {
    pub timestamp: DateTime<Utc>,
    pub origin: String,
    pub msg: Option<String>,
    pub state: T,
}

impl<T: Core> FullSnapshot<T> {
    pub fn new(
        origin: String,
        msg: Option<String>,
        state: T
    ) -> Self {
        Self { timestamp: Utc::now(), origin, msg, state }
    }
}

impl<T: Core + Default> Default for FullSnapshot<T> {
    fn default() -> Self {
        Self {
            timestamp: Utc::now(),
            origin: "default".to_string(),
            msg: None,
            state: Default::default(),
        }
    }
}

impl<T: Core + Hash> Hash for FullSnapshot<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.timestamp.hash(state);
        self.origin.hash(state);
        self.msg.hash(state);
        self.state.hash(state);
    }
}

impl<T: Core + PartialEq> PartialEq for FullSnapshot<T> {
    fn eq(&self, rhs: &Self) -> bool {
        self.timestamp == rhs.timestamp
            && self.origin == rhs.origin
            && self.msg == rhs.msg
            && self.state == rhs.state
    }
}

impl<T: Core + Eq> Eq for FullSnapshot<T> {}

#[cfg(all(test, feature = "snapshot"))]
mod tests {
    use crate::snapshot::full::FullSnapshots;
    use crate::snapshot::SnapshotCtx;
    use crate::DeltaResult;

    #[test]
    fn FullSnapshots__clear__keeps_current_invariant() {
        let mut snapshots = FullSnapshots::<u64>::default();
        snapshots.push_snapshot("x".to_string(), None, 42).unwrap();
        snapshots.clear();
        // `current()` unwraps `last()`; `clear()` must preserve the
        // "there's always at least 1 snapshot" invariant (was: panic):
        assert_eq!(snapshots.current().state, 0);
        assert_eq!(snapshots.len(), 1);
    }

    struct Ctx {
        history: FullSnapshots<u64>,
    }

    impl SnapshotCtx<u64> for Ctx {
        type History = FullSnapshots<u64>;

        fn history(&mut self) -> &mut Self::History { &mut self.history }
    }

    #[test]
    fn full_snapshot__macro__records_one_snapshot() -> DeltaResult<()> {
        let mut ctx = Ctx { history: FullSnapshots::<u64>::default() };
        let _: DeltaResult<()> =
            full_snapshot!(use result type DeltaResult<()>; [test] 5u64 => &mut ctx);
        assert_eq!(ctx.history.len(), 2); // default seed + recorded snapshot
        assert_eq!(ctx.history.current().state, 5);
        Ok(())
    }
}
