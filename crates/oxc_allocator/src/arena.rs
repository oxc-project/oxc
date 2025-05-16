// Allocation methods are small and should always be inlined
#![expect(clippy::inline_always)]

// TODO: Make all alloc methods assert that `T` is not `Drop`.

use std::{
    alloc::{self, Layout},
    cell::Cell,
    cmp::Ordering,
    marker::PhantomData,
    ptr::{self, NonNull},
    slice, str,
};

use oxc_data_structures::pointer_ext::PointerExt;

/// Configuration trait for [`Arena`].
pub trait ArenaConfig {
    /// Minimum alignment of allocations in the arena.
    ///
    /// Types with lower alignment than `MIN_ALIGN` will be aligned on `MIN_ALIGN`.
    const MIN_ALIGN: usize;
}

/// Assertions for invariants of [`ArenaConfig`].
///
/// Blanket implemented for all types that implement [`ArenaConfig`].
///
/// [`ArenaConfigAsserts::CONFIG_ASSERTS`] must be referenced in all methods which create an [`Arena`].
trait ArenaConfigExt: ArenaConfig {
    const CONFIG_ASSERTS: () = {
        assert!(Self::MIN_ALIGN.is_power_of_two(), "`MIN_ALIGN` must be a power of 2",);
        assert!(Self::MIN_ALIGN <= CHUNK_ALIGN, "`MIN_ALIGN` may not be larger than `CHUNK_ALIGN`");
    };
}

impl<C: ArenaConfig> ArenaConfigExt for C {}

/// Default [`ArenaConfig`], with `MIN_ALIGN = 1`.
pub struct ArenaConfigDefault;

impl ArenaConfig for ArenaConfigDefault {
    const MIN_ALIGN: usize = 1;
}

/// Default [`Arena`], with `MIN_ALIGN = 1`.
pub type ArenaDefault = Arena<ArenaConfigDefault>;

/// TODO: Docs
#[expect(private_bounds)]
pub struct Arena<Config: ArenaConfigExt> {
    /// The current chunk we are allocating within
    current_chunk_footer: Cell<NonNull<ChunkFooter>>,
    _config: PhantomData<Config>,
}

#[repr(C)]
struct ChunkFooter {
    /// Pointer to the start of this chunk.
    start: NonNull<u8>,
    /// Bump allocation cursor that is always in the range `self.start..=self`.
    cursor: NonNull<u8>,
    /// Link to the previous chunk.
    ///
    /// The last node in the `prev` linked list is the canonical empty chunk [`EMPTY_CHUNK`],
    /// whose `previous_chunk` link points to itself.
    previous_chunk: NonNull<ChunkFooter>,
}

/// A wrapper type for the canonical, statically allocated empty chunk.
///
/// For the canonical empty chunk to be `static`, its type must be `Sync`,
/// which is the purpose of this wrapper type.
/// This is safe because the empty chunk is immutable and never actually modified.
#[repr(transparent)]
struct EmptyChunkFooter(ChunkFooter);

// SAFETY: `EmptyChunkFooter` is never mutated (see last comment above)
unsafe impl Sync for EmptyChunkFooter {}

// SAFETY: References cannot have null pointers, so `NonNull::new_unchecked` is sound.
// Creating a `NonNull` from a `&` reference (not `&mut` reference) is allowed,
// but the `NonNull` must never be used for writing.
const EMPTY_CHUNK_PTR: NonNull<ChunkFooter> =
    unsafe { NonNull::new_unchecked(ptr::from_ref(&EMPTY_CHUNK).cast::<ChunkFooter>().cast_mut()) };

static EMPTY_CHUNK: EmptyChunkFooter = EmptyChunkFooter(ChunkFooter {
    start: EMPTY_CHUNK_PTR.cast::<u8>(),
    previous_chunk: EMPTY_CHUNK_PTR,
    cursor: EMPTY_CHUNK_PTR.cast::<u8>(),
});

