use std::{
    alloc::{self, Layout},
    mem::{align_of, size_of},
    ptr::{self, NonNull},
    slice,
};

use super::StackCapacity;

pub trait StackCommon<T>: StackCapacity<T> {
    // Getter + setter methods defined by implementer
    fn start(&self) -> NonNull<T>;
    fn end(&self) -> NonNull<T>;
    fn cursor(&self) -> NonNull<T>;
    fn set_start(&mut self, start: NonNull<T>);
    fn set_end(&mut self, end: NonNull<T>);
    fn set_cursor(&mut self, cursor: NonNull<T>);

    // Defined by implementer
    fn len(&self) -> usize;

    /// Make allocation of `capacity_bytes` bytes, aligned for `T`.
    ///
    /// # Panics
    /// Panics if out of memory (or may abort, depending on global allocator's behavior).
    ///
    /// # SAFETY
    /// * `capacity_bytes` must not be 0.
    /// * `capacity_bytes` must be a multiple of `size_of::<T>()`.
    /// * `capacity_bytes` must not exceed `Self::MAX_CAPACITY_BYTES`.
    #[inline]
    unsafe fn allocate(capacity_bytes: usize) -> (NonNull<T>, NonNull<T>) {
        // SAFETY: Caller guarantees `capacity_bytes` satisfies requirements
        let layout = unsafe { Self::layout_for(capacity_bytes) };

        // SAFETY: Caller guarantees `capacity_bytes` is not zero,
        // so `layout_for` produces a non-zero size layout
        let (start, end) = unsafe { allocate(layout) };

        // `layout_for` produces a layout with `T`'s alignment, so pointers are aligned for `T`
        (start.cast::<T>(), end.cast::<T>())
    }

    /// Grow allocation.
    ///
    /// `start` and `end` are set to the start and end of new allocation.
    /// `current` is set so distance from `start` is old `capacity_bytes`.
    /// This is where it should be if stack was previously full to capacity.
    ///
    /// # Panics
    /// Panics if stack is already at maximum capacity.
    ///
    /// # SAFETY
    /// Stack must have already allocated. i.e. `start` is not a dangling pointer.
    #[inline]
    unsafe fn grow(&mut self) {
        let old_start_ptr = self.start().cast::<u8>();
        // SAFETY: Caller guarantees stack has allocated.
        let old_layout = unsafe { Self::layout_for(self.capacity_bytes()) };

        // Grow allocation.
        // SAFETY:
        // `start` and `end` are boundaries of the allocation (`alloc` and `grow` ensure that).
        // So `old_start_ptr` and `old_layout` accurately describe the current allocation.
        // `grow` creates new allocation with byte size double what it currently is, or caps it
        // at `MAX_CAPACITY_BYTES`.
        // Old capacity in bytes was a multiple of `size_of::<T>()`, so double that must be too.
        // `MAX_CAPACITY_BYTES` is also a multiple of `size_of::<T>()`.
        // So new capacity in bytes must be a multiple of `size_of::<T>()`.
        // `MAX_CAPACITY_BYTES <= isize::MAX`.
        let (start, end, current) =
            unsafe { grow(old_start_ptr, old_layout, Self::MAX_CAPACITY_BYTES) };

        // Update pointers.
        // `start`, `end`, and `current` are all `NonNull` - just casting them.
        // All pointers returned from `grow` are aligned for `T`.
        // Old capacity and new capacity in bytes are both multiples of `size_of::<T>()`,
        // so distances `end - start` and `current - start` are both multiples of `size_of::<T>()`.
        self.set_start(start.cast::<T>());
        self.set_end(end.cast::<T>());
        self.set_cursor(current.cast::<T>());
    }

    /// Deallocate stack memory.
    ///
    /// Note: Does *not* drop the contents of the stack (the `T`s), only the memory allocated
    /// by `allocate` / `grow`. If stack is not empty, also call `drop_contents()` before calling this.
    ///
    /// # SAFETY
    /// Stack must have already allocated. i.e. `start` is not a dangling pointer.
    #[inline]
    unsafe fn deallocate(&self) {
        // SAFETY: Caller guarantees stack is allocated.
        let layout = unsafe { Self::layout_for(self.capacity_bytes()) };
        // SAFETY: `start` and `end` are boundaries of that allocation (`allocate` and `grow` ensure that).
        // So `start` and `layout` accurately describe the current allocation.
        unsafe { alloc::dealloc(self.start().as_ptr().cast::<u8>(), layout) };
    }

