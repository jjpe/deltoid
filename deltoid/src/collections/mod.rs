pub mod btreemap;
pub mod btreeset;
pub mod hashmap;
pub mod hashset;
pub mod vecdeque;

pub use btreemap::{BTreeMapDelta, EntryDelta as BTreeMapEntryDelta};
pub use btreeset::{BTreeSetDelta, EntryDelta as BTreeSetEntryDelta};
pub use hashmap::{HashMapDelta, EntryDelta as HashMapEntryDelta};
pub use hashset::{HashSetDelta, EntryDelta as HashSetEntryDelta};
pub use vecdeque::{VecDequeDelta, EltDelta as VecDequeEltDelta};
