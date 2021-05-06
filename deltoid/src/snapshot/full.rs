//!

use chrono::prelude::{DateTime, Utc};
use crate::{Apply, Core, Delta, DeltaError, DeltaResult};
use crate::snapshot::SnapshotCtx;
use crate::snapshot::delta::{DeltaSnapshot, DeltaSnapshots};
use serde_derive::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

#[macro_export]
macro_rules! full_snapshot {
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
                let result = $crate::snapshot::full::full_snapshot(
                    $ctx,
                    origin.clone(),
                    msg.clone(),
                    $new_state.clone()
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
        break Ok(()) as $result_type;
    }}
}

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



#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
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
    pub fn clear(&mut self) { self.0.clear(); }

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
        Ok(DeltaSnapshots {
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
        if self.timestamp != rhs.timestamp { return false; }
        if self.origin != rhs.origin { return false; }
        true
    }
}

impl<T: Core + Eq> Eq for FullSnapshot<T> {}

impl<T: Core + PartialOrd> PartialOrd for FullSnapshot<T> {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        let timestamp_cmp = self.timestamp.partial_cmp(&rhs.timestamp);
        if timestamp_cmp != Some(Ordering::Equal) { return timestamp_cmp }
        let origin_cmp = self.origin.partial_cmp(&rhs.origin);
        if origin_cmp != Some(Ordering::Equal) { return origin_cmp }
        Some(Ordering::Equal)
    }
}

impl<T: Core + Ord> Ord for FullSnapshot<T> {
    fn cmp(&self, rhs: &Self) -> Ordering {
        let timestamp_cmp = self.timestamp.cmp(&rhs.timestamp);
        if timestamp_cmp != Ordering::Equal { return timestamp_cmp }
        let origin_cmp = self.origin.cmp(&rhs.origin);
        if origin_cmp != Ordering::Equal { return origin_cmp }
        Ordering::Equal
    }
}
