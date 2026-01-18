// Allocation methods are small and should always be inlined
#![expect(clippy::inline_always)]

use std::{
    alloc::{self, Layout, handle_alloc_error},
    cell::Cell,
    cmp::{self, Ordering},
    iter::FusedIterator,
    marker::PhantomData,
    ptr::{self, NonNull},
    slice, str,
};

use oxc_data_structures::pointer_ext::PointerExt;

use super::{
    config::ArenaConfigExt,
    constants::{
        CHUNK_ALIGN, FIRST_CHUNK_DEFAULT_CAPACITY, FOOTER_SIZE, MAX_INITIAL_CAPACITY, OVERHEAD,
        TYPICAL_PAGE_SIZE,
    },
    footer::ChunkFooter,
};

/// Bump arena allocator.
///
/// Similar to [`bumpalo`], but with some tweaks.
///
/// [`Arena`] is configured with an [`ArenaConfig`].
///
/// TODO: More docs
///
/// [`bumpalo`]: https://docs.rs/bumpalo
/// [`ArenaConfig`]: super::config::ArenaConfig
#[expect(private_bounds)]
pub struct Arena<Config: ArenaConfigExt> {
    /// The current chunk we are allocating within
    current_chunk_footer: Cell<NonNull<ChunkFooter>>,
    _config: PhantomData<Config>,
}

#[expect(private_bounds)]
impl<Config: ArenaConfigExt> Arena<Config> {
    /// Construct a new arena to allocate into.
    ///
    /// ## Example
    /// ```
    /// # use oxc_allocator::__private::ArenaDefault;
    /// let arena = ArenaDefault::new();
    /// ```
    #[inline]
    pub fn new() -> Self {
        const { Config::ASSERTS };

        Self { current_chunk_footer: Cell::new(ChunkFooter::EMPTY), _config: PhantomData }
    }

    /// Construct a new arena with the specified byte capacity to allocate into.
    ///
    /// Actual capacity may be larger than requested.
    ///
    /// ## Example
    /// ```
    /// # use oxc_allocator::__private::ArenaDefault;
    /// let arena = ArenaDefault::with_capacity(100);
    /// ```
    ///
    /// ## Panics
    ///
    /// Panics if allocating the initial capacity fails.
    pub fn with_capacity(capacity: usize) -> Self {
        // Const validate `ArenaConfig`
        const { Config::ASSERTS };

        if capacity == 0 {
            return Self::new();
        }

        assert!(
            capacity <= MAX_INITIAL_CAPACITY,
            "`capacity` cannot exceed `MAX_INITIAL_CAPACITY`"
        );

        // TODO: Do we need this line?
        // Cannot overflow due to `capacity <= MAX_INITIAL_CAPACITY` check above
        let capacity = capacity.next_multiple_of(CHUNK_ALIGN);

        // We want our allocations to play nice with the system memory allocator, and waste as little
        // memory as possible. For small allocations, this means that the entire allocation including
        // the chunk footer and malloc's internal overhead is as close to a power of 2 as we can go
        // without going over. For larger allocations, we only need to get close to a page boundary
        // without going over.
        let capacity = if capacity < TYPICAL_PAGE_SIZE {
            // TODO: Just allocate a whole page? An arena smaller than 4 KiB is silly!
            (capacity + OVERHEAD).next_power_of_two() - OVERHEAD
        } else {
            // TODO: I think this can end up being `> isize::MAX`.
            // Maybe lower `MAX_INITIAL_CAPACITY` a bit?
            (capacity + OVERHEAD).next_multiple_of(TYPICAL_PAGE_SIZE) - OVERHEAD
        };

        // SAFETY: TODO
        let footer_ptr = unsafe {
            match Self::new_chunk(capacity, CHUNK_ALIGN, ChunkFooter::EMPTY) {
                Some(footer_ptr) => footer_ptr,
                None => alloc_failure(capacity, CHUNK_ALIGN),
            }
        };

        Self { current_chunk_footer: Cell::new(footer_ptr), _config: PhantomData }
    }

