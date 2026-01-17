//! Fast bump allocator optimized for oxc.
//!
//! Key optimizations:
//! - No `Cell` wrapper - direct pointer manipulation
//! - Hardcoded 8-byte alignment - no runtime alignment checks
//! - Backward allocation - optimal for AST traversal cache locality
//! - Minimal hot path
//!
//! # Safety
//!
//! This allocator is NOT thread-safe. The `Bump` type is `!Sync` to enforce this.

// Bump allocators intentionally return `&mut T` from `&self` - this is safe because
// each allocation returns a unique, non-overlapping region of memory.
#![expect(clippy::mut_from_ref)]
// We use `#[inline(always)]` on hot path allocation functions for performance.
#![expect(clippy::inline_always)]
// Pointer casts in this module are intentional and well-understood.
#![expect(clippy::ptr_as_ptr, clippy::cast_ptr_alignment)]

use std::{
    alloc::{Layout, alloc, dealloc, handle_alloc_error},
    cell::UnsafeCell,
    marker::PhantomData,
    mem,
    ptr::{self, NonNull},
    slice,
};

// ============================================================================
// Constants
// ============================================================================

/// Alignment for all allocations. 8 bytes covers most AST nodes.
pub const ALIGN: usize = 8;

/// Size of chunk footer.
pub const FOOTER_SIZE: usize = mem::size_of::<ChunkFooter>();

/// Default first chunk size (512 bytes).
const DEFAULT_CHUNK_SIZE: usize = 512;

/// Minimum chunk size.
const MIN_CHUNK_SIZE: usize = FOOTER_SIZE + ALIGN;

// ============================================================================
// ChunkFooter
// ============================================================================

/// Metadata at the END of each chunk.
///
/// ```text
/// ┌─────────────────────────────────────┬─────────────┐
/// │  ← ← ← allocations grow left        │   Footer    │
/// └─────────────────────────────────────┴─────────────┘
/// ↑                                     ↑             ↑
/// data                                 ptr          footer
/// ```
#[repr(C)]
pub struct ChunkFooter {
    /// Start of chunk data region.
    data: NonNull<u8>,
    /// Layout used for this chunk allocation.
    layout: Layout,
    /// Previous chunk in linked list.
    prev: *mut ChunkFooter,
    /// Current bump pointer (moves toward `data`).
    ptr: *mut u8,
    /// Total allocated bytes in this and all previous chunks.
    allocated_bytes: usize,
}

/// Static empty chunk footer - used when allocator has no chunks.
pub struct EmptyChunk;

impl EmptyChunk {
    /// Get pointer to empty chunk. This is a sentinel, never dereferenced for allocation.
    #[inline]
    fn get() -> *mut ChunkFooter {
        // We use a well-aligned dangling pointer as sentinel.
        // This is never dereferenced for actual allocation.
        ALIGN as *mut ChunkFooter
    }

    #[inline]
    fn is_empty(ptr: *mut ChunkFooter) -> bool {
        ptr as usize == ALIGN
    }
}

// ============================================================================
// Bump Allocator
// ============================================================================

/// Fast bump allocator optimized for AST allocation.
///
/// # Memory Layout
///
/// Uses backward allocation (same as bumpalo):
/// - Allocations grow from high addresses to low
/// - Footer is at the end of each chunk
/// - This gives optimal cache behavior for AST traversal
///
/// # Example
///
/// ```
/// use oxc_allocator::bump::Bump;
///
/// let bump = Bump::new();
/// let x = bump.alloc(42u64);
/// assert_eq!(*x, 42);
/// ```
pub struct Bump {
    /// Current chunk footer pointer.
    chunk: UnsafeCell<*mut ChunkFooter>,
    /// Current bump pointer (cache of chunk.ptr for fast access).
    ptr: UnsafeCell<*mut u8>,
    /// Start of current chunk's data region (cache of chunk.data).
    start: UnsafeCell<*mut u8>,
    /// Marker to prevent Send (Sync is auto-prevented by UnsafeCell).
    _marker: PhantomData<*mut u8>,
}

