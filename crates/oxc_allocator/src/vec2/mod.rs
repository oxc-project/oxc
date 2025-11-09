// This file is copied from the [Bumpalo's Vec](https://github.com/fitzgen/bumpalo/blob/1d2fbea9e3d0c2be56367b9ad5382ff33852a188/src/collections/vec.rs)

// Copyright 2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! A contiguous growable array type with heap-allocated contents, written
//! [`Vec<'a, T>`].
//!
//! Vectors have `O(1)` indexing, amortized `O(1)` push (to the end) and
//! `O(1)` pop (from the end).
//!
//! This module is a fork of the [`std::vec`] module, that uses a bump allocator.
//!
//! [`std::vec`]: https://doc.rust-lang.org/std/vec/index.html
//!
//! # Examples
//!
//! You can explicitly create a [`Vec<'a, T>`] with [`new_in`]:
//!
//! ```
//! use bumpalo::{Bump, collections::Vec};
//!
//! let b = Bump::new();
//! let v: Vec<i32> = Vec::new_in(&b);
//! ```
//!
//! You can [`push`] values onto the end of a vector (which will grow the vector
//! as needed):
//!
//! ```
//! use bumpalo::{Bump, collections::Vec};
//!
//! let b = Bump::new();
//!
//! let mut v = Vec::new_in(&b);
//!
//! v.push(1);
//! ```
//!
//! Popping values works in much the same way:
//!
//! ```
//! use bumpalo::{Bump, collections::Vec};
//!
//! let b = Bump::new();
//!
//! let mut v = Vec::from_iter_in([1, 2], &b);
//!
//! assert_eq!(v.pop(), Some(2));
//! ```
//!
//! Vectors also support indexing (through the [`Index`] and [`IndexMut`] traits):
//!
//! ```
//! use bumpalo::{Bump, collections::Vec};
//!
//! let b = Bump::new();
//!
//! let mut v = Vec::from_iter_in([1, 2, 3], &b);
//! assert_eq!(v[2], 3);
//! v[1] += 5;
//! assert_eq!(v, [1, 7, 3]);
//! ```
//!
//! [`Vec<'a, T>`]: Vec
//! [`new_in`]: Vec::new_in
//! [`push`]: Vec::push
//! [`Index`]: https://doc.rust-lang.org/std/ops/trait.Index.html
//! [`IndexMut`]: https://doc.rust-lang.org/std/ops/trait.IndexMut.html

#![expect(
    clippy::semicolon_if_nothing_returned,
    clippy::needless_pass_by_ref_mut,
    clippy::needless_for_each,
    clippy::cloned_instead_of_copied,
    clippy::checked_conversions,
    clippy::legacy_numeric_constants,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::swap_ptr_to_ref,
    clippy::ref_as_ptr,
    clippy::ptr_as_ptr,
    clippy::ptr_cast_constness,
    unsafe_op_in_unsafe_fn,
    clippy::undocumented_unsafe_blocks
)]

use std::{
    borrow::{Borrow, BorrowMut},
    cmp::Ordering,
    fmt,
    hash::{self, Hash},
    hint::assert_unchecked,
    iter::FusedIterator,
    marker::PhantomData,
    mem,
    ops::{
        self,
        Bound::{Excluded, Included, Unbounded},
        Index, IndexMut, RangeBounds,
    },
    ptr::{self, NonNull},
    slice::{self, SliceIndex},
};

// #[cfg(feature = "std")]
// use std::io;

use oxc_data_structures::assert_unchecked;

use crate::alloc::Alloc;

mod raw_vec;
use raw_vec::{AllocError, RawVec};

unsafe fn arith_offset<T>(p: *const T, offset: isize) -> *const T {
    p.offset(offset)
}

fn partition_dedup_by<T, F>(s: &mut [T], mut same_bucket: F) -> (&mut [T], &mut [T])
where
    F: FnMut(&mut T, &mut T) -> bool,
{
    // Although we have a mutable reference to `s`, we cannot make
    // *arbitrary* changes. The `same_bucket` calls could panic, so we
    // must ensure that the slice is in a valid state at all times.
    //
    // The way that we handle this is by using swaps; we iterate
    // over all the elements, swapping as we go so that at the end
    // the elements we wish to keep are in the front, and those we
    // wish to reject are at the back. We can then split the slice.
    // This operation is still O(n).
    //
    // Example: We start in this state, where `r` represents "next
    // read" and `w` represents "next_write`.
    //
    //           r
    //     +---+---+---+---+---+---+
    //     | 0 | 1 | 1 | 2 | 3 | 3 |
    //     +---+---+---+---+---+---+
    //           w
    //
    // Comparing s[r] against s[w-1], this is not a duplicate, so
    // we swap s[r] and s[w] (no effect as r==w) and then increment both
    // r and w, leaving us with:
    //
    //               r
    //     +---+---+---+---+---+---+
    //     | 0 | 1 | 1 | 2 | 3 | 3 |
    //     +---+---+---+---+---+---+
    //               w
    //
    // Comparing s[r] against s[w-1], this value is a duplicate,
    // so we increment `r` but leave everything else unchanged:
    //
    //                   r
    //     +---+---+---+---+---+---+
    //     | 0 | 1 | 1 | 2 | 3 | 3 |
    //     +---+---+---+---+---+---+
    //               w
    //
    // Comparing s[r] against s[w-1], this is not a duplicate,
    // so swap s[r] and s[w] and advance r and w:
    //
    //                       r
    //     +---+---+---+---+---+---+
    //     | 0 | 1 | 2 | 1 | 3 | 3 |
    //     +---+---+---+---+---+---+
    //                   w
    //
    // Not a duplicate, repeat:
    //
    //                           r
    //     +---+---+---+---+---+---+
    //     | 0 | 1 | 2 | 3 | 1 | 3 |
    //     +---+---+---+---+---+---+
    //                       w
    //
    // Duplicate, advance r. End of slice. Split at w.

    let len = s.len();
    if len <= 1 {
        return (s, &mut []);
    }

    let ptr = s.as_mut_ptr();
    let mut next_read: usize = 1;
    let mut next_write: usize = 1;

    unsafe {
        // Avoid bounds checks by using raw pointers.
        while next_read < len {
            let ptr_read = ptr.add(next_read);
            let prev_ptr_write = ptr.add(next_write - 1);
            if !same_bucket(&mut *ptr_read, &mut *prev_ptr_write) {
                if next_read != next_write {
                    let ptr_write = prev_ptr_write.offset(1);
                    mem::swap(&mut *ptr_read, &mut *ptr_write);
                }
                next_write += 1;
            }
            next_read += 1;
        }
    }

    s.split_at_mut(next_write)
}

unsafe fn offset_from<T>(p: *const T, origin: *const T) -> isize
where
    T: Sized,
{
    let pointee_size = size_of::<T>();
    assert!(0 < pointee_size && pointee_size <= isize::max_value() as usize);

    // This is the same sequence that Clang emits for pointer subtraction.
    // It can be neither `nsw` nor `nuw` because the input is treated as
    // unsigned but then the output is treated as signed, so neither works.
    let d = isize::wrapping_sub(p as _, origin as _);
    d / (pointee_size as isize)
}

/// A contiguous growable array type, written `Vec<'a, T, A>` but pronounced 'vector'.
///
/// # Examples
///
/// ```
/// use bumpalo::{Bump, collections::Vec};
///
/// let b = Bump::new();
///
/// let mut vec = Vec::new_in(&b);
/// vec.push(1);
/// vec.push(2);
///
/// assert_eq!(vec.len(), 2);
/// assert_eq!(vec[0], 1);
///
/// assert_eq!(vec.pop(), Some(2));
/// assert_eq!(vec.len(), 1);
///
/// vec[0] = 7;
/// assert_eq!(vec[0], 7);
///
/// vec.extend([1, 2, 3].iter().cloned());
///
/// for x in &vec {
///     println!("{}", x);
/// }
/// assert_eq!(vec, [7, 1, 2, 3]);
/// ```
///
/// Use a `Vec<'a, T>` as an efficient stack:
///
/// ```
/// use bumpalo::{Bump, collections::Vec};
///
/// let b = Bump::new();
///
/// let mut stack = Vec::new_in(&b);
///
/// stack.push(1);
/// stack.push(2);
/// stack.push(3);
///
/// while let Some(top) = stack.pop() {
///     // Prints 3, 2, 1
///     println!("{}", top);
/// }
/// ```
///
/// # Indexing
///
/// The `Vec` type allows accessing values by index, because it implements the
/// [`Index`] trait. An example will be more explicit:
///
/// ```
/// use bumpalo::{Bump, collections::Vec};
///
/// let b = Bump::new();
///
/// let v = Vec::from_iter_in([0, 2, 4, 6], &b);
/// println!("{}", v[1]); // it will display '2'
/// ```
///
/// However be careful: if you try to access an index which isn't in the `Vec`,
/// your software will panic! You cannot do this:
///
/// ```should_panic
/// use bumpalo::{Bump, collections::Vec};
///
/// let b = Bump::new();
///
/// let v = Vec::from_iter_in([0, 2, 4, 6], &b);
/// println!("{}", v[6]); // it will panic!
/// ```
///
/// In conclusion: always check if the index you want to get really exists
/// before doing it.
///
/// # Slicing
///
/// A `Vec` can be mutable. Slices, on the other hand, are read-only objects.
/// To get a slice, use `&`. Example:
///
/// ```
/// use bumpalo::{Bump, collections::Vec};
///
/// fn read_slice(slice: &[usize]) {
///     // ...
/// }
///
/// let b = Bump::new();
///
/// let v = Vec::from_iter_in([0, 1], &b);
/// read_slice(&v);
///
/// // ... and that's all!
/// // you can also do it like this:
/// let x : &[usize] = &v;
/// ```
///
/// In Rust, it's more common to pass slices as arguments rather than vectors
/// when you just want to provide a read access. The same goes for [`String`] and
/// [`&str`].
///
/// # Capacity and reallocation
///
/// The capacity of a vector is the amount of space allocated for any future
/// elements that will be added onto the vector. This is not to be confused with
/// the *length* of a vector, which specifies the number of actual elements
/// within the vector. If a vector's length exceeds its capacity, its capacity
/// will automatically be increased, but its elements will have to be
/// reallocated.
///
/// For example, a vector with capacity 10 and length 0 would be an empty vector
/// with space for 10 more elements. Pushing 10 or fewer elements onto the
/// vector will not change its capacity or cause reallocation to occur. However,
/// if the vector's length is increased to 11, it will have to reallocate, which
/// can be slow. For this reason, it is recommended to use [`Vec::with_capacity_in`]
/// whenever possible to specify how big the vector is expected to get.
///
/// # Guarantees
///
/// Due to its incredibly fundamental nature, `Vec` makes a lot of guarantees
/// about its design. This ensures that it's as low-overhead as possible in
/// the general case, and can be correctly manipulated in primitive ways
/// by unsafe code. Note that these guarantees refer to an unqualified `Vec<'a, T>`.
/// If additional type parameters are added (e.g. to support custom allocators),
/// overriding their defaults may change the behavior.
///
/// Most fundamentally, `Vec` is and always will be a (pointer, capacity, length)
/// triplet. No more, no less. The order of these fields is completely
/// unspecified, and you should use the appropriate methods to modify these.
/// The pointer will never be null, so this type is null-pointer-optimized.
///
/// However, the pointer may not actually point to allocated memory. In particular,
/// if you construct a `Vec` with capacity 0 via [`Vec::new_in`],
/// [`Vec::with_capacity_in(0)`][`Vec::with_capacity_in`], or by calling [`shrink_to_fit`]
/// on an empty Vec, it will not allocate memory. Similarly, if you store zero-sized
/// types inside a `Vec`, it will not allocate space for them. *Note that in this case
/// the `Vec` may not report a [`capacity`] of 0*. `Vec` will allocate if and only
/// if <code>[`size_of::<T>`]\() * capacity() > 0</code>. In general, `Vec`'s allocation
/// details are very subtle &mdash; if you intend to allocate memory using a `Vec`
/// and use it for something else (either to pass to unsafe code, or to build your
/// own memory-backed collection), be sure to deallocate this memory by using
/// `from_raw_parts` to recover the `Vec` and then dropping it.
///
/// If a `Vec` *has* allocated memory, then the memory it points to is
/// in the arena used to construct it, and its
/// pointer points to [`len`] initialized, contiguous elements in order (what
/// you would see if you coerced it to a slice), followed by <code>[`capacity`] -
/// [`len`]</code> logically uninitialized, contiguous elements.
///
/// `Vec` will never perform a "small optimization" where elements are actually
/// stored on the stack for two reasons:
///
/// * It would make it more difficult for unsafe code to correctly manipulate
///   a `Vec`. The contents of a `Vec` wouldn't have a stable address if it were
///   only moved, and it would be more difficult to determine if a `Vec` had
///   actually allocated memory.
///
/// * It would penalize the general case, incurring an additional branch
///   on every access.
///
/// `Vec` will never automatically shrink itself, even if completely empty. This
/// ensures no unnecessary allocations or deallocations occur. Emptying a `Vec`
/// and then filling it back up to the same [`len`] should incur no calls to
/// the allocator. If you wish to free up unused memory, use
/// [`shrink_to_fit`][`shrink_to_fit`].
///
/// [`push`] and [`insert`] will never (re)allocate if the reported capacity is
/// sufficient. [`push`] and [`insert`] *will* (re)allocate if
/// <code>[`len`] == [`capacity`]</code>. That is, the reported capacity is completely
/// accurate, and can be relied on. It can even be used to manually free the memory
/// allocated by a `Vec` if desired. Bulk insertion methods *may* reallocate, even
/// when not necessary.
///
/// `Vec` does not guarantee any particular growth strategy when reallocating
/// when full, nor when [`reserve`] is called. The current strategy is basic
/// and it may prove desirable to use a non-constant growth factor. Whatever
/// strategy is used will of course guarantee `O(1)` amortized [`push`].
///
/// [`Vec::with_capacity_in(n)`][`Vec::with_capacity_in`], will produce a
/// `Vec` with exactly the requested capacity. If <code>[`len`] == [`capacity`]</code>,
/// then a `Vec<'a, T>` can be converted to and from a [`Box<[T]>`][owned slice]
/// without reallocating or moving the elements.
///
/// `Vec` will not specifically overwrite any data that is removed from it,
/// but also won't specifically preserve it. Its uninitialized memory is
/// scratch space that it may use however it wants. It will generally just do
/// whatever is most efficient or otherwise easy to implement. Do not rely on
/// removed data to be erased for security purposes. Even if you drop a `Vec`, its
/// buffer may simply be reused by another `Vec`. Even if you zero a `Vec`'s memory
/// first, that may not actually happen because the optimizer does not consider
/// this a side-effect that must be preserved. There is one case which we will
/// not break, however: using `unsafe` code to write to the excess capacity,
/// and then increasing the length to match, is always valid.
///
/// `Vec` does not currently guarantee the order in which elements are dropped.
/// The order has changed in the past and may change again.
///
/// [`Index`]: https://doc.rust-lang.org/std/ops/trait.Index.html
/// [`String`]: ../string/struct.String.html
/// [`&str`]: https://doc.rust-lang.org/std/primitive.str.html
/// [`Vec::with_capacity_in`]: struct.Vec.html#method.with_capacity_in
/// [`Vec::new_in`]: struct.Vec.html#method.new_in
/// [`shrink_to_fit`]: struct.Vec.html#method.shrink_to_fit
/// [`capacity`]: struct.Vec.html#method.capacity
/// [`size_of::<T>`]: https://doc.rust-lang.org/std/mem/fn.size_of.html
/// [`len`]: struct.Vec.html#method.len
/// [`push`]: struct.Vec.html#method.push
/// [`insert`]: struct.Vec.html#method.insert
/// [`reserve`]: struct.Vec.html#method.reserve
/// [owned slice]: https://doc.rust-lang.org/std/boxed/struct.Box.html
#[repr(transparent)]
pub struct Vec<'a, T: 'a, A: Alloc> {
    buf: RawVec<'a, T, A>,
}