    /// Construct a static-sized [`Arena`] from an existing memory allocation.
    ///
    /// The [`Arena`] which is returned takes ownership of the memory allocation,
    /// and the allocation will be freed if the `Arena` is dropped.
    /// If caller wishes to prevent that happening, they must wrap the `Arena` in `ManuallyDrop`.
    ///
    /// # SAFETY
    ///
    /// * `ptr` must be aligned on at least [`CHUNK_ALIGN`].
    /// * `layout.size()` must be a multiple of [`CHUNK_ALIGN`].
    /// * `layout.size()` must be at least [`FOOTER_SIZE`].
    /// * `layout.align()` must be at least [`CHUNK_ALIGN`].
    /// * The memory region starting at `ptr` and encompassing `size` bytes must be within
    ///   a single allocation.
    pub unsafe fn from_raw_parts(ptr: NonNull<u8>, layout: Layout) -> Self {
        // Debug assert that `ptr` and `size` fulfill size and alignment requirements
        debug_assert!(is_multiple_of(ptr.as_ptr() as usize, CHUNK_ALIGN));
        debug_assert!(is_multiple_of(layout.size(), CHUNK_ALIGN));
        debug_assert!(layout.size() >= FOOTER_SIZE);
        debug_assert!(layout.align() >= CHUNK_ALIGN);

        // Write chunk footer into end of chunk.
        // SAFETY: TODO
        let footer_ptr = unsafe { ptr.add(layout.size() - FOOTER_SIZE).cast::<ChunkFooter>() };
        let footer = ChunkFooter {
            start: ptr,
            // Cursor starts at end of the range
            cursor: Cell::new(footer_ptr.cast::<u8>()),
            previous_chunk: ChunkFooter::EMPTY,
            alignment: layout.align(),
        };
        // SAFETY: TODO
        unsafe { footer_ptr.write(footer) };

        Self { current_chunk_footer: Cell::new(footer_ptr), _config: PhantomData }
    }

    /// Allocate an object in this [`Arena`] and return an exclusive reference to it.
    ///
    /// # Panics
    /// Panics if reserving space for `T` fails.
    ///
    /// # Example
    /// ```
    /// # use oxc_allocator::__private::ArenaDefault;
    /// let arena = ArenaDefault::new();
    /// let x = arena.alloc([1u8; 20]);
    /// assert_eq!(x, &[1u8; 20]);
    /// ```
    #[inline(always)]
    pub fn alloc<T>(&self, value: T) -> &mut T {
        const { assert!(!std::mem::needs_drop::<T>(), "Cannot allocate `Drop` type in arena") };

        self.alloc_with(|| value)
    }

    /// Pre-allocate space for a value in this [`Arena`], and initialize it using the closure.
    /// This method returns a mutable reference to the value in arena.
    ///
    /// ## Panics
    /// Panics if reserving space for `T` fails.
    ///
    /// ## Example
    /// ```
    /// # use oxc_allocator::__private::ArenaDefault;
    /// let arena = ArenaDefault::new();
    /// let n = arena.alloc_with(|| 123u64);
    /// assert_eq!(*n, 123u64);
    /// ```
    #[expect(clippy::mut_from_ref)]
    #[inline(always)]
    pub fn alloc_with<F, T>(&self, f: F) -> &mut T
    where
        F: FnOnce() -> T,
    {
        const { assert!(!std::mem::needs_drop::<T>(), "Cannot allocate `Drop` type in arena") };

        #[expect(clippy::items_after_statements)]
        #[inline(always)]
        unsafe fn inner_writer<T, F>(ptr: NonNull<T>, f: F)
        where
            F: FnOnce() -> T,
        {
            // This function is translated as:
            // * Allocate space for a `T` on the stack.
            // * Call `f()` with the return value being put onto this stack space.
            // * `memcpy` from the stack to the heap.
            //
            // Ideally we want LLVM to always realize that doing a stack allocation is unnecessary
            // and optimize the code so it writes directly into the heap instead.
            // It seems we get it to realize this most consistently if we put this critical line
            // into it's own function instead of inlining it into the surrounding code.
            // SAFETY: TODO
            unsafe { ptr.write(f()) };
        }

        let layout = Layout::new::<T>();

        // SAFETY: TODO
        unsafe {
            let ptr = self.alloc_layout(layout);
            let mut ptr = ptr.cast::<T>();
            inner_writer(ptr, f);
            ptr.as_mut()
        }
    }

