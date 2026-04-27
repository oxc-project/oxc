//! Unit tests for `Arena`.

// Note: Only tests which require private types, fields, or methods should be in here.
// Anything that can just be tested via public API surface should be in `tests/arena/*`.

use std::alloc::Layout;

use super::{
    Arena, DEFAULT_CHUNK_SIZE_WITHOUT_FOOTER,
    bumpalo_alloc::Alloc as BumpaloAlloc,
    create::{OVERHEAD, TYPICAL_PAGE_SIZE},
};

/// This function tests that `Arena` isn't `Sync`.
/// ```compile_fail
/// use oxc_allocator::arena::Arena;
/// fn _requires_sync<T: Sync>(_value: T) {}
/// fn _arena_not_sync(b: Arena) {
///    _requires_sync(b);
/// }
/// ```
#[cfg(doctest)]
fn arena_not_sync() {}

// Uses private `DEFAULT_CHUNK_SIZE_WITHOUT_FOOTER`, `OVERHEAD`, and `TYPICAL_PAGE_SIZE`
#[test]
fn allocated_and_used_bytes() {
    fn chunk_count<const MIN_ALIGN: usize>(b: &mut Arena<MIN_ALIGN>) -> usize {
        b.iter_allocated_chunks().count()
    }

    // Each new chunk is sized to twice the previous chunk's allocatable space, rounded up to a page
    // boundary (after accounting for `OVERHEAD` = 16-byte malloc overhead + footer).
    // Mirrors the formula in `Arena::new_chunk`.
    const fn next_chunk_size(prev: usize) -> usize {
        let raw = 2 * prev + OVERHEAD;
        let aligned = (raw + TYPICAL_PAGE_SIZE - 1) & !(TYPICAL_PAGE_SIZE - 1);
        aligned - OVERHEAD
    }
    const SECOND_CHUNK: usize = next_chunk_size(DEFAULT_CHUNK_SIZE_WITHOUT_FOOTER);
    const THIRD_CHUNK: usize = next_chunk_size(SECOND_CHUNK);

    let mut b = Arena::new();

    // Empty arena owns no chunks
    assert_eq!(b.allocated_bytes(), 0);
    assert_eq!(b.used_bytes(), 0);
    assert_eq!(chunk_count(&mut b), 0);

    // First allocation creates the first chunk
    b.alloc(0u8);
    assert_eq!(b.allocated_bytes(), DEFAULT_CHUNK_SIZE_WITHOUT_FOOTER);
    assert_eq!(b.used_bytes(), 1);
    assert_eq!(chunk_count(&mut b), 1);

    // Reset with a single chunk: The chunk is retained (capacity unchanged), but the cursor is reset
    b.reset();
    assert_eq!(b.allocated_bytes(), DEFAULT_CHUNK_SIZE_WITHOUT_FOOTER);
    assert_eq!(b.used_bytes(), 0);
    assert_eq!(chunk_count(&mut b), 1);

    // Allocations that fit in the current chunk don't grow the allocated total, but do grow `used_bytes`
    let small = Layout::from_size_align(64, 1).unwrap();
    b.alloc_layout(small);
    b.alloc_layout(small);
    assert_eq!(b.allocated_bytes(), DEFAULT_CHUNK_SIZE_WITHOUT_FOOTER);
    assert_eq!(b.used_bytes(), small.size() * 2);
    assert_eq!(chunk_count(&mut b), 1);

    // A request which cannot be served in first chunk forces creation of a second chunk.
    // Chunk 1 is retired with its current `used_bytes` (`2 * 64`); chunk 2 holds the `big` allocation.
    let big = Layout::from_size_align(DEFAULT_CHUNK_SIZE_WITHOUT_FOOTER, 1).unwrap();
    b.alloc_layout(big);
    let two_chunks = b.allocated_bytes();
    assert_eq!(two_chunks, DEFAULT_CHUNK_SIZE_WITHOUT_FOOTER + SECOND_CHUNK);
    assert_eq!(b.used_bytes(), small.size() * 2 + big.size());
    assert_eq!(chunk_count(&mut b), 2);

    // A request larger than every chunk seen so far forces a third chunk.
    // (The previous `big` allocation might fit alongside another `big` in chunk 2,
    // so use something definitely too big to fit anywhere existing.)
    let huge = Layout::from_size_align(DEFAULT_CHUNK_SIZE_WITHOUT_FOOTER * 2, 1).unwrap();
    b.alloc_layout(huge);
    let three_chunks = b.allocated_bytes();
    assert_eq!(three_chunks, DEFAULT_CHUNK_SIZE_WITHOUT_FOOTER + SECOND_CHUNK + THIRD_CHUNK);
    assert_eq!(b.used_bytes(), small.size() * 2 + big.size() + huge.size());
    assert_eq!(chunk_count(&mut b), 3);

    // `reset` with multiple chunks deallocates every chunk except the most recent (which is also the largest),
    // so `allocated_bytes` after reset equals just the size of that surviving chunk
    b.reset();
    let after_reset = b.allocated_bytes();
    assert_eq!(after_reset, THIRD_CHUNK);
    assert_eq!(b.used_bytes(), 0);
    assert_eq!(chunk_count(&mut b), 1);
}

