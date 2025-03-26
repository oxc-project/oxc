// This file is copied from the [Bumpalo's Vec](https://github.com/fitzgen/bumpalo/blob/1d2fbea9e3d0c2be56367b9ad5382ff33852a188/src/collections/raw_vec.rs)

// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![allow(
    unused_mut,
    unused_unsafe,
    clippy::allow_attributes,
    clippy::uninlined_format_args,
    clippy::enum_glob_use,
    clippy::equatable_if_let,
    clippy::needless_pass_by_value,
    clippy::inline_always
)]
#![allow(unstable_name_collisions)]
#![allow(dead_code)]

use allocator_api2::alloc::{AllocError, Allocator};
use bumpalo::Bump;

pub use core::alloc::Layout;
use core::cmp;
use core::mem;
use core::ptr::{self, NonNull};

use bumpalo::collections::CollectionAllocErr::{self, AllocErr, CapacityOverflow};

// use boxed::Box;

/// A low-level utility for more ergonomically allocating, reallocating, and deallocating
/// a buffer of memory on the heap without having to worry about all the corner cases
/// involved. This type is excellent for building your own data structures like Vec and VecDeque.
/// In particular:
///
/// * Produces Unique::empty() on zero-sized types
/// * Produces Unique::empty() on zero-length allocations
/// * Catches all overflows in capacity computations (promotes them to "capacity overflow" panics)
/// * Guards against 32-bit systems allocating more than isize::MAX bytes
/// * Guards against overflowing your length
/// * Aborts on OOM
/// * Avoids freeing Unique::empty()
/// * Contains a ptr::Unique and thus endows the user with all related benefits
///
/// This type does not in anyway inspect the memory that it manages. When dropped it *will*
/// free its memory, but it *won't* try to Drop its contents. It is up to the user of RawVec
/// to handle the actual things *stored* inside of a RawVec.
///
/// Note that a RawVec always forces its capacity to be usize::MAX for zero-sized types.
/// This enables you to use capacity growing logic catch the overflows in your length
/// that might occur with zero-sized types.
///
/// However this means that you need to be careful when round-tripping this type
/// with a `Box<[T]>`: `cap()` won't yield the len. However `with_capacity`,
/// `shrink_to_fit`, and `from_box` will actually set RawVec's private capacity
/// field. This allows zero-sized types to not be special-cased by consumers of
/// this type.
#[allow(missing_debug_implementations)]
pub struct RawVec<'a, T> {
    ptr: NonNull<T>,
    cap: usize,
    a: &'a Bump,
}

impl<'a, T> RawVec<'a, T> {
    /// Like `new` but parameterized over the choice of allocator for
    /// the returned RawVec.
    pub fn new_in(a: &'a Bump) -> Self {
        // `cap: 0` means "unallocated". zero-sized types are ignored.
        RawVec { ptr: NonNull::dangling(), cap: 0, a }
    }

    /// Like `with_capacity` but parameterized over the choice of
    /// allocator for the returned RawVec.
    #[inline]
    pub fn with_capacity_in(cap: usize, a: &'a Bump) -> Self {
        RawVec::allocate_in(cap, false, a)
    }

    /// Like `with_capacity_zeroed` but parameterized over the choice
    /// of allocator for the returned RawVec.
    #[inline]
    pub fn with_capacity_zeroed_in(cap: usize, a: &'a Bump) -> Self {
        RawVec::allocate_in(cap, true, a)
    }

    fn allocate_in(cap: usize, zeroed: bool, mut a: &'a Bump) -> Self {
        unsafe {
            let elem_size = mem::size_of::<T>();

            let alloc_size = cap.checked_mul(elem_size).unwrap_or_else(|| capacity_overflow());
            alloc_guard(alloc_size).unwrap_or_else(|_| capacity_overflow());

            // handles ZSTs and `cap = 0` alike
            let ptr = if alloc_size == 0 {
                NonNull::<T>::dangling()
            } else {
                let align = mem::align_of::<T>();
                let layout = Layout::from_size_align(alloc_size, align).unwrap();
                let result = if zeroed { a.allocate_zeroed(layout) } else { a.allocate(layout) };
                match result {
                    Ok(ptr) => ptr.cast(),
                    Err(_) => handle_alloc_error(layout),
                }
            };

            RawVec { ptr, cap, a }
        }
    }
}