    /// Copy a string slice into this [`Arena`] and return a reference to it.
    ///
    /// # Panics
    /// Panics if reserving space for the string fails.
    ///
    /// # Example
    /// ```
    /// # use oxc_allocator::__private::ArenaDefault;
    /// let arena = ArenaDefault::new();
    /// let hello = arena.alloc_str("hello world");
    /// assert_eq!(hello, "hello world");
    /// ```
    #[inline(always)]
    pub fn alloc_str<'arena>(&'arena self, s: &str) -> &'arena str {
        // `str` is not `Drop`, so need for const assertion about `Drop` types in arena

        let bytes = self.alloc_slice_copy(s.as_bytes());
        // SAFETY: `bytes` was created from a `&str`, so is guaranteed to be valid UTF-8
        unsafe { str::from_utf8_unchecked(bytes) }
    }

    /// Copy a slice into this [`Arena`] and return an exclusive reference to the copy in arena.
    ///
    /// ## Panics
    /// Panics if reserving space for the slice fails.
    ///
    /// ## Example
    /// ```
    /// # use oxc_allocator::__private::ArenaDefault;
    /// let arena = ArenaDefault::new();
    /// let x = arena.alloc_slice_copy(&[1, 2, 3]);
    /// assert_eq!(x, &[1, 2, 3]);
    /// ```
    #[expect(clippy::mut_from_ref)]
    #[inline(always)]
    pub fn alloc_slice_copy<'arena, T: Copy>(&'arena self, slice: &[T]) -> &'arena mut [T] {
        // `Copy` types cannot be `Drop`, so need for const assertion about `Drop` types in arena

        let layout = Layout::for_value(slice);
        let dst = self.alloc_layout(layout).cast::<T>();

        // SAFETY: TODO
        unsafe {
            ptr::copy_nonoverlapping(slice.as_ptr(), dst.as_ptr(), slice.len());
            slice::from_raw_parts_mut(dst.as_ptr(), slice.len())
        }
    }

    /// Allocate space for an object with the given [`Layout`].
    ///
    /// The returned pointer points to uninitialized memory, and should be initialized with
    /// [`std::ptr::write`].
    ///
    /// # Panics
    /// Panics if reserving space matching `layout` fails.
    #[inline(always)]
    pub fn alloc_layout(&self, layout: Layout) -> NonNull<u8> {
        if let Some(ptr) = self.alloc_layout_fast(layout) {
            ptr
        } else {
            self.alloc_layout_slow(layout)
        }
    }

    #[inline(always)]
    fn alloc_layout_fast(&self, layout: Layout) -> Option<NonNull<u8>> {
        // We don't need to check for ZSTs here since they will automatically be handled properly.
        // The pointer will be bumped by zero bytes, modulo alignment.
        // This keeps the fast path optimized for non-ZSTs, which are much more common.
        // Note: ZSTs includes zero-length slices and strings.
        //
        // `EMPTY_CHUNK` uses `Cell` for the cursor field so that it's placed in writable memory.
        // This allows ZST allocations to write to the cursor without causing SIGBUS.
        // The write is idempotent (writing the same value back) so it's safe.

        // SAFETY: TODO
        unsafe {
            let mut footer_ptr = self.current_chunk_footer.get();
            let chunk = footer_ptr.as_mut();

            debug_assert!(chunk.start <= chunk.cursor.get());
            debug_assert!(chunk.cursor.get() <= footer_ptr.cast::<u8>());
            debug_assert!(is_multiple_of(chunk.cursor.get().as_ptr() as usize, Config::MIN_ALIGN));

            // This `match` should be boiled away by LLVM. `MIN_ALIGN` is a constant and the layout's
            // alignment is also constant in practice after inlining
            let value_ptr = match layout.align().cmp(&Config::MIN_ALIGN) {
                Ordering::Less => {
                    // We need to round the size up to a multiple of `MIN_ALIGN` to preserve the
                    // minimum alignment. This cannot overflow, because `Layout` guarantees that
                    // `size <= isize::MAX`.
                    let aligned_size = layout.size().next_multiple_of(Config::MIN_ALIGN);
                    if aligned_size > chunk.free_bytes() {
                        return None;
                    }
                    chunk.cursor.get().sub(aligned_size)
                }
                Ordering::Equal => {
                    // `Layout` guarantees that rounding the size up to its align cannot overflow.
                    // But it does not guarantee that the size is initially a multiple of the alignment,
                    // which is why we need to do this rounding.
                    let aligned_size = layout.size().next_multiple_of(layout.align());

                    // TODO: `cursor_ptr` of an allocated chunk is aligned on 16.
                    // If we made `EmptyChunkFooter` aligned on 16 too, then `cursor_ptr` would
                    // always be >= 16.
                    // An unchecked assertion here to state that fact might help compiler,
                    // when `layout.size() <= 16`, to convert this code to:
                    // ```
                    // let value_ptr = cursor_ptr.wrapping_sub(aligned_size);
                    // if (value_ptr as usize) < (start_ptr as usize) {
                    //   return None;
                    // }
                    // value_ptr
                    // ```
                    // This is 2 less instructions.
                    //
                    // If `CHUNK_ALIGN` were higher (e.g. 256), and `EmptyChunkFooter` aligned on 256 too,
                    // then this optimization would apply for any allocation less than 256 size.
                    // i.e. almost all `Box` AST nodes, and `Vec<Statement>`s etc with length <= 16.
                    //
                    // Sadly unchecked assertion doesn't help: https://godbolt.org/z/vhTzx1GYs
                    // Would need to be a separate method for known size allocations.
                    if aligned_size > chunk.free_bytes() {
                        return None;
                    }
                    chunk.cursor.get().sub(aligned_size)
                }
                Ordering::Greater => {
                    // `Layout` guarantees that rounding the size up to its align cannot overflow.
                    // But it does not guarantee that the size is initially a multiple of the alignment,
                    // which is why we need to do this rounding.
                    let aligned_size = layout.size().next_multiple_of(layout.align());

                    let cursor_ptr = chunk.cursor.get().as_ptr();
                    let start_ptr = chunk.start.as_ptr();
                    // Must use `wrapping_sub` because `layout.align()` could be very large (e.g. `1 << 63`),
                    // so this could go out of bounds, or even wrap around the address space
                    let aligned_cursor_ptr =
                        cursor_ptr.wrapping_sub(cursor_ptr as usize & (layout.align() - 1));
                    let free_space = (aligned_cursor_ptr as usize).wrapping_sub(start_ptr as usize);
                    // TODO: How does this work?
                    if aligned_cursor_ptr < start_ptr || aligned_size > free_space {
                        return None;
                    }

                    let value_ptr = aligned_cursor_ptr.sub(aligned_size);
                    NonNull::new_unchecked(value_ptr)
                }
            };

            debug_assert!(is_multiple_of(value_ptr.as_ptr() as usize, layout.align()));
            debug_assert!(is_multiple_of(value_ptr.as_ptr() as usize, Config::MIN_ALIGN));
            debug_assert!(value_ptr >= chunk.start && value_ptr <= chunk.cursor.get());

            chunk.cursor.set(value_ptr);

            Some(value_ptr)
        }
    }

    /// Slow path allocation for when we need to add a new chunk because there isn't enough space
    /// in current chunk.
    #[inline(never)]
    #[cold]
    fn alloc_layout_slow(&self, layout: Layout) -> NonNull<u8> {
        let current_footer_ptr = self.current_chunk_footer.get();
        // SAFETY: TODO
        let current_chunk_capacity = unsafe { current_footer_ptr.as_ref().capacity() };

        // By default, we want our new chunk to be about twice as big as the previous chunk.
        // If the global allocator refuses it, we try to divide it by half until it works,
        // or the requested size is smaller than the default footer size.
        let align = cmp::max(layout.align(), CHUNK_ALIGN);

        let min_new_chunk_capacity = if layout.size() <= FIRST_CHUNK_DEFAULT_CAPACITY {
            FIRST_CHUNK_DEFAULT_CAPACITY
        } else {
            let size = layout.size().next_multiple_of(CHUNK_ALIGN) + FOOTER_SIZE;
            Layout::from_size_align(size, align).expect("layout too big");
            size - FOOTER_SIZE
        };

        let double_current_chunk_capacity = current_chunk_capacity * 2;
        let mut double_current_size =
            double_current_chunk_capacity.next_multiple_of(CHUNK_ALIGN) + FOOTER_SIZE;
        if double_current_size.next_multiple_of(align) > isize::MAX as usize {
            double_current_size -= align;
        }
        let target_capacity = double_current_size - FOOTER_SIZE;

        let mut try_capacity = cmp::max(min_new_chunk_capacity, target_capacity);

        let mut tried_minimum = false;
        let new_footer_ptr = loop {
            // SAFETY: TODO
            let footer_ptr = unsafe { Self::new_chunk(try_capacity, align, current_footer_ptr) };
            if let Some(footer_ptr) = footer_ptr {
                break footer_ptr;
            }

            try_capacity /= 2;
            if try_capacity < min_new_chunk_capacity {
                if tried_minimum {
                    // SAFETY: TODO
                    unsafe { alloc_failure(try_capacity, align) };
                }
                try_capacity = min_new_chunk_capacity;
                tried_minimum = true;
            }
        };

        // SAFETY: No other references exist to new `ChunkFooter`
        unsafe {
            debug_assert!(is_multiple_of(new_footer_ptr.as_ref().start.as_ptr() as usize, align));
            debug_assert!(new_footer_ptr.as_ref().capacity() >= layout.size());
        }

        // Set the new chunk as our new current chunk.
        self.current_chunk_footer.set(new_footer_ptr);

        // And then we can rely on `try_alloc_layout_fast` to allocate
        // space within this chunk.
        self.alloc_layout_fast(layout).unwrap()
    }

    /// Allocate space for `bytes` bytes at start of [`Arena`]'s current chunk.
    ///
    /// Returns a pointer to the start of an uninitialized section of `bytes` bytes.
    ///
    /// Note: [`alloc_layout`] allocates at *end* of the current chunk, because `Arena` bumps downwards,
    /// hence the need for this method, to allocate at *start* of current chunk.
    ///
    /// This method is dangerous, and should not ordinarily be used.
    ///
    /// This method moves the pointer to start of the current chunk forwards, so it no longer correctly
    /// describes the start of the allocation obtained from system allocator.
    ///
    /// The `Arena` **must not be allowed to be dropped** or it would be UB.
    /// Only use this method if you prevent that possibililty. e.g.:
    ///
    /// 1. Set the start pointer back to its correct value before it is dropped, using [`set_start_ptr`].
    /// 2. Wrap the `Arena` in `ManuallyDrop`, and deallocate it manually with the correct pointer.
    ///
    /// # Panics
    ///
    /// Panics if insufficient capacity for `bytes`
    /// (after rounding up to nearest multiple of [`CHUNK_ALIGN`]).
    ///
    /// # SAFETY
    ///
    /// `Arena` must not be dropped after calling this method (see above).
    ///
    /// [`alloc_layout`]: Self::alloc_layout
    /// [`set_start_ptr`]: Self::set_start_ptr
    pub unsafe fn alloc_bytes_start(&self, bytes: usize) -> NonNull<u8> {
        // Round up number of bytes to reserve to multiple of `CHUNK_ALIGN`,
        // so start pointer remains aligned on `CHUNK_ALIGN`
        let alloc_bytes = (bytes + CHUNK_ALIGN - 1) & !(CHUNK_ALIGN - 1);

        let start_ptr = self.start_ptr();
        let cursor_ptr = self.cursor_ptr();
        // SAFETY: Cursor pointer is always `>=` start pointer.
        // Both pointers are within same allocation, and derived from the same original pointer.
        let free_capacity = unsafe { cursor_ptr.offset_from_usize(start_ptr) };

        // Check sufficient capacity to write `alloc_bytes` bytes, without overwriting data already
        // stored in arena.
        // Could use `>=` here and it would be sufficient capacity, but use `>` instead so this assertion
        // fails if current chunk is the empty chunk and `bytes` is 0.
        assert!(free_capacity > alloc_bytes);

        // Calculate new start pointer.
        // SAFETY: We checked above that distance between start pointer and cursor is `>= alloc_bytes`,
        // so moving start pointer forwards by `alloc_bytes` cannot place it after cursor pointer.
        let new_start_ptr = unsafe { start_ptr.add(alloc_bytes) };

        // Set new start pointer.
        // SAFETY: `Arena` must have at least 1 allocated chunk or check for sufficient capacity
        // above would have failed.
        // Data pointer is always aligned on `CHUNK_ALIGN`, and we rounded `alloc_bytes` up to a multiple
        // of `CHUNK_ALIGN`, so that remains the case.
        unsafe { self.set_start_ptr(new_start_ptr) };

        // Return original start pointer
        start_ptr
    }

    /// Get start pointer for this [`Arena`]'s current chunk.
    pub fn start_ptr(&self) -> NonNull<u8> {
        // SAFETY: TODO
        let chunk_footer = unsafe { self.current_chunk_footer.get().as_ref() };
        chunk_footer.start
    }

    /// Set start pointer for this [`Arena`]'s current chunk.
    ///
    /// This is dangerous, and this method should not ordinarily be used.
    /// It is only here for manually writing data to start of the arena chunk,
    /// and then adjusting the start pointer to after it.
    ///
    /// # SAFETY
    ///
    /// * `Arena` must have at least 1 allocated chunk.
    ///   It is UB to call this method on an `Arena` which has not allocated
    ///   i.e. fresh from `Arena::new`.
    /// * `ptr` must point to within the allocation underlying this `Arena`.
    /// * `ptr` must be aligned on `CHUNK_ALIGN`.
    pub unsafe fn set_start_ptr(&self, ptr: NonNull<u8>) {
        debug_assert!(is_multiple_of(ptr.as_ptr() as usize, CHUNK_ALIGN));

        // SAFETY: Caller guarantees `Arena` has at least 1 allocated chunk,
        // so we can't be obtaining a mut reference to empty chunk footer
        let chunk_footer = unsafe { self.current_chunk_footer.get().as_mut() };
        chunk_footer.start = ptr;
    }

    /// Get cursor pointer for this [`Arena`]'s current chunk.
    fn cursor_ptr(&self) -> NonNull<u8> {
        // SAFETY: TODO
        let chunk_footer = unsafe { self.current_chunk_footer.get().as_ref() };
        chunk_footer.cursor.get()
    }

    /// Get pointer to end of the data region of this [`Arena`]'s current chunk
    /// (i.e. to the start of the `ChunkFooter`).
    pub fn data_end_ptr(&self) -> NonNull<u8> {
        self.current_chunk_footer.get().cast::<u8>()
    }

    /// Get pointer to end of this [`Arena`]'s current chunk (after the `ChunkFooter`).
    pub fn end_ptr(&self) -> NonNull<u8> {
        // SAFETY: `current_chunk_footer` returns pointer to a valid `ChunkFooter`,
        // so stepping past it cannot be out of bounds of the chunk's allocation.
        // If `Arena` has not allocated, so `current_chunk_footer` returns a pointer to the static
        // empty chunk, it's still valid.
        unsafe { self.current_chunk_footer.get().add(1).cast::<u8>() }
    }

    /// Allocate a new chunk with capacity for `capacity` bytes of data.
    ///
    /// # SAFETY
    /// * TODO: `capacity` constraint?
    /// * `alignment` must be a power of 2.
    /// * `alignment` must >= `CHUNK_ALIGN`.
    unsafe fn new_chunk(
        capacity: usize,
        alignment: usize,
        previous_chunk_ptr: NonNull<ChunkFooter>,
    ) -> Option<NonNull<ChunkFooter>> {
        // SAFETY: TODO
        unsafe {
            // Allocate a slice of memory, large enough for `capacity` bytes + chunk footer
            let size = capacity + FOOTER_SIZE;
            let layout = Layout::from_size_align_unchecked(size, alignment);
            let start_ptr = alloc::alloc(layout);
            let start_ptr = NonNull::new(start_ptr)?;

            // The `ChunkFooter` is at the end of the chunk
            let footer_ptr = start_ptr.add(capacity).cast::<ChunkFooter>();

            debug_assert!(is_multiple_of(start_ptr.as_ptr() as usize, alignment));
            debug_assert!(is_multiple_of(footer_ptr.as_ptr() as usize, alignment));

            let footer = ChunkFooter {
                start: start_ptr,
                // Cursor starts at end of the range
                cursor: Cell::new(footer_ptr.cast::<u8>()),
                previous_chunk: previous_chunk_ptr,
                alignment,
            };
            footer_ptr.write(footer);

            Some(footer_ptr)
        }
    }

    /// Reset this [`Arena`]`.
    ///
    /// Performs mass deallocation of everything allocated in this arena, by resetting the pointer
    /// into the underlying chunk of memory to the start of the chunk.
    ///
    /// If this arena has allocated multiple chunks, all the chunks except the last (the biggest)
    /// are returned to the global allocator.
    ///
    /// ## Example
    /// ```
    /// # use oxc_allocator::__private::ArenaDefault;
    /// let mut arena = ArenaDefault::new();
    ///
    /// // Allocate a bunch of things
    /// for i in 0..100 {
    ///     arena.alloc(i);
    /// }
    ///
    /// // Reset the arena
    /// arena.reset();
    ///
    /// // Allocate some new things in the space previously occupied by the original things
    /// for j in 200..400 {
    ///     arena.alloc(j);
    /// }
    /// ```
    pub fn reset(&mut self) {
        // Takes `&mut self` to guarantee no active borrows of data in the arena,
        // which would be invalidated by resetting

        let mut last_chunk_ptr = *self.current_chunk_footer.get_mut();
        if last_chunk_ptr == ChunkFooter::EMPTY {
            // `Arena` never allocated. No chunks to deallocate.
            return;
        }

        // Reset cursor of last chunk, and get pointer to previous chunk
        let mut chunk_ptr = {
            // SAFETY: Pointer always points to a valid initialized `ChunkFooter`.
            // Reference only lives during this block, so chunk is not referenced when it's deallocated below.
            let last_chunk = unsafe { last_chunk_ptr.as_mut() };

            // Reset cursor of last chunk
            last_chunk.cursor.set(last_chunk_ptr.cast::<u8>());

            // If only one chunk, exit
            let previous_chunk_ptr = last_chunk.previous_chunk;
            if previous_chunk_ptr == ChunkFooter::EMPTY {
                return;
            }

            // Unlink last chunk from rest of the chain
            last_chunk.previous_chunk = ChunkFooter::EMPTY;

            previous_chunk_ptr
        };

        // Deallocate all previous chunks
        loop {
            let (start_ptr, layout) = {
                // SAFETY: Pointer always points to a valid initialized `ChunkFooter`.
                // Reference only lives during this block, so chunk is not referenced when it's deallocated below.
                let chunk = unsafe { chunk_ptr.as_ref() };
                chunk_ptr = chunk.previous_chunk;
                chunk.start_ptr_and_layout()
            };

            // SAFETY: `start_ptr_and_layout` returns original layout chunk was allocated with
            unsafe { alloc::dealloc(start_ptr, layout) };

            // If was first chunk, exit
            if chunk_ptr == ChunkFooter::EMPTY {
                break;
            }
        }
    }

    /// Calculate the total capacity of this [`Arena`] across all its chunks, in bytes.
    ///
    /// Note: This is the total amount of memory the [`Arena`] owns NOT the total size of data
    /// that's been allocated in it. If you want the latter, use [`used_bytes`] instead.
    ///
    /// # Example
    /// ```
    /// # use oxc_allocator::__private::ArenaDefault;
    /// let capacity = 64 * 1024; // 64 KiB
    /// let mut arena = ArenaDefault::with_capacity(capacity);
    /// arena.alloc(123u64); // 8 bytes
    ///
    /// // Result is the capacity (64 KiB), not the size of allocated data (8 bytes).
    /// // `Arena::with_capacity` may allocate a bit more than requested.
    /// assert!(arena.capacity() >= capacity);
    /// ```
    ///
    /// [`used_bytes`]: Arena::used_bytes
    pub fn capacity(&self) -> usize {
        // SAFETY: We do not create any other references to `ChunkFooter`s while this iterator exists
        let iter = unsafe { ChunkFooterIter::new(self) };
        iter.map(ChunkFooter::capacity).sum()
    }

    /// Calculate the total size of data used in this [`Arena`], in bytes.
    ///
    /// This is the total amount of memory that has been *used* in the [`Arena`], NOT the amount of
    /// memory the [`Arena`] owns. If you want the latter, use [`capacity`] instead.
    ///
    /// The result includes:
    ///
    /// 1. Padding bytes between objects which have been allocated to preserve alignment of types
    ///    where they have different alignments or have larger-than-typical alignment.
    ///
    /// 2. Excess capacity in [`Vec`]s, [`String`]s and [`HashMap`]s.
    ///
    /// 3. Objects which were allocated but later dropped. [`Arena`] does not re-use allocations,
    ///    so anything which is allocated into arena continues to take up "dead space", even after it's
    ///    no longer referenced anywhere.
    ///
    /// 4. "Dead space" left over where a [`Vec`], [`StringBuilder`] or [`HashMap`] has grown
    ///    and had to make a new allocation to accommodate its new larger size.
    ///    Its old allocation continues to take up "dead" space in the arena.
    ///
    /// In practice, this almost always means that the result returned from this function will be an
    /// over-estimate vs the amount of "live" data in the arena.
    ///
    /// However, if you are using the result of this method to create a new `Arena` to clone
    /// an AST into, it is theoretically possible (though very unlikely) that it may be a slight
    /// under-estimate of the capacity required in new arena to clone the AST into, depending
    /// on the order that `&str`s were allocated into arena in parser vs the order they get allocated
    /// during cloning. The order allocations are made in affects the amount of padding bytes required.
    ///
    /// # Example
    /// ```ignore
    /// # use oxc_allocator::{__private::ArenaDefault, Vec};
    /// let capacity = 64 * 1024; // 64 KiB
    /// let mut arena = ArenaDefault::with_capacity(capacity);
    ///
    /// arena.alloc(1u8); // 1 byte with alignment 1
    /// arena.alloc(2u8); // 1 byte with alignment 1
    /// arena.alloc(3u64); // 8 bytes with alignment 8
    ///
    /// // Only 10 bytes were allocated, but 16 bytes were used, in order to align `3u64` on 8
    /// assert_eq!(arena.used_bytes(), 16);
    ///
    /// arena.reset();
    ///
    /// let mut vec = Vec::<u64>::with_capacity_in(2, &arena);
    ///
    /// // Allocate something else, so `vec`'s allocation is not the most recent
    /// arena.alloc(123u64);
    ///
    /// // `vec` has to grow beyond it's initial capacity
    /// vec.extend([1, 2, 3, 4]);
    ///
    /// // `vec` takes up 32 bytes, and `123u64` takes up 8 bytes = 40 total.
    /// // But there's an additional 16 bytes consumed for `vec`'s original capacity of 2,
    /// // which is still using up space
    /// assert_eq!(arena.used_bytes(), 56);
    /// ```
    ///
    /// [`capacity`]: Arena::capacity
    /// [`Vec`]: crate::Vec
    /// [`StringBuilder`]: crate::StringBuilder
    /// [`HashMap`]: crate::HashMap
    pub fn used_bytes(&self) -> usize {
        // SAFETY: We do not create any other references to `ChunkFooter`s while this iterator exists
        let iter = unsafe { ChunkFooterIter::new(self) };
        iter.map(ChunkFooter::used_bytes).sum()
    }
}

