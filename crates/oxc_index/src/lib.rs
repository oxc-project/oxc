//! This crate is a fork of `index_vec`, <https://github.com/thomcc/index_vec>
//! It helps with defining "newtype"-style wrappers around `usize` (or
//! other integers), `Vec<T>`, and `[T]` so that some additional type safety can
//! be gained at zero cost.
//!
//! ## Example / Overview
//! ```rust
//! use index_vec::{IndexVec, IndexSlice, index_vec};
//!
//! index_vec::define_index_type! {
//!     // Define StrIdx to use only 32 bits internally (you can use usize, u16,
//!     // and even u8).
//!     pub struct StrIdx = u32;
//!
//!     // The defaults are very reasonable, but this macro can let
//!     // you customize things quite a bit:
//!
//!     // By default, creating a StrIdx would check an incoming `usize against
//!     // `u32::max_value()`, as u32 is the wrapped index type. Lets imagine that
//!     // StrIdx has to interface with an external system that uses signed ints.
//!     // We can change the checking behavior to complain on i32::max_value()
//!     // instead:
//!     MAX_INDEX = i32::max_value() as usize;
//!
//!     // We can also disable checking all-together if we are more concerned with perf
//!     // than any overflow problems, or even do so, but only for debug builds: Quite
//!     // pointless here, but an okay example
//!     DISABLE_MAX_INDEX_CHECK = cfg!(not(debug_assertions));
//!
//!     // And more too, see this macro's docs for more info.
//! }
//!
//! // Create a vector which can be accessed using `StrIdx`s.
//! let mut strs: IndexVec<StrIdx, &'static str> = index_vec!["strs", "bar", "baz"];
//!
//! // l is a `StrIdx`
//! let l = strs.last_idx();
//! assert_eq!(strs[l], "baz");
//!
//! let new_i = strs.push("quux");
//! assert_eq!(strs[new_i], "quux");
//!
//! // The slice APIs are wrapped as well.
//! let s: &IndexSlice<StrIdx, [&'static str]> = &strs[StrIdx::new(1)..];
//! assert_eq!(s[0], "bar");
//!
//! // Indices are mostly interoperable with `usize`, and support
//! // a lot of what you might want to do to an index.
//!
//! // Comparison
//! assert_eq!(StrIdx::new(0), 0usize);
//!
//! // Addition
//! assert_eq!(StrIdx::new(0) + 1, 1usize);
//!
//! // Subtraction
//! assert_eq!(StrIdx::new(1) - 1, 0usize);
//!
//! // Wrapping
//! assert_eq!(StrIdx::new(5) % strs.len(), 1usize);
//! // ...
//! ```
//! ## Background
//!
//! The goal is to help with the pattern of using a `type FooIdx = usize` to
//! access a `Vec<Foo>` with something that can statically prevent using a
//! `FooIdx` in a `Vec<Bar>`. It's most useful if you have a bunch of indices
//! referring to different sorts of vectors.
//!
//! The code was originally based on `rustc`'s `IndexVec` code, however that has
//! been almost entirely rewritten (except for the cases where it's trivial,
//! e.g. the Vec wrapper).
//!
//! ## Other crates
//!
//! The [`indexed_vec`](https://crates.io/crates/indexed_vec) crate predates
//! this, and is a much closer copy of the code from `rustc`. Unfortunately,
//! this means it does not compile on stable.
//!
//! If you're looking for something further from a vec and closer to a map, you
//! might find [`handy`](https://crates.io/crates/handy),
//! [`slotmap`](https://crates.io/crates/slotmap), or
//! [`slab`](https://crates.io/crates/slab) to be closer what you want.
//!
//! ## FAQ
//!
//! #### Wouldn't `define_index_type` be better as a proc macro?
//!
//! Probably. It's not a proc macro because I tend to avoid them where possible
//! due to wanting to minimize compile times. If the issues around proc-macro
//! compile times are fixed, then I'll revisit this.
//!
//! I also may eventually add a proc-macro feature which is not required, but
//! avoids some of the grossness.
//!
//! #### Does `define_index_type` do too much?
//!
//! Possibly. It defines a type, implements a bunch of functions on it, and
//! quite a few traits. That said, it's intended to be a very painless journey
//! from `Vec<T>` + `usize` to `IndexVec<I, T>`. If it left it up to the
//! developer to do those things, it would be too annoying to be worth using.
//!
//! #### The syntax for the options in `define_index_type` is terrible.
//!
//! I'm open to suggestions.
//!
//! #### Does it support no_std?
//!
//! Yes, although it uses `extern crate alloc;`, of course.
//!
//! #### Does it support serde?
//!
//! Yes, but only if you turn on the `serialize` feature.
//!
//! #### What features are planned?
//!
//! Planned is a bit strong but here are the things I would find useful.
//!
//! - Support any remaining parts of the slice/vec api.
//! - Add typesafe wrappers for SmallVec/ArrayVec (behind a cargo `feature`, of
//!   course).
//! - Better syntax for the define_index_type macro (no concrete ideas).
//! - Allow the generated type to be a tuple struct, or use a specific field
//!   name.
//! - Allow use of indices for string types (the primary benefit here would
//!   probably be the ability to e.g. use u32 without too much pain rather than
//!   mixing up indices from different strings -- but you never know!)
//! - Allow index types such as NonZeroU32 and such, if it can be done sanely.
//! - ...
//!
#![allow(clippy::inline_always)]
#![allow(clippy::partialeq_ne_impl)]
#![no_std]
extern crate alloc;

use core::fmt::Debug;
use core::hash::Hash;

pub mod non_zero;
pub use non_zero::vec::NonZeroIndexVec;
pub use non_zero::NonZeroIdx;

pub mod indexing;
pub mod slice;
pub mod vec;

pub use indexing::{IdxRangeBounds, IdxSliceIndex};
pub use slice::{IndexBox, IndexSlice};
pub use vec::IndexVec;

#[macro_use]
mod macros;

#[cfg(any(test, feature = "example_generated"))]
pub mod example_generated;

/// Represents a wrapped value convertible to and from a `usize`.
///
/// Generally you implement this via the [`define_index_type!`] macro, rather
/// than manually implementing it.
///
/// # Overflow
///
/// `Idx` impls are allowed to be smaller than `usize`, which means converting
/// `usize` to an `Idx` implementation might have to handle overflow.
///
/// The way overflow is handled is up to the implementation of `Idx`, but it's
/// generally panicking, unless it was turned off via the
/// `DISABLE_MAX_INDEX_CHECK` option in [`define_index_type!`]. If you need more
/// subtle handling than this, then you're on your own (or, well, either handle
/// it earlier, or pick a bigger index type).
///
/// Note: I'm open for suggestions on how to handle this case, but do not want
/// the typical cases (E.g. Idx is a newtyped `usize` or `u32`), to become more
/// complex.
pub trait Idx: Copy + 'static + Ord + Debug + Hash {
    /// Construct an Index from a `usize`. This is equivalent to `From<usize>`.
    ///
    /// # Panics
    ///
    /// Note that this will panic if `idx` does not fit (unless checking has
    /// been disabled, as mentioned above). Also note that `Idx` implementations
    /// are free to define what "fit" means as they desire.
    fn from_usize(idx: usize) -> Self;

    /// Get the underlying index. This is equivalent to `Into<usize>`
    fn index(self) -> usize;
}