// SAFETY: Bump is Send because:
// - The raw pointers point to memory owned by this allocator
// - Moving the allocator between threads transfers ownership of that memory
// - Single-threaded use is enforced by !Sync (UnsafeCell is not Sync)
unsafe impl Send for Bump {}

impl Default for Bump {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Bump {
    /// Create a new allocator with no initial capacity.
    ///
    /// First allocation will trigger chunk creation.
    #[inline]
    pub fn new() -> Self {
        Self {
            chunk: UnsafeCell::new(EmptyChunk::get()),
            ptr: UnsafeCell::new(ptr::null_mut()),
            start: UnsafeCell::new(ptr::null_mut()),
            _marker: PhantomData,
        }
    }

    /// Create a new allocator with specified initial capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        let bump = Self::new();
        if capacity > 0 {
            bump.alloc_chunk(capacity.max(MIN_CHUNK_SIZE));
        }
        bump
    }

    // ========================================================================
    // Hot Path - Allocation
    // ========================================================================

    /// Allocate a value in the arena.
    ///
    /// # Panics
    ///
    /// Panics if allocation fails.
    #[inline(always)]
    pub fn alloc<T>(&self, val: T) -> &mut T {
        let ptr = self.alloc_layout(Layout::new::<T>()).as_ptr().cast::<T>();
        // SAFETY: ptr is valid, aligned, and we have exclusive access
        unsafe {
            ptr::write(ptr, val);
            &mut *ptr
        }
    }

    /// Allocate memory with the given layout.
    ///
    /// This is the core allocation method, optimized for minimal instructions.
    #[inline(always)]
    pub fn alloc_layout(&self, layout: Layout) -> NonNull<u8> {
        // SAFETY: Single-threaded access guaranteed by !Sync
        unsafe {
            let ptr = *self.ptr.get();
            let start = *self.start.get();

            // Round size up to alignment (compile-time for known sizes)
            let size = round_up_to(layout.size(), ALIGN);

            // Fast path: have a chunk, fits, and alignment <= 8
            // Note: if ptr is null (no chunk), ptr.sub() would be UB, so check ptr > start
            // which is false when both are null
            if likely(layout.align() <= ALIGN && !ptr.is_null()) {
                let new_ptr = ptr.sub(size);
                if new_ptr >= start {
                    *self.ptr.get() = new_ptr;
                    return NonNull::new_unchecked(new_ptr);
                }
            }

            // Slow path
            self.alloc_slow(layout, size)
        }
    }

    /// Slow path: either need new chunk or special alignment.
    #[cold]
    #[inline(never)]
    fn alloc_slow(&self, layout: Layout, size: usize) -> NonNull<u8> {
        // SAFETY: Single-threaded access guaranteed by !Sync. All pointer arithmetic
        // is within allocated chunk bounds.
        unsafe {
            // Handle over-aligned allocations
            if layout.align() > ALIGN {
                return self.alloc_overaligned(layout);
            }

            // Need new chunk
            let chunk = *self.chunk.get();
            let current_capacity =
                if EmptyChunk::is_empty(chunk) { 0 } else { (*chunk).layout.size() };

            // Double size, minimum DEFAULT_CHUNK_SIZE
            let new_capacity =
                (current_capacity * 2).max(size + FOOTER_SIZE).max(DEFAULT_CHUNK_SIZE);

            self.alloc_chunk(new_capacity);

            // Now allocate from new chunk
            let ptr = *self.ptr.get();
            let new_ptr = ptr.sub(size);
            *self.ptr.get() = new_ptr;
            NonNull::new_unchecked(new_ptr)
        }
    }