impl ChunkFooter {
    /// Get data capacity of chunk (including both used and unused regions).
    #[inline]
    fn capacity(&self) -> usize {
        // SAFETY: `self.start` is always before `self`, and both are within same allocation
        unsafe { ptr::from_ref(self).cast::<u8>().offset_from_usize(self.start.as_ptr()) }
    }

    /// Get number of bytes used to store data in this chunk.
    #[inline]
    fn used_bytes(&self) -> usize {
        // SAFETY: `self.cursor` is always before `self`, and both are within same allocation
        unsafe { ptr::from_ref(self).cast::<u8>().offset_from_usize(self.cursor.as_ptr()) }
    }

    /// Get number of bytes remaining which are free to store data in this chunk.
    #[inline]
    fn free_bytes(&self) -> usize {
        // SAFETY: `self.start` is always before `self.cursor`, and both are within same allocation
        unsafe { self.cursor.offset_from_usize(self.start) }
    }

    /// Returns the start and length of the region of this chunk which is currently filled with data.
    ///
    /// Does not include the `ChunkFooter`, or region of the chunk which is not filled with data.
    // TODO: Is this function needed?
    #[expect(dead_code)]
    fn as_raw_parts(&self) -> (*const u8, usize) {
        let start_ptr = self.start.as_ptr().cast::<u8>().cast_const();
        let cursor_ptr = self.cursor.as_ptr().cast::<u8>().cast_const();
        debug_assert!(start_ptr <= cursor_ptr);
        debug_assert!(cursor_ptr <= ptr::from_ref(self).cast::<u8>());
        // SAFETY: Chunk footer is always after cursor. Both are within the chunk's allocation.
        let len = unsafe { ptr::from_ref(self).cast::<u8>().offset_from_usize(cursor_ptr) };
        (cursor_ptr, len)
    }
}

/// The typical page size these days.
///
/// Note that we don't need to exactly match page size for correctness, and it's OK if this is smaller
/// than the real page size in practice. It isn't worth the portability concerns and lack of const
/// propagation that dynamically looking up the actual page size implies.
const TYPICAL_PAGE_SIZE: usize = 0x1000; // 4 KiB

/// Maximum typical overhead per allocation imposed by allocators.
const MALLOC_OVERHEAD: usize = 16;

/// We only support alignments of up to 16 bytes for `Arena::iter_allocated_chunks`.
// TODO: Why?
const SUPPORTED_ITER_ALIGNMENT: usize = 16;
const CHUNK_ALIGN: usize = SUPPORTED_ITER_ALIGNMENT;
const FOOTER_SIZE: usize = size_of::<ChunkFooter>();

// Ensure that `ChunkFooter` doesn't require higher alignment than `CHUNK_ALIGN`.
const _: () = assert!(align_of::<ChunkFooter>() <= CHUNK_ALIGN);

/// Total overhead from malloc, footer and alignment.
///
/// For instance, if we want to request a chunk of memory that has at least X bytes usable for
/// allocations (where `X` is aligned to `CHUNK_ALIGN`), then we expect that after adding a footer,
/// malloc overhead and alignment, the chunk of memory the allocator actually sets aside for us is
/// `X + OVERHEAD` rounded up to the nearest suitable size boundary.
const OVERHEAD: usize = (MALLOC_OVERHEAD + FOOTER_SIZE).next_multiple_of(CHUNK_ALIGN);

/// Maximum initial capacity for an [`Arena`].
pub const MAX_INITIAL_CAPACITY: usize = (isize::MAX as usize + 1) - FOOTER_SIZE - CHUNK_ALIGN;

#[expect(private_bounds)]
impl<Config: ArenaConfigExt> Arena<Config> {
    /// Construct a new arena to allocate into.
    ///
    /// ## Example
    /// ```
    /// # use oxc_allocator::__arena::ArenaDefault as Arena;
    /// let arena = Arena::new();
    /// ```
    #[inline]
    pub fn new() -> Self {
        const { Config::CONFIG_ASSERTS };

        Self { current_chunk_footer: Cell::new(EMPTY_CHUNK_PTR), _config: PhantomData }
    }

