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

use std::{
    alloc::Layout,
    cmp,
    ptr::{self, NonNull},
};

use crate::alloc::Alloc;

/// Error type for fallible methods:
/// [`RawVec::try_reserve`], [`RawVec::try_reserve_exact`].
pub enum AllocError {
    AllocErr,
    CapacityOverflow,
}

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
/// Note that a RawVec always forces its capacity to be u32::MAX for zero-sized types.
/// This enables you to use capacity growing logic catch the overflows in your length
/// that might occur with zero-sized types.
///
/// However this means that you need to be careful when round-tripping this type
/// with a `Box<[T]>`: `cap()` won't yield the len. However `with_capacity`,
/// `shrink_to_fit`, and `from_box` will actually set RawVec's private capacity
/// field. This allows zero-sized types to not be special-cased by consumers of
/// this type.
#[allow(missing_debug_implementations)]
#[repr(C)]
pub struct RawVec<'a, T, A: Alloc> {
    ptr: NonNull<T>,
    len: u32,
    cap: u32,
    // SAFETY: Methods must not mutate the allocator (e.g. allocate into it), unless they can guarantee
    // they have exclusive access to it, by taking `&mut self`
    alloc: &'a A,
}

impl<'a, T, A: Alloc> RawVec<'a, T, A> {
    /// Like `new` but parameterized over the choice of allocator for
    /// the returned RawVec.
    #[inline(always)]
    pub fn new_in(alloc: &'a A) -> Self {
        // `cap: 0` means "unallocated". zero-sized types are ignored.
        RawVec { ptr: NonNull::dangling(), alloc, cap: 0, len: 0 }
    }

    /// Like `with_capacity` but parameterized over the choice of
    /// allocator for the returned RawVec.
    ///
    /// # Panics
    ///
    /// Panics if `cap` is too large.
    #[inline]
    pub fn with_capacity_in(cap: usize, alloc: &'a A) -> Self {
        unsafe {
            let elem_size = size_of::<T>();

            let alloc_size = cap.checked_mul(elem_size).unwrap_or_else(|| capacity_overflow());
            alloc_guard(alloc_size).unwrap_or_else(|_| capacity_overflow());

            // handles ZSTs and `cap = 0` alike
            let ptr = if alloc_size == 0 {
                NonNull::<T>::dangling()
            } else {
                let align = align_of::<T>();
                let layout = Layout::from_size_align(alloc_size, align).unwrap();
                alloc.alloc(layout).cast::<T>()
            };

            // `cap as u32` is safe because `alloc_guard` ensures that `cap`
            // cannot exceed `u32::MAX`.
            #[expect(clippy::cast_possible_truncation)]
            let cap = cap as u32;
            RawVec { ptr, alloc, cap, len: 0 }
        }
    }

    /// Reconstitutes a RawVec from a pointer, capacity, and allocator.
    ///
    /// # SAFETY
    ///
    /// * `ptr` must be allocated (via the given allocator `a`), and with the given capacity.
    /// * `cap` cannot exceed `u32::MAX`, as capacity is stored as `u32`.
    /// * The capacity in bytes (`cap * size_of::<T>()`) cannot exceed `isize::MAX`
    ///   (only a concern on 32-bit systems).
    /// * `len` must be `<= cap`. `len` is also therefore subject to same restrictions as `cap`.
    ///
    /// If all these values came from a `Vec` created in allocator `a`, then these requirements
    /// are guaranteed to be fulfilled.
    #[inline(always)]
    pub unsafe fn from_raw_parts_in(ptr: *mut T, len: usize, cap: usize, alloc: &'a A) -> Self {
        // SAFETY: Caller guarantees `ptr` was allocated, which implies it's not null
        let ptr = unsafe { NonNull::new_unchecked(ptr) };

        // Caller guarantees `cap` and `len` are `<= u32::MAX`, so `as u32` cannot truncate them
        #[expect(clippy::cast_possible_truncation)]
        let len = len as u32;
        #[expect(clippy::cast_possible_truncation)]
        let cap = cap as u32;

        RawVec { ptr, len, cap, alloc }
    }