/// Iterator over all [`ChunkFooter`]s belonging to an [`Arena`], not including `EMPTY_CHUNK`.
struct ChunkFooterIter<'arena> {
    footer_ptr: NonNull<ChunkFooter>,
    _marker: PhantomData<&'arena ()>,
}

impl<'arena> ChunkFooterIter<'arena> {
    /// Create iterator over all [`ChunkFooter`]s belonging to an [`Arena`].
    ///
    /// # SAFETY
    ///
    /// Caller must not create a `&mut` reference to any [`ChunkFooter`] belonging to the [`Arena`]
    /// while this iterator exists.
    ///
    /// The iterator holds a borrow of [`Arena`] for the duration of the iterator's existence, but this
    /// is not sufficient to prevent aliasing violations, because [`Arena`] uses interior mutability.
    unsafe fn new<C: ArenaConfigExt>(arena: &'arena Arena<C>) -> Self {
        let footer_ptr = arena.current_chunk_footer.get();
        Self { footer_ptr, _marker: PhantomData }
    }
}

impl<'arena> Iterator for ChunkFooterIter<'arena> {
    type Item = &'arena ChunkFooter;

    fn next(&mut self) -> Option<&'arena ChunkFooter> {
        if self.footer_ptr == ChunkFooter::EMPTY {
            return None;
        }