    /// Construct a new arena with the specified byte capacity to allocate into.
    ///
    /// Actual capacity may be larger than requested.
    ///
    /// ## Example
    /// ```
    /// # use oxc_allocator::__arena::ArenaDefault as Arena;
    /// let arena = Arena::with_capacity(100);
    /// ```
    ///
    /// ## Panics
    ///
    /// Panics if allocating the initial capacity fails.
    pub fn with_capacity(capacity: usize) -> Self {
        // Const validate `ArenaConfig`
        const { Config::CONFIG_ASSERTS };

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
        let footer_ptr = unsafe { Self::new_chunk(capacity, EMPTY_CHUNK_PTR) };

        Self { current_chunk_footer: Cell::new(footer_ptr), _config: PhantomData }
    }

    /// Allocate a new chunk with capacity for `capacity` bytes of data.
    ///
    /// # SAFETY
    /// TODO
    unsafe fn new_chunk(
        capacity: usize,
        previous_chunk_ptr: NonNull<ChunkFooter>,
    ) -> NonNull<ChunkFooter> {
        // SAFETY: TODO
        unsafe {
            // Allocate a slice of memory, large enough for `capacity` bytes + chunk footer
            let size = capacity + FOOTER_SIZE;
            let layout = Layout::from_size_align_unchecked(size, CHUNK_ALIGN);
            let start_ptr = alloc::alloc(layout);
            let start_ptr = NonNull::new(start_ptr).expect("Allocating chunk failed");

            // The `ChunkFooter` is at the end of the chunk
            let footer_ptr = start_ptr.add(capacity).cast::<ChunkFooter>();

            debug_assert_eq!((start_ptr.as_ptr() as usize) % CHUNK_ALIGN, 0);
            debug_assert_eq!((footer_ptr.as_ptr() as usize) % CHUNK_ALIGN, 0);

            let footer = ChunkFooter {
                start: start_ptr,
                // Cursor starts at end of the range
                cursor: footer_ptr.cast::<u8>(),
                previous_chunk: previous_chunk_ptr,
            };
            footer_ptr.write(footer);

            footer_ptr
        }
    }

    /// Reset this arena.
    ///
    /// Performs mass deallocation on everything allocated in this arena by resetting the pointer
    /// into the underlying chunk of memory to the start of the chunk.
    /// Does not run any `Drop` implementations on deallocated objects.
    ///
    /// If this arena has allocated multiple chunks to allocate into, all the chunks except the last
    /// (the biggest) are returned to the global allocator.
    ///
    /// ## Example
    /// ```
    /// # use oxc_allocator::__arena::ArenaDefault as Arena;
    /// let mut arena = Arena::new();
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
        // Takes `&mut self` so `self` must be unique and there can't be any
        // borrows active that would get invalidated by resetting

