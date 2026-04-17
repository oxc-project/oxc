//! Unit tests for `Arena`.

// Note: Only tests which require private types, fields, or methods should be in here.
// Anything that can just be tested via public API surface should be in `tests/arena/*`.

use std::alloc::Layout;

use super::{
    Arena, ChunkFooter, DEFAULT_CHUNK_SIZE_WITHOUT_FOOTER, bumpalo_alloc::Alloc as BumpaloAlloc,
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

// Uses private type `ChunkFooter`
#[cfg(target_pointer_width = "64")]
#[test]
fn chunk_footer_is_six_words_on_64_bit() {
    assert_eq!(size_of::<ChunkFooter>(), size_of::<usize>() * 6);
}

// Uses private type `ChunkFooter`
#[cfg(target_pointer_width = "32")]
#[test]
fn chunk_footer_is_eight_words_on_32_bit() {
    assert_eq!(size_of::<ChunkFooter>(), size_of::<usize>() * 8);
}

// Uses private `DEFAULT_CHUNK_SIZE_WITHOUT_FOOTER`
#[test]
fn allocated_bytes() {
    let mut b = Arena::new();

    assert_eq!(b.allocated_bytes(), 0);

    b.alloc(0u8);

    assert_eq!(b.allocated_bytes(), DEFAULT_CHUNK_SIZE_WITHOUT_FOOTER);

    b.reset();

    assert_eq!(b.allocated_bytes(), DEFAULT_CHUNK_SIZE_WITHOUT_FOOTER);
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
        assert_eq!(q.as_ptr() as usize, p.as_ptr() as usize - 1);
        b.reset();

        // `realloc` will allocate a new chunk when growing the last allocation, if need be
        let layout = Layout::from_size_align(1, 1).unwrap();
        let p = b.alloc_layout(layout);
        let q = (&b).realloc(p, layout, CAPACITY + 1).unwrap();
        assert_ne!(q.as_ptr() as usize, p.as_ptr() as usize - CAPACITY);
        b.reset();

        // `realloc` will allocate and copy when reallocating anything that wasn't the last allocation
        let layout = Layout::from_size_align(1, 1).unwrap();
        let p = b.alloc_layout(layout);
        let _ = b.alloc_layout(layout);
        let q = (&b).realloc(p, layout, 2).unwrap();
        assert!(q.as_ptr() as usize != p.as_ptr() as usize - 1);
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
