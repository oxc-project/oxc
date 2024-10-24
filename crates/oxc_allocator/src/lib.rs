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
//! ```ignore
//! use oxc::{allocator::Allocator, parser::Parser, span::SourceType};
//!
//! let allocator = Allocator::default();
//! let parsed = Parser::new(&allocator, "let x = 1;", SourceType::default());
//! assert!(parsed.errors.is_empty());
//! ```
#![warn(missing_docs)]
use std::{
    convert::From,
    ops::{Deref, DerefMut},
};

use allocator_api2::alloc::Global;

/// A bump-allocated string.
type BumpScope<'a> = bump_scope::BumpScope<'a>;

mod address;
mod boxed;
mod clone_in;
mod convert;
mod string;
pub mod vec;

pub use address::{Address, GetAddress};
pub use boxed::Box;
pub use clone_in::CloneIn;
pub use convert::{FromIn, IntoIn};
pub use string::String;
pub use vec::Vec;

const BUMP_UPWARDS: bool = true;
const MINIMUM_ALIGNMENT: usize = 1;

type BumpImpl = bump_scope::Bump<Global, MINIMUM_ALIGNMENT, BUMP_UPWARDS>;
type VecImpl<'a, T> = bump_scope::BumpVec<'a, 'a, T, Global, MINIMUM_ALIGNMENT, BUMP_UPWARDS>;
type StringImpl<'a> = bump_scope::BumpString<'a, 'a, Global, MINIMUM_ALIGNMENT, BUMP_UPWARDS>;

/// A bump-allocated memory arena based on [bump-scope].
///
/// ## No `Drop`s
///
/// Objects that are bump-allocated will never have their [`Drop`] implementation
/// called &mdash; unless you do it manually yourself. This makes it relatively
/// easy to leak memory or other resources.
#[derive(Default)]
pub struct Allocator {
    bump: BumpImpl,
}

impl<'a> From<&'a Allocator> for &'a BumpScope<'a> {
    fn from(value: &'a Allocator) -> Self {
        value.bump.as_scope()
    }
}

impl From<BumpImpl> for Allocator {
    fn from(bump: BumpImpl) -> Self {
        Self { bump }
    }
}

impl Deref for Allocator {
    type Target = BumpImpl;

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

    use super::BumpImpl;

    use crate::Allocator;

    #[test]
    fn test_api() {
        let bump = BumpImpl::new();
        let allocator: Allocator = bump.into();
        #[allow(clippy::explicit_deref_methods)]
        {
            _ = allocator.deref();
        }
    }
}