////////////////////////////////////////////////////////////////////////////////
// Inherent methods
////////////////////////////////////////////////////////////////////////////////

impl<'a, T: 'a, A: Alloc> Vec<'a, T, A> {
    /// Constructs a new, empty `Vec<'a, T>`.
    ///
    /// The vector will not allocate until elements are pushed onto it.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![allow(unused_mut)]
    /// use bumpalo::{Bump, collections::Vec};
    ///
    /// let b = Bump::new();
    /// let mut vec: Vec<i32> = Vec::new_in(&b);
    /// ```
    #[inline]
    pub fn new_in(alloc: &'a A) -> Vec<'a, T, A> {
        Vec { buf: RawVec::new_in(alloc) }
    }

    /// Constructs a new, empty `Vec<'a, T>` with the specified capacity.
    ///
    /// The vector will be able to hold exactly `capacity` elements without
    /// reallocating. If `capacity` is 0, the vector will not allocate.
    ///
    /// It is important to note that although the returned vector has the
    /// *capacity* specified, the vector will have a zero *length*. For an
    /// explanation of the difference between length and capacity, see
    /// *[Capacity and reallocation]*.
    ///
    /// [Capacity and reallocation]: #capacity-and-reallocation
    ///
    /// # Examples
    ///
    /// ```
    /// use bumpalo::{Bump, collections::Vec};
    ///
    /// let b = Bump::new();
    ///
    /// let mut vec = Vec::with_capacity_in(10, &b);
    ///
    /// // The vector contains no items, even though it has capacity for more
    /// assert_eq!(vec.len(), 0);
    ///
    /// // These are all done without reallocating...
    /// for i in 0..10 {
    ///     vec.push(i);
    /// }
    ///
    /// // ...but this may make the vector reallocate
    /// vec.push(11);
    /// ```
    #[inline]
    pub fn with_capacity_in(capacity: usize, alloc: &'a A) -> Vec<'a, T, A> {
        Vec { buf: RawVec::with_capacity_in(capacity, alloc) }
    }

    /// Construct a new `Vec` from the given iterator's items.
    ///
    /// # Examples
    ///
    /// ```
    /// use bumpalo::{Bump, collections::Vec};
    /// use std::iter;
    ///
    /// let b = Bump::new();
    /// let v = Vec::from_iter_in(iter::repeat(7).take(3), &b);
    /// assert_eq!(v, [7, 7, 7]);
    /// ```
    pub fn from_iter_in<I: IntoIterator<Item = T>>(iter: I, alloc: &'a A) -> Vec<'a, T, A> {
        let mut v = Vec::new_in(alloc);
        v.extend(iter);
        v
    }

    /// Creates a `Vec<'a, T>` directly from the raw components of another vector.
    ///
    /// # Safety
    ///
    /// This is highly unsafe, due to the number of invariants that aren't
    /// checked:
    ///
    /// * `ptr` needs to have been previously allocated via [`String`] / `Vec<'a, T>`
    ///   (at least, it's highly likely to be incorrect if it wasn't).
    /// * `ptr`'s `T` needs to have the same size and alignment as it was allocated with.
    /// * `length` needs to be less than or equal to `capacity`.
    /// * `capacity` needs to be the capacity that the pointer was allocated with.
    ///
    /// Violating these may cause problems like corrupting the allocator's
    /// internal data structures. For example it is **not** safe
    /// to build a `Vec<u8>` from a pointer to a C `char` array and a `size_t`.
    ///
    /// The ownership of `ptr` is effectively transferred to the
    /// `Vec<'a, T>` which may then deallocate, reallocate or change the
    /// contents of memory pointed to by the pointer at will. Ensure
    /// that nothing else uses the pointer after calling this
    /// function.
    ///
    /// [`String`]: ../string/struct.String.html
    ///
    /// # Examples
    ///
    /// ```
    /// use bumpalo::{Bump, collections::Vec};
    ///
    /// use std::ptr;
    /// use std::mem;
    ///
    /// let b = Bump::new();
    ///
    /// let mut v = Vec::from_iter_in([1, 2, 3], &b);
    ///
    /// // Pull out the various important pieces of information about `v`
    /// let p = v.as_mut_ptr();
    /// let len = v.len();
    /// let cap = v.capacity();
    ///
    /// unsafe {
    ///     // Cast `v` into the void: no destructor run, so we are in
    ///     // complete control of the allocation to which `p` points.
    ///     mem::forget(v);
    ///
    ///     // Overwrite memory with 4, 5, 6
    ///     for i in 0..len as isize {
    ///         ptr::write(p.offset(i), 4 + i);
    ///     }
    ///
    ///     // Put everything back together into a Vec
    ///     let rebuilt = Vec::from_raw_parts_in(p, len, cap, &b);
    ///     assert_eq!(rebuilt, [4, 5, 6]);
    /// }
    /// ```
    pub unsafe fn from_raw_parts_in(
        ptr: *mut T,
        length: usize,
        capacity: usize,
        alloc: &'a A,
    ) -> Vec<'a, T, A> {
        Vec { buf: RawVec::from_raw_parts_in(ptr, length, capacity, alloc) }
    }

    /// Returns the number of elements in the vector, also referred to as its 'length'.
    ///
    /// # Examples
    ///
    /// ```
    /// use bumpalo::{Bump, collections::Vec};
    ///
    /// let b = Bump::new();
    ///
    /// let a = Vec::from_iter_in([1, 2, 3], &b);
    /// assert_eq!(a.len(), 3);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.buf.len_usize()
    }

    /// Get number of elements in the vector as a `usize`.
    #[inline]
    fn len_usize(&self) -> usize {
        self.buf.len_usize()
    }

    /// Get number of elements in the vector as a `u32`.
    #[inline]
    fn len_u32(&self) -> u32 {
        self.buf.len_u32()
    }

    /// Returns the number of elements the vector can hold without reallocating.
    ///
    /// # Examples
    ///
    /// ```
    /// use bumpalo::{Bump, collections::Vec};
    ///
    /// let b = Bump::new();
    /// let vec: Vec<i32> = Vec::with_capacity_in(10, &b);
    /// assert_eq!(vec.capacity(), 10);
    /// ```
    #[inline]
    pub fn capacity(&self) -> usize {
        self.buf.capacity_usize()
    }

    /// Get capacity of the vector as a `usize`.
    #[inline]
    fn capacity_usize(&self) -> usize {
        self.buf.capacity_usize()
    }

    /// Get capacity of the vector as a `u32`.
    #[inline]
    fn capacity_u32(&self) -> u32 {
        self.buf.capacity_u32()
    }

    /// Sets the length of a vector.
    ///
    /// This will explicitly set the size of the vector, without actually modifying its buffers,
    /// so it is up to the caller to ensure that the vector is actually the specified size.
    ///
    /// # Safety
    ///
    /// * `new_len` must be less than or equal to `u32::MAX`.
    /// * `new_len` must be less than or equal to [`capacity()`].
    /// * The elements at `old_len..new_len` must be initialized.
    ///
    /// [`capacity()`]: Vec::capacity
    ///
    /// # Examples
    ///
    /// ```
    /// use bumpalo::{Bump, collections::Vec};
    ///
    /// use std::ptr;
    ///
    /// let b = Bump::new();
    ///
    /// let mut vec = Vec::from_iter_in(['r', 'u', 's', 't'], &b);
    ///
    /// unsafe {
    ///     ptr::drop_in_place(&mut vec[3]);
    ///     vec.set_len(3);
    /// }
    /// assert_eq!(vec, ['r', 'u', 's']);
    /// ```
    ///
    /// In this example, there is a memory leak since the memory locations
    /// owned by the inner vectors were not freed prior to the `set_len` call:
    ///
    /// ```
    /// use bumpalo::{Bump, collections::Vec};
    ///
    /// let b = Bump::new();
    ///
    /// let mut vec = Vec::new_in(&b);
    /// vec.push("foo".to_string());
    ///
    /// unsafe {
    ///     vec.set_len(0);
    /// }
    /// ```
    ///
    /// In this example, the vector gets expanded from zero to four items
    /// but we directly initialize uninitialized memory:
    ///
    // TODO: rely upon `spare_capacity_mut`
    /// ```
    /// use bumpalo::{Bump, collections::Vec};
    ///
    /// let len = 4;
    /// let b = Bump::new();
    ///
    /// let mut vec: Vec<u8> = Vec::with_capacity_in(len, &b);
    ///
    /// for i in 0..len {
    ///     // SAFETY: we initialize memory via `pointer::write`
    ///     unsafe { vec.as_mut_ptr().add(i).write(b'a') }
    /// }
    ///
    /// unsafe {
    ///     vec.set_len(len);
    /// }
    ///
    /// assert_eq!(b"aaaa", &*vec);
    /// ```
    #[inline]
    pub unsafe fn set_len(&mut self, new_len: usize) {
        // Caller guarantees `new_len <= u32::MAX`, so `new_len as u32` cannot truncate `new_len`
        #[expect(clippy::cast_possible_truncation)]
        let new_len = new_len as u32;
        self.buf.set_len(new_len);
    }

    /// Reserves capacity for at least `additional` more elements to be inserted
    /// in the given `Vec<'a, T>`. The collection may reserve more space to avoid
    /// frequent reallocations. After calling `reserve`, capacity will be
    /// greater than or equal to `self.len() + additional`. Does nothing if
    /// capacity is already sufficient.
    ///
    /// # Panics
    ///
    /// Panics if the new capacity overflows `u32`.
    ///
    /// # Examples
    ///
    /// ```
    /// use bumpalo::{Bump, collections::Vec};
    ///
    /// let b = Bump::new();
    /// let mut vec = Vec::from_iter_in([1], &b);
    /// vec.reserve(10);
    /// assert!(vec.capacity() >= 11);
    /// ```
    pub fn reserve(&mut self, additional: usize) {
        self.buf.reserve(self.len_u32(), additional);
    }

    /// Reserves the minimum capacity for exactly `additional` more elements to
    /// be inserted in the given `Vec<'a, T>`. After calling `reserve_exact`,
    /// capacity will be greater than or equal to `self.len() + additional`.
    /// Does nothing if the capacity is already sufficient.
    ///
    /// Note that the allocator may give the collection more space than it
    /// requests. Therefore capacity can not be relied upon to be precisely
    /// minimal. Prefer `reserve` if future insertions are expected.
    ///
    /// # Panics
    ///
    /// Panics if the new capacity overflows `u32`.
    ///
    /// # Examples
    ///
    /// ```
    /// use bumpalo::{Bump, collections::Vec};
    ///
    /// let b = Bump::new();
    /// let mut vec = Vec::from_iter_in([1], &b);
    /// vec.reserve_exact(10);
    /// assert!(vec.capacity() >= 11);
    /// ```
    pub fn reserve_exact(&mut self, additional: usize) {
        self.buf.reserve_exact(self.len_u32(), additional);
    }

    /// Attempts to reserve capacity for at least `additional` more elements to be inserted
    /// in the given `Vec<'a, T>`. The collection may reserve more space to avoid
    /// frequent reallocations. After calling `try_reserve`, capacity will be
    /// greater than or equal to `self.len() + additional`. Does nothing if
    /// capacity is already sufficient.
    ///
    /// # Errors
    ///
    /// Returns `Err(AllocError)` if unable to reserve requested space in the `Vec`.
    ///
    /// # Examples
    ///
    /// ```
    /// use bumpalo::{Bump, collections::Vec};
    ///
    /// let b = Bump::new();
    /// let mut vec = Vec::from_iter_in([1], &b);
    /// vec.try_reserve(10).unwrap();
    /// assert!(vec.capacity() >= 11);
    /// ```
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), AllocError> {
        self.buf.try_reserve(self.len_u32(), additional)
    }

    /// Attempts to reserve the minimum capacity for exactly `additional` more elements to
    /// be inserted in the given `Vec<'a, T>`. After calling `try_reserve_exact`,
    /// capacity will be greater than or equal to `self.len() + additional`.
    /// Does nothing if the capacity is already sufficient.
    ///
    /// Note that the allocator may give the collection more space than it
    /// requests. Therefore capacity can not be relied upon to be precisely
    /// minimal. Prefer `try_reserve` if future insertions are expected.
    ///
    /// # Errors
    ///
    /// Returns `Err(AllocError)` if unable to reserve requested space in the `Vec`.
    ///
    /// # Examples
    ///
    /// ```
    /// use bumpalo::{Bump, collections::Vec};
    ///
    /// let b = Bump::new();
    /// let mut vec = Vec::from_iter_in([1], &b);
    /// vec.try_reserve_exact(10).unwrap();
    /// assert!(vec.capacity() >= 11);
    /// ```
    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), AllocError> {
        self.buf.try_reserve_exact(self.len_u32(), additional)
    }

    /// Shrinks the capacity of the vector as much as possible.
    ///
    /// It will drop down as close as possible to the length but the allocator
    /// may still inform the vector that there is space for a few more elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use bumpalo::{Bump, collections::Vec};
    ///
    /// let b = Bump::new();
    ///
    /// let mut vec = Vec::with_capacity_in(10, &b);
    /// vec.extend([1, 2, 3].iter().cloned());
    /// assert_eq!(vec.capacity(), 10);
    /// vec.shrink_to_fit();
    /// assert!(vec.capacity() >= 3);
    /// ```
    pub fn shrink_to_fit(&mut self) {
        if self.len_u32() != self.capacity_u32() {
            self.buf.shrink_to_fit(self.len_u32());
        }
    }

    /// Converts the vector into `&'a [T]`.
    ///
    /// # Examples
    ///
    /// ```
    /// use bumpalo::{Bump, collections::Vec};
    ///
    /// let b = Bump::new();
    /// let v = Vec::from_iter_in([1, 2, 3], &b);
    ///
    /// let slice = v.into_bump_slice();
    /// assert_eq!(slice, [1, 2, 3]);
    /// ```
    pub fn into_bump_slice(self) -> &'a [T] {
        unsafe {
            let ptr = self.as_ptr();
            let len = self.len_usize();
            // Don't need `mem::forget(self)` here, because `Vec` does not implement `Drop`.
            slice::from_raw_parts(ptr, len)
        }
    }

    /// Converts the vector into `&'a mut [T]`.
    ///
    /// # Examples
    ///
    /// ```
    /// use bumpalo::{Bump, collections::Vec};
    ///
    /// let b = Bump::new();
    /// let v = Vec::from_iter_in([1, 2, 3], &b);
    ///
    /// let mut slice = v.into_bump_slice_mut();
    ///
    /// slice[0] = 3;
    /// slice[2] = 1;
    ///
    /// assert_eq!(slice, [3, 2, 1]);
    /// ```
    pub fn into_bump_slice_mut(mut self) -> &'a mut [T] {
        let ptr = self.as_mut_ptr();
        let len = self.len_usize();
        // Don't need `mem::forget(self)` here, because `Vec` does not implement `Drop`.

        unsafe { slice::from_raw_parts_mut(ptr, len) }
    }

    /// Shortens the vector, keeping the first `len` elements and dropping
    /// the rest.
    ///
    /// If `len` is greater than the vector's current length, this has no
    /// effect.
    ///
    /// The [`drain`] method can emulate `truncate`, but causes the excess
    /// elements to be returned instead of dropped.
    ///
    /// Note that this method has no effect on the allocated capacity
    /// of the vector.
    ///
    /// # Examples
    ///
    /// Truncating a five element vector to two elements:
    ///
    /// ```
    /// use bumpalo::{Bump, collections::Vec};
    ///
    /// let b = Bump::new();
    ///
    /// let mut vec = Vec::from_iter_in([1, 2, 3, 4, 5], &b);
    /// vec.truncate(2);
    /// assert_eq!(vec, [1, 2]);
    /// ```
    ///
    /// No truncation occurs when `len` is greater than the vector's current
    /// length:
    ///
    /// ```
    /// use bumpalo::{Bump, collections::Vec};
    ///
    /// let b = Bump::new();
    ///
    /// let mut vec = Vec::from_iter_in([1, 2, 3], &b);
    /// vec.truncate(8);
    /// assert_eq!(vec, [1, 2, 3]);
    /// ```
    ///
    /// Truncating when `len == 0` is equivalent to calling the [`clear`]
    /// method.
    ///
    /// ```
    /// use bumpalo::{Bump, collections::Vec};
    ///
    /// let b = Bump::new();
    ///
    /// let mut vec = Vec::from_iter_in([1, 2, 3], &b);
    /// vec.truncate(0);
    /// assert_eq!(vec, []);
    /// ```
    ///
    /// [`clear`]: #method.clear
    /// [`drain`]: #method.drain
    pub fn truncate(&mut self, len: usize) {
        let current_len = self.len_usize();
        if len < current_len {
            // SAFETY: `len` is less than current len, so cannot be greater than `u32::MAX`,
            // and cannot be greater than `self.capacity()`.
            // We are shrinking `len`, so there are no concerns about initialized elements.
            unsafe { self.set_len(len) };
        }
    }

    /// Extracts a slice containing the entire vector.
    ///
    /// Equivalent to `&s[..]`.
    ///
    /// # Examples
    ///
    /// ```
    /// use bumpalo::{Bump, collections::Vec};
    /// use std::io::{self, Write};
    ///
    /// let b = Bump::new();
    ///
    /// let buffer = Vec::from_iter_in([1, 2, 3, 5, 8], &b);
    /// io::sink().write(buffer.as_slice()).unwrap();
    /// ```
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        self
    }

    /// Extracts a mutable slice of the entire vector.
    ///
    /// Equivalent to `&mut s[..]`.
    ///
    /// # Examples
    ///
    /// ```
    /// use bumpalo::{Bump, collections::Vec};
    /// use std::io::{self, Read};
    ///
    /// let b = Bump::new();
    /// let mut buffer = Vec::from_iter_in([0; 3], &b);
    /// io::repeat(0b101).read_exact(buffer.as_mut_slice()).unwrap();
    /// ```
    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        self
    }

    /// Returns a raw pointer to the vector's buffer, or a dangling raw pointer
    /// valid for zero sized reads if the vector didn't allocate.
    ///
    /// The caller must ensure that the vector outlives the pointer this
    /// function returns, or else it will end up pointing to garbage.
    /// Modifying the vector may cause its buffer to be reallocated,
    /// which would also make any pointers to it invalid.
    ///
    /// The caller must also ensure that the memory the pointer (non-transitively) points to
    /// is never written to (except inside an `UnsafeCell`) using this pointer or any pointer
    /// derived from it. If you need to mutate the contents of the slice, use [`as_mut_ptr`].
    ///
    /// # Examples
    ///
    /// ```
    /// use bumpalo::{Bump, collections::Vec};
    ///
    /// let arena = Bump::new();
    ///
    /// let x = Vec::from_iter_in([1, 2, 4], &arena);
    /// let x_ptr = x.as_ptr();
    ///
    /// unsafe {
    ///     for i in 0..x.len() {
    ///         assert_eq!(*x_ptr.add(i), 1 << i);
    ///     }
    /// }
    /// ```
    ///
    /// [`as_mut_ptr`]: Vec::as_mut_ptr
    #[inline]
    pub fn as_ptr(&self) -> *const T {
        // We shadow the slice method of the same name to avoid going through
        // `deref`, which creates an intermediate reference.
        let ptr = self.buf.ptr();
        // Inform compiler that `ptr` is not null. Very possibly compiler already knows this, as `ptr` is derived
        // from a `NonNull<T>`, but Bumpalo includes this line, and it shouldn't hurt, so we keep it.
        unsafe { assert_unchecked(!ptr.is_null()) };
        ptr
    }

    /// Returns an unsafe mutable pointer to the vector's buffer, or a dangling
    /// raw pointer valid for zero sized reads if the vector didn't allocate.
    ///
    /// The caller must ensure that the vector outlives the pointer this
    /// function returns, or else it will end up pointing to garbage.
    /// Modifying the vector may cause its buffer to be reallocated,
    /// which would also make any pointers to it invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// use bumpalo::{Bump, collections::Vec};
    ///
    /// let arena = Bump::new();
    ///
    /// // Allocate vector big enough for 4 elements.
    /// let size = 4;
    /// let mut x: Vec<i32> = Vec::with_capacity_in(size, &arena);
    /// let x_ptr = x.as_mut_ptr();
    ///
    /// // Initialize elements via raw pointer writes, then set length.
    /// unsafe {
    ///     for i in 0..size {
    ///         x_ptr.add(i).write(i as i32);
    ///     }
    ///     x.set_len(size);
    /// }
    /// assert_eq!(&*x, &[0, 1, 2, 3]);
    /// ```
    #[inline]
    pub fn as_mut_ptr(&mut self) -> *mut T {
        // We shadow the slice method of the same name to avoid going through
        // `deref_mut`, which creates an intermediate reference.
        let ptr = self.buf.ptr();
        // Inform compiler that `ptr` is not null. Very possibly compiler already knows this, as `ptr` is derived
        // from a `NonNull<T>`, but Bumpalo includes this line, and it shouldn't hurt, so we keep it.
        unsafe { assert_unchecked(!ptr.is_null()) };
        ptr
    }

    /// Removes an element from the vector and returns it.
    ///
    /// The removed element is replaced by the last element of the vector.
    ///
    /// This does not preserve ordering, but is O(1).
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use bumpalo::{Bump, collections::Vec};
    ///
    /// let b = Bump::new();
    ///
    /// let mut v = Vec::from_iter_in(["foo", "bar", "baz", "qux"], &b);
    ///
    /// assert_eq!(v.swap_remove(1), "bar");
    /// assert_eq!(v, ["foo", "qux", "baz"]);
    ///
    /// assert_eq!(v.swap_remove(0), "foo");
    /// assert_eq!(v, ["baz", "qux"]);
    /// ```
    #[inline]
    pub fn swap_remove(&mut self, index: usize) -> T {
        unsafe {
            // We replace self[index] with the last element. Note that if the
            // bounds check on hole succeeds there must be a last element (which
            // can be self[index] itself).
            let hole: *mut T = &raw mut self[index];
            let last = ptr::read(self.get_unchecked(self.len_usize() - 1));
            self.buf.decrease_len(1);
            ptr::replace(hole, last)
        }
    }

    /// Inserts an element at position `index` within the vector, shifting all
    /// elements after it to the right.
    ///
    /// # Panics
    ///
    /// Panics if `index > len`.
    ///
    /// # Examples
    ///
    /// ```
    /// use bumpalo::{Bump, collections::Vec};
    ///
    /// let b = Bump::new();
    ///
    /// let mut vec = Vec::from_iter_in([1, 2, 3], &b);
    /// vec.insert(1, 4);
    /// assert_eq!(vec, [1, 4, 2, 3]);
    /// vec.insert(4, 5);
    /// assert_eq!(vec, [1, 4, 2, 3, 5]);
    /// ```
    pub fn insert(&mut self, index: usize, element: T) {
        let len = self.len_usize();
        assert!(index <= len);

        // space for the new element
        if self.len_u32() == self.capacity_u32() {
            self.buf.grow_one();
        }

        unsafe {
            // infallible
            // The spot to put the new value
            {
                let p = self.as_mut_ptr().add(index);
                // Shift everything over to make space. (Duplicating the
                // `index`th element into two consecutive places.)
                ptr::copy(p, p.offset(1), len - index);
                // Write it in, overwriting the first copy of the `index`th
                // element.
                ptr::write(p, element);
            }
            self.buf.increase_len(1);
        }
    }

    /// Removes and returns the element at position `index` within the vector,
    /// shifting all elements after it to the left.
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use bumpalo::{Bump, collections::Vec};
    ///
    /// let b = Bump::new();
    ///
    /// let mut v = Vec::from_iter_in([1, 2, 3], &b);
    /// assert_eq!(v.remove(1), 2);
    /// assert_eq!(v, [1, 3]);
    /// ```
    pub fn remove(&mut self, index: usize) -> T {
        let len = self.len_usize();
        assert!(index < len);
        unsafe {
            // infallible
            let ret;
            {
                // the place we are taking from.
                let ptr = self.as_mut_ptr().add(index);
                // copy it out, unsafely having a copy of the value on
                // the stack and in the vector at the same time.
                ret = ptr::read(ptr);

                // Shift everything down to fill in that spot.
                ptr::copy(ptr.offset(1), ptr, len - index - 1);
            }
            self.set_len(len - 1);
            ret
        }
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all elements `e` for which `f(&e)` returns `false`.
    /// This method operates in place, visiting each element exactly once in the
    /// original order, and preserves the order of the retained elements.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use bumpalo::Bump;
    /// let arena = Bump::new();
    /// let mut vec = Vec::from_iter_in([1, 2, 3, 4], &arena);
    /// vec.retain(|&x| x % 2 == 0);
    /// assert_eq!(vec, [2, 4]);
    /// ```
    ///
    /// Because the elements are visited exactly once in the original order,
    /// external state may be used to decide which elements to keep.
    ///
    /// ```ignore
    /// use bumpalo::Bump;
    /// let arena = Bump::new();
    /// let mut vec = Vec::from_iter_in([1, 2, 3, 4, 5], &arena);
    /// let keep = [false, true, true, false, true];
    /// let mut iter = keep.iter();
    /// vec.retain(|_| *iter.next().unwrap());
    /// assert_eq!(vec, [2, 3, 5]);
    /// ```
    pub fn retain<F>(&mut self, mut f: F)
    where
        F: FnMut(&T) -> bool,
    {
        self.retain_mut(|x| f(x));
    }

    /// Retains only the elements specified by the predicate, passing a mutable reference to it.
    ///
    /// In other words, remove all elements `e` such that `f(&mut e)` returns `false`.
    /// This method operates in place, visiting each element exactly once in the
    /// original order, and preserves the order of the retained elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use bumpalo::{Bump, collections::Vec};
    ///
    /// let b = Bump::new();
    ///
    /// let mut vec = Vec::from_iter_in([1, 2, 3, 4], &b);
    /// vec.retain_mut(|x| if *x <= 3 {
    ///     *x += 1;
    ///     true
    /// } else {
    ///     false
    /// });
    /// assert_eq!(vec, [2, 3, 4]);
    /// ```
    // The implementation is based on the [`std::vec::Vec::retain_mut`].
    //
    // Allowing the following clippy rules just to make the code same as the original implementation.
    #[expect(clippy::items_after_statements, clippy::redundant_else)]
    pub fn retain_mut<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut T) -> bool,
    {
        let original_len = self.len_usize();

        if original_len == 0 {
            // Empty case: explicit return allows better optimization, vs letting compiler infer it
            return;
        }

        // Avoid double drop if the drop guard is not executed,
        // since we may make some holes during the process.
        unsafe { self.set_len(0) };

        // Vec: [Kept, Kept, Hole, Hole, Hole, Hole, Unchecked, Unchecked]
        //      |<-              processed len   ->| ^- next to check
        //                  |<-  deleted cnt     ->|
        //      |<-              original_len                          ->|
        // Kept: Elements which predicate returns true on.
        // Hole: Moved or dropped element slot.
        // Unchecked: Unchecked valid elements.
        //
        // This drop guard will be invoked when predicate or `drop` of element panicked.
        // It shifts unchecked elements to cover holes and `set_len` to the correct length.
        // In cases when predicate and `drop` never panick, it will be optimized out.
        struct BackshiftOnDrop<'a, 'v, T, A: Alloc> {
            v: &'v mut Vec<'a, T, A>,
            processed_len: usize,
            deleted_cnt: usize,
            original_len: usize,
        }

        impl<T, A: Alloc> Drop for BackshiftOnDrop<'_, '_, T, A> {
            fn drop(&mut self) {
                if self.deleted_cnt > 0 {
                    // SAFETY: Trailing unchecked items must be valid since we never touch them.
                    unsafe {
                        ptr::copy(
                            self.v.as_ptr().add(self.processed_len),
                            self.v.as_mut_ptr().add(self.processed_len - self.deleted_cnt),
                            self.original_len - self.processed_len,
                        );
                    }
                }
                // SAFETY: After filling holes, all items are in contiguous memory.
                unsafe {
                    self.v.set_len(self.original_len - self.deleted_cnt);
                }
            }
        }

        let mut g = BackshiftOnDrop { v: self, processed_len: 0, deleted_cnt: 0, original_len };

        fn process_loop<F, T, A: Alloc, const DELETED: bool>(
            original_len: usize,
            f: &mut F,
            g: &mut BackshiftOnDrop<'_, '_, T, A>,
        ) where
            F: FnMut(&mut T) -> bool,
        {
            while g.processed_len != original_len {
                // SAFETY: Unchecked element must be valid.
                let cur = unsafe { &mut *g.v.as_mut_ptr().add(g.processed_len) };
                if !f(cur) {
                    // Advance early to avoid double drop if `drop_in_place` panicked.
                    g.processed_len += 1;
                    g.deleted_cnt += 1;
                    // SAFETY: We never touch this element again after dropped.
                    unsafe { ptr::drop_in_place(cur) };
                    // We already advanced the counter.
                    if DELETED {
                        continue;
                    } else {
                        break;
                    }
                }
                if DELETED {
                    // SAFETY: `deleted_cnt` > 0, so the hole slot must not overlap with current element.
                    // We use copy for move, and never touch this element again.
                    unsafe {
                        let hole_slot = g.v.as_mut_ptr().add(g.processed_len - g.deleted_cnt);
                        ptr::copy_nonoverlapping(cur, hole_slot, 1);
                    }
                }
                g.processed_len += 1;
            }
        }

        // Stage 1: Nothing was deleted.
        process_loop::<F, T, A, false>(original_len, &mut f, &mut g);

        // Stage 2: Some elements were deleted.
        process_loop::<F, T, A, true>(original_len, &mut f, &mut g);

        // All item are processed. This can be optimized to `set_len` by LLVM.
        drop(g);
    }

    /// Creates an iterator that removes the elements in the vector
    /// for which the predicate returns `true` and yields the removed items.
    ///
    /// # Examples
    ///
    /// ```
    /// use bumpalo::Bump;
    /// use bumpalo::collections::{CollectIn, Vec};
    ///
    /// let b = Bump::new();
    ///
    /// let mut numbers = Vec::from_iter_in([1, 2, 3, 4, 5], &b);
    ///
    /// let evens: Vec<_> = numbers.drain_filter(|x| *x % 2 == 0).collect_in(&b);
    ///
    /// assert_eq!(numbers, &[1, 3, 5]);
    /// assert_eq!(evens, &[2, 4]);
    /// ```
    pub fn drain_filter<'v, F>(&'v mut self, filter: F) -> DrainFilter<'a, 'v, T, A, F>
    where
        F: FnMut(&mut T) -> bool,
    {
        let old_len = self.len_usize();

        // Guard against us getting leaked (leak amplification)
        unsafe {
            self.set_len(0);
        }

        DrainFilter { vec: self, idx: 0, del: 0, old_len, pred: filter }
    }

    /// Removes all but the first of consecutive elements in the vector that resolve to the same
    /// key.
    ///
    /// If the vector is sorted, this removes all duplicates.
    ///
    /// # Examples
    ///
    /// ```
    /// use bumpalo::{Bump, collections::Vec};
    ///
    /// let b = Bump::new();
    ///
    /// let mut vec = Vec::from_iter_in([10, 20, 21, 30, 20], &b);
    ///
    /// vec.dedup_by_key(|i| *i / 10);
    ///
    /// assert_eq!(vec, [10, 20, 30, 20]);
    /// ```
    #[inline]
    pub fn dedup_by_key<F, K>(&mut self, mut key: F)
    where
        F: FnMut(&mut T) -> K,
        K: PartialEq,
    {
        self.dedup_by(|a, b| key(a) == key(b))
    }

    /// Removes all but the first of consecutive elements in the vector satisfying a given equality
    /// relation.
    ///
    /// The `same_bucket` function is passed references to two elements from the vector and
    /// must determine if the elements compare equal. The elements are passed in opposite order
    /// from their order in the slice, so if `same_bucket(a, b)` returns `true`, `a` is removed.
    ///
    /// If the vector is sorted, this removes all duplicates.
    ///
    /// # Examples
    ///
    /// ```
    /// use bumpalo::{Bump, collections::Vec};
    ///
    /// let b = Bump::new();
    ///
    /// let mut vec = Vec::from_iter_in(["foo", "bar", "Bar", "baz", "bar"], &b);
    ///
    /// vec.dedup_by(|a, b| a.eq_ignore_ascii_case(b));
    ///
    /// assert_eq!(vec, ["foo", "bar", "baz", "bar"]);
    /// ```
    pub fn dedup_by<F>(&mut self, same_bucket: F)
    where
        F: FnMut(&mut T, &mut T) -> bool,
    {
        let len = {
            let (dedup, _) = partition_dedup_by(self.as_mut_slice(), same_bucket);
            dedup.len()
        };
        self.truncate(len);
    }

    /// Appends an element to the back of a vector.
    ///
    /// # Panics
    ///
    /// Panics if the number of elements in the vector overflows a `u32`.
    ///
    /// # Examples
    ///
    /// ```
    /// use bumpalo::{Bump, collections::Vec};
    ///
    /// let b = Bump::new();
    ///
    /// let mut vec = Vec::from_iter_in([1, 2], &b);
    /// vec.push(3);
    /// assert_eq!(vec, [1, 2, 3]);
    /// ```
    #[inline]
    pub fn push(&mut self, value: T) {
        // This will panic or abort if we would allocate > isize::MAX bytes
        // or if the length increment would overflow for zero-sized types.
        if self.len_u32() == self.capacity_u32() {
            self.buf.grow_one();
        }
        unsafe {
            let end = self.buf.ptr().add(self.len_usize());
            ptr::write(end, value);
            self.buf.increase_len(1);
        }
    }

    /// Removes the last element from a vector and returns it, or [`None`] if it
    /// is empty.
    ///
    /// [`None`]: https://doc.rust-lang.org/std/option/enum.Option.html#variant.None
    ///
    /// # Examples
    ///
    /// ```
    /// use bumpalo::{Bump, collections::Vec};
    ///
    /// let b = Bump::new();
    ///
    /// let mut vec = Vec::from_iter_in([1, 2, 3], &b);
    /// assert_eq!(vec.pop(), Some(3));
    /// assert_eq!(vec, [1, 2]);
    /// ```
    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        if self.len_u32() == 0 {
            None
        } else {
            unsafe {
                self.buf.decrease_len(1);
                Some(ptr::read(self.as_ptr().add(self.len_usize())))
            }
        }
    }

    /// Moves all the elements of `other` into `Self`, leaving `other` empty.
    ///
    /// # Panics
    ///
    /// Panics if the number of elements in the vector overflows a `u32`.
    /// Panics if the length of `Self` and `other` add up to more than `u32::MAX`.
    ///
    /// # Examples
    ///
    /// ```
    /// use bumpalo::{Bump, collections::Vec};
    ///
    /// let b = Bump::new();
    ///
    /// let mut vec = Vec::from_iter_in([1, 2, 3], &b);
    /// let mut vec2 = Vec::from_iter_in([4, 5, 6], &b);
    /// vec.append(&mut vec2);
    /// assert_eq!(vec, [1, 2, 3, 4, 5, 6]);
    /// assert_eq!(vec2, []);
    /// ```
    #[inline]
    pub fn append(&mut self, other: &mut Self) {
        unsafe {
            // SAFETY: The elements in `other` are made inaccessible by `other.set_len(0)` straight after
            self.append_elements(other.as_slice() as _);
            other.set_len(0);
        }
    }

    /// Appends elements to `Self` from other buffer.
    ///
    /// # SAFETY
    ///
    /// Elements from `other` will be copied into `self`'s buffer.
    /// Caller must ensure either that `T` is `Copy`, or the elements of `other` are not accessible
    /// except by the pointer `other`, and that they are not read after this call.
    #[inline]
    unsafe fn append_elements(&mut self, other: *const [T]) {
        // See https://github.com/oxc-project/oxc/pull/11092 for why this `#[allow]` attribute.
        // TODO: Remove this once we bump MSRV and it's no longer required.
        #[allow(clippy::needless_borrow, clippy::allow_attributes)]
        let count = (&*other).len();
        self.reserve(count);
        let len = self.len_usize();
        ptr::copy_nonoverlapping(other as *const T, self.as_mut_ptr().add(len), count);
        // `count` cannot be `> u32::MAX`, so `count as u32` cannot truncate `count`.
        // `self.buf.len + count` cannot be `> u32::MAX`.
        // If either of these conditions was violated, `self.reserve(count)` above would have panicked.
        // So this addition cannot wrap around.
        #[expect(clippy::cast_possible_truncation)]
        let count = count as u32;
        self.buf.increase_len(count);
    }

    /// Creates a draining iterator that removes the specified range in the vector
    /// and yields the removed items.
    ///
    /// Note 1: The element range is removed even if the iterator is only
    /// partially consumed or not consumed at all.
    ///
    /// Note 2: It is unspecified how many elements are removed from the vector
    /// if the `Drain` value is leaked.
    ///
    /// # Panics
    ///
    /// Panics if the starting point is greater than the end point or if
    /// the end point is greater than the length of the vector.
    ///
    /// # Examples
    ///
    /// ```
    /// use bumpalo::Bump;
    /// use bumpalo::collections::{CollectIn, Vec};
    ///
    /// let b = Bump::new();
    ///
    /// let mut v = Vec::from_iter_in([1, 2, 3], &b);
    ///
    /// let u: Vec<_> = v.drain(1..).collect_in(&b);
    ///
    /// assert_eq!(v, &[1]);
    /// assert_eq!(u, &[2, 3]);
    ///
    /// // A full range clears the vector
    /// v.drain(..);
    /// assert_eq!(v, &[]);
    /// ```
    pub fn drain<R>(&mut self, range: R) -> Drain<'_, '_, T, A>
    where
        R: RangeBounds<usize>,
    {
        // Memory safety
        //
        // When the Drain is first created, it shortens the length of
        // the source vector to make sure no uninitialized or moved-from elements
        // are accessible at all if the Drain's destructor never gets to run.
        //
        // Drain will ptr::read out the values to remove.
        // When finished, remaining tail of the vec is copied back to cover
        // the hole, and the vector length is restored to the new length.
        //
        let len = self.len_usize();
        let start = match range.start_bound() {
            Included(&n) => n,
            Excluded(&n) => n + 1,
            Unbounded => 0,
        };
        let end = match range.end_bound() {
            Included(&n) => n + 1,
            Excluded(&n) => n,
            Unbounded => len,
        };
        assert!(start <= end);
        assert!(end <= len);

        unsafe {
            // set self.vec length's to start, to be safe in case Drain is leaked
            self.set_len(start);
            // Use the borrow in the IterMut to indicate borrowing behavior of the
            // whole Drain iterator (like &mut T).
            let range_slice = slice::from_raw_parts_mut(self.as_mut_ptr().add(start), end - start);
            Drain {
                tail_start: end,
                tail_len: len - end,
                iter: range_slice.iter(),
                vec: NonNull::from(self),
            }
        }
    }

    /// Clears the vector, removing all values.
    ///
    /// Note that this method has no effect on the allocated capacity
    /// of the vector.
    ///
    /// # Examples
    ///
    /// ```
    /// use bumpalo::{Bump, collections::Vec};
    ///
    /// let b = Bump::new();
    ///
    /// let mut v = Vec::from_iter_in([1, 2, 3], &b);
    ///
    /// v.clear();
    ///
    /// assert!(v.is_empty());
    /// ```
    #[inline]
    pub fn clear(&mut self) {
        self.truncate(0)
    }

    /// Returns `true` if the vector contains no elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use bumpalo::{Bump, collections::Vec};
    ///
    /// let b = Bump::new();
    ///
    /// let mut v = Vec::new_in(&b);
    /// assert!(v.is_empty());
    ///
    /// v.push(1);
    /// assert!(!v.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.len_u32() == 0
    }

    /// Splits the collection into two at the given index.
    ///
    /// Returns a newly allocated vector. `self` contains elements `[0, at)`,
    /// and the returned vector contains elements `[at, len)`.
    ///
    /// Note that the capacity of `self` does not change.
    ///
    /// # Panics
    ///
    /// Panics if `at > len`.
    ///
    /// # Examples
    ///
    /// ```
    /// use bumpalo::{Bump, collections::Vec};
    ///
    /// let b = Bump::new();
    ///
    /// let mut vec = Vec::from_iter_in([1, 2, 3], &b);
    /// let vec2 = vec.split_off(1);
    /// assert_eq!(vec, [1]);
    /// assert_eq!(vec2, [2, 3]);
    /// ```
    #[inline]
    #[must_use]
    pub fn split_off(&mut self, at: usize) -> Self {
        assert!(at <= self.len_usize(), "`at` out of bounds");

        let other_len = self.len_usize() - at;
        // SAFETY: This method takes a `&mut self`. It lives for the duration of this method
        // - longer than we use `bump` for.
        let bump = unsafe { self.buf.bump() };
        let mut other = Vec::with_capacity_in(other_len, bump);

        // Unsafely `set_len` and copy items to `other`.
        unsafe {
            self.set_len(at);
            other.set_len(other_len);

            ptr::copy_nonoverlapping(self.as_ptr().add(at), other.as_mut_ptr(), other.len_usize());
        }
        other
    }
}

