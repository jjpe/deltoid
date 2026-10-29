//!

use chrono::prelude::{DateTime, Utc};
use crate::{Apply, Core, Delta, DeltaError, DeltaResult};
use crate::snapshot::SnapshotCtx;
use crate::snapshot::full::{FullSnapshot, FullSnapshots};
use serde_derive::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

#[cfg(feature = "snapshot")]
#[inline(always)]
pub fn delta_snapshot<S, E, C>(
    ctx: &mut C,
    origin: String,
    msg: Option<String>,
    new_state: &S,
) -> Result<(), E>
where
    S: Delta + Apply + Default,
    E: From<DeltaError>,
    C: SnapshotCtx<S, History = DeltaSnapshots<S>>,
{
    let history: &mut <C as SnapshotCtx<S>>::History = ctx.history();
    let old_state: &S = &history.current().state;
    let delta: <S as Core>::Delta = old_state.delta(new_state)?;
    history.update_current(origin.clone(), new_state);
    let timestamp = history.current().timestamp.clone();
    history.add_snapshot(DeltaSnapshot { timestamp, origin, msg, delta });
    Ok(())
}



#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct DeltaSnapshots<T: Core> {
    pub(crate) base: T,
    pub(crate) snapshots: Vec<DeltaSnapshot<T>>,
    pub(crate) current: FullSnapshot<T>,
}

impl<T: Apply + Delta + Default> DeltaSnapshots<T> {
    #[inline(always)]
    pub fn current(&self) -> &FullSnapshot<T> { &self.current }

    pub fn update_current(&mut self, origin: String, state: &T) {
        self.current.state = state.clone();
        self.current.origin = origin;
        self.current.timestamp = Utc::now();
    }

    pub fn clear(&mut self) {
        self.snapshots.clear();
        self.base = Default::default();
        self.current = Default::default();
    }

    #[inline(always)]
    pub fn len(&self) -> usize { self.snapshots.len() }

    #[inline(always)]
    pub fn is_empty(&self) -> bool { self.snapshots.is_empty() }

    pub fn push_snapshot(
        &mut self,
        origin: String,
        msg: Option<String>,
        state: T
    ) -> DeltaResult<()> {
        let old: &T = &self.current.state;
        let delta = old.delta(&state)?;
        let full = FullSnapshot { timestamp: Utc::now(), origin, msg, state };
        self.add_snapshot(DeltaSnapshot {
            timestamp: full.timestamp.clone(),
            origin:    full.origin.clone(),
            msg:       full.msg.clone(),
            delta,
        });
        self.current = full;
        Ok(())
    }

    #[inline(always)]
    pub fn add_snapshot(&mut self, snapshot: DeltaSnapshot<T>) {
        self.snapshots.push(snapshot);
    }

    pub fn take_snapshots(&mut self) -> (T, Vec<DeltaSnapshot<T>>) {
        let base = std::mem::replace(&mut self.base, self.current.state.clone());
        (base, std::mem::take(&mut self.snapshots))
    }

    pub fn to_full_snapshots(self) -> DeltaResult<FullSnapshots<T>> {
        let mut uncompressed: Vec<FullSnapshot<T>> = vec![];
        let mut state = self.base;
        for snapshot in self.snapshots {
            state = state.apply(snapshot.delta)?;
            uncompressed.push(FullSnapshot {
                timestamp: snapshot.timestamp,
                origin:    snapshot.origin,
                msg:       snapshot.msg,
                state:     state.clone(),
            });
        }
        Ok(FullSnapshots(uncompressed))
    }

    #[inline(always)]
    pub fn into_iter(self) -> impl Iterator<Item = DeltaSnapshot<T>> {
        self.snapshots.into_iter()
    }

    #[inline(always)]
    pub fn iter(&self) -> impl Iterator<Item = &DeltaSnapshot<T>> {
        self.snapshots.iter()
    }
}

impl<T: Core + Default> Default for DeltaSnapshots<T> {
    fn default() -> Self {
        Self {
            base: T::default(),
            snapshots: vec![],
            current: FullSnapshot::default(),
        }
    }
}