    /// Handle allocations requiring alignment > 8.
    #[cold]
    #[inline(never)]
    fn alloc_overaligned(&self, layout: Layout) -> NonNull<u8> {
        // SAFETY: Single-threaded access guaranteed by !Sync. Alignment arithmetic
        // ensures returned pointer meets layout requirements.
        unsafe {
            let ptr = *self.ptr.get();
            let start = *self.start.get();
            let size = round_up_to(layout.size(), ALIGN);

            // Try to fit in current chunk if we have one
            if !ptr.is_null() {
                // Align ptr down to required alignment
                let aligned_ptr = round_down_to_ptr(ptr, layout.align());
                let new_ptr = aligned_ptr.sub(size);

                if new_ptr >= start {
                    *self.ptr.get() = new_ptr;
                    return NonNull::new_unchecked(new_ptr);
                }
            }

            // Need new chunk with enough space + alignment padding
            let padding = layout.align();
            let new_capacity = (size + padding + FOOTER_SIZE).max(DEFAULT_CHUNK_SIZE);

            self.alloc_chunk(new_capacity);

            // Allocate with alignment
            let ptr = *self.ptr.get();
            let aligned_ptr = round_down_to_ptr(ptr, layout.align());
            let new_ptr = aligned_ptr.sub(size);
            *self.ptr.get() = new_ptr;
            NonNull::new_unchecked(new_ptr)
        }
    }

    // ========================================================================
    // Chunk Management
    // ========================================================================

    /// Allocate a new chunk with given capacity.
    #[cold]
    fn alloc_chunk(&self, capacity: usize) {
        // SAFETY: We allocate memory with proper layout, initialize the footer,
        // and update internal state atomically. Single-threaded access guaranteed by !Sync.
        unsafe {
            // Ensure capacity fits footer and has room for allocations
            let capacity = capacity.max(MIN_CHUNK_SIZE);
            let layout = Layout::from_size_align_unchecked(capacity, ALIGN);

            let data = alloc(layout);
            if data.is_null() {
                handle_alloc_error(layout);
            }

            // Footer is at the end of the chunk
            let footer_ptr = data.add(capacity - FOOTER_SIZE) as *mut ChunkFooter;

            // Get previous chunk info
            let prev_chunk = *self.chunk.get();
            let prev_allocated =
                if EmptyChunk::is_empty(prev_chunk) { 0 } else { (*prev_chunk).allocated_bytes };

            // Initialize footer
            // allocated_bytes tracks total capacity across all chunks
            ptr::write(
                footer_ptr,
                ChunkFooter {
                    data: NonNull::new_unchecked(data),
                    layout,
                    prev: prev_chunk,
                    ptr: footer_ptr as *mut u8,
                    allocated_bytes: prev_allocated + capacity,
                },
            );

            // Update allocator state
            let start = data;
            let ptr = footer_ptr as *mut u8;

            *self.chunk.get() = footer_ptr;
            *self.ptr.get() = ptr;
            *self.start.get() = start;
        }
    }

    /// Reset allocator, keeping only the largest chunk.
    pub fn reset(&mut self) {
        // SAFETY: We have exclusive access via `&mut self`. All chunk pointers are valid
        // as they were allocated by this allocator. Deallocation uses correct layouts.
        unsafe {
            let chunk = *self.chunk.get();
            if EmptyChunk::is_empty(chunk) {
                return;
            }

            // If only one chunk, just reset its pointer
            let prev = (*chunk).prev;
            if EmptyChunk::is_empty(prev) {
                // Single chunk - just reset the bump pointer
                (*chunk).ptr = chunk as *mut u8;
                *self.ptr.get() = chunk as *mut u8;
                return;
            }

            // Multiple chunks - find largest and free the rest
            let mut largest = chunk;
            let mut largest_size = (*chunk).layout.size();
            let mut current = prev;

            while !EmptyChunk::is_empty(current) {
                let size = (*current).layout.size();
                let next = (*current).prev;

                if size > largest_size {
                    // Deallocate previous largest
                    dealloc((*largest).data.as_ptr(), (*largest).layout);
                    largest = current;
                    largest_size = size;
                } else {
                    // Deallocate this chunk
                    dealloc((*current).data.as_ptr(), (*current).layout);
                }
                current = next;
            }

            // Keep only largest chunk
            (*largest).prev = EmptyChunk::get();
            (*largest).ptr = largest as *mut u8;
            (*largest).allocated_bytes = largest_size;

            // Reset allocator state
            *self.chunk.get() = largest;
            *self.ptr.get() = largest as *mut u8;
            *self.start.get() = (*largest).data.as_ptr();
        }
    }

    // ========================================================================
    // Convenience Methods
    // ========================================================================