/*
#[cfg(feature = "boxed")]
impl<'a, T> Vec<'a, T> {
    /// Converts the vector into [`Box<[T]>`][owned slice].
    ///
    /// Note that this will drop any excess capacity.
    ///
    /// [owned slice]: ../../boxed/struct.Box.html
    ///
    /// # Examples
    ///
    /// ```
    /// use bumpalo::{Bump, collections::Vec, vec};
    ///
    /// let b = Bump::new();
    ///
    /// let v = Vec::from_iter_in([1, 2, 3], &b);
    ///
    /// let slice = v.into_boxed_slice();
    /// ```
    pub fn into_boxed_slice(mut self) -> crate::boxed::Box<'a, [T]> {
        use crate::boxed::Box;

        // Unlike `alloc::vec::Vec` shrinking here isn't necessary as `bumpalo::boxed::Box` doesn't own memory.
        unsafe {
            let slice = slice::from_raw_parts_mut(self.as_mut_ptr(), self.len);
            let output: Box<'a, [T]> = Box::from_raw(slice);
            mem::forget(self);
            output
        }
    }
}
*/

impl<'a, T: 'a + Clone, A: Alloc> Vec<'a, T, A> {
    /// Resizes the `Vec` in-place so that `len` is equal to `new_len`.
    ///
    /// If `new_len` is greater than `len`, the `Vec` is extended by the
    /// difference, with each additional slot filled with `value`.
    /// If `new_len` is less than `len`, the `Vec` is simply truncated.
    ///
    /// This method requires [`Clone`] to be able clone the passed value. If
    /// you need more flexibility (or want to rely on [`Default`] instead of
    /// [`Clone`]), use [`resize_with`].
    ///
    /// # Examples
    ///
    /// ```
    /// use bumpalo::{Bump, collections::Vec};
    ///
    /// let b = Bump::new();
    ///
    /// let mut vec = Vec::from_iter_in(["hello"], &b);
    /// vec.resize(3, "world");
    /// assert_eq!(vec, ["hello", "world", "world"]);
    ///
    /// let mut vec = Vec::from_iter_in([1, 2, 3, 4], &b);
    /// vec.resize(2, 0);
    /// assert_eq!(vec, [1, 2]);
    /// ```
    ///
    /// [`Clone`]: https://doc.rust-lang.org/std/clone/trait.Clone.html
    /// [`Default`]: https://doc.rust-lang.org/std/default/trait.Default.html
    /// [`resize_with`]: #method.resize_with
    pub fn resize(&mut self, new_len: usize, value: T) {
        let len = self.len_usize();

        if new_len > len {
            self.extend_with(new_len - len, ExtendElement(value))
        } else {
            self.truncate(new_len);
        }
    }

    /// Clones and appends all elements in a slice to the `Vec`.
    ///
    /// Iterates over the slice `other`, clones each element, and then appends
    /// it to this `Vec`. The `other` vector is traversed in-order.
    ///
    /// Note that this function is same as [`extend`] except that it is
    /// specialized to work with slices instead. If and when Rust gets
    /// specialization this function will likely be deprecated (but still
    /// available).
    ///
    /// # Examples
    ///
    /// ```
    /// use bumpalo::{Bump, collections::Vec};
    ///
    /// let b = Bump::new();
    ///
    /// let mut vec = Vec::from_iter_in([1], &b);
    /// vec.extend_from_slice(&[2, 3, 4]);
    /// assert_eq!(vec, [1, 2, 3, 4]);
    /// ```
    ///
    /// [`extend`]: #method.extend
    pub fn extend_from_slice(&mut self, other: &[T]) {
        self.extend(other.iter().cloned())
    }
}

