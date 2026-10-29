//! Snapshotting infrastructure.

// The snapshot macros are defined here, in the ungated module root: the
// `snapshot` feature is evaluated when *deltoid* compiles by selecting one
// of the two macro definitions below. A `#[cfg(feature = "snapshot")]`
// written *inside* a macro body would instead be evaluated in the expanding
// user crate (which may not have a feature of that name), silently dropping
// snapshots; and macros living in the feature-gated submodules would not
// exist at all when this crate is built without the feature.
#[cfg(feature = "snapshot")]
#[macro_export]
macro_rules! full_snapshot {
    (
        use result type $result_type:ty;
        [$($origin:ident)::*] $($new_state:expr),+ => $ctx:expr
        $(; $fmt:expr $(, $arg:expr)* )?
    ) => { loop {
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
        break Ok(()) as $result_type;
    }}
}

#[cfg(not(feature = "snapshot"))]
#[macro_export]
macro_rules! full_snapshot {
    (
        use result type $result_type:ty;
        [$($origin:ident)::*] $($new_state:expr),+ => $ctx:expr
        $(; $fmt:expr $(, $arg:expr)* )?
    ) => { loop {
        #[allow(redundant_semicolons, unused)] {
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
#[macro_export]
macro_rules! delta_snapshot {
    (
        use result type $result_type:ty;
        [$($origin:ident)::*] $($new_state:expr),+ => $ctx:expr
        $(; $fmt:expr $(, $arg:expr)* )?
    ) => { loop {
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
        break Ok(()) as $result_type
    }}
}

#[cfg(not(feature = "snapshot"))]
#[macro_export]
macro_rules! delta_snapshot {
    (
        use result type $result_type:ty;
        [$($origin:ident)::*] $($new_state:expr),+ => $ctx:expr
        $(; $fmt:expr $(, $arg:expr)* )?
    ) => { loop {
        #[allow(redundant_semicolons, unused)] {
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

#[cfg(feature = "snapshot")] pub mod delta;
#[cfg(feature = "snapshot")] pub mod full;

#[cfg(feature = "snapshot")] use crate::core::Core;
#[cfg(feature = "snapshot")] pub use crate::snapshot::delta::*;
#[cfg(feature = "snapshot")] pub use crate::snapshot::full::*;

#[cfg(feature = "snapshot")]
pub trait SnapshotCtx<T>
where T: Core + Default {
    // NOTE: Ideally `Self::History` would be a GAT defined ± as follows:
    //       ```
    //           type History<S>: Default = FullSnapshots<S>;
    //       ```
    //       Note the absence of `S` in the type level generic parameter set.
    //       Defining it as above would have the effect that `History` merely
    //       becomes a way to specify whether to use delta's or not, while `T`
    //       becomes the sole way to specify what the snapshot's state type is.
    type History: Default;

    fn history(&mut self) -> &mut Self::History;

    #[inline]
    fn take_history(&mut self) -> Self::History {
        std::mem::take(self.history())
    }
}

#[cfg(all(test, feature = "snapshot"))]
mod tests {
    use crate::snapshot::delta::DeltaSnapshots;
    use crate::snapshot::SnapshotCtx;
    use crate::DeltaResult;

    struct Ctx {
        history: DeltaSnapshots<u64>,
    }

    impl SnapshotCtx<u64> for Ctx {
        type History = DeltaSnapshots<u64>;

        fn history(&mut self) -> &mut Self::History { &mut self.history }
    }

    #[test]
    fn delta_snapshot__macro__records_one_snapshot() -> DeltaResult<()> {
        let mut ctx = Ctx { history: DeltaSnapshots::<u64>::default() };
        let _: DeltaResult<()> =
            delta_snapshot!(use result type DeltaResult<()>; [test] &5u64 => &mut ctx);
        assert_eq!(ctx.history.len(), 1);
        assert_eq!(ctx.history.current().state, 5);
        Ok(())
    }
}