impl<'a, T> RawVec<'a, T> {
    /// Reconstitutes a RawVec from a pointer, capacity, and allocator.
    ///
    /// # Undefined Behavior
    ///
    /// The ptr must be allocated (via the given allocator `a`), and with the given capacity. The
    /// capacity cannot exceed `isize::MAX` (only a concern on 32-bit systems).
    /// If the ptr and capacity come from a RawVec created via `a`, then this is guaranteed.
    pub unsafe fn from_raw_parts_in(ptr: *mut T, cap: usize, a: &'a Bump) -> Self {
        RawVec { ptr: NonNull::new_unchecked(ptr), cap, a }
    }
}

impl<'a, T> RawVec<'a, T> {
    /// Gets a raw pointer to the start of the allocation. Note that this is
    /// Unique::empty() if `cap = 0` or T is zero-sized. In the former case, you must
    /// be careful.
    pub fn ptr(&self) -> *mut T {
        self.ptr.as_ptr()
    }

    /// Gets the capacity of the allocation.
    ///
    /// This will always be `usize::MAX` if `T` is zero-sized.
    #[inline(always)]
    pub fn cap(&self) -> usize {
        if mem::size_of::<T>() == 0 { !0 } else { self.cap }
    }

    /// Returns a shared reference to the allocator backing this RawVec.
    pub fn bump(&self) -> &'a Bump {
        self.a
    }

    fn current_layout(&self) -> Option<Layout> {
        if self.cap == 0 {
            None
        } else {
            // We have an allocated chunk of memory, so we can bypass runtime
            // checks to get our current layout.
            unsafe {
                let align = mem::align_of::<T>();
                let size = mem::size_of::<T>() * self.cap;
                Some(Layout::from_size_align_unchecked(size, align))
            }
        }
    }

    /*
    /// Doubles the size of the type's backing allocation. This is common enough
    /// to want to do that it's easiest to just have a dedicated method. Slightly
    /// more efficient logic can be provided for this than the general case.
    ///
    /// This function is ideal for when pushing elements one-at-a-time because
    /// you don't need to incur the costs of the more general computations
    /// reserve needs to do to guard against overflow. You do however need to
    /// manually check if your `len == cap`.
    ///
    /// # Panics
    ///
    /// * Panics if T is zero-sized on the assumption that you managed to exhaust
    ///   all `usize::MAX` slots in your imaginary buffer.
    /// * Panics on 32-bit platforms if the requested capacity exceeds
    ///   `isize::MAX` bytes.
    ///
    /// # Aborts
    ///
    /// Aborts on OOM
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # #![feature(alloc, raw_vec_internals)]
    /// # extern crate alloc;
    /// # use std::ptr;
    /// # use alloc::raw_vec::RawVec;
    /// struct MyVec<T> {
    ///     buf: RawVec<T>,
    ///     len: usize,
    /// }
    ///
    /// impl<T> MyVec<T> {
    ///     pub fn push(&mut self, elem: T) {
    ///         if self.len == self.buf.cap() { self.buf.double(); }
    ///         // double would have aborted or panicked if the len exceeded
    ///         // `isize::MAX` so this is safe to do unchecked now.
    ///         unsafe {
    ///             ptr::write(self.buf.ptr().add(self.len), elem);
    ///         }
    ///         self.len += 1;
    ///     }
    /// }
    /// # fn main() {
    /// #   let mut vec = MyVec { buf: RawVec::new(), len: 0 };
    /// #   vec.push(1);
    /// # }
    /// ```
    #[inline(never)]
    #[cold]
    pub fn double(&mut self) {
        unsafe {
            let elem_size = mem::size_of::<T>();

            // since we set the capacity to usize::MAX when elem_size is
            // 0, getting to here necessarily means the RawVec is overfull.
            assert!(elem_size != 0, "capacity overflow");

            let (new_cap, uniq) = match self.current_layout() {
                Some(cur) => {
                    // Since we guarantee that we never allocate more than
                    // isize::MAX bytes, `elem_size * self.cap <= isize::MAX` as
                    // a precondition, so this can't overflow. Additionally the
                    // alignment will never be too large as to "not be
                    // satisfiable", so `Layout::from_size_align` will always
                    // return `Some`.
                    //
                    // tl;dr; we bypass runtime checks due to dynamic assertions
                    // in this module, allowing us to use
                    // `from_size_align_unchecked`.
                    let new_cap = 2 * self.cap;
                    let new_size = new_cap * elem_size;
                    alloc_guard(new_size).unwrap_or_else(|_| capacity_overflow());
                    let ptr_res = self.a.realloc(self.ptr.cast(), cur, new_size);
                    match ptr_res {
                        Ok(ptr) => (new_cap, ptr.cast()),
                        Err(_) => handle_alloc_error(Layout::from_size_align_unchecked(
                            new_size,
                            cur.align(),
                        )),
                    }
                }
                None => {
                    // skip to 4 because tiny Vec's are dumb; but not if that
                    // would cause overflow
                    let new_cap = if elem_size > (!0) / 8 { 1 } else { 4 };
                    match self.a.alloc_array::<T>(new_cap) {
                        Ok(ptr) => (new_cap, ptr),
                        Err(_) => handle_alloc_error(Layout::array::<T>(new_cap).unwrap()),
                    }
                }
            };
            self.ptr = uniq;
            self.cap = new_cap;
        }
    }
    */

    /*
    /// Attempts to double the size of the type's backing allocation in place. This is common
    /// enough to want to do that it's easiest to just have a dedicated method. Slightly
    /// more efficient logic can be provided for this than the general case.
    ///
    /// Returns true if the reallocation attempt has succeeded, or false otherwise.
    ///
    /// # Panics
    ///
    /// * Panics if T is zero-sized on the assumption that you managed to exhaust
    ///   all `usize::MAX` slots in your imaginary buffer.
    /// * Panics on 32-bit platforms if the requested capacity exceeds
    ///   `isize::MAX` bytes.
    #[inline(never)]
    #[cold]
    pub fn double_in_place(&mut self) -> bool {
        unsafe {
            let elem_size = mem::size_of::<T>();
            let old_layout = match self.current_layout() {
                Some(layout) => layout,
                None => return false, // nothing to double
            };

            // since we set the capacity to usize::MAX when elem_size is
            // 0, getting to here necessarily means the RawVec is overfull.
            assert!(elem_size != 0, "capacity overflow");

            // Since we guarantee that we never allocate more than isize::MAX
            // bytes, `elem_size * self.cap <= isize::MAX` as a precondition, so
            // this can't overflow.
            //
            // Similarly like with `double` above we can go straight to
            // `Layout::from_size_align_unchecked` as we know this won't
            // overflow and the alignment is sufficiently small.
            let new_cap = 2 * self.cap;
            let new_size = new_cap * elem_size;
            alloc_guard(new_size).unwrap_or_else(|_| capacity_overflow());
            match self.a.grow_in_place(self.ptr.cast(), old_layout, new_size) {
                Ok(_) => {
                    // We can't directly divide `size`.
                    self.cap = new_cap;
                    true
                }
                Err(_) => false,
            }
        }
    }
    */

    /// The same as `reserve_exact`, but returns on errors instead of panicking or aborting.
    pub fn try_reserve_exact(
        &mut self,
        len: usize,
        additional: usize,
    ) -> Result<(), CollectionAllocErr> {
        if self.needs_to_grow(len, additional) {
            self.grow_exact(len, additional)?
        }

        Ok(())
    }

    /// Ensures that the buffer contains at least enough space to hold
    /// `len + additional` elements. If it doesn't already,
    /// will reallocate the minimum possible amount of memory necessary.
    /// Generally this will be exactly the amount of memory necessary,
    /// but in principle the allocator is free to give back more than
    /// we asked for.
    ///
    /// If `len` exceeds `self.cap()`, this may fail to actually allocate
    /// the requested space. This is not really unsafe, but the unsafe
    /// code *you* write that relies on the behavior of this function may break.
    ///
    /// # Panics
    ///
    /// * Panics if the requested capacity exceeds `usize::MAX` bytes.
    /// * Panics on 32-bit platforms if the requested capacity exceeds
    ///   `isize::MAX` bytes.
    ///
    /// # Aborts
    ///
    /// Aborts on OOM
    pub fn reserve_exact(&mut self, len: usize, additional: usize) {
        if let Err(err) = self.try_reserve_exact(len, additional) {
            handle_error(err)
        }
    }

    /// Calculates the buffer's new size given that it'll hold `len +
    /// additional` elements. This logic is used in amortized reserve methods.
    /// Returns `(new_capacity, new_alloc_size)`.
    fn amortized_new_size(
        &self,
        len: usize,
        additional: usize,
    ) -> Result<usize, CollectionAllocErr> {
        // Nothing we can really do about these checks :(
        let required_cap = len.checked_add(additional).ok_or(CapacityOverflow)?;
        // Cannot overflow, because `cap <= isize::MAX`, and type of `cap` is `usize`.
        let double_cap = self.cap * 2;
        // `double_cap` guarantees exponential growth.
        Ok(cmp::max(double_cap, required_cap))
    }

    /// The same as `reserve`, but returns on errors instead of panicking or aborting.
    pub fn try_reserve(&mut self, len: usize, additional: usize) -> Result<(), CollectionAllocErr> {
        if self.needs_to_grow(len, additional) {
            self.grow_amortized(len, additional)?;
        }

        Ok(())
    }

    /// Ensures that the buffer contains at least enough space to hold
    /// `len + additional` elements. If it doesn't already have
    /// enough capacity, will reallocate enough space plus comfortable slack
    /// space to get amortized `O(1)` behavior. Will limit this behavior
    /// if it would needlessly cause itself to panic.
    ///
    /// If `len` exceeds `self.cap()`, this may fail to actually allocate
    /// the requested space. This is not really unsafe, but the unsafe
    /// code *you* write that relies on the behavior of this function may break.
    ///
    /// This is ideal for implementing a bulk-push operation like `extend`.
    ///
    /// # Panics
    ///
    /// * Panics if the requested capacity exceeds `usize::MAX` bytes.
    /// * Panics on 32-bit platforms if the requested capacity exceeds
    ///   `isize::MAX` bytes.
    ///
    /// # Aborts
    ///
    /// Aborts on OOM
    ///
    /// # Examples
    ///
    /// ```ignore
    /// # #![feature(alloc, raw_vec_internals)]
    /// # extern crate alloc;
    /// # use std::ptr;
    /// # use alloc::raw_vec::RawVec;
    /// struct MyVec<T> {
    ///     buf: RawVec<T>,
    ///     len: usize,
    /// }
    ///
    /// impl<T: Clone> MyVec<T> {
    ///     pub fn push_all(&mut self, elems: &[T]) {
    ///         self.buf.reserve(self.len, elems.len());
    ///         // reserve would have aborted or panicked if the len exceeded
    ///         // `isize::MAX` so this is safe to do unchecked now.
    ///         for x in elems {
    ///             unsafe {
    ///                 ptr::write(self.buf.ptr().add(self.len), x.clone());
    ///             }
    ///             self.len += 1;
    ///         }
    ///     }
    /// }
    /// # fn main() {
    /// #   let mut vector = MyVec { buf: RawVec::new(), len: 0 };
    /// #   vector.push_all(&[1, 3, 5, 7, 9]);
    /// # }
    /// ```
    #[inline]
    pub fn reserve(&mut self, len: usize, additional: usize) {
        if let Err(err) = self.try_reserve(len, additional) {
            handle_error(err)
        }
    }

    /*
    /// Attempts to ensure that the buffer contains at least enough space to hold
    /// `len + additional` elements. If it doesn't already have
    /// enough capacity, will reallocate in place enough space plus comfortable slack
    /// space to get amortized `O(1)` behavior. Will limit this behaviour
    /// if it would needlessly cause itself to panic.
    ///
    /// If `len` exceeds `self.cap()`, this may fail to actually allocate
    /// the requested space. This is not really unsafe, but the unsafe
    /// code *you* write that relies on the behavior of this function may break.
    ///
    /// Returns true if the reallocation attempt has succeeded, or false otherwise.
    ///
    /// # Panics
    ///
    /// * Panics if the requested capacity exceeds `usize::MAX` bytes.
    /// * Panics on 32-bit platforms if the requested capacity exceeds
    ///   `isize::MAX` bytes.
    pub fn reserve_in_place(&mut self, len: usize, additional: usize) -> bool {
        unsafe {
            // NOTE: we don't early branch on ZSTs here because we want this
            // to actually catch "asking for more than usize::MAX" in that case.
            // If we make it past the first branch then we are guaranteed to
            // panic.

            // Don't actually need any more capacity. If the current `cap` is 0, we can't
            // reallocate in place.
            // Wrapping in case they give a bad `len`
            let old_layout = match self.current_layout() {
                Some(layout) => layout,
                None => return false,
            };
            if self.cap().wrapping_sub(len) >= additional {
                return false;
            }

            let new_cap = self
                .amortized_new_size(len, additional)
                .unwrap_or_else(|_| capacity_overflow());

            // Here, `cap < len + additional <= new_cap`
            // (regardless of whether `self.cap - len` wrapped).
            // Therefore we can safely call grow_in_place.

            let new_layout = Layout::new::<T>().repeat(new_cap).unwrap().0;
            // FIXME: may crash and burn on over-reserve
            alloc_guard(new_layout.size()).unwrap_or_else(|_| capacity_overflow());
            match self.a.grow_in_place(self.ptr.cast(), old_layout, new_layout.size()) {
                Ok(_) => {
                    self.cap = new_cap;
                    true
                }
                Err(_) => false,
            }
        }
    }
    */

    /// Shrinks the allocation down to the specified amount. If the given amount
    /// is 0, actually completely deallocates.
    ///
    /// # Panics
    ///
    /// Panics if the given amount is *larger* than the current capacity.
    ///
    /// # Aborts
    ///
    /// Aborts on OOM.
    pub fn shrink_to_fit(&mut self, amount: usize) {
        let elem_size = mem::size_of::<T>();

        // Set the `cap` because they might be about to promote to a `Box<[T]>`
        if elem_size == 0 {
            self.cap = amount;
            return;
        }

        // This check is my waterloo; it's the only thing Vec wouldn't have to do.
        assert!(self.cap >= amount, "Tried to shrink to a larger capacity");

        if amount == 0 {
            // We want to create a new zero-length vector within the
            // same allocator.  We use ptr::write to avoid an
            // erroneous attempt to drop the contents, and we use
            // ptr::read to sidestep condition against destructuring
            // types that implement Drop.

            unsafe {
                let a = self.a;
                self.dealloc_buffer();
                ptr::write(self, RawVec::new_in(a));
            }
        } else if self.cap != amount {
            unsafe {
                // We know here that our `amount` is greater than zero. This
                // implies, via the assert above, that capacity is also greater
                // than zero, which means that we've got a current layout that
                // "fits"
                //
                // We also know that `self.cap` is greater than `amount`, and
                // consequently we don't need runtime checks for creating either
                // layout
                let old_size = elem_size * self.cap;
                let new_size = elem_size * amount;
                let align = mem::align_of::<T>();
                let old_layout = Layout::from_size_align_unchecked(old_size, align);
                match realloc(self.a, self.ptr.cast(), old_layout, new_size) {
                    Ok(p) => self.ptr = p.cast(),
                    Err(_) => {
                        handle_alloc_error(Layout::from_size_align_unchecked(new_size, align))
                    }
                }
            }
            self.cap = amount;
        }
    }
}