impl<'a, T: 'a + Copy, A: Alloc> Vec<'a, T, A> {
    /// Helper method to copy all of the items in `other` and append them to the end of `self`.
    ///
    /// SAFETY:
    ///   * The caller is responsible for:
    ///       * calling [`reserve`](Self::reserve) beforehand to guarantee that there is enough
    ///         capacity to store `other.len()` more items.
    ///       * guaranteeing that `self` and `other` do not overlap.
    unsafe fn extend_from_slice_copy_unchecked(&mut self, other: &[T]) {
        let old_len = self.len_usize();
        debug_assert!(old_len + other.len() <= self.capacity_usize());

        // SAFETY:
        // * `src` is valid for reads of `other.len()` values by virtue of being a `&[T]`.
        // * `dst` is valid for writes of `other.len()` bytes because the caller of this
        //   method is required to `reserve` capacity to store at least `other.len()` items
        //   beforehand.
        // * Because `src` is a `&[T]` and dst is a `&[T]` within the `Vec<T>`,
        //   `copy_nonoverlapping`'s alignment requirements are met.
        // * Caller is required to guarantee that the source and destination ranges cannot overlap
        unsafe {
            let src = other.as_ptr();
            let dst = self.as_mut_ptr().add(old_len);
            ptr::copy_nonoverlapping(src, dst, other.len());
            self.set_len(old_len + other.len());
        }
    }

    /// Copies all elements in the slice `other` and appends them to the `Vec`.
    ///
    /// Note that this function is same as [`extend_from_slice`] except that it is optimized for
    /// slices of types that implement the `Copy` trait. If and when Rust gets specialization
    /// this function will likely be deprecated (but still available).
    ///
    /// To copy and append the data from multiple source slices at once, see
    /// [`extend_from_slices_copy`].
    ///
    /// # Examples
    ///
    /// ```
    /// use bumpalo::{Bump, collections::Vec};
    ///
    /// let b = Bump::new();
    ///
    /// let mut vec = Vec::from_iter_in([1], &b);
    /// vec.extend_from_slice_copy(&[2, 3, 4]);
    /// assert_eq!(vec, [1, 2, 3, 4]);
    /// ```
    ///
    /// ```
    /// use bumpalo::{Bump, collections::Vec};
    ///
    /// let b = Bump::new();
    ///
    /// let mut vec = Vec::from_iter_in(['H' as u8], &b);
    /// vec.extend_from_slice_copy("ello, world!".as_bytes());
    /// assert_eq!(vec, "Hello, world!".as_bytes());
    /// ```
    ///
    /// [`extend_from_slice`]: #method.extend_from_slice
    /// [`extend_from_slices_copy`]: #method.extend_from_slices_copy
    pub fn extend_from_slice_copy(&mut self, other: &[T]) {
        // Reserve space in the Vec for the values to be added
        self.reserve(other.len());

        // Copy values into the space that was just reserved
        // SAFETY:
        // * `self` has enough capacity to store `other.len()` more items as `self.reserve(other.len())`
        //   above guarantees that.
        // * Source and destination data ranges cannot overlap as we just reserved the destination
        //   range from the allocator.
        unsafe {
            self.extend_from_slice_copy_unchecked(other);
        }
    }

    /// For each slice in `slices`, copies all elements in the slice and appends them to the `Vec`.
    ///
    /// This method is equivalent to calling [`extend_from_slice_copy`] in a loop, but is able
    /// to precompute the total amount of space to reserve in advance. This reduces the potential
    /// maximum number of reallocations needed from one-per-slice to just one.
    ///
    /// # Panics
    ///
    /// Panics if unable to reserve sufficient capacity in the `Vec` for the slices.
    ///
    /// # Examples
    ///
    /// ```
    /// use bumpalo::{Bump, collections::Vec};
    ///
    /// let b = Bump::new();
    ///
    /// let mut vec = Vec::from_iter_in([1], &b);
    /// vec.extend_from_slices_copy(&[&[2, 3], &[], &[4]]);
    /// assert_eq!(vec, [1, 2, 3, 4]);
    /// ```
    ///
    /// ```
    /// use bumpalo::{Bump, collections::Vec};
    ///
    /// let b = Bump::new();
    ///
    /// let mut vec = Vec::from_iter_in(['H' as u8], &b);
    /// vec.extend_from_slices_copy(&["ello,".as_bytes(), &[], " world!".as_bytes()]);
    /// assert_eq!(vec, "Hello, world!".as_bytes());
    /// ```
    ///
    /// [`extend_from_slice_copy`]: #method.extend_from_slice_copy
    pub fn extend_from_slices_copy(&mut self, slices: &[&[T]]) {
        // Reserve the total amount we need to append the aggregated contents of all slices in `slices`.
        // We have to use checked addition here to guard against overflow if the total length would
        // exceed `usize::MAX`. Otherwise, we could reserve too little, and write out of bounds.
        let total_len = slices.iter().fold(0usize, |total_len, slice| {
            let len = slice.len();
            // Tell compiler that slices have maximum length of `isize::MAX`,
            // to help it remove checks on addition in some cases.
            // Strangely the compiler is unaware of this invariant.
            // SAFETY: Slices have maximum length of `isize::MAX`.
            unsafe { assert_unchecked!(len <= (isize::MAX as usize)) };
            total_len.checked_add(len).unwrap()
        });

        self.reserve(total_len);

        // SAFETY:
        // * `dst` is valid for writes of `total_len` items as `self.reserve(total_len)` above
        //   guarantees that.
        // * Source and destination ranges cannot overlap as we just reserved the destination
        //   range from the allocator.
        unsafe {
            // Copy the contents of each slice onto the end of `self`
            slices.iter().for_each(|slice| {
                self.extend_from_slice_copy_unchecked(slice);
            });
        }
    }
}

