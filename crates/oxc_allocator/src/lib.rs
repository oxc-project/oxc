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
//!
//! See [`Allocator`] docs for information on efficient use of [`Allocator`].
//!
//! ## Features
//!
//! * `serialize` - Enables serialization support for [`Box`] and [`Vec`] with `serde` and `oxc_estree`.
//!
//! * `from_raw_parts` - Adds [`Allocator::from_raw_parts`] method.
//!   Usage of this feature is not advisable, and it will be removed as soon as we're able to.
//!
//! * `fixed_size` - Makes [`AllocatorPool`] create large fixed-size allocators, instead of
//!   flexibly-sized ones.
//!   Only supported on 64-bit little-endian platforms at present.
//!   Usage of this feature is not advisable, and it will be removed as soon as we're able to.
//!
//! * `disable_fixed_size` - Disables `fixed_size` feature.
//!   Purpose is to prevent `--all-features` enabling fixed sized allocators.

#![warn(missing_docs)]

mod accessor;
mod address;
mod alloc;
mod allocator;
mod allocator_api2;
mod boxed;
mod clone_in;
mod convert;
#[cfg(feature = "from_raw_parts")]
mod from_raw_parts;
pub mod hash_map;
mod string_builder;
mod take_in;
mod vec;
mod vec2;

pub use accessor::AllocatorAccessor;
pub use address::{Address, GetAddress};
pub use allocator::Allocator;
pub use boxed::Box;
pub use clone_in::CloneIn;
pub use convert::{FromIn, IntoIn};
pub use hash_map::HashMap;
pub use string_builder::StringBuilder;
pub use take_in::{Dummy, TakeIn};
pub use vec::Vec;

// Fixed size allocators are only supported on 64-bit little-endian platforms at present

#[cfg(not(all(
    feature = "fixed_size",
    not(feature = "disable_fixed_size"),
    target_pointer_width = "64",
    target_endian = "little"
)))]
mod pool;

#[cfg(all(
    feature = "fixed_size",
    not(feature = "disable_fixed_size"),
    target_pointer_width = "64",
    target_endian = "little"
))]
mod pool_fixed_size;
#[cfg(all(
    feature = "fixed_size",
    not(feature = "disable_fixed_size"),
    target_pointer_width = "64",
    target_endian = "little"
))]
use pool_fixed_size as pool;
// Import here so `generated/assert_layouts.rs` can access it
#[cfg(all(
    feature = "fixed_size",
    not(feature = "disable_fixed_size"),
    target_pointer_width = "64",
    target_endian = "little"
))]
use pool_fixed_size::FixedSizeAllocatorMetadata;

pub use pool::{AllocatorGuard, AllocatorPool};

#[cfg(all(
    feature = "fixed_size",
    not(feature = "disable_fixed_size"),
    target_pointer_width = "64",
    target_endian = "little"
))]
mod generated {
    #[cfg(debug_assertions)]
    pub mod assert_layouts;
    pub mod fixed_size_constants;
}
#[cfg(all(
    feature = "fixed_size",
    not(feature = "disable_fixed_size"),
    target_pointer_width = "64",
    target_endian = "little"
))]
use generated::fixed_size_constants;