    /// Copy a string into the allocator.
    #[inline]
    pub fn alloc_str(&self, s: &str) -> &mut str {
        let bytes = self.alloc_slice_copy(s.as_bytes());
        // SAFETY: input was valid UTF-8
        unsafe { std::str::from_utf8_unchecked_mut(bytes) }
    }

    /// Copy a slice into the allocator.
    #[inline]
    pub fn alloc_slice_copy<T: Copy>(&self, src: &[T]) -> &mut [T] {
        if src.is_empty() {
            return &mut [];
        }

        let layout = Layout::for_value(src);
        let dst = self.alloc_layout(layout).as_ptr().cast::<T>();

        // SAFETY: `dst` is freshly allocated with correct size and alignment.
        // `src` is a valid slice. No overlap possible with fresh allocation.
        unsafe {
            ptr::copy_nonoverlapping(src.as_ptr(), dst, src.len());
            slice::from_raw_parts_mut(dst, src.len())
        }
    }

    /// Total bytes allocated from system.
    pub fn allocated_bytes(&self) -> usize {
        // SAFETY: Single-threaded access guaranteed by !Sync.
        unsafe {
            let chunk = *self.chunk.get();
            if EmptyChunk::is_empty(chunk) { 0 } else { (*chunk).allocated_bytes }
        }
    }

    // ========================================================================
    // Raw Parts Support (for fixed_size allocator)
    // ========================================================================

    /// Construct a [`Bump`] from an existing memory allocation.
    ///
    /// The allocator takes ownership of the memory and will free it on drop.
    /// The allocator cannot grow beyond this single chunk.
    ///
    /// # SAFETY
    ///
    /// * `ptr` must be aligned on [`ALIGN`].
    /// * `size` must be a multiple of [`ALIGN`].
    /// * `size` must be at least [`FOOTER_SIZE`].
    /// * The memory region must be a valid allocation.
    pub unsafe fn from_raw_parts(ptr: NonNull<u8>, size: usize) -> Self {
        // SAFETY: Caller guarantees memory region is valid and properly aligned.
        // We initialize the footer and set up internal state correctly.
        unsafe {
            debug_assert!((ptr.as_ptr() as usize).is_multiple_of(ALIGN));
            debug_assert!(size.is_multiple_of(ALIGN));
            debug_assert!(size >= FOOTER_SIZE);

            // Footer is at the end of the chunk
            let footer_ptr = ptr.as_ptr().add(size - FOOTER_SIZE) as *mut ChunkFooter;

            // Initialize footer
            let layout = Layout::from_size_align_unchecked(size, ALIGN);
            ptr::write(
                footer_ptr,
                ChunkFooter {
                    data: ptr,
                    layout,
                    prev: EmptyChunk::get(),
                    ptr: footer_ptr as *mut u8,
                    allocated_bytes: size,
                },
            );

            Self {
                chunk: UnsafeCell::new(footer_ptr),
                ptr: UnsafeCell::new(footer_ptr as *mut u8),
                start: UnsafeCell::new(ptr.as_ptr()),
                _marker: PhantomData,
            }
        }
    }

    /// Get data pointer for this allocator's current chunk.
    ///
    /// # SAFETY
    ///
    /// Caller must ensure allocator has at least 1 allocated chunk.
    pub unsafe fn data_ptr(&self) -> NonNull<u8> {
        // SAFETY: Caller guarantees chunk exists.
        unsafe {
            let chunk = *self.chunk.get();
            debug_assert!(!EmptyChunk::is_empty(chunk));
            (*chunk).data
        }
    }

    /// Set data pointer for this allocator's current chunk.
    ///
    /// # SAFETY
    ///
    /// * Allocator must have at least 1 allocated chunk.
    /// * `ptr` must point within the chunk and be properly aligned.
    pub unsafe fn set_data_ptr(&self, ptr: NonNull<u8>) {
        // SAFETY: Caller guarantees chunk exists and ptr is valid.
        unsafe {
            let chunk = *self.chunk.get();
            debug_assert!(!EmptyChunk::is_empty(chunk));
            (*chunk).data = ptr;
            *self.start.get() = ptr.as_ptr();
        }
    }