        // SAFETY: TODO
        unsafe {
            let mut last_chunk_ptr = self.current_chunk_footer.get();
            if last_chunk_ptr == EMPTY_CHUNK_PTR {
                // Never allocated. No chunks to free.
                return;
            }

            // Reset cursor of last chunk
            let last_chunk = last_chunk_ptr.as_mut();
            last_chunk.cursor = last_chunk_ptr.cast::<u8>();

            // If only one chunk, exit
            let previous_chunk_ptr = last_chunk.previous_chunk;
            if previous_chunk_ptr == EMPTY_CHUNK_PTR {
                return;
            }

            // Unlink last chunk from rest of the chain
            last_chunk.previous_chunk = EMPTY_CHUNK_PTR;

            // Free all previous chunks
            Self::deallocate_chunks(previous_chunk_ptr);
        }
    }

    /// # SAFETY
    /// TODO
    unsafe fn deallocate_chunks(mut chunk_ptr: NonNull<ChunkFooter>) {
        // SAFETY: TODO
        unsafe {
            loop {
                let chunk = chunk_ptr.as_ref();

                // Have to get pointer to next chunk before we deallocate this chunk
                let next_chunk_ptr = chunk.previous_chunk;

                // Deallocate chunk
                let start_ptr = chunk.start.as_ptr();
                let end_ptr = chunk_ptr.cast::<u8>().as_ptr();
                let size = end_ptr.offset_from_usize(start_ptr);
                let layout = Layout::from_size_align_unchecked(size, CHUNK_ALIGN);
                alloc::dealloc(start_ptr, layout);

                // If was first chunk, exit
                if next_chunk_ptr == EMPTY_CHUNK_PTR {
                    return;
                }

                // Continue on to the next
                chunk_ptr = next_chunk_ptr;
            }
        }
    }

    /// Allocate an object in this [`Arena`] and return an exclusive reference to it.
    ///
    /// # Panics
    /// Panics if reserving space for `T` fails.
    ///
    /// # Example
    /// ```
    /// # use oxc_allocator::__arena::ArenaDefault as Arena;
    /// let arena = Arena::new();
    /// let x = arena.alloc([1u8; 20]);
    /// assert_eq!(x, &[1u8; 20]);
    /// ```
    #[inline(always)]
    pub fn alloc<T>(&self, value: T) -> &mut T {
        const { assert!(!std::mem::needs_drop::<T>(), "Cannot allocate Drop type in arena") };

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
    /// # use oxc_allocator::__arena::ArenaDefault as Arena;
    /// let arena = Arena::new();
    /// let n = arena.alloc_with(|| 123u64);
    /// assert_eq!(*n, 123u64);
    /// ```
    #[expect(clippy::mut_from_ref)]
    #[inline(always)]
    pub fn alloc_with<F, T>(&self, f: F) -> &mut T
    where
        F: FnOnce() -> T,
    {
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
    /// # use oxc_allocator::__arena::ArenaDefault as Arena;
    /// let arena = Arena::new();
    /// let hello = arena.alloc_str("hello world");
    /// assert_eq!(hello, "hello world");
    /// ```
    #[inline(always)]
    pub fn alloc_str<'arena>(&'arena self, s: &str) -> &'arena str {
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
    /// # use oxc_allocator::__arena::ArenaDefault as Arena;
    /// let arena = Arena::new();
    /// let x = arena.alloc_slice_copy(&[1, 2, 3]);
    /// assert_eq!(x, &[1, 2, 3]);
    /// ```
    #[expect(clippy::mut_from_ref)]
    #[inline(always)]
    pub fn alloc_slice_copy<'arena, T: Copy>(&'arena self, slice: &[T]) -> &'arena mut [T] {
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
    /// The returned pointer points at uninitialized memory, and should be initialized with
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

        // SAFETY: TODO
        unsafe {
            let mut footer_ptr = self.current_chunk_footer.get();
            let chunk = footer_ptr.as_mut();

            debug_assert!(chunk.start <= chunk.cursor);
            debug_assert!(chunk.cursor <= footer_ptr.cast::<u8>());
            debug_assert!(chunk.cursor.as_ptr() as usize % Config::MIN_ALIGN == 0);

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
                    chunk.cursor.sub(aligned_size)
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
                    chunk.cursor.sub(aligned_size)
                }
                Ordering::Greater => {
                    // `Layout` guarantees that rounding the size up to its align cannot overflow.
                    // But it does not guarantee that the size is initially a multiple of the alignment,
                    // which is why we need to do this rounding.
                    let aligned_size = layout.size().next_multiple_of(layout.align());

                    let cursor_ptr = chunk.cursor.as_ptr();
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

            debug_assert_eq!((value_ptr.as_ptr() as usize) % layout.align(), 0);
            debug_assert_eq!((value_ptr.as_ptr() as usize) % Config::MIN_ALIGN, 0);
            debug_assert!(value_ptr >= chunk.start && value_ptr <= chunk.cursor);

            chunk.cursor = value_ptr;

            Some(value_ptr)
        }
    }

    /// Slow path allocation for when we need to allocate a new chunk because there isn't enough space
    /// in current chunk.
    #[inline(never)]
    #[cold]
    fn alloc_layout_slow(&self, _layout: Layout) -> NonNull<u8> {
        todo!(); // TODO
    }

    /// Calculate the total capacity of this [`Arena`] including all chunks, in bytes.
    ///
    /// Note: This is the total amount of memory the [`Arena`] owns NOT the total size of data
    /// that's been allocated in it. If you want the latter, use [`used_bytes`] instead.
    ///
    /// # Example
    /// ```
    /// # use oxc_allocator::__arena::ArenaDefault as Arena;
    /// let capacity = 64 * 1024; // 64 KiB
    /// let mut arena = Arena::with_capacity(capacity);
    /// arena.alloc(123u64); // 8 bytes
    ///
    /// // Result is the capacity (64 KiB), not the size of allocated data (8 bytes).
    /// // `Arena::with_capacity` may allocate a bit more than requested.
    /// assert!(arena.capacity() >= capacity);
    /// ```
    ///
    /// [`used_bytes`]: Arena::used_bytes
    pub fn capacity(&self) -> usize {
        let mut capacity = 0;
        let mut chunk_ptr = self.current_chunk_footer.get();
        while chunk_ptr != EMPTY_CHUNK_PTR {
            // SAFETY: `self.current_chunk_footer` always points to a valid initialized `ChunkFooter`
            let chunk = unsafe { chunk_ptr.as_ref() };
            capacity += chunk.capacity();
            chunk_ptr = chunk.previous_chunk;
        }
        capacity
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
    /// 2. Excess capacity in [`Vec`]s, [`String`]s and [`HashMap`]s.
    /// 3. Objects which were allocated but later dropped. [`Arena`] does not re-use allocations,
    ///    so anything which is allocated into arena continues to take up "dead space", even after it's
    ///    no longer referenced anywhere.
    /// 4. "Dead space" left over where a [`Vec`], [`String`] or [`HashMap`] has grown and had to make
    ///    a new allocation to accommodate its new larger size. Its old allocation continues to take up
    ///    "dead" space in the allocator.
    ///
    /// In practice, this almost always means that the result returned from this function will be an
    /// over-estimate vs the amount of "live" data in the arena.
    ///
    /// However, if you are using the result of this method to create a new `Arena` to clone
    /// an AST into, it is theoretically possible (though very unlikely) that it may be a slight
    /// under-estimate of the capacity required in new allocator to clone the AST into, depending
    /// on the order that `&str`s were allocated into arena in parser vs the order they get allocated
    /// during cloning. The order allocations are made in affects the amount of padding bytes required.
    ///
    /// # Example
    /// ```
    /// # use oxc_allocator::{__arena::ArenaDefault as Arena, Vec};
    /// let capacity = 64 * 1024; // 64 KiB
    /// let mut arena = Arena::with_capacity(capacity);
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
    /// [`String`]: crate::String
    /// [`HashMap`]: crate::HashMap
    pub fn used_bytes(&self) -> usize {
        let mut used_bytes = 0;
        let mut chunk_ptr = self.current_chunk_footer.get();
        while chunk_ptr != EMPTY_CHUNK_PTR {
            // SAFETY: `self.current_chunk_footer` always points to a valid initialized `ChunkFooter`
            let chunk = unsafe { chunk_ptr.as_ref() };
            used_bytes += chunk.used_bytes();
            chunk_ptr = chunk.previous_chunk;
        }
        used_bytes
    }
}

impl<Config: ArenaConfigExt> Default for Arena<Config> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
