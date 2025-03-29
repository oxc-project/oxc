//! [`VecBuilder`] type and methods on [`Vec`] to utilize it to build a [`Vec`].
//!
//! [`VecBuilder`] stores a small number ([`VEC_BUILDER_STACK_CAPACITY`]) of items on the stack.
//!
//! If that capacity is not exceeded, no allocations will occur during the closure passed to
//! [`Vec::build_in`]. Only at the end of the closure, when the exact length is known,
//! then space for `len` items is allocated in the arena, and the contents are copied into that space.
//!
//! This results in a [`Vec`] with no excess capacity, so makes efficent use of memory in the arena.
//!
//! If more than [`VEC_BUILDER_STACK_CAPACITY`] items are pushed, it allocates to arena, and behaves
//! like a normal [`Vec`].
//!
//! The other advantage of `VecBuilder` is that [`VecBuilder::push`] is very cheap.
//! It will not need to grow until more than [`VEC_BUILDER_STACK_CAPACITY`] items have been pushed,
//! so the "full to capacity, need to grow" branch can be marked `#[cold]` and `#[inline(never)]`.
//! This makes `push` very small and it can be inlined.
//!
//! Checking whether the contents are stored inline or in arena ([`VecBuilder::is_on_stack`])
//! only happens on the cold path of `push` and in the final call to `into_vec`.
//! The only branch on hot path of `push` is checking if capacity has been reached yet.

use std::{
    alloc::Layout,
    mem::MaybeUninit,
    ptr::{self, NonNull},
};

use oxc_data_structures::assert_unchecked;

use crate::{Allocator, Vec};

/// Maximum capacity stored on stack while building a [`Vec`] with a [`VecBuilder`].
pub const VEC_BUILDER_STACK_CAPACITY: usize = 8;

