//!

use chrono::prelude::{DateTime, Utc};
use crate::{Apply, Core, Delta, DeltaError, DeltaResult};
use crate::snapshot::SnapshotCtx;
use crate::snapshot::full::{FullSnapshot, FullSnapshots};
use serde_derive::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

#[macro_export]
macro_rules! delta_snapshot {
    (
        use result type $result_type:ty;
        [$($origin:ident)::*] $($new_state:expr),+ => $ctx:expr
        $(; $fmt:expr $(, $arg:expr)* )?
    ) => { loop {
        #[cfg(feature = "snapshot")]
        #[allow(redundant_semicolons, unused)] {
            let mut origin = String::new();
            $(
                if !origin.is_empty() { origin.push_str("::"); }
                origin.push_str(stringify!($origin));
            )* ;
            let mut msg: Option<String> = None;
            $(
                msg = Some(format!($fmt $(, $arg)*));
            )? ;
            $(
                let result = $crate::snapshot::delta::delta_snapshot(
                    $ctx,
                    origin.clone(),
                    msg.clone(),
                    $new_state
                );
                if let Err(err) = result {
                    break Err(err) as $result_type;
                }
            )+ ;
        }
        #[cfg(not(feature = "snapshot"))] {
            $(
                let _ = $new_state;
            )+ ;
            let _ = $ctx;
            $(
                let _ = $fmt;
                $(
                    let _ = $arg;
                )*
            )? ;
        }
        break Ok(()) as $result_type
    }}
}

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



#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub struct DeltaSnapshots<T: Core> {
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

    #[inline(always)]
    pub fn take_snapshots(&mut self) -> Vec<DeltaSnapshot<T>> {
        self.snapshots.drain(..).collect()
    }

    pub fn to_full_snapshots(self) -> DeltaResult<FullSnapshots<T>> {
        let initial = FullSnapshot::default();
        let mut uncompressed: Vec<FullSnapshot<T>> = vec![];
        for snapshot in self.snapshots {
            let old: &T = &uncompressed.last().unwrap_or(&initial).state;
            let new: T = old.apply(snapshot.delta)?;
            uncompressed.push(FullSnapshot {
                timestamp: snapshot.timestamp,
                origin:    snapshot.origin,
                msg:       snapshot.msg.clone(),
                state:     new,
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
            snapshots: vec![],
            current: FullSnapshot::default(),
        }
    }
}

impl<T: Core + Hash> Hash for DeltaSnapshots<T>
where <T as Core>::Delta: Hash
{
    fn hash<H: Hasher>(&self, state: &mut H) {
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
        if self.timestamp != rhs.timestamp { return false; }
        if self.origin != rhs.origin { return false; }
        true
    }
}

impl<T: Core + Eq> Eq for DeltaSnapshot<T> {}

impl<T: Core + PartialOrd> PartialOrd for DeltaSnapshot<T> {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        let timestamp_cmp = self.timestamp.partial_cmp(&rhs.timestamp);
        if timestamp_cmp != Some(Ordering::Equal) { return timestamp_cmp }
        let origin_cmp = self.origin.partial_cmp(&rhs.origin);
        if origin_cmp != Some(Ordering::Equal) { return origin_cmp }
        Some(Ordering::Equal)
    }
}

impl<T: Core + Ord> Ord for DeltaSnapshot<T> {
    fn cmp(&self, rhs: &Self) -> Ordering {
        let timestamp_cmp = self.timestamp.cmp(&rhs.timestamp);
        if timestamp_cmp != Ordering::Equal { return timestamp_cmp }
        let origin_cmp = self.origin.cmp(&rhs.origin);
        if origin_cmp != Ordering::Equal { return origin_cmp }
        Ordering::Equal
    }
}