impl<T: Core + Hash> Hash for DeltaSnapshots<T>
where <T as Core>::Delta: Hash
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.base.hash(state);
        self.snapshots.hash(state);
        self.current.hash(state);
    }
}




#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DeltaSnapshot<T: Core> {
    pub timestamp: DateTime<Utc>,
    pub origin: String,
    pub msg: Option<String>,
    pub delta: <T as Core>::Delta,
}

impl<T: Core> DeltaSnapshot<T> {
    pub fn new(
        origin: String,
        msg: Option<String>,
        delta: <T as Core>::Delta
    ) -> Self {
        Self { timestamp: Utc::now(), origin, msg, delta }
    }
}

impl<T: Core> Hash for DeltaSnapshot<T>
where <T as Core>::Delta: Hash {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.timestamp.hash(state);
        self.origin.hash(state);
        self.msg.hash(state);
        self.delta.hash(state);
    }
}

impl<T: Core + PartialEq> PartialEq for DeltaSnapshot<T> {
    fn eq(&self, rhs: &Self) -> bool {
        self.timestamp == rhs.timestamp
            && self.origin == rhs.origin
            && self.msg == rhs.msg
            && self.delta == rhs.delta
    }
}

impl<T: Core + Eq> Eq for DeltaSnapshot<T> {}

#[cfg(all(test, feature = "snapshot"))]
mod tests {
    use crate::snapshot::delta::{DeltaSnapshot, DeltaSnapshots};
    use crate::snapshot::SnapshotCtx;
    use crate::{Apply, DeltaResult, IntoDelta};
    use std::collections::HashSet;

    struct Ctx {
        history: DeltaSnapshots<u64>,
    }

    impl SnapshotCtx<u64> for Ctx {
        type History = DeltaSnapshots<u64>;

        fn history(&mut self) -> &mut Self::History { &mut self.history }
    }

    #[test]
    fn DeltaSnapshot__eq__compares_full_content() -> DeltaResult<()> {
        let a = DeltaSnapshot::<u64>::new("x".to_string(), None, 5u64.into_delta()?);
        let b = a.clone();
        assert_eq!(a, b);
        let c = DeltaSnapshot {
            timestamp: a.timestamp.clone(),
            origin:    a.origin.clone(),
            msg:       a.msg.clone(),
            delta:     6u64.into_delta()?,
        };
        // was: equal (equality ignored `delta`/`msg`) despite differing content
        assert_ne!(a, c);

        // Eq/Hash contract: `==`-equal snapshots hash to one bucket,
        // content-differing snapshots hash apart:
        let mut set = HashSet::new();
        set.insert(&a);
        set.insert(&b);
        assert_eq!(set.len(), 1);
        set.insert(&c);
        assert_eq!(set.len(), 2);
        Ok(())
    }

    #[test]
    fn DeltaSnapshots__take_snapshots__reanchors_base() -> DeltaResult<()> {
        let mut ctx = Ctx { history: DeltaSnapshots::<u64>::default() };
        ctx.history.push_snapshot("a".to_string(), None, 5)?;
        ctx.history.push_snapshot("b".to_string(), None, 9)?;

        // The taken chain replays correctly from the returned pre-take base:
        let (base, chain) = ctx.history.take_snapshots();
        assert_eq!(base, 0);
        let mut state = base;
        let mut states = vec![];
        for snapshot in chain {
            state = state.apply(snapshot.delta)?;
            states.push(state);
        }
        assert_eq!(states, vec![5, 9]);

        // Post-take, the chain is re-anchored at the previous head (9), so a
        // further push replays without silently re-basing on T::default():
        ctx.history.push_snapshot("c".to_string(), None, 12)?;
        let fulls = ctx.history.to_full_snapshots()?;
        let replayed: Vec<u64> = fulls.iter().map(|f| f.state).collect();
        assert_eq!(replayed, vec![12]);
        Ok(())
    }
}