    /// Gets a raw pointer to the start of the allocation.
    /// Note that this is `NonNull::dangling()` if `cap = 0` or T is zero-sized.
    /// In the former case, you must be careful.
    #[inline(always)]
    pub fn ptr(&self) -> *mut T {
        self.ptr.as_ptr()
    }

    /// Gets the number of elements as `u32`.
    #[inline(always)]
    pub fn len_u32(&self) -> u32 {
        self.len
    }

    /// Gets the number of elements as `usize`.
    #[inline(always)]
    pub fn len_usize(&self) -> usize {
        self.len as usize
    }

    /// Set the number of elements.
    #[inline(always)]
    pub fn set_len(&mut self, new_len: u32) {
        self.len = new_len;
    }

    /// Increase the number of elements by `increment`.
    #[inline(always)]
    pub fn increase_len(&mut self, increment: u32) {
        self.len += increment;
    }

    /// Decrease the number of elements by `decrement`.
    #[inline(always)]
    pub fn decrease_len(&mut self, decrement: u32) {
        self.len -= decrement;
    }

    /// Gets the capacity of the allocation as `u32`.
    ///
    /// This will always be `u32::MAX` if `T` is zero-sized.
    #[inline(always)]
    pub fn capacity_u32(&self) -> u32 {
        if size_of::<T>() == 0 { !0 } else { self.cap }
    }

    /// Gets the capacity of the allocation as `usize`.
    ///
    /// This will always be `usize::MAX` if `T` is zero-sized.
    #[inline(always)]
    pub fn capacity_usize(&self) -> usize {
        if size_of::<T>() == 0 { !0 } else { self.cap as usize }
    }

