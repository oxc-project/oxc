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
    /// Create a new [`Allocator`] with no initial capacity.
    ///
    /// This method does not reserve any memory to back the allocator. Memory for allocator's initial
    /// chunk will be reserved lazily, when you make the first allocation into this [`Allocator`]
    /// (e.g. with [`Allocator::alloc`], [`Box::new_in`], [`Vec::new_in`], [`HashMap::new_in`]).
    ///
    /// If you can estimate the amount of memory the allocator will require to fit what you intend to
    /// allocate into it, it is generally preferable to create that allocator with [`with_capacity`]
    /// which reserves that amount of memory upfront. This will avoid further system calls to allocate
    /// further chunks later on.
    ///
    /// [`with_capacity`]: Allocator::with_capacity
    //
    // `#[inline(always)]` because just delegates to `bumpalo` method
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub fn new() -> Self {
        Self { bump: Bump::new() }
    }

    /// Create a new [`Allocator`] with specified capacity.
    //
    // `#[inline(always)]` because just delegates to `bumpalo` method
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub fn with_capacity(capacity: usize) -> Self {
        Self { bump: Bump::with_capacity(capacity) }
    }

    /// Allocate an object in this [`Allocator`] and return an exclusive reference to it.
    ///
    /// # Panics
    /// Panics if reserving space for `T` fails.
    ///
    /// # Examples
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
    /// # Examples
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
    /// # Examples
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

    /// Calculate the total capacity of this [`Allocator`] including all chunks, in bytes.
    ///
    /// Note: This is the total amount of memory the [`Allocator`] owns NOT the total size of data
    /// that's been allocated in it. If you want the latter, use [`used_bytes`] instead.
    ///
    /// # Examples
    /// ```
    /// use oxc_allocator::Allocator;
    ///
    /// let capacity = 64 * 1024; // 64 KiB
    /// let mut allocator = Allocator::with_capacity(capacity);
    /// allocator.alloc(123u64); // 8 bytes
    ///
    /// // Result is the capacity (64 KiB), not the size of allocated data (8 bytes).
    /// // `Allocator::with_capacity` may allocate a bit more than requested.
    /// assert!(allocator.capacity() >= capacity);
    /// ```
    ///
    /// [`used_bytes`]: Allocator::used_bytes
    //
    // `#[inline(always)]` because it just delegates to `bumpalo`
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.bump.allocated_bytes()
    }

    /// Calculate the total size of data used in this [`Allocator`], in bytes.
    ///
    /// This is the total amount of memory that has been *used* in the [`Allocator`], NOT the amount of
    /// memory the [`Allocator`] owns. If you want the latter, use [`capacity`] instead.
    ///
    /// The result includes:
    ///
    /// 1. Padding bytes between objects which have been allocated to preserve alignment of types
    ///    where they have different alignments or have larger-than-typical alignment.
    /// 2. Excess capacity in [`Vec`]s, [`String`]s and [`HashMap`]s.
    /// 3. Objects which were allocated but later dropped. [`Allocator`] does not re-use allocations,
    ///    so anything which is allocated into arena continues to take up "dead space", even after it's
    ///    no longer referenced anywhere.
    /// 4. "Dead space" left over where a [`Vec`], [`String`] or [`HashMap`] has grown and had to make
    ///    a new allocation to accommodate its new larger size. Its old allocation continues to take up
    ///    "dead" space in the allocator, unless it was the most recent allocation.
    ///
    /// In practice, this almost always means that the result returned from this function will be an
    /// over-estimate vs the amount of "live" data in the arena.
    ///
    /// However, if you are using the result of this method to create a new `Allocator` to clone
    /// an AST into, it is theoretically possible (though very unlikely) that it may be a slight
    /// under-estimate of the capacity required in new allocator to clone the AST into, depending
    /// on the order that `&str`s were allocated into arena in parser vs the order they get allocated
    /// during cloning. The order allocations are made in affects the amount of padding bytes required.
    ///
    /// # Examples
    /// ```
    /// use oxc_allocator::{Allocator, Vec};
    ///
    /// let capacity = 64 * 1024; // 64 KiB
    /// let mut allocator = Allocator::with_capacity(capacity);
    ///
    /// allocator.alloc(1u8); // 1 byte with alignment 1
    /// allocator.alloc(2u8); // 1 byte with alignment 1
    /// allocator.alloc(3u64); // 8 bytes with alignment 8
    ///
    /// // Only 10 bytes were allocated, but 16 bytes were used, in order to align `3u64` on 8
    /// assert_eq!(allocator.used_bytes(), 16);
    ///
    /// allocator.reset();
    ///
    /// let mut vec = Vec::<u64>::with_capacity_in(2, &allocator);
    ///
    /// // Allocate something else, so `vec`'s allocation is not the most recent
    /// allocator.alloc(123u64);
    ///
    /// // `vec` has to grow beyond it's initial capacity
    /// vec.extend([1, 2, 3, 4]);
    ///
    /// // `vec` takes up 32 bytes, and `123u64` takes up 8 bytes = 40 total.
    /// // But there's an additional 16 bytes consumed for `vec`'s original capacity of 2,
    /// // which is still using up space
    /// assert_eq!(allocator.used_bytes(), 56);
    /// ```
    ///
    /// [`capacity`]: Allocator::capacity
    pub fn used_bytes(&self) -> usize {
        let mut bytes = 0;
        // SAFETY: No allocations are made while `chunks_iter` is alive. No data is read from the chunks.
        let chunks_iter = unsafe { self.bump.iter_allocated_chunks_raw() };
        for (_, size) in chunks_iter {
            bytes += size;
        }
        bytes
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
