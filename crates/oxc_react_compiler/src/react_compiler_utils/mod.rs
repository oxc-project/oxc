pub mod disjoint_set;

pub use disjoint_set::DisjointSet;

/// `IndexMap` keyed with the fast `FxHasher` (rustc-hash) instead of the default SipHash.
pub type FxIndexMap<K, V> = indexmap::IndexMap<K, V, rustc_hash::FxBuildHasher>;
/// `IndexSet` keyed with the fast `FxHasher` (rustc-hash) instead of the default SipHash.
pub type FxIndexSet<T> = indexmap::IndexSet<T, rustc_hash::FxBuildHasher>;
