//!


use chrono::prelude::{DateTime, Utc};
use crate::{DeltaError, DeltaResult, Deltoid};
use crate::snapshot::delta::{DeltaSnapshot, DeltaSnapshots};
use serde_derive::{Deserialize, Serialize};
use std::cmp::Ordering;


#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub struct FullSnapshots<T: Deltoid + Default>(pub(crate) Vec<FullSnapshot<T>>);

impl<T: Deltoid + Default> FullSnapshots<T> {
    pub fn new() -> Self { Self(vec![]) }

    pub fn clear(&mut self) { self.0.clear(); }

    pub fn len(&self) -> usize { self.0.len() }

    pub fn is_empty(&self) -> bool { self.0.is_empty() }

    pub fn push_snapshot(&mut self, origin: String, state: T) -> DeltaResult<()> {
        self.add_snapshot(FullSnapshot {
            timestamp: Utc::now(),
            origin: origin,
            state: state,
        });
        Ok(())
    }

    pub fn add_snapshot(&mut self, snapshot: FullSnapshot<T>) {
        self.0.push(snapshot);
    }

    pub fn snapshot_ref(&self, idx: usize) -> DeltaResult<&FullSnapshot<T>> {
        self.0.get(idx).ok_or(DeltaError::ExpectedValue)
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
                delta:     old.delta(new)?,
            });
        }
        Ok(DeltaSnapshots {
            snapshots: deltas,
            current: self.0.pop().unwrap_or(initial),
        })
    }
}

impl<T: Deltoid + Default> Default for FullSnapshots<T> {
    fn default() -> Self { Self::new() }
}




#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FullSnapshot<T: Deltoid> {
    pub timestamp: DateTime<Utc>,
    pub origin: String,
    pub state: T,
}

impl<T: Deltoid> FullSnapshot<T> {
    pub fn new(origin: String, state: T) -> Self {
        Self { timestamp: Utc::now(), origin, state }
    }
}

impl<T: Deltoid + Default> Default for FullSnapshot<T> {
    fn default() -> Self {
        Self {
            timestamp: Utc::now(),
            origin: "default".to_string(),
            state: Default::default(),
        }
    }
}

impl<T: Deltoid> PartialEq for FullSnapshot<T> {
    fn eq(&self, rhs: &Self) -> bool {
        if self.timestamp != rhs.timestamp { return false; }
        if self.origin != rhs.origin { return false; }
        true
    }
}

impl<T: Deltoid> Eq for FullSnapshot<T> {}

impl<T: Deltoid> PartialOrd for FullSnapshot<T> {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        let timestamp_cmp = self.timestamp.partial_cmp(&rhs.timestamp);
        if timestamp_cmp != Some(Ordering::Equal) { return timestamp_cmp }
        let origin_cmp = self.origin.partial_cmp(&rhs.origin);
        if origin_cmp != Some(Ordering::Equal) { return origin_cmp }
        Some(Ordering::Equal)
    }
}

impl<T: Deltoid> Ord for FullSnapshot<T> {
    fn cmp(&self, rhs: &Self) -> Ordering {
        let timestamp_cmp = self.timestamp.cmp(&rhs.timestamp);
        if timestamp_cmp != Ordering::Equal { return timestamp_cmp }
        let origin_cmp = self.origin.cmp(&rhs.origin);
        if origin_cmp != Ordering::Equal { return origin_cmp }
        Ordering::Equal
    }
}