    /// Get cursor pointer for this allocator's current chunk.
    ///
    /// # SAFETY
    ///
    /// Caller must ensure allocator has at least 1 allocated chunk.
    pub unsafe fn cursor_ptr(&self) -> NonNull<u8> {
        // SAFETY: Caller guarantees chunk exists, so ptr is non-null.
        unsafe { NonNull::new_unchecked(*self.ptr.get()) }
    }

    /// Set cursor pointer for this allocator's current chunk.
    ///
    /// # SAFETY
    ///
    /// * Allocator must have at least 1 allocated chunk.
    /// * `ptr` must point within the chunk.
    pub unsafe fn set_cursor_ptr(&self, ptr: NonNull<u8>) {
        // SAFETY: Caller guarantees chunk exists and ptr is valid.
        unsafe {
            let chunk = *self.chunk.get();
            debug_assert!(!EmptyChunk::is_empty(chunk));
            (*chunk).ptr = ptr.as_ptr();
            *self.ptr.get() = ptr.as_ptr();
        }
    }

    /// Get pointer to current chunk's [`ChunkFooter`].
    ///
    /// # SAFETY
    ///
    /// Caller must ensure allocator has at least 1 allocated chunk for valid dereferencing.
    pub unsafe fn chunk_footer_ptr(&self) -> NonNull<ChunkFooter> {
        // SAFETY: Caller guarantees chunk exists, so chunk pointer is non-null.
        unsafe { NonNull::new_unchecked(*self.chunk.get()) }
    }
}

impl Drop for Bump {
    fn drop(&mut self) {
        // SAFETY: We have exclusive access via `&mut self`. All chunks were allocated
        // by this allocator with the stored layouts.
        unsafe {
            let mut chunk = *self.chunk.get();
            while !EmptyChunk::is_empty(chunk) {
                let prev = (*chunk).prev;
                dealloc((*chunk).data.as_ptr(), (*chunk).layout);
                chunk = prev;
            }
        }
    }
}

// ============================================================================
// allocator_api2 Implementation
// ============================================================================

use allocator_api2::alloc::{AllocError, Allocator as AllocatorTrait};

// SAFETY: Bump allocator returns valid, properly aligned memory for the requested layout.
// Memory remains valid until the allocator is dropped. Deallocate is a no-op (bump semantics).
unsafe impl AllocatorTrait for Bump {
    #[inline(always)]
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        let ptr = self.alloc_layout(layout);
        let slice = NonNull::slice_from_raw_parts(ptr, layout.size());
        Ok(slice)
    }

    #[inline(always)]
    unsafe fn deallocate(&self, _ptr: NonNull<u8>, _layout: Layout) {
        // Bump allocator doesn't deallocate individual allocations
    }

    #[inline(always)]
    unsafe fn grow(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        debug_assert!(new_layout.size() >= old_layout.size());

        // Try to grow in place if this is the last allocation
        let old_size = old_layout.size();
        let new_size = new_layout.size();

        // SAFETY: Caller guarantees ptr was allocated by this allocator with old_layout.
        // Single-threaded access guaranteed by !Sync.
        unsafe {
            let cursor = *self.ptr.get();
            let old_end = ptr.as_ptr().add(old_size);

            // Check if this allocation ends at the current cursor
            // (i.e., it's the most recent allocation)
            if old_end == cursor {
                let additional = new_size - old_size;
                let start = *self.start.get();
                let new_cursor = cursor.sub(additional);

                if new_cursor >= start {
                    // Can grow in place
                    *self.ptr.get() = new_cursor;
                    let slice = NonNull::slice_from_raw_parts(ptr, new_size);
                    return Ok(slice);
                }
            }
        }

        // Can't grow in place - allocate new and copy
        let new_ptr = self.alloc_layout(new_layout);
        // SAFETY: old_ptr is valid for old_size bytes, new_ptr is freshly allocated
        // with new_size >= old_size, no overlap possible with fresh allocation.
        unsafe {
            ptr::copy_nonoverlapping(ptr.as_ptr(), new_ptr.as_ptr(), old_size);
        }
        let slice = NonNull::slice_from_raw_parts(new_ptr, new_size);
        Ok(slice)
    }

    #[inline(always)]
    unsafe fn grow_zeroed(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        // SAFETY: Caller guarantees safety requirements
        unsafe {
            let result = AllocatorTrait::grow(self, ptr, old_layout, new_layout)?;
            // Zero the new bytes
            let old_size = old_layout.size();
            let new_size = new_layout.size();
            if new_size > old_size {
                let new_ptr = result.as_ptr().cast::<u8>();
                ptr::write_bytes(new_ptr.add(old_size), 0, new_size - old_size);
            }
            Ok(result)
        }
    }

    #[inline(always)]
    unsafe fn shrink(
        &self,
        ptr: NonNull<u8>,
        _old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        // Bump allocator can't reclaim space, just return the same pointer
        let slice = NonNull::slice_from_raw_parts(ptr, new_layout.size());
        Ok(slice)
    }
}