// This code generalises `extend_with_{element,default}`.
trait ExtendWith<T> {
    fn next(&mut self) -> T;
    fn last(self) -> T;
}

struct ExtendElement<T>(T);
impl<T: Clone> ExtendWith<T> for ExtendElement<T> {
    fn next(&mut self) -> T {
        self.0.clone()
    }
    fn last(self) -> T {
        self.0
    }
}

impl<'a, T: 'a, A: Alloc> Vec<'a, T, A> {
    /// Extend the vector by `n` values, using the given generator.
    fn extend_with<E: ExtendWith<T>>(&mut self, n: usize, mut value: E) {
        self.reserve(n);

        // std library version of this method updates `len` after writing each element,
        // because `value.next()` or `value.last()` could panic.
        // If that happens, `len` needs to contain all the elements written so far,
        // so they get dropped when the `Vec` is dropped.
        // But our `Vec` requires that `T` is not `Drop`, so we don't need to worry about that.
        unsafe {
            let mut ptr = self.as_mut_ptr().add(self.len_usize());

            // Write all elements except the last one
            for _ in 1..n {
                ptr::write(ptr, value.next());
                ptr = ptr.offset(1);
            }

            if n > 0 {
                // We can write the last element directly without cloning needlessly
                ptr::write(ptr, value.last());
            }

            // `n` and `self.len() + n` must both be `<= u32::MAX`, otherwise `self.reserve(n)` above
            // would have panicked. So `n as u32` cannot truncate `n`, and `len + n` cannot wrap.
            #[expect(clippy::cast_possible_truncation)]
            let n = n as u32;
            self.buf.increase_len(n);
        }
    }
}

