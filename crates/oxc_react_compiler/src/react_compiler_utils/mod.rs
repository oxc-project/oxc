use indexmap::{IndexMap, IndexSet};
use rustc_hash::FxBuildHasher;

pub mod disjoint_set;
pub mod ordered_map;

pub use disjoint_set::DisjointSet;
pub use ordered_map::{OrderedMap, OrderedSet};

/// `IndexMap` keyed with the fast `FxHasher` (rustc-hash) instead of the default SipHash.
pub type FxIndexMap<K, V> = IndexMap<K, V, FxBuildHasher>;
/// `IndexSet` keyed with the fast `FxHasher` (rustc-hash) instead of the default SipHash.
pub type FxIndexSet<T> = IndexSet<T, FxBuildHasher>;
/// `IndexMap` keyed by [`oxc_str::Ident`], reusing its precomputed hash.
pub type IdentIndexMap<'a, V> = IndexMap<oxc_str::Ident<'a>, V, oxc_str::IdentBuildHasher>;
