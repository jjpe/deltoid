//! Snapshotting infrastructure.

#[cfg(feature = "snapshot")] mod delta;
#[cfg(feature = "snapshot")] mod full;

#[cfg(feature = "snapshot")] pub use crate::snapshot::delta::*;
#[cfg(feature = "snapshot")] pub use crate::snapshot::full::*;


#[cfg(feature = "snapshot")]
pub trait SnapshotCtx<T>
where T: crate::Core + Default {
    fn history(&mut self) -> &mut DeltaSnapshots<T>;

    #[inline]
    fn take_history(&mut self) -> DeltaSnapshots<T> {
        std::mem::take(self.history())
    }
}

#[macro_export]
macro_rules! snapshot {
    (
        use result type $result_type:ty;
        [$($origin:ident)::*] $new_state:expr => $context:expr
        $(; $fmt:expr $(, $arg:expr)* )?
    ) => { loop /* used as a do-block rather than a loop */ {
        #[cfg(feature = "snapshot")]
        #[allow(redundant_semicolons, unused)] {
            use deltoid::{Core, Apply, Delta, FromDelta, IntoDelta};
            use deltoid::snapshot::{DeltaSnapshot, DeltaSnapshots, SnapshotCtx};
            let mut origin = String::new();
            $(
                if !origin.is_empty() { origin.push_str("::"); }
                origin.push_str(stringify!($origin));
            )* ;
            let mut msg: Option<String> = None;
            $(
                msg = Some(format!($fmt $(, $arg)*));
            )? ;
            let ctx: &mut SnapshotCtx<_> = $context;
            let mut history_guard = match ctx.history() {
                Ok(guard) => guard,
                Err(err) => break Err(err.into()) as $result_type,
            };
            let history: &mut DeltaSnapshots<_> = &mut *history_guard;
            let old_state: &_ = &history.current().state;
            let new_state: &_ = &$new_state;
            let delta = match old_state.delta(new_state) {
                Ok(delta) => delta,
                Err(derr) => break Err(derr.into()) as $result_type,
            };
            history.update_current(origin, new_state);
            history.add_snapshot(DeltaSnapshot {
                timestamp: history.current().timestamp.clone(),
                origin:    history.current().origin.clone(),
                msg,
                delta,
            });
        }
        #[cfg(not(feature = "snapshot"))] {
            let _ = $new_state;
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