/// Methods for constructing a [`Vec`] using a [`VecBuilder`].
impl<'a, T> Vec<'a, T> {
    /// Build [`Vec`] using a [`VecBuilder`].
    ///
    /// This uses "scratch space" on the stack for first [`VEC_BUILDER_STACK_CAPACITY`] items,
    /// and only allocates into the arena during the closure if `push` is called more times than that.
    ///
    /// At end of the closure, if on-stack capacity was not exceeded (less than
    /// [`VEC_BUILDER_STACK_CAPACITY`] items pushed), the contents stored on stack are moved into arena.
    /// A `Vec` is created which has no excess capacity - it uses up exactly as much memory as is
    /// required for the contents, and nothing more.
    ///
    /// The advantages of using `build_in` are:
    /// * For small `Vec`s, more efficient use of memory in the arena.
    /// * `push` is very cheap, because the slow path for "full to capacity, need to grow" is rarely hit.
    ///
    /// The disadvantages are:
    /// * Additional memory copying for `Vec`s with length of <= `VEC_BUILDER_STACK_CAPACITY`,
    ///   because values are stored on stack first and then copied into arena.
    ///
    /// Use this method if:
    /// * You do not know in advance how large the `Vec` will be.
    /// * `T` is small e.g. `Expression` or `Statement`.
    ///
    /// # Example
    /// ```
    /// use oxc_allocator::{Allocator, Vec};
    ///
    /// let allocator = Allocator::new();
    /// let vec = Vec::build_in(|v| {
    ///     v.push(100u64);
    ///     v.push(200u64);
    ///     v.push(300u64);
    /// }, &allocator);
    ///
    /// // `vec` has exactly the right capacity
    /// assert_eq!(vec.len(), 3);
    /// assert_eq!(vec.capacity(), 3);
    /// ```
    pub fn build_in<F>(f: F, allocator: &'a Allocator) -> Self
    where
        F: FnOnce(&mut VecBuilder<'a, T>),
    {
        const { Self::ASSERT_T_IS_NOT_DROP };

        let mut data = StackData::<T>::new();
        // SAFETY: `data` does not move until it's dropped at end of this function.
        // By that time, the `VecBuilder` has been consumed.
        let mut builder = unsafe { VecBuilder::<T>::new(&mut data, allocator) };
        f(&mut builder);
        builder.into_vec()
    }

    /// Build [`Vec`] using a [`VecBuilder`], with specified initial capacity.
    ///
    /// TODO: Docs
    ///
    /// # Panics
    ///
    /// Panics if `capacity` exceeds maximum.
    pub fn build_in_with_capacity<F>(capacity: usize, f: F, allocator: &'a Allocator) -> Self
    where
        F: FnOnce(&mut VecBuilder<'a, T>),
    {
        const { Self::ASSERT_T_IS_NOT_DROP };

        let mut data = StackData::<T>::new();
        // SAFETY: `data` does not move until it's dropped at end of this function.
        // By that time, the `VecBuilder` has been consumed.
        let mut builder = unsafe { VecBuilder::<T>::with_capacity(capacity, &mut data, allocator) };
        f(&mut builder);
        builder.into_vec()
    }

    /// Build [`Vec`] from an iterator using a [`VecBuilder`].
    ///
    /// The same pros / cons apply to this method as to [`Vec::build_in`].
    ///
    /// If you have an array, use [`Vec::from_array_in`].
    /// TODO: Other suggestions.
    pub fn build_from_iter_in<I>(iter: I, allocator: &'a Allocator) -> Self
    where
        I: IntoIterator<Item = T>,
    {
        const { Self::ASSERT_T_IS_NOT_DROP };

        Self::build_in(
            |builder| {
                let iter = iter.into_iter();
                for item in iter {
                    builder.push(item);
                }
            },
            allocator,
        )
    }
}

/// Data stored on stack while building a [`Vec`] with a [`VecBuilder`].
#[repr(transparent)]
struct StackData<T> {
    data: [MaybeUninit<T>; VEC_BUILDER_STACK_CAPACITY],
}

impl<T> StackData<T> {
    fn new() -> Self {
        Self { data: [const { MaybeUninit::uninit() }; VEC_BUILDER_STACK_CAPACITY] }
    }

    fn as_non_null(&mut self) -> NonNull<T> {
        // SAFETY: A `&mut` ref cannot be a null pointer.
        // `StackData` is `#[repr(transparent)]`, so a pointer to `StackData` is equivalent to a pointer
        // to start of `data` array.
        unsafe { NonNull::new_unchecked(ptr::from_mut(self).cast::<T>()) }
    }
}

/// Vec builder.
///
/// Provided to closure passed to [`Vec::build_in`].
pub struct VecBuilder<'a, T> {
    ptr: NonNull<T>,
    len: usize,
    capacity: usize,
    allocator: &'a Allocator,
}

impl<'a, T> VecBuilder<'a, T> {
    const MAX_CAPACITY: usize = {
        // This assertion is not needed as next line will cause a compile failure anyway
        // if `size_of::<T>() == 0`, due to division by zero.
        // But keep it anyway as soundness depends on it.
        assert!(size_of::<T>() > 0, "Zero sized types are not supported");
        // As it's always true that `size_of::<T>() >= align_of::<T>()` and `/` rounds down,
        // this fulfills `Layout`'s alignment requirement.
        // TODO: Or does it? What about `#[repr(packed)]`?
        let max_capacity = isize::MAX as usize / size_of::<T>();
        // Check `VEC_BUILDER_STACK_CAPACITY * 2` does not exceed `MAX_CAPACITY`.
        // This is relied upon by `grow_for_one`.
        assert!(VEC_BUILDER_STACK_CAPACITY * 2 <= max_capacity);
        max_capacity
    };

    /// Create a new [`VecBuilder`].
    ///
    /// # SAFETY
    /// `data` must not move for the duration that this [`VecBuilder`] exists.
    //
    // TODO: Enforce this with lifetime and make this method safe.
    unsafe fn new(data: &mut StackData<T>, allocator: &'a Allocator) -> Self {
        // Ensure size and alignment of `T` fulfills requirements
        let _ = Self::MAX_CAPACITY;

        let ptr = data.as_non_null();
        Self { ptr, len: 0, capacity: VEC_BUILDER_STACK_CAPACITY, allocator }
    }

    /// Create a new [`VecBuilder`] with specified capacity.
    ///
    /// If `capacity` is less than or equal to [`VEC_BUILDER_STACK_CAPACITY`], will not allocate into
    /// arena and will use on-stack storage. In that case, capacity of the `VecBuilder` will be
    /// `VEC_BUILDER_STACK_CAPACITY` (which may be more than requested).
    ///
    /// # Panics
    /// Panics if `capacity` is larger than [`Self::MAX_CAPACITY`].
    ///
    /// # SAFETY
    /// `data` must not move for the duration that this [`VecBuilder`] exists.
    //
    // TODO: Enforce this with lifetime and make this method safe.
    #[expect(clippy::todo)]
    unsafe fn with_capacity(
        capacity: usize,
        data: &mut StackData<T>,
        allocator: &'a Allocator,
    ) -> Self {
        if capacity <= VEC_BUILDER_STACK_CAPACITY {
            // SAFETY: Caller guarantees `data` does not move while this `VecBuilder` exists
            unsafe { Self::new(data, allocator) }
        } else {
            assert!(capacity <= Self::MAX_CAPACITY);
            // TODO: Allocate into arena
            todo!()
        }
    }

    /// Push a value to the [`VecBuilder`].
    //
    // `#[inline(always)]` because this is very cheap - the more expensive reallocation logic
    // is out of line in `push_slow`.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub fn push(&mut self, value: T) {
        if self.len == self.capacity {
            // SAFETY: We just checked that `self.len == self.capacity`
            unsafe { self.push_slow(value) };
        } else {
            // SAFETY: We just checked that there is capacity to write one more item
            unsafe { self.ptr.add(self.len).write(value) };
            self.len += 1;
        }
    }

    /// # SAFETY
    /// `VecBuilder` must be full to capacity. i.e. `self.len == self.capacity`.
    #[cold]
    #[inline(never)]
    unsafe fn push_slow(&mut self, value: T) {
        // SAFETY: Caller guarantees that `self.len == self.capacity`
        unsafe { self.grow_for_one() };

        // SAFETY: `grow_for_one` ensures there is sufficient capacity to write one more item
        unsafe { self.ptr.add(self.len).write(value) };
        self.len += 1;
    }

    /// Push a value created by closure to the [`VecBuilder`].
    ///
    /// This may be more performant than [`VecBuilder::push`], because it makes it easier
    /// for compiler to see that the value can be constructed directly in place.
    //
    // `#[inline(always)]` because this is very cheap - the more expensive reallocation logic
    // is out of line in `grow_for_one_cold`.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub fn push_with<F: FnOnce() -> T>(&mut self, f: F) {
        if self.len == self.capacity {
            // SAFETY: We just checked that `self.len == self.capacity`
            unsafe { self.grow_for_one_cold() };
        }

        // SAFETY: If there was not sufficient capacity, `grow_for_one_cold` ensured there is now,
        // so `ptr` is guaranteed to be in bounds
        unsafe {
            let ptr = self.ptr.add(self.len);
            let value = f();
            ptr.write(value);
        }

        self.len += 1;
    }

    /// Grow capacity enough to accommodate one more item.
    ///
    /// Just a wrapper around `grow_for_one` which is marked `#[cold]` and `#[inline(never)`.
    ///
    /// # SAFETY
    /// `VecBuilder` must be full to capacity. i.e. `self.len == self.capacity`.
    #[cold]
    #[inline(never)]
    unsafe fn grow_for_one_cold(&mut self) {
        // SAFETY: Caller guarantees that `self.len == self.capacity`
        unsafe { self.grow_for_one() };
    }

    /// Grow capacity enough to accommodate one more item.
    ///
    /// # SAFETY
    /// `VecBuilder` must be full to capacity. i.e. `self.len == self.capacity`.
    #[inline]
    unsafe fn grow_for_one(&mut self) {
        if self.is_on_stack() {
            // Allocate space for 16 in arena, and copy existing 8 from stack into that allocation.
            // SAFETY: Caller guarantees that `self.len == self.capacity`.
            // `is_on_stack` only returns `true` if `self.capacity == VEC_BUILDER_STACK_CAPACITY`.
            unsafe { assert_unchecked!(self.len == VEC_BUILDER_STACK_CAPACITY) };
            // SAFETY: Data cannot have been copied into arena already because capacity only increases,
            // and it's currently at minimum. We increase capacity below.
            // `self.len == VEC_BUILDER_STACK_CAPACITY`, which satisfies both:
            // * `self.len > 0`.
            // * `VEC_BUILDER_STACK_CAPACITY * 2 > self.len`.
            // Assertions in `const MAX_CAPACITY` ensure `VEC_BUILDER_STACK_CAPACITY * 2` is valid.
            self.ptr = unsafe { self.copy_into_arena(VEC_BUILDER_STACK_CAPACITY * 2) };
            self.capacity = VEC_BUILDER_STACK_CAPACITY * 2;
        } else {
            // TODO: Reallocate - double capacity
        }
    }

    /// Convert [`VecBuilder`] to [`Vec`].
    ///
    /// If the `VecBuilder` has not grown to above [`VEC_BUILDER_STACK_CAPACITY`], this moves the data
    /// from the `VecBuilder`'s stack storage into the arena, resulting in a `Vec` while no spare capacity.
    fn into_vec(self) -> Vec<'a, T> {
        if self.is_on_stack() {
            // Data is stored inline.
            // Allocate it into arena and create a `Vec`.
            if self.len == 0 {
                Vec::new_in(self.allocator)
            } else {
                // SAFETY: Data cannot have been copied into arena already because when it is copied over,
                // capacity is increased. Capacity is currently at minimum.
                // We checked that `self.len > 0` above.
                // `self.len` trivially satisfies requirement that `capacity >= self.len`.
                // `self.len` must be `<= self.capacity`, and existing capacity must be in legal range,
                // so `self.len` must be too.
                let ptr = unsafe { self.copy_into_arena(self.len).as_ptr() };
                // SAFETY: TODO
                unsafe { Vec::from_raw_parts_in(ptr, self.len, self.len, self.allocator) }
            }
        } else {
            // Data already stored in arena.
            // Convert to `Vec`.
            let ptr = self.ptr.as_ptr();
            // SAFETY: TODO
            unsafe { Vec::from_raw_parts_in(ptr, self.len, self.capacity, self.allocator) }
        }
    }

    /// Copy data from stack into arena.
    ///
    /// Returns a pointer to the start of the allocation.
    ///
    /// # SAFETY
    /// * Data must not have already been copied into arena.
    /// * `self.len` must be > 0.
    /// * `capacity` must be >= `self.len`.
    /// * `capacity` must be in range where `capacity * size_of::<T>() <= isize::MAX`.
    //
    // `#[inline(always)]` so that `self.len` and `capacity` can be statically known in the call to
    // this method in `grow_for_one`. That likely will mean `ptr::copy_nonoverlapping` call gets inlined
    // as a series of 16-byte xmm register loads/stores, rather than a `memcpy` function call.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    unsafe fn copy_into_arena(&self, capacity: usize) -> NonNull<T> {
        // TODO: Is it necessary to `assume_init` on the `MaybeUninit` array?

        let src = self.ptr.as_ptr().cast_const();

        // SAFETY: Caller guarantees that `capacity` is within legal range
        let layout = unsafe {
            Layout::from_size_align_unchecked(capacity * size_of::<T>(), align_of::<T>())
        };
        let dst = self.allocator.bump().alloc_layout(layout);
        let dst = dst.cast::<T>();

        // SAFETY: We have allocated space for `capacity` items at pointer `dst`.
        // Caller guarantees `capacity >= self.len`, so writing `self.len` x `T`s is within bounds
        // of the allocation.
        // Allocation is aligned for `T`.
        // `src` is aligned for `T` and points to `self.len` initialized `T`s.
        unsafe { ptr::copy_nonoverlapping(src, dst.as_ptr(), self.len) };

        dst
    }

    /// Returns `true` if contents of this [`VecBuilder`] are stored on stack.
    pub fn is_on_stack(&self) -> bool {
        self.capacity == VEC_BUILDER_STACK_CAPACITY
    }

    /// Get current length of `Vec` being built.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if this [`VecBuilder`] is empty (nothing pushed yet).
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Get current capacity of this [`VecBuilder`].
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}
