//! # ⚓ Oxc Memory Allocator
//!
//! Oxc uses a bump-based memory arena for faster AST allocations. This crate
//! contains an [`Allocator`] for creating such arenas, as well as ports of
//! memory management data types from `std` adapted to use this arena.
//!
//! ## No `Drop`s
//!
//! Objects allocated into Oxc memory arenas are never [`Dropped`](Drop).
//! Memory is released in bulk when the allocator is dropped, without dropping the individual
//! objects in the arena.
//!
//! Therefore, it would produce a memory leak if you allocated [`Drop`] types into the arena
//! which own memory allocations outside the arena.
//!
//! Static checks make this impossible to do. [`Allocator::alloc`], [`Box::new_in`], [`Vec::new_in`],
//! and all other methods which store data in the arena will refuse to compile if called with
//! a [`Drop`] type.
//!
//! ## Examples
//!
//! ```ignore
//! use oxc_allocator::{Allocator, Box};
//!
//! struct Foo {
//!     pub a: i32
//! }
//!
//! impl std::ops::Drop for Foo {
//!     fn drop(&mut self) {}
//! }
//!
//! struct Bar {
//!     v: std::vec::Vec<u8>,
//! }
//!
//! let allocator = Allocator::default();
//!
//! // This will fail to compile because `Foo` implements `Drop`
//! let foo = Box::new_in(Foo { a: 0 }, &allocator);
//! // This will fail to compile because `Bar` contains a `std::vec::Vec`, and it implements `Drop`
//! let bar = Box::new_in(Bar { v: vec![1, 2, 3] }, &allocator);
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

use std::mem::needs_drop;

use bumpalo::Bump;

mod address;
mod allocator_api2;
mod boxed;
mod clone_in;
mod convert;
pub mod hash_map;
pub mod string;
mod vec;

pub use address::{Address, GetAddress};
pub use boxed::Box;
pub use clone_in::CloneIn;
pub use convert::{FromIn, IntoIn};
pub use hash_map::HashMap;
pub use string::String;
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

impl Allocator {
    /// Allocate an object in this [`Allocator`] and return an exclusive reference to it.
    ///
    /// # Panics
    /// Panics if reserving space for `T` fails.
    ///
    /// # Example
    /// ```
    /// use oxc_allocator::Allocator;
    ///
    /// let allocator = Allocator::default();
    /// let x = allocator.alloc([1u8; 20]);
    /// assert_eq!(x, &[1u8; 20]);
    /// ```
    //
    // `#[inline(always)]` because this is a very hot path and `Bump::alloc` is a very small function.
    // We always want it to be inlined.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub fn alloc<T>(&self, val: T) -> &mut T {
        const {
            assert!(!needs_drop::<T>(), "Cannot allocate Drop type in arena");
        }

        self.bump.alloc(val)
    }

    /// Copy a string slice into this [`Allocator`] and return a reference to it.
    ///
    /// # Panics
    /// Panics if reserving space for the string fails.
    ///
    /// # Example
    /// ```
    /// use oxc_allocator::Allocator;
    /// let allocator = Allocator::default();
    /// let hello = allocator.alloc_str("hello world");
    /// assert_eq!(hello, "hello world");
    /// ```
    //
    // `#[inline(always)]` because this is a hot path and `Bump::alloc_str` is a very small function.
    // We always want it to be inlined.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub fn alloc_str<'alloc>(&'alloc self, src: &str) -> &'alloc mut str {
        self.bump.alloc_str(src)
    }

    /// Reset this allocator.
    ///
    /// Performs mass deallocation on everything allocated in this arena by resetting the pointer
    /// into the underlying chunk of memory to the start of the chunk.
    /// Does not run any `Drop` implementations on deallocated objects.
    ///
    /// If this arena has allocated multiple chunks to bump allocate into, then the excess chunks
    /// are returned to the global allocator.
    ///
    /// ## Example
    ///
    /// ```
    /// use oxc_allocator::Allocator;
    ///
    /// let mut allocator = Allocator::default();
    ///
    /// // Allocate a bunch of things.
    /// {
    ///     for i in 0..100 {
    ///         allocator.alloc(i);
    ///     }
    /// }
    ///
    /// // Reset the arena.
    /// allocator.reset();
    ///
    /// // Allocate some new things in the space previously occupied by the
    /// // original things.
    /// for j in 200..400 {
    ///     allocator.alloc(j);
    /// }
    /// ```
    //
    // `#[inline(always)]` because it just delegates to `bumpalo`
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub fn reset(&mut self) {
        self.bump.reset();
    }

    /// Get inner [`bumpalo::Bump`].
    ///
    /// This method is not public. We don't want to expose `bumpalo::Allocator` to user.
    /// The fact that we're using `bumpalo` is an internal implementation detail.
    //
    // `#[inline(always)]` because it's a no-op
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub(crate) fn bump(&self) -> &Bump {
        &self.bump
    }
}

/// SAFETY: Not actually safe, but for enabling `Send` for downstream crates.
unsafe impl Send for Allocator {}
/// SAFETY: Not actually safe, but for enabling `Sync` for downstream crates.
unsafe impl Sync for Allocator {}

#[cfg(test)]
mod test {
    use crate::Allocator;

    #[test]
    fn test_api() {
        let mut allocator = Allocator::default();
        {
            let array = allocator.alloc([123; 10]);
            assert_eq!(array, &[123; 10]);
            let str = allocator.alloc_str("hello");
            assert_eq!(str, "hello");
        }
        allocator.reset();
    }
}
