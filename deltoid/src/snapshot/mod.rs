//! Snapshotting infrastructure.

#[cfg(feature = "snapshot")] mod delta;
#[cfg(feature = "snapshot")] mod full;

#[cfg(feature = "snapshot")] pub use crate::snapshot::delta::*;
#[cfg(feature = "snapshot")] pub use crate::snapshot::full::*;


#[macro_export]
macro_rules! snapshot {
    (
        use result type $result_type:ty;
        [$($origin:ident)::*] $new:expr => $context:expr
        $(; $fmt:expr $(, $arg:expr)* )?
    ) => { loop {
        #[cfg(feature = "snapshot")]
        #[allow(redundant_semicolons)]
        #[allow(unused)] {
            use deltoid::{Core, Apply, Delta, FromDelta, IntoDelta};
            use deltoid::snapshot::{DeltaSnapshot, DeltaSnapshots};
            let mut origin = String::new();
            $(
                if !origin.is_empty() { origin.push_str("::"); }
                origin.push_str(stringify!($origin));
            )* ;
            let mut msg: Option<String> = None;
            $(
                msg = Some(format!($fmt $(, $arg)*));
            )?
            let mut history_guard = match $context.history() {
                Ok(guard) => guard,
                Err(err) => break Err(err.into()) as $result_type,
            };
            let history: &mut DeltaSnapshots<_> = &mut *history_guard;
            let old: &_ = &history.current().state;
            let new: &_ = &$new;
            if &*old != &**new {
                let delta = match old.delta(new) {
                    Ok(delta) => delta,
                    Err(derr) => break Err(derr.into()) as $result_type,
                };
                history.update_current(origin, new);
                history.add_snapshot(DeltaSnapshot {
                    timestamp: history.current().timestamp.clone(),
                    origin:    history.current().origin.clone(),
                    msg,
                    delta,
                });
            }
        }
        #[cfg(not(feature = "snapshot"))] {
            let _ = $new;
            let _ = $context;
            $(
                let _ = $fmt;
                $(
                    let _ = $arg;
                )*
            )?
        }
        break Ok(()) as $result_type;
    }}
}