impl<'a, T: 'a + PartialEq, A: Alloc> Vec<'a, T, A> {
    /// Removes consecutive repeated elements in the vector according to the
    /// [`PartialEq`] trait implementation.
    ///
    /// If the vector is sorted, this removes all duplicates.
    ///
    /// # Examples
    ///
    /// ```
    /// use bumpalo::{Bump, collections::Vec};
    ///
    /// let b = Bump::new();
    ///
    /// let mut vec = Vec::from_iter_in([1, 2, 2, 3, 2], &b);
    ///
    /// vec.dedup();
    ///
    /// assert_eq!(vec, [1, 2, 3, 2]);
    /// ```
    #[inline]
    pub fn dedup(&mut self) {
        self.dedup_by(|a, b| a == b)
    }
}

////////////////////////////////////////////////////////////////////////////////
// Common trait implementations for Vec
////////////////////////////////////////////////////////////////////////////////

impl<'a, T: 'a + Hash, A: Alloc> Hash for Vec<'a, T, A> {
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        Hash::hash(&**self, state)
    }
}

impl<T, A: Alloc, I: SliceIndex<[T]>> Index<I> for Vec<'_, T, A> {
    type Output = I::Output;

    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        Index::index(&**self, index)
    }
}

impl<T, A: Alloc, I: SliceIndex<[T]>> IndexMut<I> for Vec<'_, T, A> {
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        IndexMut::index_mut(&mut **self, index)
    }
}

impl<'a, T: 'a, A: Alloc> ops::Deref for Vec<'a, T, A> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        unsafe {
            let p = self.buf.ptr();
            // assume(!p.is_null());
            slice::from_raw_parts(p, self.len_usize())
        }
    }
}

