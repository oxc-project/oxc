//! # ⚓ Oxc Memory Allocator
//!
//! Oxc uses a bump-based memory arena for faster AST allocations. This crate
//! contains an [`Allocator`] for creating such arenas, as well as ports of
//! memory management data types from `std` adapted to use this arena.
//!
//! ## No `Drop`s
//! Objects allocated into oxc memory arenas are never [`Dropped`](Drop), making
//! it relatively easy to leak memory if you're not careful. Memory is released
//! in bulk when the allocator is dropped.
//!
//! ## Examples
//! ```
//! use oxc_allocator::{Allocator, Box};
//!
//! struct Foo {
//!     pub a: i32
//! }
//! impl std::ops::Drop for Foo {
//!     fn drop(&mut self) {
//!         // Arena boxes are never dropped.
//!         unreachable!();
//!     }
//! }
//!
//! let allocator = Allocator::default();
//! let foo = Box::new_in(Foo { a: 0 }, &allocator);
//! drop(foo);
//! ```
//!
//! Consumers of the [`oxc` umbrella crate](https://crates.io/crates/oxc) pass
//! [`Allocator`] references to other tools.
//!
//! ```
//! use oxc::{allocator::Allocator, parser::Parser, span::SourceType};
//!
//! let allocator = Allocator::default()
//! let parsed = Parser::new(&allocator, "let x = 1;", SourceType::default());
//! assert!(parsed.errors.is_empty());
//! ```
#![warn(missing_docs)]
use std::{
    convert::From,
    ops::{Deref, DerefMut},
};

pub use bumpalo::collections::String;
use bumpalo::Bump;

mod boxed;
mod clone_in;
mod convert;
mod vec;

pub use boxed::{Address, Box};
pub use clone_in::CloneIn;
pub use convert::{FromIn, IntoIn};
pub use vec::Vec;

/// A bump-allocated memory arena based on [bumpalo].
///
/// ## No `Drop`s
///
/// Objects that are bump-allocated will never have their [`Drop`] implementation
/// called &mdash; unless you do it manually yourself. This makes it relatively
/// easy to leak memory or other resources.
#[derive(Default)]
pub struct Allocator {
    bump: Bump,
}

impl From<Bump> for Allocator {
    fn from(bump: Bump) -> Self {
        Self { bump }
    }
}

impl Deref for Allocator {
    type Target = Bump;

    fn deref(&self) -> &Self::Target {
        &self.bump
    }
}

impl DerefMut for Allocator {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.bump
    }
}

#[cfg(test)]
mod test {
    use std::ops::Deref;

    use bumpalo::Bump;

    use crate::Allocator;

    #[test]
    fn test_api() {
        let bump = Bump::new();
        let allocator: Allocator = bump.into();
        #[allow(clippy::explicit_deref_methods)]
        {
            _ = allocator.deref();
        }
    }
}
