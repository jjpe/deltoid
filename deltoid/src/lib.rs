//!

// TODO:
// Can a delta be applied to a value of:
//   + a slice type  e.g. &[T]  and  &str?    (Very unlikely for borrowed types)

#[macro_use] pub mod error;
#[macro_use] pub mod snapshot;
pub mod core;

pub mod arrays;
pub mod borrow;
pub mod boxed;
pub mod collections;
pub mod option;
pub mod range;
pub mod result;
pub mod rc;
pub mod string;
pub mod sync;
pub mod tuple;
pub mod vec;


pub use crate::core::*;
pub use crate::borrow::CowDelta;
pub use crate::boxed::*;
pub use crate::collections::*;
pub use crate::error::{DeltaError, DeltaResult};
pub use crate::option::OptionDelta;
pub use crate::range::RangeDelta;
pub use crate::rc::*;
pub use crate::string::{Str, StringDelta};
pub use crate::sync::*;
pub use crate::tuple::*;
pub use crate::vec::{EltDelta, VecDelta};