/*
#[cfg(feature = "boxed")]
impl<'a, T> RawVec<'a, T> {
    /// Converts the entire buffer into `Box<[T]>`.
    ///
    /// Note that this will correctly reconstitute any `cap` changes
    /// that may have been performed. (See description of type for details.)
    ///
    /// # Undefined Behavior
    ///
    /// All elements of `RawVec<T>` must be initialized. Notice that
    /// the rules around uninitialized boxed values are not finalized yet,
    /// but until they are, it is advisable to avoid them.
    pub unsafe fn into_box(self) -> crate::boxed::Box<'a, [T]> {
        use crate::boxed::Box;

        // NOTE: not calling `cap()` here; actually using the real `cap` field!
        let slice = core::slice::from_raw_parts_mut(self.ptr(), self.cap);
        let output: Box<'a, [T]> = Box::from_raw(slice);
        mem::forget(self);
        output
    }
}
*/

impl<'a, T> RawVec<'a, T> {
    #[inline]
    fn needs_to_grow(&self, len: usize, additional: usize) -> bool {
        additional > self.cap().wrapping_sub(len)
    }

    /// Helper method to reserve additional space, reallocating the backing memory.
    /// The caller is responsible for confirming that there is not already enough space available.
    fn grow_exact(&mut self, len: usize, additional: usize) -> Result<(), CollectionAllocErr> {
        unsafe {
            // NOTE: we don't early branch on ZSTs here because we want this
            // to actually catch "asking for more than usize::MAX" in that case.
            // If we make it past the first branch then we are guaranteed to
            // panic.

            let new_cap = len.checked_add(additional).ok_or(CapacityOverflow)?;
            let new_layout = Layout::array::<T>(new_cap).map_err(|_| CapacityOverflow)?;

            self.ptr = self.finish_grow(new_layout)?.cast();

            self.cap = new_cap;

            Ok(())
        }
    }

