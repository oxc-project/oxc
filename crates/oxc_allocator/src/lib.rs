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
//! * `allocator_api` - Enables the nightly Rust `allocator_api` feature via dependencies.
//!   This feature enables `allocator-api2/nightly`, `hashbrown/nightly`, and `bumpalo/allocator_api`.
//!   Note that enabling this feature will require all other crates in your dependency tree
//!   that use these libraries to be compatible with their nightly features. Only enable this
//!   if you are certain all dependencies are compatible, otherwise you'd get a compile error.

#![warn(missing_docs)]
#![cfg_attr(feature = "allocator_api", feature(allocator_api))]

mod address;
mod allocator;
mod allocator_api2;
mod boxed;
mod clone_in;
mod convert;
#[cfg(feature = "from_raw_parts")]
mod from_raw_parts;
pub mod hash_map;
pub mod string;
mod take_in;
mod vec;
mod vec2;

pub use address::{Address, GetAddress};
pub use allocator::Allocator;
pub use boxed::Box;
pub use clone_in::CloneIn;
pub use convert::{FromIn, IntoIn};
pub use hash_map::HashMap;
pub use string::String;
pub use take_in::{Dummy, TakeIn};
pub use vec::Vec;
