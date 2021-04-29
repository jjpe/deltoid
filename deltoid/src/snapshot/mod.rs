//! Snapshotting infrastructure.

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