// Uses private `bumpalo_alloc` module
#[test]
fn test_realloc() {
    unsafe {
        const CAPACITY: usize = DEFAULT_CHUNK_SIZE_WITHOUT_FOOTER;
        let mut b = Arena::<1>::with_min_align_and_capacity(CAPACITY);

        // `realloc` doesn't shrink allocations that aren't "worth it"
        let layout = Layout::from_size_align(100, 1).unwrap();
        let p = b.alloc_layout(layout);
        let q = (&b).realloc(p, layout, 51).unwrap();
        assert_eq!(p, q);
        b.reset();

        // `realloc` will shrink allocations that are "worth it"
        let layout = Layout::from_size_align(100, 1).unwrap();
        let p = b.alloc_layout(layout);
        let q = (&b).realloc(p, layout, 50).unwrap();
        assert!(p != q);
        b.reset();

        // `realloc` will reuse the last allocation when growing
        let layout = Layout::from_size_align(10, 1).unwrap();
        let p = b.alloc_layout(layout);
        let q = (&b).realloc(p, layout, 11).unwrap();
        assert_eq!(q.as_ptr().addr(), p.as_ptr().addr() - 1);
        b.reset();

        // `realloc` will allocate a new chunk when growing the last allocation, if need be
        let layout = Layout::from_size_align(1, 1).unwrap();
        let p = b.alloc_layout(layout);
        let q = (&b).realloc(p, layout, CAPACITY + 1).unwrap();
        assert_ne!(q.as_ptr().addr(), p.as_ptr().addr() - CAPACITY);
        b.reset();

        // `realloc` will allocate and copy when reallocating anything that wasn't the last allocation
        let layout = Layout::from_size_align(1, 1).unwrap();
        let p = b.alloc_layout(layout);
        let _ = b.alloc_layout(layout);
        let q = (&b).realloc(p, layout, 2).unwrap();
        assert!(q.as_ptr().addr() != p.as_ptr().addr() - 1);
        b.reset();
    }
}

// Uses our private `bumpalo_alloc` module
#[test]
fn invalid_read() {
    let mut b = &Arena::new();

    unsafe {
        let l1 = Layout::from_size_align(12000, 4).unwrap();
        let p1 = BumpaloAlloc::alloc(&mut b, l1).unwrap();

        let l2 = Layout::from_size_align(1000, 4).unwrap();
        BumpaloAlloc::alloc(&mut b, l2).unwrap();

        let p1 = b.realloc(p1, l1, 24000).unwrap();
        let l3 = Layout::from_size_align(24000, 4).unwrap();
        b.realloc(p1, l3, 48000).unwrap();
    }
}