// ============================================================================
// Alloc Trait Implementation
// ============================================================================

use crate::alloc::Alloc;

impl Alloc for Bump {
    #[inline(always)]
    fn alloc(&self, layout: Layout) -> NonNull<u8> {
        self.alloc_layout(layout)
    }

    #[inline(always)]
    unsafe fn dealloc(&self, _ptr: NonNull<u8>, _layout: Layout) {
        // Bump allocator doesn't deallocate
    }

    #[inline(always)]
    unsafe fn grow(&self, ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) -> NonNull<u8> {
        // SAFETY: Caller guarantees safety requirements
        unsafe {
            match AllocatorTrait::grow(self, ptr, old_layout, new_layout) {
                Ok(slice) => slice.cast(),
                Err(_) => std::alloc::handle_alloc_error(new_layout),
            }
        }
    }

    #[inline(always)]
    unsafe fn shrink(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> NonNull<u8> {
        // SAFETY: Caller guarantees safety requirements
        unsafe {
            match AllocatorTrait::shrink(self, ptr, old_layout, new_layout) {
                Ok(slice) => slice.cast(),
                Err(_) => std::alloc::handle_alloc_error(new_layout),
            }
        }
    }
}

// ============================================================================
// Chunk Iterator (for used_bytes)
// ============================================================================

impl Bump {
    /// Iterate over allocated chunks.
    ///
    /// # Safety
    ///
    /// Caller must not allocate into this bump while iterating.
    pub unsafe fn iter_allocated_chunks_raw(&self) -> ChunkIter<'_> {
        // SAFETY: Caller guarantees no concurrent allocations
        unsafe {
            ChunkIter { chunk: *self.chunk.get(), cursor: *self.ptr.get(), _marker: PhantomData }
        }
    }
}

/// Iterator over allocated chunks.
pub struct ChunkIter<'a> {
    chunk: *mut ChunkFooter,
    cursor: *mut u8,
    _marker: PhantomData<&'a Bump>,
}

