use std::{
    alloc::Layout,
    cell::Cell,
    ptr::{self, NonNull},
};

use oxc_data_structures::pointer_ext::PointerExt;

use super::constants::CHUNK_ALIGN;

/// Chunk footer.
///
/// Each chunk contains `ChunkFooter` at the very end of the chunk.
///
/// The arena uses bidirectional allocation:
/// - `cursor` bumps downward from the footer (for AST nodes with 8-byte alignment)
/// - `up_cursor` bumps upward from `start` (for strings with 1-byte alignment)
///
/// The chunk is full when `up_cursor` meets `cursor`.
///
/// `cursor` and `up_cursor` use `Cell` to allow interior mutability. This is necessary so that
/// `EMPTY_CHUNK` static is placed in writable memory, allowing ZST allocations
/// to write to the cursor without causing SIGBUS.
#[repr(C)]
pub struct ChunkFooter {
    /// Pointer to the start of this chunk.
    pub start: NonNull<u8>,
    /// Bump allocation cursor that bumps downward, always in the range `self.up_cursor..=self`.
    /// Uses `Cell` for interior mutability so `EMPTY_CHUNK` can be in writable memory.
    pub cursor: Cell<NonNull<u8>>,
    /// Upward bump allocation cursor for strings, always in the range `self.start..=self.cursor`.
    /// Uses `Cell` for interior mutability so `EMPTY_CHUNK` can be in writable memory.
    pub up_cursor: Cell<NonNull<u8>>,
    /// Link to the previous chunk.
    ///
    /// The last node in the `prev` linked list is the canonical empty chunk [`EMPTY_CHUNK`],
    /// whose `previous_chunk` link points to itself.
    pub previous_chunk: NonNull<ChunkFooter>,
    /// Alignment this chunk was allocated with.
    pub alignment: usize,
}

/// Size of `ChunkFooter`.
pub const FOOTER_SIZE: usize = size_of::<ChunkFooter>();

// Ensure that `ChunkFooter` doesn't require higher alignment than `CHUNK_ALIGN`
pub const _: () = assert!(align_of::<ChunkFooter>() <= CHUNK_ALIGN);

/// A wrapper type for the canonical, statically allocated empty chunk.
///
/// For the canonical empty chunk to be `static`, its type must be `Sync`,
/// which is the purpose of this wrapper type.
/// This is safe because the `Cell` fields use interior mutability which is `Sync`-safe
/// when properly synchronized (we only write the same value back for ZST allocations).
#[repr(transparent)]
struct EmptyChunkFooter(ChunkFooter);

// SAFETY: `EmptyChunkFooter` only contains `Cell` for mutable fields, which is safe
// because writes to EMPTY_CHUNK are idempotent (writing the same value back for ZSTs).
unsafe impl Sync for EmptyChunkFooter {}

// SAFETY: References cannot have null pointers, so `NonNull::new_unchecked` is sound.
const EMPTY_CHUNK_PTR: NonNull<ChunkFooter> =
    unsafe { NonNull::new_unchecked(ptr::from_ref(&EMPTY_CHUNK).cast::<ChunkFooter>().cast_mut()) };

static EMPTY_CHUNK: EmptyChunkFooter = EmptyChunkFooter(ChunkFooter {
    start: EMPTY_CHUNK_PTR.cast::<u8>(),
    cursor: Cell::new(EMPTY_CHUNK_PTR.cast::<u8>()),
    up_cursor: Cell::new(EMPTY_CHUNK_PTR.cast::<u8>()),
    previous_chunk: EMPTY_CHUNK_PTR,
    alignment: CHUNK_ALIGN,
});

impl ChunkFooter {
    /// Pointer to canonical empty chunk.
    pub const EMPTY: NonNull<ChunkFooter> = EMPTY_CHUNK_PTR;

    /// Get data capacity of chunk (including both used and unused regions).
    #[inline]
    pub fn capacity(&self) -> usize {
        // SAFETY: `self.start` is always before `self`, and both are within same allocation
        unsafe { ptr::from_ref(self).cast::<u8>().offset_from_usize(self.start.as_ptr()) }
    }

    /// Get number of bytes used to store data in this chunk.
    /// This includes both downward allocations (from cursor to footer) and upward allocations (from start to up_cursor).
    #[inline]
    pub fn used_bytes(&self) -> usize {
        // Downward allocations: from cursor to footer
        // SAFETY: `self.cursor` is always before `self`, and both are within same allocation
        let downward = unsafe {
            ptr::from_ref(self).cast::<u8>().offset_from_usize(self.cursor.get().as_ptr())
        };
        // Upward allocations: from start to up_cursor
        // SAFETY: `self.start` is always before `self.up_cursor`, and both are within same allocation
        let upward = unsafe { self.up_cursor.get().offset_from_usize(self.start) };
        downward + upward
    }

    /// Get number of bytes remaining which are free to store data in this chunk.
    /// This is the space between `up_cursor` and `cursor`.
    #[inline]
    pub fn free_bytes(&self) -> usize {
        // SAFETY: `self.up_cursor` is always before `self.cursor`, and both are within same allocation
        unsafe { self.cursor.get().offset_from_usize(self.up_cursor.get()) }
    }

    /// Get pointer to start of chunk's allocation, and its [`Layout`].
    #[inline]
    pub fn start_ptr_and_layout(&self) -> (*mut u8, Layout) {
        let start_ptr = self.start.as_ptr();
        // SAFETY: `self` + the size of `self` is clearly within same allocation
        let end_ptr = unsafe { ptr::from_ref(self).cast::<u8>().add(FOOTER_SIZE) };
        // SAFETY: `self.start` is always before `self`, and both are within same allocation
        let size = unsafe { end_ptr.offset_from_usize(start_ptr) };
        // SAFETY: If this chunk was allocated, must be a valid layout.
        // `EMPTY_CHUNK` also produces a valid `Layout`.
        let layout = unsafe { Layout::from_size_align_unchecked(size, self.alignment) };
        (start_ptr, layout)
    }
}