impl<'a, T: 'a, A: Alloc> ops::DerefMut for Vec<'a, T, A> {
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe {
            let ptr = self.buf.ptr();
            // assume(!ptr.is_null());
            slice::from_raw_parts_mut(ptr, self.len_usize())
        }
    }
}

impl<'a, T: 'a, A: Alloc> IntoIterator for Vec<'a, T, A> {
    type Item = T;
    type IntoIter = IntoIter<'a, T>;

    /// Creates a consuming iterator, that is, one that moves each value out of
    /// the vector (from start to end). The vector cannot be used after calling
    /// this.
    ///
    /// # Examples
    ///
    /// ```
    /// use bumpalo::{Bump, collections::Vec};
    ///
    /// let b = Bump::new();
    ///
    /// let v = Vec::from_iter_in(["a".to_string(), "b".to_string()], &b);
    /// for s in v.into_iter() {
    ///     // s has type String, not &String
    ///     println!("{}", s);
    /// }
    /// ```
    #[inline]
    fn into_iter(mut self) -> IntoIter<'a, T> {
        unsafe {
            let begin = self.as_mut_ptr();
            // assume(!begin.is_null());
            let end = if size_of::<T>() == 0 {
                arith_offset(begin as *const i8, self.len_u32() as isize) as *const T
            } else {
                begin.add(self.len_usize()) as *const T
            };
            IntoIter { phantom: PhantomData, ptr: begin, end }
        }
    }
}

impl<'a, T, A: Alloc> IntoIterator for &'a Vec<'_, T, A> {
    type Item = &'a T;
    type IntoIter = slice::Iter<'a, T>;

    fn into_iter(self) -> slice::Iter<'a, T> {
        self.iter()
    }
}

impl<'a, T, A: Alloc> IntoIterator for &'a mut Vec<'_, T, A> {
    type Item = &'a mut T;
    type IntoIter = slice::IterMut<'a, T>;

    fn into_iter(self) -> slice::IterMut<'a, T> {
        self.iter_mut()
    }
}

impl<'a, T: 'a, A: Alloc> Extend<T> for Vec<'a, T, A> {
    #[inline]
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        let iterator = iter.into_iter();
        self.extend_desugared(iterator);
    }
}

impl<'a, T: 'a, A: Alloc> Vec<'a, T, A> {
    // leaf method to which various SpecFrom/SpecExtend implementations delegate when
    // they have no further optimizations to apply
    #[track_caller]
    fn extend_desugared<I: Iterator<Item = T>>(&mut self, mut iterator: I) {
        // This is the case for a general iterator.
        //
        // This function should be the moral equivalent of:
        //
        //      for item in iterator {
        //          self.push(item);
        //      }
        while let Some(element) = iterator.next() {
            let len = self.len_usize();
            if len == self.capacity_usize() {
                // This reallocation path is rarely taken, especially with prior reservation,
                // so mark it `#[cold]` and `#[inline(never)]` helps the compiler optimize the
                // common case, and prevents this cold path from being inlined to the `while` loop,
                // which increases the execution instructions and hits the performance.
                #[cold]
                #[inline(never)]
                fn reserve_slow<T, A: Alloc>(v: &mut Vec<T, A>, iterator: &impl Iterator) {
                    let (lower, _) = iterator.size_hint();
                    v.reserve(lower.saturating_add(1));
                }

                reserve_slow(self, &iterator);
            }
            unsafe {
                ptr::write(self.as_mut_ptr().add(len), element);
                // Since next() executes user code which can panic we have to bump the length
                // after each step.
                // NB can't overflow since we would have had to alloc the address space
                self.set_len(len + 1);
            }
        }
    }
}

impl<'a, T: 'a, A: Alloc> Vec<'a, T, A> {
    /// Creates a splicing iterator that replaces the specified range in the vector
    /// with the given `replace_with` iterator and yields the removed items.
    /// `replace_with` does not need to be the same length as `range`.
    ///
    /// Note 1: The element range is removed even if the iterator is not
    /// consumed until the end.
    ///
    /// Note 2: It is unspecified how many elements are removed from the vector,
    /// if the `Splice` value is leaked.
    ///
    /// Note 3: The input iterator `replace_with` is only consumed
    /// when the `Splice` value is dropped.
    ///
    /// Note 4: This is optimal if:
    ///
    /// * The tail (elements in the vector after `range`) is empty,
    /// * or `replace_with` yields fewer elements than `range`’s length
    /// * or the lower bound of its `size_hint()` is exact.
    ///
    /// Otherwise, a temporary vector is allocated and the tail is moved twice.
    ///
    /// # Panics
    ///
    /// Panics if the starting point is greater than the end point or if
    /// the end point is greater than the length of the vector.
    ///
    /// # Examples
    ///
    /// ```
    /// use bumpalo::{Bump, collections::Vec};
    ///
    /// let b = Bump::new();
    ///
    /// let mut v = Vec::from_iter_in([1, 2, 3], &b);
    /// let new = [7, 8];
    /// let u: Vec<_> = Vec::from_iter_in(v.splice(..2, new.iter().cloned()), &b);
    /// assert_eq!(v, &[7, 8, 3]);
    /// assert_eq!(u, &[1, 2]);
    /// ```
    #[inline]
    pub fn splice<R, I>(&mut self, range: R, replace_with: I) -> Splice<'_, '_, I::IntoIter, A>
    where
        R: RangeBounds<usize>,
        I: IntoIterator<Item = T>,
    {
        Splice { drain: self.drain(range), replace_with: replace_with.into_iter() }
    }
}

/// Extend implementation that copies elements out of references before pushing them onto the Vec.
///
/// This implementation is specialized for slice iterators, where it uses [`copy_from_slice`] to
/// append the entire slice at once.
///
/// [`copy_from_slice`]: https://doc.rust-lang.org/std/primitive.slice.html#method.copy_from_slice
impl<'a, T: 'a + Copy, A: Alloc> Extend<&'a T> for Vec<'_, T, A> {
    fn extend<I: IntoIterator<Item = &'a T>>(&mut self, iter: I) {
        self.extend(iter.into_iter().cloned())
    }
}

////////////////////////////////////////////////////////////////////////////////
// Comparison
////////////////////////////////////////////////////////////////////////////////

macro_rules! __impl_slice_eq1 {
    ($Rhs: ty) => {
        __impl_slice_eq1! { $Rhs, }
    };
    ($Rhs: ty, $($bounds:tt)*) => {
        impl<T, U, A, $($bounds)*> PartialEq<$Rhs> for Vec<'_, T, A>
        where
            T: PartialEq<U>,
            A: Alloc,
        {
            #[inline]
            fn eq(&self, other: &$Rhs) -> bool {
                self[..] == other[..]
            }
        }
    };
}

__impl_slice_eq1! { Vec<'_, U, A2>, A2: Alloc }
__impl_slice_eq1! { &[U] }
__impl_slice_eq1! { &mut [U] }

macro_rules! __impl_slice_eq1_array {
    ($Rhs: ty) => {
        impl<T, U, A, const N: usize> PartialEq<$Rhs> for Vec<'_, T, A>
        where
            T: PartialEq<U>,
            A: Alloc,
        {
            #[inline]
            fn eq(&self, other: &$Rhs) -> bool {
                self[..] == other[..]
            }
        }
    };
}

__impl_slice_eq1_array! { [U; N] }
__impl_slice_eq1_array! { &[U; N] }
__impl_slice_eq1_array! { &mut [U; N] }

/// Implements comparison of vectors, lexicographically.
impl<'a, T: 'a + PartialOrd, A: Alloc, A2: Alloc> PartialOrd<Vec<'a, T, A2>> for Vec<'a, T, A> {
    #[inline]
    fn partial_cmp(&self, other: &Vec<'a, T, A2>) -> Option<Ordering> {
        PartialOrd::partial_cmp(&**self, &**other)
    }
}

impl<'a, T: 'a + Eq, A: Alloc> Eq for Vec<'a, T, A> {}

/// Implements ordering of vectors, lexicographically.
impl<'a, T: 'a + Ord, A: Alloc> Ord for Vec<'a, T, A> {
    #[inline]
    fn cmp(&self, other: &Vec<'a, T, A>) -> Ordering {
        Ord::cmp(&**self, &**other)
    }
}

////////////////////////////////////////////////////////////////////////////////
// Misc
////////////////////////////////////////////////////////////////////////////////

impl<'a, T: 'a + fmt::Debug, A: Alloc> fmt::Debug for Vec<'a, T, A> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<'a, T: 'a, A: Alloc> AsRef<Vec<'a, T, A>> for Vec<'a, T, A> {
    fn as_ref(&self) -> &Vec<'a, T, A> {
        self
    }
}

impl<'a, T: 'a, A: Alloc> AsMut<Vec<'a, T, A>> for Vec<'a, T, A> {
    fn as_mut(&mut self) -> &mut Vec<'a, T, A> {
        self
    }
}

impl<'a, T: 'a, A: Alloc> AsRef<[T]> for Vec<'a, T, A> {
    fn as_ref(&self) -> &[T] {
        self
    }
}

impl<'a, T: 'a, A: Alloc> AsMut<[T]> for Vec<'a, T, A> {
    fn as_mut(&mut self) -> &mut [T] {
        self
    }
}

/*
#[cfg(feature = "boxed")]
impl<'a, T: 'a, A: Alloc> From<Vec<'a, T, A>> for crate::boxed::Box<'a, [T]> {
    fn from(v: Vec<'a, T, A>) -> crate::boxed::Box<'a, [T]> {
        v.into_boxed_slice()
    }
}
*/

impl<'a, T: 'a, A: Alloc> Borrow<[T]> for Vec<'a, T, A> {
    #[inline]
    fn borrow(&self) -> &[T] {
        &self[..]
    }
}

impl<'a, T: 'a, A: Alloc> BorrowMut<[T]> for Vec<'a, T, A> {
    #[inline]
    fn borrow_mut(&mut self) -> &mut [T] {
        &mut self[..]
    }
}

////////////////////////////////////////////////////////////////////////////////
// Clone-on-write
////////////////////////////////////////////////////////////////////////////////

// impl<'a, 'v, T: Clone, A: Alloc> From<Vec<'a, T, A>> for Cow<'v, [T]> {
//     fn from(v: Vec<'a, T, A>) -> Cow<'v, [T]> {
//         Cow::Owned(v)
//     }
// }

// impl<'a, 'v, T: Clone, A: Alloc> From<&'v Vec<'a, T, A>> for Cow<'v, [T]> {
//     fn from(v: &'v Vec<'a, T, A>) -> Cow<'v, [T]> {
//         Cow::Borrowed(v.as_slice())
//     }
// }

////////////////////////////////////////////////////////////////////////////////
// Iterators
////////////////////////////////////////////////////////////////////////////////

/// An iterator that moves out of a vector.
///
/// This `struct` is created by the [`Vec::into_iter`] method
/// (provided by the [`IntoIterator`] trait).
///
/// [`IntoIterator`]: https://doc.rust-lang.org/std/iter/trait.IntoIterator.html
pub struct IntoIter<'a, T> {
    phantom: PhantomData<&'a [T]>,
    ptr: *const T,
    end: *const T,
}

impl<T: fmt::Debug> fmt::Debug for IntoIter<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("IntoIter").field(&self.as_slice()).finish()
    }
}

