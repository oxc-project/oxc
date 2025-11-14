//! # ⚓ Oxc Memory Allocator
//!
//! Oxc uses a bump-based memory arena for faster AST allocations.
//!
//! This crate contains an [`Allocator`] for creating such arenas, as well as ports of data types
//! from `std` adapted to use this arena:
//!
//! * [`Box`]
//! * [`Vec`]
//! * [`String`]
//! * [`HashMap`]
//! * [`HashSet`]
//!
//! See [`Allocator`] docs for information on efficient use of [`Allocator`].
//!
//! ## Features
//!
//! * `serialize` - Enables serialization support for [`Box`] and [`Vec`] with `serde` and `oxc_estree`.
//!
//! * `pool` - Enables [`AllocatorPool`].
//!
//! * `bitset` - Enables [`BitSet`].
//!
//! * `from_raw_parts` - Adds [`Allocator::from_raw_parts`] method.
//!   Usage of this feature is not advisable, and it will be removed as soon as we're able to.
//!
//! * `fixed_size` - Makes [`AllocatorPool`] create large fixed-size allocators, instead of
//!   flexibly-sized ones.
//!   Only supported on 64-bit little-endian platforms at present.
//!   Usage of this feature is not advisable, and it will be removed as soon as we're able to.
//!
//! * `track_allocations` - Count allocations and reallocations.
//!   For internal use only. The APIs provided by this feature are sketchy at best, and possibly
//!   undefined behavior. Do not enable this feature under any circumstances in production code.
//!
//! * `disable_track_allocations` - Disables `track_allocations` feature.
//!   Purpose is to prevent `--all-features` enabling allocation tracking.

#![warn(missing_docs)]

mod accessor;
mod address;
mod alloc;
mod allocator;
mod allocator_api2;
#[cfg(feature = "bitset")]
mod bitset;
mod boxed;
mod clone_in;
mod convert;
#[cfg(feature = "from_raw_parts")]
mod from_raw_parts;
pub mod hash_map;
pub mod hash_set;
#[cfg(feature = "pool")]
mod pool;
mod string_builder;
mod take_in;
#[cfg(all(feature = "track_allocations", not(feature = "disable_track_allocations")))]
mod tracking;
mod vec;
mod vec2;

pub use accessor::AllocatorAccessor;
pub use address::{Address, GetAddress, UnstableAddress};
pub use allocator::Allocator;
#[cfg(feature = "bitset")]
pub use bitset::BitSet;
pub use boxed::Box;
pub use clone_in::CloneIn;
pub use convert::{FromIn, IntoIn};
pub use hash_map::HashMap;
pub use hash_set::HashSet;
#[cfg(feature = "pool")]
pub use pool::*;
pub use string_builder::StringBuilder;
pub use take_in::{Dummy, TakeIn};
pub use vec::Vec;

// Fixed size allocators are only supported on 64-bit little-endian platforms at present.
//
// Note: Importing the `fixed_size_constants` module would cause a compilation error on 32-bit systems.
#[cfg(all(feature = "fixed_size", target_pointer_width = "64", target_endian = "little"))]
mod generated {
    #[cfg(debug_assertions)]
    mod assert_layouts;
    pub mod fixed_size_constants;
}