    /// Helper method to reserve additional space, reallocating the backing memory.
    /// The caller is responsible for confirming that there is not already enough space available.
    fn grow_amortized(&mut self, len: usize, additional: usize) -> Result<(), CollectionAllocErr> {
        unsafe {
            // NOTE: we don't early branch on ZSTs here because we want this
            // to actually catch "asking for more than usize::MAX" in that case.
            // If we make it past the first branch then we are guaranteed to
            // panic.

            let new_cap = self.amortized_new_size(len, additional)?;
            let new_layout = Layout::array::<T>(new_cap).map_err(|_| CapacityOverflow)?;

            self.ptr = self.finish_grow(new_layout)?.cast();

            self.cap = new_cap;

            Ok(())
        }
    }

    // Given a new layout, completes the grow operation.
    #[inline]
    fn finish_grow(&self, new_layout: Layout) -> Result<NonNull<[u8]>, CollectionAllocErr> {
        alloc_guard(new_layout.size())?;

        let res = match self.current_layout() {
            Some(layout) => unsafe {
                debug_assert!(new_layout.align() == layout.align());
                realloc(self.a, self.ptr.cast(), layout, new_layout.size())
            },
            None => self.a.allocate(new_layout),
        };

        res.map_err(|_| AllocErr)
    }
}