impl<'a, T: 'a> IntoIter<'a, T> {
    /// Returns the remaining items of this iterator as a slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use bumpalo::{Bump, collections::Vec};
    ///
    /// let b = Bump::new();
    ///
    /// let vec = Vec::from_iter_in(['a', 'b', 'c'], &b);
    /// let mut into_iter = vec.into_iter();
    /// assert_eq!(into_iter.as_slice(), &['a', 'b', 'c']);
    /// let _ = into_iter.next().unwrap();
    /// assert_eq!(into_iter.as_slice(), &['b', 'c']);
    /// ```
    pub fn as_slice(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.ptr, self.len()) }
    }

    /// Returns the remaining items of this iterator as a mutable slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use bumpalo::{Bump, collections::Vec};
    ///
    /// let b = Bump::new();
    ///
    /// let vec = Vec::from_iter_in(['a', 'b', 'c'], &b);
    /// let mut into_iter = vec.into_iter();
    /// assert_eq!(into_iter.as_slice(), &['a', 'b', 'c']);
    /// into_iter.as_mut_slice()[2] = 'z';
    /// assert_eq!(into_iter.next().unwrap(), 'a');
    /// assert_eq!(into_iter.next().unwrap(), 'b');
    /// assert_eq!(into_iter.next().unwrap(), 'z');
    /// ```
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(self.ptr as *mut T, self.len()) }
    }
}

unsafe impl<T: Send> Send for IntoIter<'_, T> {}
unsafe impl<T: Sync> Sync for IntoIter<'_, T> {}

impl<'a, T: 'a> Iterator for IntoIter<'a, T> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<T> {
        unsafe {
            if std::ptr::eq(self.ptr, self.end) {
                None
            } else if size_of::<T>() == 0 {
                // purposefully don't use 'ptr.offset' because for
                // vectors with 0-size elements this would return the
                // same pointer.
                self.ptr = arith_offset(self.ptr as *const i8, 1) as *mut T;

                // Make up a value of this ZST.
                Some(mem::zeroed())
            } else {
                let old = self.ptr;
                self.ptr = self.ptr.offset(1);

                Some(ptr::read(old))
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let exact = if size_of::<T>() == 0 {
            (self.end as usize).wrapping_sub(self.ptr as usize)
        } else {
            unsafe { offset_from(self.end, self.ptr) as usize }
        };
        (exact, Some(exact))
    }

    #[inline]
    fn count(self) -> usize {
        self.len()
    }
}

impl<'a, T: 'a> DoubleEndedIterator for IntoIter<'a, T> {
    #[inline]
    fn next_back(&mut self) -> Option<T> {
        unsafe {
            if self.end == self.ptr {
                None
            } else if size_of::<T>() == 0 {
                // See above for why 'ptr.offset' isn't used
                self.end = arith_offset(self.end as *const i8, -1) as *mut T;

                // Make up a value of this ZST.
                Some(mem::zeroed())
            } else {
                self.end = self.end.offset(-1);

                Some(ptr::read(self.end))
            }
        }
    }
}

impl<'a, T: 'a> ExactSizeIterator for IntoIter<'a, T> {}

impl<'a, T: 'a> FusedIterator for IntoIter<'a, T> {}

impl<T> Drop for IntoIter<'_, T> {
    fn drop(&mut self) {
        // drop all remaining elements
        self.for_each(drop);
    }
}

/// A draining iterator for `Vec<'a, T>`.
///
/// This `struct` is created by the [`Vec::drain`] method.
pub struct Drain<'a, 's, T: 'a + 's, A: Alloc> {
    /// Index of tail to preserve
    tail_start: usize,
    /// Length of tail
    tail_len: usize,
    /// Current remaining range to remove
    iter: slice::Iter<'s, T>,
    vec: NonNull<Vec<'a, T, A>>,
}

impl<'a, 's, T: 'a + 's + fmt::Debug, A: Alloc> fmt::Debug for Drain<'a, 's, T, A> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("Drain").field(&self.iter.as_slice()).finish()
    }
}

// TODO: Should these also require `A: Send` / `A: Sync` bounds?
unsafe impl<T: Sync, A: Alloc> Sync for Drain<'_, '_, T, A> {}
unsafe impl<T: Send, A: Alloc> Send for Drain<'_, '_, T, A> {}

impl<T, A: Alloc> Iterator for Drain<'_, '_, T, A> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<T> {
        self.iter.next().map(|elt| unsafe { ptr::read(elt as *const _) })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<T, A: Alloc> DoubleEndedIterator for Drain<'_, '_, T, A> {
    #[inline]
    fn next_back(&mut self) -> Option<T> {
        self.iter.next_back().map(|elt| unsafe { ptr::read(elt as *const _) })
    }
}

impl<T, A: Alloc> Drop for Drain<'_, '_, T, A> {
    fn drop(&mut self) {
        // exhaust self first
        self.for_each(drop);

        if self.tail_len > 0 {
            unsafe {
                let source_vec = self.vec.as_mut();
                // memmove back untouched tail, update to new length
                let start = source_vec.len_usize();
                let tail = self.tail_start;
                if tail != start {
                    let src = source_vec.as_ptr().add(tail);
                    let dst = source_vec.as_mut_ptr().add(start);
                    ptr::copy(src, dst, self.tail_len);
                }
                source_vec.set_len(start + self.tail_len);
            }
        }
    }
}

impl<T, A: Alloc> ExactSizeIterator for Drain<'_, '_, T, A> {}

impl<T, A: Alloc> FusedIterator for Drain<'_, '_, T, A> {}

/// A splicing iterator for `Vec`.
///
/// This struct is created by the [`Vec::splice`] method. See its
/// documentation for more information.
#[derive(Debug)]
pub struct Splice<'a, 'd, I: Iterator + 'a + 'd, A: Alloc> {
    drain: Drain<'a, 'd, I::Item, A>,
    replace_with: I,
}

impl<I: Iterator, A: Alloc> Iterator for Splice<'_, '_, I, A> {
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.drain.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.drain.size_hint()
    }
}

impl<I: Iterator, A: Alloc> DoubleEndedIterator for Splice<'_, '_, I, A> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.drain.next_back()
    }
}

impl<I: Iterator, A: Alloc> ExactSizeIterator for Splice<'_, '_, I, A> {}

impl<I: Iterator, A: Alloc> Drop for Splice<'_, '_, I, A> {
    fn drop(&mut self) {
        self.drain.by_ref().for_each(drop);

        unsafe {
            if self.drain.tail_len == 0 {
                self.drain.vec.as_mut().extend(self.replace_with.by_ref());
                return;
            }

            // First fill the range left by drain().
            if !self.drain.fill(&mut self.replace_with) {
                return;
            }

            // There may be more elements. Use the lower bound as an estimate.
            // FIXME: Is the upper bound a better guess? Or something else?
            let (lower_bound, _upper_bound) = self.replace_with.size_hint();
            if lower_bound > 0 {
                self.drain.move_tail(lower_bound);
                if !self.drain.fill(&mut self.replace_with) {
                    return;
                }
            }

            // Collect any remaining elements.
            // This is a zero-length vector which does not allocate if `lower_bound` was exact.

            // SAFETY: `Splice` iterator is created in `Vec::splice`, which takes a `&mut self`.
            // `Splice` inherits the lifetime of `&mut self` from that method, so the mut borrow
            // of the `Vec` is held for the life of the `Splice`.
            // Therefore we have exclusive access to the `Vec` until end of this method.
            // That is longer than we use `bump` for.
            let bump = self.drain.vec.as_ref().buf.bump();

            let mut collected = Vec::new_in(bump);
            collected.extend(self.replace_with.by_ref());
            let mut collected = collected.into_iter();
            // Now we have an exact count.
            if collected.len() > 0 {
                self.drain.move_tail(collected.len());
                let filled = self.drain.fill(&mut collected);
                debug_assert!(filled);
                debug_assert_eq!(collected.len(), 0);
            }
        }
        // Let `Drain::drop` move the tail back if necessary and restore `vec.len`.
    }
}

/// Private helper methods for `Splice::drop`
impl<T, A: Alloc> Drain<'_, '_, T, A> {
    /// The range from `self.vec.len` to `self.tail_start` contains elements
    /// that have been moved out.
    /// Fill that range as much as possible with new elements from the `replace_with` iterator.
    /// Return whether we filled the entire range. (`replace_with.next()` didn’t return `None`.)
    unsafe fn fill<I: Iterator<Item = T>>(&mut self, replace_with: &mut I) -> bool {
        let vec = self.vec.as_mut();
        let range_start = vec.len_usize();
        let range_end = self.tail_start;
        let range_slice =
            slice::from_raw_parts_mut(vec.as_mut_ptr().add(range_start), range_end - range_start);

        for place in range_slice {
            if let Some(new_item) = replace_with.next() {
                ptr::write(place, new_item);
                vec.buf.increase_len(1);
            } else {
                return false;
            }
        }
        true
    }

    /// Make room for inserting more elements before the tail.
    unsafe fn move_tail(&mut self, extra_capacity: usize) {
        let vec = self.vec.as_mut();
        let used_capacity = self.tail_start + self.tail_len;
        // `used_capacity as u32` is safe because the [`Vec::drain`] method has ensured that
        // `self.tail_start` is less than or equal to `self.len()`, and `self.tail_len` calculated
        // from `self.len() - self.tail_start`, and `self.len()` is `u32`.
        #[expect(clippy::cast_possible_truncation)]
        let used_capacity = used_capacity as u32;
        vec.buf.reserve(used_capacity, extra_capacity);

        let new_tail_start = self.tail_start + extra_capacity;
        let src = vec.as_ptr().add(self.tail_start);
        let dst = vec.as_mut_ptr().add(new_tail_start);
        ptr::copy(src, dst, self.tail_len);
        self.tail_start = new_tail_start;
    }
}

/// An iterator produced by calling [`Vec::drain_filter`].
#[derive(Debug)]
pub struct DrainFilter<'a: 'v, 'v, T: 'a + 'v, A: Alloc, F>
where
    F: FnMut(&mut T) -> bool,
{
    vec: &'v mut Vec<'a, T, A>,
    idx: usize,
    del: usize,
    old_len: usize,
    pred: F,
}

impl<T, A: Alloc, F> Iterator for DrainFilter<'_, '_, T, A, F>
where
    F: FnMut(&mut T) -> bool,
{
    type Item = T;

    fn next(&mut self) -> Option<T> {
        unsafe {
            while self.idx != self.old_len {
                let i = self.idx;
                self.idx += 1;
                let v = slice::from_raw_parts_mut(self.vec.as_mut_ptr(), self.old_len);
                if (self.pred)(&mut v[i]) {
                    self.del += 1;
                    return Some(ptr::read(&raw const v[i]));
                } else if self.del > 0 {
                    let del = self.del;
                    let src: *const T = &raw const v[i];
                    let dst: *mut T = &raw mut v[i - del];
                    // This is safe because self.vec has length 0
                    // thus its elements will not have Drop::drop
                    // called on them in the event of a panic.
                    ptr::copy_nonoverlapping(src, dst, 1);
                }
            }
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.old_len - self.idx))
    }
}

impl<T, A: Alloc, F> Drop for DrainFilter<'_, '_, T, A, F>
where
    F: FnMut(&mut T) -> bool,
{
    fn drop(&mut self) {
        self.for_each(drop);
        unsafe {
            self.vec.set_len(self.old_len - self.del);
        }
    }
}

/*
#[cfg(feature = "std")]
impl<'a> io::Write for Vec<'a, u8> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.extend_from_slice_copy(buf);
        Ok(buf.len())
    }

    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.extend_from_slice_copy(buf);
        Ok(())
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
*/

/*
#[cfg(feature = "serde")]
mod serialize {
    use super::*;

    use serde::{Serialize, Serializer, ser::SerializeSeq};

    impl<'a, T> Serialize for Vec<'a, T>
    where
        T: Serialize,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let mut seq = serializer.serialize_seq(Some(self.len))?;
            for e in self.iter() {
                seq.serialize_element(e)?;
            }
            seq.end()
        }
    }
}
*/