    /// Get a shared reference to the allocator backing this `RawVec`.
    ///
    /// This method is hazardous.
    ///
    /// `Vec` is `Sync`, but `Bump` is not, because it utilizes interior mutability.
    /// It is possible to make allocations into the arena while holding only a `&Bump`.
    /// Because `Vec` is `Sync`, it's possible for multiple `&Vec` references to the same `Vec`,
    /// or references to multiple `Vec`s attached to the same `Bump`, to exist simultaneously
    /// on different threads.
    ///
    /// So this method could be used to obtain 2 `&Bump` references simultaneously on different threads.
    /// Utilizing those references to allocate into the arena simultaneously from different threads
    /// would be UB.
    ///
    /// We cannot rely on the type system or borrow checker to ensure correct synchronization.
    ///
    /// Therefore callers must ensure by other means that they have exclusive access, by:
    ///
    /// 1. Taking a `&mut self`.
    ///    No methods of `Vec` or `RawVec` which do not hold a `&mut Vec` / `&mut RawVec` can use this method.
    ///
    /// 2. That `&mut self` must be held for at least as long as the `&'a A` reference returned by
    ///    this method is held.
    ///
    /// Note: It's tempting to think we could make this a safe method by making it take `&mut self`,
    /// but that's insufficient. That would enforce that the caller holds a `&mut self`, but the `&'a A`
    /// returned by this method outlives the lifetime of `&self` that the method takes, so it would
    /// NOT guarantee anything about *how long* they hold it for.
    /// Taking a `&'a mut self` *would* be safe, but it'd be impractical.
    ///
    /// For further information, see comments on the `impl Sync` implementation of `Vec`.
    ///
    /// # IMPORTANT
    /// The ability to obtain a reference to the allocator MUST NOT be exposed to user,
    /// outside of `Vec`'s internals.
    ///
    /// # SAFETY
    /// Caller must ensure they have exclusive access, but holding a `&mut Vec` or `&mut RawVec`
    /// for the duration that the reference returned by this method is held.
    /// See text above for further detail.
    #[inline(always)]
    pub unsafe fn bump(&self) -> &'a A {
        self.alloc
    }

    #[inline]
    fn current_layout(&self) -> Option<Layout> {
        if self.cap == 0 {
            None
        } else {
            // We have an allocated chunk of memory, so we can bypass runtime
            // checks to get our current layout.
            unsafe {
                let align = align_of::<T>();
                // `self.cap as usize` is safe because it's is `u32`
                // so it must be less than `usize::MAX`.
                let size = size_of::<T>() * self.cap as usize;
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
    ///   all `u32::MAX` slots in your imaginary buffer.
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
            let elem_size = size_of::<T>();

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
    ///   all `u32::MAX` slots in your imaginary buffer.
    /// * Panics on 32-bit platforms if the requested capacity exceeds
    ///   `isize::MAX` bytes.
    #[inline(never)]
    #[cold]
    pub fn double_in_place(&mut self) -> bool {
        unsafe {
            let elem_size = size_of::<T>();
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
    ///
    /// # Errors
    ///
    /// Returns `Err(AllocError)` if unable to reserve requested space in the `RawVec`.
    pub fn try_reserve_exact(&mut self, len: u32, additional: usize) -> Result<(), AllocError> {
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
    /// * Panics if the requested capacity exceeds `u32::MAX` bytes.
    /// * Panics on 32-bit platforms if the requested capacity exceeds
    ///   `isize::MAX` bytes.
    /// * Panics if the new number of elements would overflow `u32`.
    ///
    /// # Aborts
    ///
    /// Aborts on OOM
    pub fn reserve_exact(&mut self, len: u32, additional: usize) {
        if let Err(err) = self.try_reserve_exact(len, additional) {
            handle_error(err)
        }
    }

    /// The same as `reserve`, but returns on errors instead of panicking or aborting.
    ///
    /// # Errors
    ///
    /// Returns `Err(AllocError)` if unable to reserve requested space in the `RawVec`.
    pub fn try_reserve(&mut self, len: u32, additional: usize) -> Result<(), AllocError> {
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
    /// * Panics if the requested capacity exceeds `u32::MAX` bytes.
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
    pub fn reserve(&mut self, len: u32, additional: usize) {
        // Callers expect this function to be very cheap when there is already sufficient capacity.
        // Therefore, we move all the resizing and error-handling logic from grow_amortized and
        // handle_reserve behind a call, while making sure that this function is likely to be
        // inlined as just a comparison and a call if the comparison fails.
        #[cold]
        fn do_reserve_and_handle<T, A: Alloc>(slf: &mut RawVec<T, A>, len: u32, additional: usize) {
            if let Err(err) = slf.grow_amortized(len, additional) {
                handle_error(err);
            }
        }

        if self.needs_to_grow(len, additional) {
            do_reserve_and_handle(self, len, additional);
        }
    }

    /// A specialized version of `self.reserve(len, 1)` which requires the
    /// caller to ensure `len == self.capacity()`.
    //
    // Unlike standard library implementation marked as `#[inline(never)]`, we need to
    // mark as `#[inline]` because this function is common case in the oxc_parser.
    #[inline]
    pub fn grow_one(&mut self) {
        if let Err(err) = self.grow_amortized(self.cap, 1) {
            handle_error(err);
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
    /// * Panics if the requested capacity exceeds `u32::MAX` bytes.
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
    pub fn shrink_to_fit(&mut self, amount: u32) {
        let elem_size = size_of::<T>();

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
                let a = self.alloc;
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
                //
                // `self.cap as usize` and `amount as usize` are safe because
                // they are `u32` so they must be less than `usize::MAX`.
                let old_size = elem_size * self.cap as usize;
                let new_size = elem_size * amount as usize;
                let align = align_of::<T>();
                let old_layout = Layout::from_size_align_unchecked(old_size, align);
                let new_layout = Layout::from_size_align_unchecked(new_size, align);
                self.ptr =
                    self.alloc.shrink(self.ptr.cast::<u8>(), old_layout, new_layout).cast::<T>();
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
        let slice = std::slice::from_raw_parts_mut(self.ptr(), self.cap);
        let output: Box<'a, [T]> = Box::from_raw(slice);
        mem::forget(self);
        output
    }
}
*/

impl<T, A: Alloc> RawVec<'_, T, A> {
    #[inline]
    fn needs_to_grow(&self, len: u32, additional: usize) -> bool {
        // `self.cap().wrapping_sub(len) as usize` is safe because
        // `self.cap()` is `u32` and `len` is `u32`, so the result
        // is guaranteed to be less than `usize::MAX`.
        additional > self.capacity_u32().wrapping_sub(len) as usize
    }

    /// Helper method to reserve additional space, reallocating the backing memory.
    /// The caller is responsible for confirming that there is not already enough space available.
    fn grow_exact(&mut self, len: u32, additional: usize) -> Result<(), AllocError> {
        unsafe {
            // NOTE: we don't early branch on ZSTs here because we want this
            // to actually catch "asking for more than u32::MAX" in that case.
            // If we make it past the first branch then we are guaranteed to
            // panic.

            // `len as usize` is safe because `len` is `u32`, so it must be
            // less than `usize::MAX`.
            let new_cap =
                (len as usize).checked_add(additional).ok_or(AllocError::CapacityOverflow)?;
            let new_layout =
                Layout::array::<T>(new_cap).map_err(|_| AllocError::CapacityOverflow)?;

            self.ptr = self.finish_grow(new_layout)?.cast();

            // `cap as u32` is safe because `finish_grow` called `alloc_guard`, and
            // `alloc_guard` ensures that `cap` cannot exceed `u32::MAX`.
            #[expect(clippy::cast_possible_truncation)]
            let new_cap = new_cap as u32;
            self.cap = new_cap;

            Ok(())
        }
    }

    /// Helper method to reserve additional space, reallocating the backing memory.
    /// The caller is responsible for confirming that there is not already enough space available.
    fn grow_amortized(&mut self, len: u32, additional: usize) -> Result<(), AllocError> {
        unsafe {
            // NOTE: we don't early branch on ZSTs here because we want this
            // to actually catch "asking for more than u32::MAX" in that case.
            // If we make it past the first branch then we are guaranteed to
            // panic.

            // Nothing we can really do about these checks, sadly.
            // `len as usize` is safe because `len` is `u32`, so it must be
            // less than `usize::MAX`.
            let required_cap =
                (len as usize).checked_add(additional).ok_or(AllocError::CapacityOverflow)?;

            // This guarantees exponential growth. The doubling cannot overflow
            // because `cap <= isize::MAX` and the type of `cap` is `u32`.
            let cap = cmp::max((self.capacity_u32() as usize) * 2, required_cap);

            // The following commented-out code is copied from the standard library.
            // We don't use it because this would cause notable performance regression
            // in the oxc_transformer, oxc_minifier and oxc_mangler, but would only get a
            // tiny performance improvement in the oxc_parser.
            //
            // The reason is that only the oxc_parser has a lot of tiny `Vec`s without
            // pre-reserved capacity, which can benefit from this change. Other
            // crates don't have such cases, so the `cmp::max` calculation costs more
            // than the potential performance improvement.
            //
            // ------------------ Copied from the standard library ------------------
            //
            // Tiny Vecs are dumb. Skip to:
            // - 8 if the element size is 1, because any heap allocators is likely
            //   to round up a request of less than 8 bytes to at least 8 bytes.
            // - 4 if elements are moderate-sized (<= 1 KiB).
            // - 1 otherwise, to avoid wasting too much space for very short Vecs.
            // const fn min_non_zero_cap(size: usize) -> usize {
            //     if size == 1 {
            //         8
            //     } else if size <= 1024 {
            //         4
            //     } else {
            //         1
            //     }
            // }
            // let cap = cmp::max(Self::MIN_NON_ZERO_CAP, cap);

            let new_layout = Layout::array::<T>(cap).map_err(|_| AllocError::CapacityOverflow)?;

            self.ptr = self.finish_grow(new_layout)?.cast();

            // `cap as u32` is safe because `finish_grow` called `alloc_guard`, and
            // `alloc_guard` ensures that `cap` cannot exceed `u32::MAX`.
            #[expect(clippy::cast_possible_truncation)]
            let cap = cap as u32;
            self.cap = cap;

            Ok(())
        }
    }

    // Given a new layout, completes the grow operation.
    #[inline]
    fn finish_grow(&self, new_layout: Layout) -> Result<NonNull<u8>, AllocError> {
        alloc_guard(new_layout.size())?;

        let new_ptr = match self.current_layout() {
            Some(layout) => unsafe {
                // Marking this function as `#[cold]` and `#[inline(never)]` because grow method is
                // relatively expensive and we want to avoid inlining it into the caller.
                #[cold]
                #[inline(never)]
                unsafe fn grow<T, A: Alloc>(
                    alloc: &A,
                    ptr: NonNull<T>,
                    old_layout: Layout,
                    new_layout: Layout,
                ) -> NonNull<u8> {
                    alloc.grow(ptr.cast(), old_layout, new_layout)
                }
                debug_assert!(new_layout.align() == layout.align());
                grow(self.alloc, self.ptr, layout, new_layout)
            },
            None => self.alloc.alloc(new_layout),
        };

        Ok(new_ptr)
    }
}

impl<T, A: Alloc> RawVec<'_, T, A> {
    /// Frees the memory owned by the RawVec *without* trying to Drop its contents.
    ///
    /// # SAFETY
    ///
    /// Not sure what safety invariants of this method are! TODO
    pub unsafe fn dealloc_buffer(&mut self) {
        let elem_size = size_of::<T>();
        if elem_size != 0
            && let Some(layout) = self.current_layout()
        {
            self.alloc.dealloc(self.ptr.cast(), layout);
        }
    }
}

// We need to guarantee the following:
// * We don't ever allocate `> isize::MAX` byte-size objects
// * We don't overflow `u32::MAX` and actually allocate too little
//
// On 64-bit we need to check for overflow since trying to allocate `> u32::MAX`
// bytes is overflow because `cap` and `len` are both `u32`s.
// On 32-bit and 16-bit we need to add an extra guard for this in case we're
// running on a platform which can use all 4GB in user-space. e.g. PAE or x32

#[inline]
fn alloc_guard(alloc_size: usize) -> Result<(), AllocError> {
    if size_of::<usize>() < 8 {
        if alloc_size > isize::MAX as usize {
            return Err(AllocError::CapacityOverflow);
        }
    } else if alloc_size > u32::MAX as usize {
        return Err(AllocError::CapacityOverflow);
    }
    Ok(())
}

// One central function responsible for reporting capacity overflows. This'll
// ensure that the code generation related to these panics is minimal as there's
// only one location which panics rather than a bunch throughout the module.
fn capacity_overflow() -> ! {
    panic!("capacity overflow")
}

/// Handle collection allocation errors
///
// Causing a collection alloc error is rare case, so marked as `#[cold]` and `#[inline(never)]`
// to make the call site function as small as possible, so it can be inlined.
#[inline(never)]
#[cold]
fn handle_error(error: AllocError) -> ! {
    match error {
        AllocError::CapacityOverflow => capacity_overflow(),
        // TODO: call `handle_alloc_error` instead of `panic!` once the AllocErr stored a Layout,
        AllocError::AllocErr => panic!("encountered allocation error"),
    }
}

#[cfg(test)]
mod tests {
    use bumpalo::Bump;

    use super::*;

    #[test]
    fn reserve_does_not_overallocate() {
        let arena = Bump::new();
        {
            let mut v: RawVec<u32, _> = RawVec::new_in(&arena);
            // First `reserve` allocates like `reserve_exact`
            v.reserve(0, 9);
            assert_eq!(9, v.capacity_u32());
        }

        {
            let mut v: RawVec<u32, _> = RawVec::new_in(&arena);
            v.reserve(0, 7);
            assert_eq!(7, v.capacity_u32());
            // 97 if more than double of 7, so `reserve` should work
            // like `reserve_exact`.
            v.reserve(7, 90);
            assert_eq!(97, v.capacity_u32());
        }

        {
            let mut v: RawVec<u32, _> = RawVec::new_in(&arena);
            v.reserve(0, 12);
            assert_eq!(12, v.capacity_u32());
            v.reserve(12, 3);
            // 3 is less than half of 12, so `reserve` must grow
            // exponentially. At the time of writing this test grow
            // factor is 2, so new capacity is 24, however, grow factor
            // of 1.5 is OK too. Hence `>= 18` in assert.
            assert!(v.capacity_u32() >= 12 + 12 / 2);
        }
    }
}