    /// Drop contents of stack.
    ///
    /// This function will be optimized out if `T` is non-drop, as `drop_in_place` calls
    /// `std::mem::needs_drop` internally and is a no-op if it returns true.
    ///
    /// # SAFETY
    /// * Stack must be allocated.
    /// * Stack must contain `self.len()` initialized entries, starting at `self.start()`.
    #[inline]
    unsafe fn drop_contents(&self) {
        // Drop contents. Next line copied from `std`'s `Vec`.
        // SAFETY: Caller guarantees stack contains `len` initialized entries, starting at `start`.
        unsafe {
            ptr::drop_in_place(ptr::slice_from_raw_parts_mut(self.start().as_ptr(), self.len()));
        }
    }

    /// Get layout for allocation of `capacity_bytes` bytes.
    ///
    /// # SAFETY
    /// * `capacity_bytes` must not be 0.
    /// * `capacity_bytes` must be a multiple of `size_of::<T>()`.
    /// * `capacity_bytes` must not exceed `Self::MAX_CAPACITY_BYTES`.
    #[inline]
    unsafe fn layout_for(capacity_bytes: usize) -> Layout {
        // `capacity_bytes` must not be 0 because cannot make 0-size allocations.
        debug_assert!(capacity_bytes > 0);
        // `capacity_bytes` must be a multiple of `size_of::<T>()` so that `cursor == end`
        // checks in `push` methods accurately detects when full to capacity
        debug_assert!(capacity_bytes.is_multiple_of(size_of::<T>()));
        // `capacity_bytes` must not exceed `Self::MAX_CAPACITY_BYTES` to prevent creating
        // an allocation of illegal size
        debug_assert!(capacity_bytes <= Self::MAX_CAPACITY_BYTES);

        // SAFETY: `align_of::<T>()` trivially satisfies alignment requirements.
        // Caller guarantees `capacity_bytes <= MAX_CAPACITY_BYTES`.
        // `MAX_CAPACITY_BYTES` takes into account the rounding-up by alignment requirement.
        unsafe { Layout::from_size_align_unchecked(capacity_bytes, align_of::<T>()) }
    }

    /// Get offset of `cursor` in number of `T`s.
    ///
    /// # SAFETY
    /// * `self.cursor()` and `self.start()` must be derived from same pointer.
    /// * `self.cursor()` must be `>= self.start()`.
    /// * Byte distance between `self.cursor()` and `self.start()` must be a multiple of `size_of::<T>()`.
    unsafe fn cursor_offset(&self) -> usize {
        // `offset_from_usize` returns offset in units of `T`.
        // SAFETY: Caller guarantees `cursor` and `start` are derived from same pointer.
        // This implies that both pointers are always within bounds of a single allocation.
        // Caller guarantees `cursor >= start`.
        // Caller guarantees distance between pointers is a multiple of `size_of::<T>()`.
        unsafe { self.cursor().offset_from_unsigned(self.start()) }
    }

    /// Get capacity.
    #[inline]
    fn capacity(&self) -> usize {
        // SAFETY: `allocate` and `grow` both ensure:
        // * `start` and `end` are both derived from same pointer
        // * `start` and `end` are both within bounds of a single allocation.
        // * `end` is always >= `start`.
        // * Distance between `start` and `end` is always a multiple of `size_of::<T>()`.
        unsafe { self.end().offset_from_unsigned(self.start()) }
    }

    /// Get capacity in bytes.
    #[inline]
    fn capacity_bytes(&self) -> usize {
        // SAFETY: `allocate` and `grow` both ensure:
        // * `start` and `end` are both derived from same pointer
        // * `start` and `end` are both within bounds of a single allocation.
        // * `end` is always >= `start`.
        // * Distance between `start` and `end` is always a multiple of `size_of::<T>()`.
        unsafe { self.end().byte_offset_from_unsigned(self.start()) }
    }

    /// Get contents of stack as a slice `&[T]`.
    #[inline]
    fn as_slice(&self) -> &[T] {
        // SAFETY: Stack always contains `self.len()` entries, starting at `self.start()`
        unsafe { slice::from_raw_parts(self.start().as_ptr(), self.len()) }
    }

    /// Get contents of stack as a mutable slice `&mut [T]`.
    #[inline]
    fn as_mut_slice(&mut self) -> &mut [T] {
        // SAFETY: Stack always contains `self.len()` entries, starting at `self.start()`
        unsafe { slice::from_raw_parts_mut(self.start().as_ptr(), self.len()) }
    }
}