impl<'a, T> RawVec<'a, T> {
    /// Frees the memory owned by the RawVec *without* trying to Drop its contents.
    pub unsafe fn dealloc_buffer(&mut self) {
        let elem_size = mem::size_of::<T>();
        if elem_size != 0 {
            if let Some(layout) = self.current_layout() {
                self.a.deallocate(self.ptr.cast(), layout);
            }
        }
    }
}

// We need to guarantee the following:
// * We don't ever allocate `> isize::MAX` byte-size objects
// * We don't overflow `usize::MAX` and actually allocate too little
//
// On 64-bit we just need to check for overflow since trying to allocate
// `> isize::MAX` bytes will surely fail. On 32-bit and 16-bit we need to add
// an extra guard for this in case we're running on a platform which can use
// all 4GB in user-space. e.g. PAE or x32

#[inline]
fn alloc_guard(alloc_size: usize) -> Result<(), CollectionAllocErr> {
    if mem::size_of::<usize>() < 8 && alloc_size > ::core::isize::MAX as usize {
        Err(CapacityOverflow)
    } else {
        Ok(())
    }
}

// One central function responsible for reporting capacity overflows. This'll
// ensure that the code generation related to these panics is minimal as there's
// only one location which panics rather than a bunch throughout the module.
fn capacity_overflow() -> ! {
    panic!("capacity overflow")
}

