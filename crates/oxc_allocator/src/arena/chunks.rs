//! Methods to get info about the chunks of memory allocated by an `Arena`, and associated iterator types.

use std::{
    iter::FusedIterator,
    marker::PhantomData,
    mem,
    ptr::{self, NonNull},
    slice,
};

use super::{Arena, CHUNK_FOOTER_SIZE, ChunkFooter};

impl<const MIN_ALIGN: usize> Arena<MIN_ALIGN> {
    /// Gets the remaining capacity in the current chunk (in bytes).
    ///
    /// # Example
    ///
    /// ```
    /// # use oxc_allocator::arena::Arena;
    ///
    /// let arena = Arena::with_capacity(100);
    ///
    /// let capacity = arena.chunk_capacity();
    /// assert!(capacity >= 100);
    /// ```
    pub fn chunk_capacity(&self) -> usize {
        self.cursor_ptr.get().as_ptr() as usize - self.start_ptr.get().as_ptr() as usize
    }

    /// Get an iterator over each chunk of allocated memory that this arena has allocated into.
    ///
    /// The chunks are returned ordered by allocation time, with the most recently allocated chunk
    /// returned first, and the least recently allocated chunk returned last.
    ///
    /// The values inside each chunk are also ordered by allocation time, with the most recent allocation
    /// earlier in the slice, and the least recent allocation towards the end of the slice.
    ///
    /// # SAFETY
    ///
    /// Because this method takes `&mut self`, we know that the arena reference is unique and therefore there
    /// aren't any active references to any of the objects we've allocated in it either. This potential
    /// aliasing of exclusive references is one common footgun for unsafe code that we don't need to worry
    /// about here.
    ///
    /// However, there could be regions of uninitialized memory used as padding between allocations,
    /// which is why this iterator has items of type `[MaybeUninit<u8>]`, instead of simply `[u8]`.
    ///
    /// The only way to guarantee that there is no padding between allocations or within allocated objects is
    /// if all of these properties hold:
    ///
    /// 1. Every object allocated in this arena has the same alignment, and that alignment is at most 16.
    /// 2. Every object's size is a multiple of its alignment.
    /// 3. None of the objects allocated in this arena contain any internal padding.
    ///
    /// If you want to use this `iter_allocated_chunks` method, it is *your* responsibility to ensure that
    /// these properties hold before calling `MaybeUninit::assume_init` or otherwise reading the returned values.
    ///
    /// Finally, you must also ensure that any values allocated into the arena have not had their `Drop`
    /// implementations called on them.
    ///
    /// # Example
    ///
    /// ```
    /// # use oxc_allocator::arena::Arena;
    ///
    /// let mut arena = Arena::new();
    ///
    /// // Allocate a bunch of `i32`s in this arena,
    /// // potentially causing additional memory chunks to be reserved
    /// for i in 0..10000 {
    ///     arena.alloc(i);
    /// }
    ///
    /// // Iterate over each chunk we've allocated into. This is safe
    /// // because we have only allocated `i32`s in this arena, which fulfills
    /// // the above requirements.
    /// for ch in arena.iter_allocated_chunks() {
    ///     println!("Used a chunk that is {} bytes long", ch.len());
    ///     println!("The first byte is {:?}", unsafe {
    ///         ch[0].assume_init()
    ///     });
    /// }
    ///
    /// // Within a chunk, allocations are ordered from most recent to least
    /// // recent. If we allocated 'a', then 'b', then 'c', when we iterate
    /// // through the chunk's data, we get them in the order 'c', then 'b',
    /// // then 'a'.
    ///
    /// arena.reset();
    /// arena.alloc(b'a');
    /// arena.alloc(b'b');
    /// arena.alloc(b'c');
    ///
    /// assert_eq!(arena.iter_allocated_chunks().count(), 1);
    /// let chunk = arena.iter_allocated_chunks().nth(0).unwrap();
    /// assert_eq!(chunk.len(), 3);
    ///
    /// // Safe because we've only allocated `u8`s in this arena,
    /// // which fulfills the above requirements
    /// unsafe {
    ///     assert_eq!(chunk[0].assume_init(), b'c');
    ///     assert_eq!(chunk[1].assume_init(), b'b');
    ///     assert_eq!(chunk[2].assume_init(), b'a');
    /// }
    /// ```
    pub fn iter_allocated_chunks(&mut self) -> ChunkIter<'_, MIN_ALIGN> {
        // SAFETY: Ensured by mutable borrow of `self`
        let raw = unsafe { self.iter_allocated_chunks_raw() };
        ChunkIter { raw, arena: PhantomData }
    }

    /// Get an iterator over raw pointers to chunks of allocated memory that this arena has allocated into.
    ///
    /// This is an unsafe version of [`iter_allocated_chunks()`](Arena::iter_allocated_chunks), with the caller
    /// responsible for safe usage of the returned pointers as well as ensuring that the iterator is not
    /// invalidated by new allocations.
    ///
    /// # SAFETY
    ///
    /// Allocations from this arena must not be performed while the returned iterator is alive. If reading the
    /// chunk data (or casting to a reference) the caller must ensure that there exist no mutable references to
    /// previously allocated data.
    ///
    /// In addition, all of the caveats when reading the chunk data from
    /// [`iter_allocated_chunks()`](Arena::iter_allocated_chunks) still apply.
    pub unsafe fn iter_allocated_chunks_raw(&self) -> ChunkRawIter<'_, MIN_ALIGN> {
        ChunkRawIter {
            footer: self.current_chunk_footer.get(),
            // Authoritative cursor for the current chunk lives on `Arena`, not on the chunk's footer.
            // The iterator consumes this value on its first step, then reads cursors from each
            // retired chunk's footer.
            current_chunk_cursor_ptr: Some(self.cursor_ptr.get()),
            arena: PhantomData,
        }
    }

    /// Calculate the number of bytes currently allocated across all chunks in this arena.
    ///
    /// If you allocate types of different alignments or types with larger-than-typical alignment in the same
    /// arena, some padding bytes might get allocated in the arena. Note that those padding bytes will add to
    /// this method's resulting sum, so you cannot rely on it only counting the sum of the sizes of the things
    /// you've allocated in the arena.
    ///
    /// The allocated bytes do not include the size of arena metadata, so the amount of memory requested from
    /// the Rust allocator is higher than the returned value.
    ///
    /// # Example
    ///
    /// ```
    /// # use oxc_allocator::arena::Arena;
    ///
    /// let arena = Arena::new();
    /// let _x = arena.alloc_slice_fill_default::<u32>(5);
    /// let bytes = arena.allocated_bytes();
    /// assert!(bytes >= size_of::<u32>() * 5);
    /// ```
    pub fn allocated_bytes(&self) -> usize {
        let mut total = 0;
        let mut footer_ptr = self.current_chunk_footer.get();
        // SAFETY: Walk the chunk list until the empty sentinel chunk.
        // Every non-empty chunk is a live allocation whose `layout.size()` includes the footer.
        unsafe {
            while !footer_ptr.as_ref().is_empty() {
                total += footer_ptr.as_ref().layout.size() - CHUNK_FOOTER_SIZE;
                footer_ptr = footer_ptr.as_ref().previous_chunk_footer_ptr.get();
            }
        }
        total
    }
}