/// Make allocation of with provided layout.
///
/// This is a separate non-generic function to improve compile time
/// (same pattern as `std::vec::Vec` uses).
///
/// # Panics
/// Panics if out of memory (or may abort, depending on global allocator's behavior).
///
/// # SAFETY
/// `layout` must have non-zero size.
#[inline]
unsafe fn allocate(layout: Layout) -> (/* start */ NonNull<u8>, /* end */ NonNull<u8>) {
    // SAFETY: Caller guarantees `layout` has non-zero-size
    let ptr = unsafe { alloc::alloc(layout) };

    let Some(start) = NonNull::new(ptr) else {
        alloc::handle_alloc_error(layout);
    };

    // SAFETY: We allocated `layout.size()` bytes, so `end` is end of allocation
    let end = unsafe { start.add(layout.size()) };

    (start, end)
}

/// Grow existing allocation.
///
/// Grow by doubling size, with high bound of `max_capacity_bytes`.
///
/// # SAFETY
/// * `old_start` and `old_layout` must describe an existing allocation.
/// * `max_capacity_bytes` must be `>= old_layout.size()`.
/// * `max_capacity_bytes` must be `<= isize::MAX`.
unsafe fn grow(
    old_start: NonNull<u8>,
    old_layout: Layout,
    max_capacity_bytes: usize,
) -> (/* start */ NonNull<u8>, /* end */ NonNull<u8>, /* current */ NonNull<u8>) {
    // Get new capacity.
    // Capacity in bytes cannot be larger than `isize::MAX`, so `* 2` cannot overflow.
    let old_capacity_bytes = old_layout.size();
    let mut new_capacity_bytes = old_capacity_bytes * 2;
    if new_capacity_bytes > max_capacity_bytes {
        assert!(old_capacity_bytes < max_capacity_bytes, "Cannot grow beyond `Self::MAX_CAPACITY`");
        new_capacity_bytes = max_capacity_bytes;
    }
    debug_assert!(new_capacity_bytes > old_capacity_bytes);

    // Reallocate.
    // SAFETY: Caller guarantees `old_start` and `old_layout` describe an existing allocation.
    // Caller guarantees that `max_capacity_bytes <= isize::MAX`.
    // `new_capacity_bytes` is capped above at `max_capacity_bytes`, so is a legal allocation size.

    // `start` and `end` are boundaries of that allocation (`alloc` and `grow` ensure that).
    // So `start` and `old_layout` accurately describe the current allocation.
    // `old_capacity_bytes` was a multiple of `size_of::<T>()`, so double that must be too.
    // `MAX_CAPACITY_BYTES` is also a multiple of `size_of::<T>()`.
    // So `new_capacity_bytes` must be a multiple of `size_of::<T>()`.
    // `new_capacity_bytes` is `<= MAX_CAPACITY_BYTES`, so is a legal allocation size.
    // `layout_for` produces a layout with `T`'s alignment, so `new_ptr` is aligned for `T`.
    let new_ptr = unsafe { alloc::realloc(old_start.as_ptr(), old_layout, new_capacity_bytes) };
    let Some(new_start) = NonNull::new(new_ptr) else {
        // SAFETY: See above
        let new_layout =
            unsafe { Layout::from_size_align_unchecked(new_capacity_bytes, old_layout.align()) };
        alloc::handle_alloc_error(new_layout);
    };

    // Update pointers.
    //
    // Stack was full to capacity, so new last index after push is the old capacity.
    // i.e. `new_cursor - new_start == old_end - old_start`.
    // Note: All pointers need to be updated even if allocation grew in place.
    // From docs for `GlobalAlloc::realloc`:
    // "Any access to the old `ptr` is Undefined Behavior, even if the allocation remained in-place."
    // <https://doc.rust-lang.org/std/alloc/trait.GlobalAlloc.html#method.realloc>
    // `end` changes whatever happens, so always need to be updated.
    // `cursor` needs to be derived from `start` to make `offset_from` valid, so also needs updating.
    //
    // SAFETY: We checked that `new_ptr` is non-null.
    // `old_capacity_bytes < new_capacity_bytes` (ensured above), so `new_cursor` must be in bounds.
    unsafe {
        let new_end = new_start.add(new_capacity_bytes);
        let new_cursor = new_start.add(old_capacity_bytes);
        (new_start, new_end, new_cursor)
    }
}