        // SAFETY: This reference is returned to caller, and lives as long as the iterator.
        // `ChunkFooterIter::new` requires caller not to create `&mut` reference to any `ChunkFooter`
        // belonging to the arena while this iterator exists, so this reference cannot alias.
        let footer = unsafe { self.footer_ptr.as_ref() };
        self.footer_ptr = footer.previous_chunk;
        Some(footer)
    }
}

impl FusedIterator for ChunkFooterIter<'_> {}

/// [`Drop`] impl for [`Arena`].
impl<Config: ArenaConfigExt> Drop for Arena<Config> {
    /// Deallocate all the `Arena`'s chunks, returning them to global allocator.
    fn drop(&mut self) {
        let mut chunk_ptr = *self.current_chunk_footer.get_mut();
        while chunk_ptr != ChunkFooter::EMPTY {
            let (start_ptr, layout) = {
                // SAFETY: These pointers always point to a valid initialized `ChunkFooter`.
                // Reference only lives during this block, so chunk is not referenced when it's deallocated below.
                let chunk = unsafe { chunk_ptr.as_ref() };
                chunk_ptr = chunk.previous_chunk;
                chunk.start_ptr_and_layout()
            };

            // Deallocate chunk.
            // SAFETY: `start_ptr_and_layout` returns original layout chunk was allocated with
            unsafe { alloc::dealloc(start_ptr, layout) };
        }
    }
}