/// An iterator over each chunk of allocated memory that an arena has allocated into.
///
/// The chunks are returned ordered by allocation time, with the most recently allocated chunk returned first.
///
/// The values inside each chunk are also ordered by allocation time, with the most recent allocation
/// earlier in the slice.
///
/// This struct is created by the [`iter_allocated_chunks`] method on [`Arena`].
/// See that function for a safety description regarding reading from the returned items.
///
/// [`iter_allocated_chunks`]: Arena::iter_allocated_chunks
#[derive(Debug)]
pub struct ChunkIter<'a, const MIN_ALIGN: usize = 1> {
    raw: ChunkRawIter<'a, MIN_ALIGN>,
    arena: PhantomData<&'a mut Arena<MIN_ALIGN>>,
}

impl<'a, const MIN_ALIGN: usize> Iterator for ChunkIter<'a, MIN_ALIGN> {
    type Item = &'a [mem::MaybeUninit<u8>];

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let (ptr, len) = self.raw.next()?;
            let slice = slice::from_raw_parts(ptr as *const mem::MaybeUninit<u8>, len);
            Some(slice)
        }
    }
}

impl<const MIN_ALIGN: usize> FusedIterator for ChunkIter<'_, MIN_ALIGN> {}

/// An iterator over raw pointers to chunks of allocated memory that this arena has allocated into.
///
/// See [`ChunkIter`] for details regarding the returned chunks.
///
/// This struct is created by the [`iter_allocated_chunks_raw`] method on [`Arena`].
/// See that function for a safety description regarding reading from the returned items.
///
/// [`iter_allocated_chunks_raw`]: Arena::iter_allocated_chunks_raw
#[derive(Debug)]
pub struct ChunkRawIter<'a, const MIN_ALIGN: usize = 1> {
    footer: NonNull<ChunkFooter>,
    /// Cursor for the current chunk, taken from `Arena::cursor_ptr` at iterator creation.
    /// Consumed on the first iteration. Subsequent iterations read the cursor from each retired chunk's footer.
    current_chunk_cursor_ptr: Option<NonNull<u8>>,
    arena: PhantomData<&'a Arena<MIN_ALIGN>>,
}

impl<const MIN_ALIGN: usize> Iterator for ChunkRawIter<'_, MIN_ALIGN> {
    type Item = (*mut u8, usize);

    fn next(&mut self) -> Option<(*mut u8, usize)> {
        unsafe {
            let foot = self.footer.as_ref();
            if foot.is_empty() {
                return None;
            }

            let start_ptr = foot.start_ptr.as_ptr();
            let cursor_ptr = self
                .current_chunk_cursor_ptr
                .take()
                .unwrap_or_else(|| foot.cursor_ptr.get())
                .as_ptr();
            let end_ptr = ptr::from_ref(foot).cast::<u8>();

            debug_assert!(start_ptr <= cursor_ptr);
            debug_assert!(cursor_ptr.cast_const() <= end_ptr);

            // SAFETY: `cursor_ptr` is always before or equal to `end_ptr`
            let len = end_ptr.offset_from_unsigned(cursor_ptr.cast_const());
            self.footer = foot.previous_chunk_footer_ptr.get();

            Some((cursor_ptr, len))
        }
    }
}

impl<const MIN_ALIGN: usize> FusedIterator for ChunkRawIter<'_, MIN_ALIGN> {}
