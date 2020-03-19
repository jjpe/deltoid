//! Snapshotting infrastructure.

#[cfg(feature = "snapshot")] mod delta;
#[cfg(feature = "snapshot")] mod full;

#[cfg(feature = "snapshot")]
pub use crate::snapshot::delta::{DeltaSnapshot, DeltaSnapshots};
#[cfg(feature = "snapshot")]
pub use crate::snapshot::full::{FullSnapshot, FullSnapshots};


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
            let history: &mut DeltaSnapshots<_> = &mut *$context.history()?;
            let old: &_ = &history.current().state;
            let new: &_ = &$new;
            let delta = old.delta(new)?;
            history.update_current(origin, new);
            history.add_snapshot(DeltaSnapshot {
                timestamp: history.current().timestamp.clone(),
                origin:    history.current().origin.clone(),
                delta:     delta,
            });
        }
        #[cfg(not(feature = "snapshot"))] {
            let _ = $context;
        }
        $crate::error::DeltaResult::Ok(())
    }}
}
