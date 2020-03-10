//! Snapshotting infrastructure.

#[cfg(feature = "snapshot")] use chrono::prelude::{DateTime, Utc};
#[cfg(feature = "snapshot")] use crate::Deltoid;
#[cfg(feature = "snapshot")] use serde_derive::{Deserialize, Serialize};
#[cfg(feature = "snapshot")] use std::cmp::Ordering;

#[macro_export]
macro_rules! snap {
    ([$($location:ident)::*] $new:expr => $context:expr) => {{
        #[cfg(feature = "snapshot")]
        #[allow(redundant_semicolon)] {
            use $crate::Deltoid;
            use $crate::snapshot::Snapshot;
            let mut origin = String::new();
            $(
                if !origin.is_empty() { origin.push_str("::"); }
                origin.push_str(stringify!($location));
            )* ;
            let context = $context;
            let history: &mut History<_> = &mut *context.history_mut()?;
            let old = history.current_state();
            let new: &_ = &$new;
            let delta = old.delta(new)?;
            if history.is_empty() {
                history.set_initial_state(new.clone());
            }
            history.set_current_state(new.clone());
            history.add_snapshot(Snapshot::new(origin, delta));
        }
        #[cfg(not(feature = "snapshot"))] {
            let _ = $context;
        }
        $crate::error::DeltaResult::Ok(())
    }}
}


#[cfg(feature = "snapshot")]
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Default,
    Deserialize, Serialize
)]
pub struct History<T: Deltoid> {
    snapshots:  Vec<Snapshot<T>>,
    initial_state: T,
    current_state: T,
}

#[cfg(feature = "snapshot")]
impl<T: Deltoid + Default> History<T> {
    pub fn new() -> Self {
        Self {
            snapshots:     vec![],
            initial_state: <T>::default(),
            current_state: <T>::default(),
        }
    }

    pub fn reset(&mut self) { *self = Self::new(); }

    pub fn is_empty(&self) -> bool {
        self.snapshot_count() == 0
    }

    pub fn snapshot_count(&self) -> usize {
        self.snapshots.len()
    }

    pub fn snapshots(&self) -> impl Iterator<Item = &Snapshot<T>> {
        self.snapshots.iter()
    }

    pub fn add_snapshot(&mut self, snapshot: Snapshot<T>) {
        self.snapshots.push(snapshot);
    }

    pub fn take_snapshots<'s>(&'s mut self) -> impl Iterator<Item = Snapshot<T>> + 's {
        self.snapshots.drain(..)
    }

    pub fn initial_state(&self) -> &T { &self.initial_state }

    pub fn set_initial_state(&mut self, state: &T) {
        self.initial_state = state.clone();
    }

    pub fn current_state(&self) -> &T { &self.current_state }

    pub fn set_current_state(&mut self, state: &T) {
        self.current_state = state.clone();
    }
}



#[cfg(feature = "snapshot")]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Snapshot<T: Deltoid> {
    timestamp: DateTime<Utc>,
    origin: String,
    contents: <T as Deltoid>::Delta,
}

#[cfg(feature = "snapshot")]
impl<T: Deltoid> Snapshot<T> {
    pub fn new(
        origin: String,
        contents: <T as Deltoid>::Delta
    ) -> Self {
        Self {
            timestamp: Utc::now(),
            origin: origin,
            contents: contents,
        }
    }

    pub fn timestamp(&self) -> &DateTime<Utc> { &self.timestamp }

    pub fn origin(&self) -> &str { &self.origin }

    pub fn contents(&self) -> &T { &self.contents }
}

#[cfg(feature = "snapshot")]
impl<T: Deltoid> PartialEq for Snapshot<T> {
    fn eq(&self, rhs: &Self) -> bool {
        if self.timestamp != rhs.timestamp { return false; }
        if self.origin != rhs.origin { return false; }
        true
    }
}

#[cfg(feature = "snapshot")]
impl<T: Deltoid> Eq for Snapshot<T> {}

#[cfg(feature = "snapshot")]
impl<T: Deltoid> PartialOrd for Snapshot<T> {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        let timestamp_cmp = self.timestamp.partial_cmp(&rhs.timestamp);
        if timestamp_cmp != Some(Ordering::Equal) { return timestamp_cmp }
        let origin_cmp = self.origin.partial_cmp(&rhs.origin);
        if origin_cmp != Some(Ordering::Equal) { return origin_cmp }
        Some(Ordering::Equal)
    }
}

#[cfg(feature = "snapshot")]
impl<T: Deltoid> Ord for Snapshot<T> {
    fn cmp(&self, rhs: &Self) -> Ordering {
        let timestamp_cmp = self.timestamp.cmp(&rhs.timestamp);
        if timestamp_cmp != Ordering::Equal { return timestamp_cmp }
        let origin_cmp = self.origin.cmp(&rhs.origin);
        if origin_cmp != Ordering::Equal { return origin_cmp }
        Ordering::Equal
    }
}
