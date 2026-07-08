//! Struct-of-arrays vectors.
//!
//! The public interface is the [`multi_vec!`] macro. From one struct declaration it generates
//! a named table type which stores the struct's fields as struct-of-arrays, with typed-index
//! access, reference and slice views, iterators, and per-field accessors. See the macro's docs.
//!
//! Everything else in this module is internal machinery, reachable only through the macro's
//! expansion (via `__private`). It divides into three layers:
//!
//! # Storage
//!
//! Generic code which manages memory but never knows the field types - only their layouts.
//!
//! * [`MultiVec`] (`vec` module) - the vector, generic over an index type `I` and a fields
//!   type `F`. Owns the elements and the allocation, and provides the API the generated table
//!   forwards to (`push`, `get`, views, iteration, clone, drop).
//! * `Columns<F>` (`columns`) - the single allocation and its geometry: `(base_ptr, len,
//!   capacity)`, plus the pointer arithmetic to locate any field of any element.
//!   Owned by `MultiVec`, snapshotted by the iterators.
//! * `Shape` (`shape`) - the layout constants that arithmetic uses (alignment, element size,
//!   field offsets), precomputed at compile time from the field types' `Layout`s.
//!
//! # Typing
//!
//! The boundary where bytes acquire types. The macro implements these traits on the element
//! struct, and they are the only code which casts the untyped field pointers to real types.
//!
//! * `Fields` (`fields`) - reads, writes, drops, and per-element views. Pure casts, no logic.
//! * `SliceFields` (`fields`) - the whole-column slice views. Split from `Fields` because
//!   they also name the index type.
//! * `CloneFields` (`fields`) - column-by-column cloning, only for `#[derive(Clone)]` tables.
//!
//! # Support
//!
//! * `iter` - `Iter` / `IterMut` / `IntoIter`, all wrapping one column-walking core.
//! * `clone` - the per-column clone machinery, with panic-safety drop guards.
//! * `utils` - small unsafe helpers called from the macro's expansion.
//!
//! All logic containing unsafe code lives in these modules, written once as ordinary generic
//! Rust. The macro's expansion contains almost none - the `Fields`-family impls it generates
//! are pure pointer casts and calls to the helpers here, with no arithmetic or control flow.
//!
//! # Thread safety
//!
//! Tables and iterators are `Send` / `Sync` exactly when the field types are.
//! The manual impls (in `vec` and `iter`) bound on the fields type `F` - not on `I`,
//! and not on the item types - see their SAFETY comments for why.
//! Both directions are checked in `tests.rs`. The doctests here illustrate the negative direction.
//!
//! A table of non-`Sync` fields is not `Sync`:
//!
//! ```compile_fail,E0277
//! # use std::cell::Cell;
//! # use oxc_data_structures::multi_vec::multi_vec;
//! # oxc_index::define_index_type! { struct Id = u32; }
//! multi_vec! {
//!     table CellTable<Id, CellItem>;
//!
//!     struct CellItem {
//!         value: Cell<u32>,
//!     }
//! }
//!
//! // Error: `Cell<u32>` cannot be shared between threads
//! fn requires_sync<T: Sync>(_: &T) {}
//! requires_sync(&CellTable::new());
//! ```
//!
//! and its borrowing iterator is not `Send` - sending it would give another thread shared
//! access to the stored values:
//!
//! ```compile_fail,E0277
//! # use std::cell::Cell;
//! # use oxc_data_structures::multi_vec::multi_vec;
//! # oxc_index::define_index_type! { struct Id = u32; }
//! multi_vec! {
//!     table CellTable<Id, CellItem>;
//!
//!     struct CellItem {
//!         value: Cell<u32>,
//!     }
//! }
//!
//! // Error: `Cell<u32>` cannot be shared between threads
//! fn requires_send<T: Send>(_: T) {}
//! requires_send(CellTable::new().iter());
//! ```

mod clone;
mod columns;
mod fields;
mod iter;
mod macros;
mod shape;
mod utils;
mod vec;

use vec::MultiVec;

pub use macros::multi_vec;

/// Not public API. Referenced by the expansion of the [`multi_vec!`] macro.
#[doc(hidden)]
pub mod __private {
    pub use std::alloc::Layout;

    pub use oxc_index::IndexSlice;
    pub use pastey::paste;

    pub use super::{
        clone::{SrcAndDstPtrs, clone_column},
        fields::{CloneFields, Fields, SliceFields},
        iter::{IntoIter, Iter, IterMut},
        shape::Shape,
        utils::{drop_column, index_slice_from_raw_parts, index_slice_from_raw_parts_mut},
        vec::MultiVec,
    };
}

#[cfg(test)]
mod tests;