/// [`Default`] impl for [`Arena`].
impl<Config: ArenaConfigExt> Default for Arena<Config> {
    /// Create an empty [`Arena`] without pre-allocated capacity.
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// [`Send`] impl for [`Arena`].
///
/// SAFETY:
/// `Arena`s are safe to send between threads because nothing aliases its owned data until you start
/// allocating from it. But once you've allocated from it, the returned references to allocations
/// borrow the `Arena` and therefore prevent sending the `Arena` across threads until the borrows end.
///
/// IMPORTANT: `Arena` *cannot* be `Sync`, because it utilizes interior mutability. An `&Arena` cannot
/// be sent between threads without being wrapped in a structure which provides synchronization.
unsafe impl<Config: ArenaConfigExt> Send for Arena<Config> {}

/// Abort due to allocation failure (out of memory).
///
/// # SAFETY
/// TODO
#[cold]
#[inline(never)]
unsafe fn alloc_failure(capacity: usize, alignment: usize) -> ! {
    let size = capacity + FOOTER_SIZE;
    // SAFETY: TODO
    let layout = unsafe { Layout::from_size_align_unchecked(size, alignment) };
    handle_alloc_error(layout);
}

/// Returns `true` if `n` is a multiple of `divisor`.
#[inline]
const fn is_multiple_of(n: usize, divisor: usize) -> bool {
    n.is_multiple_of(divisor)
}