// Copied from https://github.com/fitzgen/bumpalo/blob/1d2fbea9e3d0c2be56367b9ad5382ff33852a188/src/alloc.rs#L29-L31
fn handle_alloc_error(layout: Layout) -> ! {
    panic!("encountered allocation error: {:?}", layout)
}

// Copied from https://github.com/fitzgen/bumpalo/blob/1d2fbea9e3d0c2be56367b9ad5382ff33852a188/src/lib.rs#L482-L486
/// Wrapper around `Layout::from_size_align` that adds debug assertions.
#[inline]
fn layout_from_size_align(size: usize, align: usize) -> Result<Layout, AllocError> {
    Layout::from_size_align(size, align).map_err(|_| AllocError)
}

// Code copied from https://github.com/fitzgen/bumpalo/blob/1d2fbea9e3d0c2be56367b9ad5382ff33852a188/src/lib.rs#L1917-L1936
// Doc comment from https://github.com/fitzgen/bumpalo/blob/1d2fbea9e3d0c2be56367b9ad5382ff33852a188/src/alloc.rs#L322-L402
//
/// Returns a pointer suitable for holding data described by
/// a new layout with `layout`’s alignment and a size given
/// by `new_size`. To
/// accomplish this, this may extend or shrink the allocation
/// referenced by `ptr` to fit the new layout.
///
/// If this returns `Ok`, then ownership of the memory block
/// referenced by `ptr` has been transferred to this
/// allocator. The memory may or may not have been freed, and
/// should be considered unusable (unless of course it was
/// transferred back to the caller again via the return value of
/// this method).
///
/// If this method returns `Err`, then ownership of the memory
/// block has not been transferred to this allocator, and the
/// contents of the memory block are unaltered.
///
/// # Safety
///
/// This function is unsafe because undefined behavior can result
/// if the caller does not ensure all of the following:
///
/// * `ptr` must be currently allocated via this allocator,
///
/// * `layout` must *fit* the `ptr` (see above). (The `new_size`
///   argument need not fit it.)
///
/// * `new_size` must be greater than zero.
///
/// * `new_size`, when rounded up to the nearest multiple of `layout.align()`,
///   must not overflow (i.e. the rounded value must be less than `usize::MAX`).
///
/// (Extension subtraits might provide more specific bounds on
/// behavior, e.g. guarantee a sentinel address or a null pointer
/// in response to a zero-size allocation request.)
///
/// # Errors
///
/// Returns `Err` only if the new layout
/// does not meet the allocator's size
/// and alignment constraints of the allocator, or if reallocation
/// otherwise fails.
///
/// Implementations are encouraged to return `Err` on memory
/// exhaustion rather than panicking or aborting, but this is not
/// a strict requirement. (Specifically: it is *legal* to
/// implement this trait atop an underlying native allocation
/// library that aborts on memory exhaustion.)
///
/// Clients wishing to abort computation in response to a
/// reallocation error are encouraged to call the [`handle_alloc_error`] function,
/// rather than directly invoking `panic!` or similar.
unsafe fn realloc(
    bump: &Bump,
    ptr: NonNull<u8>,
    layout: Layout,
    new_size: usize,
) -> Result<NonNull<[u8]>, AllocError> {
    let old_size = layout.size();

    if old_size == 0 {
        return bump.allocate(layout);
    }

    let new_layout = layout_from_size_align(new_size, layout.align())?;
    if new_size <= old_size {
        bump.shrink(ptr, layout, new_layout)
    } else {
        bump.grow(ptr, layout, new_layout)
    }
}