impl Iterator for ChunkIter<'_> {
    type Item = (*mut u8, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if EmptyChunk::is_empty(self.chunk) {
            return None;
        }

        // SAFETY: We checked chunk is not empty, so it's safe to dereference
        unsafe {
            let footer = &*self.chunk;
            let cursor = self.cursor;

            // For backward allocation, used bytes are from cursor to footer
            // ┌─────────────────────────────────────┬─────────────┐
            // │  ← ← ← allocations grow left        │   Footer    │
            // └─────────────────────────────────────┴─────────────┘
            // ↑                                     ↑             ↑
            // start                                cursor       footer
            //                                       │────used────│
            let footer_ptr = self.chunk as *mut u8;
            let used = (footer_ptr as usize).saturating_sub(cursor as usize);

            // Move to previous chunk
            let prev = footer.prev;
            if !EmptyChunk::is_empty(prev) {
                self.cursor = (*prev).ptr;
            }
            self.chunk = prev;

            Some((cursor, used))
        }
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Round up to next multiple of align (align must be power of 2).
#[inline(always)]
const fn round_up_to(n: usize, align: usize) -> usize {
    (n + align - 1) & !(align - 1)
}

/// Round pointer down to alignment.
#[inline(always)]
fn round_down_to_ptr(ptr: *mut u8, align: usize) -> *mut u8 {
    let addr = ptr as usize;
    (addr & !(align - 1)) as *mut u8
}

/// Branch hint - likely to be true.
#[inline(always)]
const fn likely(b: bool) -> bool {
    if !b {
        cold_path();
    }
    b
}

/// Hint that this path is cold.
#[inline(always)]
#[cold]
const fn cold_path() {}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        let bump = Bump::new();
        let x = bump.alloc(42u64);
        assert_eq!(*x, 42);
    }

    #[test]
    fn test_multiple() {
        let bump = Bump::new();
        let a = bump.alloc(1u64);
        let b = bump.alloc(2u64);
        let c = bump.alloc(3u64);
        assert_eq!(*a, 1);
        assert_eq!(*b, 2);
        assert_eq!(*c, 3);

        // Verify backward allocation (addresses decrease)
        assert!(b as *const _ < a as *const _);
        assert!(c as *const _ < b as *const _);
    }

    #[test]
    fn test_alignment() {
        let bump = Bump::new();
        let _byte = bump.alloc(1u8);
        let num = bump.alloc(42u64);
        assert_eq!((num as *const _ as usize) % 8, 0);
    }

    #[test]
    fn test_overaligned() {
        #[repr(align(64))]
        struct Aligned64([u8; 64]);

        let bump = Bump::new();
        let x = bump.alloc(Aligned64([42; 64]));
        assert_eq!((x as *const _ as usize) % 64, 0);
    }

    #[test]
    fn test_large() {
        let bump = Bump::new();
        let arr = bump.alloc([0u8; 10000]);
        assert_eq!(arr.len(), 10000);
    }

    #[test]
    fn test_reset() {
        let mut bump = Bump::new();

        for i in 0..1000 {
            bump.alloc(i as u64);
        }

        let _cap_before = bump.allocated_bytes();
        bump.reset();

        // Capacity preserved
        assert!(bump.allocated_bytes() > 0);

        // Can allocate again
        let x = bump.alloc(999u64);
        assert_eq!(*x, 999);
    }

    #[test]
    fn test_str() {
        let bump = Bump::new();
        let s = bump.alloc_str("hello world");
        assert_eq!(s, "hello world");
    }

    #[test]
    fn test_slice() {
        let bump = Bump::new();
        let s = bump.alloc_slice_copy(&[1, 2, 3, 4, 5]);
        assert_eq!(s, &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_with_capacity() {
        let bump = Bump::with_capacity(10000);
        assert!(bump.allocated_bytes() >= 10000);
    }

    #[test]
    fn test_many_chunks() {
        let bump = Bump::new();

        // Force multiple chunk allocations
        for i in 0..100 {
            bump.alloc([i as u8; 1000]);
        }

        assert!(bump.allocated_bytes() >= 100_000);
    }

    #[test]
    fn test_used_bytes_simple() {
        let bump = Bump::with_capacity(1024);

        // Calculate used bytes before any allocations
        let used_before = unsafe {
            let mut total = 0;
            for (_, size) in bump.iter_allocated_chunks_raw() {
                total += size;
            }
            total
        };
        assert_eq!(used_before, 0, "No allocations yet, used should be 0");

        // Allocate one u64 (8 bytes)
        let _ = bump.alloc(42u64);

        let used_after = unsafe {
            let mut total = 0;
            for (_, size) in bump.iter_allocated_chunks_raw() {
                total += size;
            }
            total
        };
        assert_eq!(used_after, 8, "One u64 allocated, used should be 8");
    }

    #[test]
    fn test_used_bytes_doctest() {
        // Mimics the failing doctest
        let bump = Bump::with_capacity(64 * 1024);

        bump.alloc(1u8); // 1 byte with alignment 1
        bump.alloc(2u8); // 1 byte with alignment 1
        bump.alloc(3u64); // 8 bytes with alignment 8

        let used = unsafe {
            let mut total = 0;
            for (_, size) in bump.iter_allocated_chunks_raw() {
                total += size;
            }
            total
        };

        // With our 8-byte alignment: 8 + 8 + 8 = 24 bytes
        // (each allocation is rounded up to 8)
        assert_eq!(used, 24, "Three allocations should use 24 bytes (8-byte aligned)");
    }
}
