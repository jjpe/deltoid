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
            use $crate::snapshot::DeltaSnapshot;
            let mut origin = String::new();
            $(
                if !origin.is_empty() { origin.push_str("::"); }
                origin.push_str(stringify!($location));
            )* ;
            let history: &mut History<_> = &mut *$context.history_mut()?;
            let old: &_ = history.current_state();
            let new: &_ = &$new;
            let delta = old.delta(new)?;
            history.set_current_state(origin.clone(), new);
            history.add_snapshot(DeltaSnapshot::new(origin, delta));
        }
        #[cfg(not(feature = "snapshot"))] {
            let _ = $context;
        }
        $crate::error::DeltaResult::Ok(())
    }}
}


#[cfg(feature = "snapshot")]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub struct History<T: Deltoid> {
    snapshots: Vec<DeltaSnapshot<T>>,
    initial: FullSnapshot<T>,
    current: FullSnapshot<T>,
}

#[cfg(feature = "snapshot")]
impl<T: Deltoid + Default> History<T> {
    pub fn new() -> Self {
        Self {
            snapshots: vec![],
            initial: FullSnapshot::default(),
            current: FullSnapshot::default(),
        }
    }

    pub fn reset(&mut self) { *self = Self::new(); }

    pub fn is_empty(&self) -> bool { self.snapshot_count() == 0 }

    pub fn snapshot_count(&self) -> usize { self.snapshots.len() }

    pub fn add_snapshot(&mut self, snapshot: DeltaSnapshot<T>) {
        self.snapshots.push(snapshot);
    }

    pub fn snapshots(&self) -> impl Iterator<Item = &DeltaSnapshot<T>> {
        self.snapshots.iter()
    }

    pub fn take_snapshots<'s>(&'s mut self) ->
        impl Iterator<Item = DeltaSnapshot<T>> + 's
    {
        self.snapshots.drain(..)
    }

    pub fn initial(&self) -> &FullSnapshot<T> { &self.initial }

    pub fn initial_state(&self) -> &T { &self.initial.state }

    pub fn current(&self) -> &FullSnapshot<T> { &self.current }

    pub fn current_state(&self) -> &T { &self.current.state }

    pub fn set_current_state(&mut self, origin: String, state: &T) {
        self.current.state = state.clone();
        self.current.origin = origin;
        self.current.timestamp = Utc::now();
    }
}

#[cfg(feature = "snapshot")]
impl<T: Deltoid + Default> Default for History<T> {
    fn default() -> Self { Self::new() }
}




#[cfg(feature = "snapshot")]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FullSnapshot<T: Deltoid> {
    pub timestamp: DateTime<Utc>,
    pub origin: String,
    pub state: T,
}

#[cfg(feature = "snapshot")]
impl<T: Deltoid> FullSnapshot<T> {
    pub fn new(origin: String, state: T) -> Self {
        Self { timestamp: Utc::now(), origin, state }
    }
}

#[cfg(feature = "snapshot")]
impl<T: Deltoid + Default> Default for FullSnapshot<T> {
    fn default() -> Self {
        Self {
            timestamp: Utc::now(),
            origin: "default".to_string(),
            state: Default::default(),
        }
    }
}

#[cfg(feature = "snapshot")]
impl<T: Deltoid> PartialEq for FullSnapshot<T> {
    fn eq(&self, rhs: &Self) -> bool {
        if self.timestamp != rhs.timestamp { return false; }
        if self.origin != rhs.origin { return false; }
        true
    }
}

#[cfg(feature = "snapshot")]
impl<T: Deltoid> Eq for FullSnapshot<T> {}

#[cfg(feature = "snapshot")]
impl<T: Deltoid> PartialOrd for FullSnapshot<T> {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        let timestamp_cmp = self.timestamp.partial_cmp(&rhs.timestamp);
        if timestamp_cmp != Some(Ordering::Equal) { return timestamp_cmp }
        let origin_cmp = self.origin.partial_cmp(&rhs.origin);
        if origin_cmp != Some(Ordering::Equal) { return origin_cmp }
        Some(Ordering::Equal)
    }
}

#[cfg(feature = "snapshot")]
impl<T: Deltoid> Ord for FullSnapshot<T> {
    fn cmp(&self, rhs: &Self) -> Ordering {
        let timestamp_cmp = self.timestamp.cmp(&rhs.timestamp);
        if timestamp_cmp != Ordering::Equal { return timestamp_cmp }
        let origin_cmp = self.origin.cmp(&rhs.origin);
        if origin_cmp != Ordering::Equal { return origin_cmp }
        Ordering::Equal
    }
}




#[cfg(feature = "snapshot")]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DeltaSnapshot<T: Deltoid> {
    pub timestamp: DateTime<Utc>,
    pub origin: String,
    pub delta: <T as Deltoid>::Delta,
}

#[cfg(feature = "snapshot")]
impl<T: Deltoid> DeltaSnapshot<T> {
    pub fn new(origin: String, delta: <T as Deltoid>::Delta) -> Self {
        Self { timestamp: Utc::now(), origin, delta }
    }
}

#[cfg(feature = "snapshot")]
impl<T: Deltoid> PartialEq for DeltaSnapshot<T> {
    fn eq(&self, rhs: &Self) -> bool {
        if self.timestamp != rhs.timestamp { return false; }
        if self.origin != rhs.origin { return false; }
        true
    }
}

#[cfg(feature = "snapshot")]
impl<T: Deltoid> Eq for DeltaSnapshot<T> {}

#[cfg(feature = "snapshot")]
impl<T: Deltoid> PartialOrd for DeltaSnapshot<T> {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        let timestamp_cmp = self.timestamp.partial_cmp(&rhs.timestamp);
        if timestamp_cmp != Some(Ordering::Equal) { return timestamp_cmp }
        let origin_cmp = self.origin.partial_cmp(&rhs.origin);
        if origin_cmp != Some(Ordering::Equal) { return origin_cmp }
        Some(Ordering::Equal)
    }
}

#[cfg(feature = "snapshot")]
impl<T: Deltoid> Ord for DeltaSnapshot<T> {
    fn cmp(&self, rhs: &Self) -> Ordering {
        let timestamp_cmp = self.timestamp.cmp(&rhs.timestamp);
        if timestamp_cmp != Ordering::Equal { return timestamp_cmp }
        let origin_cmp = self.origin.cmp(&rhs.origin);
        if origin_cmp != Ordering::Equal { return origin_cmp }
        Ordering::Equal
    }
}