/// Handle collection allocation errors
///
// Causing a collection alloc error is rare case, so marked as `#[cold]` and `#[inline(never)]`
// to make the call site function as small as possible, so it can be inlined.
#[inline(never)]
#[cold]
fn handle_error(error: CollectionAllocErr) -> ! {
    match error {
        CapacityOverflow => capacity_overflow(),
        // TODO: call `handle_alloc_error` instead of `panic!` once the AllocErr stored a Layout,
        AllocErr => panic!("encountered allocation error"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reserve_does_not_overallocate() {
        let bump = Bump::new();
        {
            let mut v: RawVec<u32> = RawVec::new_in(&bump);
            // First `reserve` allocates like `reserve_exact`
            v.reserve(0, 9);
            assert_eq!(9, v.cap());
        }

        {
            let mut v: RawVec<u32> = RawVec::new_in(&bump);
            v.reserve(0, 7);
            assert_eq!(7, v.cap());
            // 97 if more than double of 7, so `reserve` should work
            // like `reserve_exact`.
            v.reserve(7, 90);
            assert_eq!(97, v.cap());
        }

        {
            let mut v: RawVec<u32> = RawVec::new_in(&bump);
            v.reserve(0, 12);
            assert_eq!(12, v.cap());
            v.reserve(12, 3);
            // 3 is less than half of 12, so `reserve` must grow
            // exponentially. At the time of writing this test grow
            // factor is 2, so new capacity is 24, however, grow factor
            // of 1.5 is OK too. Hence `>= 18` in assert.
            assert!(v.cap() >= 12 + 12 / 2);
        }
    }
}
