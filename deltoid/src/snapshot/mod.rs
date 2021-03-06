//! Snapshotting infrastructure.

#[cfg(feature = "snapshot")] mod delta;
#[cfg(feature = "snapshot")] mod full;

#[cfg(feature = "snapshot")] use crate::core::{Apply, Core, Delta};
#[cfg(feature = "snapshot")] use crate::error::DeltaError;
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

#[cfg(feature = "snapshot")]
pub fn snapshot<S, E>(
    ctx: &mut dyn SnapshotCtx<S>,
    origin: String,
    msg: Option<String>,
    new_state: &S,
) -> Result<(), E>
where
    S: Delta + Apply + Default,
    E: From<DeltaError>
{
    let history: &mut DeltaSnapshots<S> = ctx.history();
    let old_state: &S = &history.current().state;
    let timestamp = history.current().timestamp.clone();
    let delta: <S as Core>::Delta = match old_state.delta(new_state) {
        Ok(delta) => delta,
        Err(derr) => return Err(derr.into()),
    };
    history.update_current(origin.clone(), new_state);
    // let origin = history.current().origin.clone();
    history.add_snapshot(DeltaSnapshot { timestamp, origin, msg, delta });
    Ok(())
}

#[macro_export]
macro_rules! snapshot {
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
                let result = $crate::snapshot::snapshot(
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
