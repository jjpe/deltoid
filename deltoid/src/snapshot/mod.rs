//! Snapshotting infrastructure.

#[cfg(feature = "snapshot")] mod core;
#[cfg(feature = "snapshot")] mod delta;
#[cfg(feature = "snapshot")] mod full;

#[cfg(feature = "snapshot")] pub use crate::snapshot::delta::*;
#[cfg(feature = "snapshot")] pub use crate::snapshot::full::*;
