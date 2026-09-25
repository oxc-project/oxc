use std::{alloc::Layout, fmt::Debug, mem, ptr};

use oxc_allocator::arena::Arena;
use oxc_data_structures::types::implements;

#[test]
fn can_iterate_over_allocated_things() {
    let mut arena = Arena::new();

    #[cfg(not(miri))]
    const MAX: u64 = 131_072;

    #[cfg(miri)] // Miri is very slow, pick a smaller max that runs in a reasonable amount of time
    const MAX: u64 = 1024;

    let mut chunk_ends = vec![];
    let mut last = None;

    for i in 0..MAX {
        let this = arena.alloc(i);
        assert_eq!(*this, i);
        let this = this as *const _ as usize;

        if match last {
            Some(last) if last - mem::size_of::<u64>() == this => false,
            _ => true,
        } {
            let chunk_end = this + mem::size_of::<u64>();
            println!("new chunk ending @ 0x{:x}", chunk_end);
            assert!(
                !chunk_ends.contains(&chunk_end),
                "should not have already allocated this chunk"
            );
            chunk_ends.push(chunk_end);
        }

        last = Some(this);
    }

    let mut seen = vec![false; MAX as usize];

    // Safe because we always allocated objects of the same type in this arena,
    // and their size >= their align.
    for ch in arena.iter_allocated_chunks() {
        let chunk_end = ch.as_ptr() as usize + ch.len();
        println!("iter chunk ending @ {:#x}", chunk_end);
        assert_eq!(
            chunk_ends.pop().unwrap(),
            chunk_end,
            "should iterate over each chunk once, in order they were allocated in"
        );

        let (before, mid, after) = unsafe { ch.align_to::<u64>() };
        assert!(before.is_empty());
        assert!(after.is_empty());
        for i in mid {
            assert!(*i < MAX, "{} < {} (aka {:x} < {:x})", i, MAX, i, MAX);
            seen[*i as usize] = true;
        }
    }

    assert!(seen.iter().all(|s| *s));
}

// Miri does not panic on OOM, the interpreter halts
#[cfg(not(miri))]
// Cannot run this test on 32-bit targets as we can't guarantee that heap allocations are in top half of address space.
// There is no valid `Layout` which would always underflow the bump pointer.
// There are ample unit tests for `try_alloc_layout_fast` which cover the same thing as this test.
#[cfg(target_pointer_width = "64")]
#[test]
#[should_panic(expected = "out of memory")]
fn oom_instead_of_bump_pointer_underflow() {
    let arena = Arena::new();
    let x = arena.alloc(0_u8);
    let addr = ptr::from_ref(x).addr();

    // If heap allocations are made in top half of address space, then this test isn't testing what it's meant to,
    // because `alloc_layout` won't cause bump pointer to underflow.
    // Make sure `addr` is lower than the size of `LAYOUT` which we allocate below.
    if addr >= isize::MAX as usize {
        // Return on error so that we don't panic and the test fails
        eprintln!("bump pointer in top half of memory: {addr:#x} > `isize::MAX`");
        return;
    }

    // A layout guaranteed to underflow the bump pointer
    const LAYOUT: Layout = match Layout::from_size_align(isize::MAX as usize, 1) {
        Ok(layout) => layout,
        Err(_) => panic!("`Layout::from_size_align` failed"),
    };

    // This should panic.
    // If it doesn't, `alloc_layout` incorrectly returned a pointer which wrapped around address space.
    arena.alloc_layout(LAYOUT);
}

#[test]
fn force_new_chunk_fits_well() {
    let b = Arena::new();

    // Use the first chunk for something
    b.alloc_layout(Layout::from_size_align(1, 1).unwrap());

    // Next force allocation of some new chunks.
    b.alloc_layout(Layout::from_size_align(100_001, 1).unwrap());
    b.alloc_layout(Layout::from_size_align(100_003, 1).unwrap());
}

#[test]
fn alloc_with_strong_alignment() {
    let b = Arena::new();

    // 64 is probably the strongest alignment we'll see in practice
    // e.g. AVX-512 types, or cache line padding optimizations
    b.alloc_layout(Layout::from_size_align(4096, 64).unwrap());
}

// Constants and helper for the `*_updates_cursor` tests.
//
// Each of those tests makes 2 consecutive allocations and checks:
// 1. The 2nd allocation is placed directly below the 1st (the arena bumps downwards).
// 2. The 2 values don't overwrite each other.
//
// Each runs on both the fast path (arena with spare capacity) and the slow path
// (1st allocation in an empty arena has to allocate a chunk).

const A: u64 = 0x1111_1111_1111_1111;
const B: u64 = 0x2222_2222_2222_2222;
const C: u64 = 0x3333_3333_3333_3333;

fn addr<T>(r: &T) -> usize {
    ptr::from_ref(r).addr()
}

#[test]
fn alloc_updates_cursor() {
    fn check(arena: &Arena) {
        let a = arena.alloc(A);
        let b = arena.alloc(B);
        assert_eq!(addr(b), addr(a) - 8);
        assert_eq!((*a, *b), (A, B));
    }

    // Fast path
    check(&Arena::with_capacity(0x1000));
    // Slow path
    check(&Arena::new());
}

#[test]
fn try_alloc_updates_cursor() {
    fn check(arena: &Arena) {
        let a = arena.try_alloc(A).unwrap();
        let b = arena.try_alloc(B).unwrap();
        assert_eq!(addr(b), addr(a) - 8);
        assert_eq!((*a, *b), (A, B));
    }

    // Fast path
    check(&Arena::with_capacity(0x1000));
    // Slow path
    check(&Arena::new());
}

#[test]
fn alloc_layout_updates_cursor() {
    fn check(arena: &Arena) {
        let layout = Layout::new::<u64>();
        let a = arena.alloc_layout(layout).cast::<u64>();
        unsafe { a.write(A) };
        let b = arena.alloc_layout(layout).cast::<u64>();
        unsafe { b.write(B) };
        assert_eq!(b.addr().get(), a.addr().get() - 8);
        assert_eq!(unsafe { (a.read(), b.read()) }, (A, B));
    }

    // Fast path
    check(&Arena::with_capacity(0x1000));
    // Slow path
    check(&Arena::new());
}

#[test]
fn try_alloc_layout_updates_cursor() {
    fn check(arena: &Arena) {
        let layout = Layout::new::<u64>();
        let a = arena.try_alloc_layout(layout).unwrap().cast::<u64>();
        unsafe { a.write(A) };
        let b = arena.try_alloc_layout(layout).unwrap().cast::<u64>();
        unsafe { b.write(B) };
        assert_eq!(b.addr().get(), a.addr().get() - 8);
        assert_eq!(unsafe { (a.read(), b.read()) }, (A, B));
    }

    // Fast path
    check(&Arena::with_capacity(0x1000));
    // Slow path
    check(&Arena::new());
}

#[test]
fn alloc_str_updates_cursor() {
    fn check(arena: &Arena) {
        let a = arena.alloc_str("hello");
        let b = arena.alloc_str("world!");
        assert_eq!(b.as_ptr().addr(), a.as_ptr().addr() - 6);
        assert_eq!((&*a, &*b), ("hello", "world!"));
    }

    // Fast path
    check(&Arena::with_capacity(0x1000));
    // Slow path
    check(&Arena::new());
}

#[test]
fn alloc_slice_copy() {
    let b = Arena::new();

    let src: &[u16] = &[0xFEED, 0xFACE, 0xA7, 0xCAFE];
    let dst = b.alloc_slice_copy(src);

    assert_eq!(src, dst);
}

#[test]
fn alloc_slice_copy_updates_cursor() {
    fn check(arena: &Arena) {
        let a = arena.alloc_slice_copy(&[A, A]);
        let b = arena.alloc_slice_copy(&[B, B]);
        assert_eq!(addr(&b[0]), addr(&a[0]) - 16);
        assert_eq!((a[0], a[1], b[0], b[1]), (A, A, B, B));
    }

    // Fast path
    check(&Arena::with_capacity(0x1000));
    // Slow path
    check(&Arena::new());
}

#[test]
fn alloc_slice_clone() {
    let b = Arena::new();

    // Original bumpalo test uses `Vec<Vec<i32>>`, but bump allocators don't run
    // destructors, so the inner Vecs' heap buffers would leak. Use a non-Copy
    // Clone type that doesn't heap-allocate to avoid Miri leak detection.
    // (bumpalo works around this with `-Zmiri-ignore-leaks`.)
    #[derive(Clone, Debug, PartialEq, Eq)]
    struct Val(i32);

    let src = vec![Val(0), Val(1), Val(2), Val(3)];
    let dst = b.alloc_slice_clone(&src);

    assert_eq!(src, dst);
}

/// `alloc_slice_clone` must commit the bump pointer after each allocation,
/// on both the fast path and the slow path (1st allocation in an empty arena has to allocate a chunk).
///
/// It must commit *before* cloning the elements, so a `Clone` impl which itself allocates from
/// the same arena does not get overlapping memory.
#[test]
fn alloc_slice_clone_updates_cursor() {
    #[derive(Clone)]
    struct Val(u64);

    fn check(arena: &Arena) {
        let a = arena.alloc_slice_clone(&[Val(A), Val(A)]);
        let b = arena.alloc_slice_clone(&[Val(B), Val(B)]);
        assert_eq!(addr(&b[0]), addr(&a[0]) - 16);
        assert_eq!((a[0].0, a[1].0, b[0].0, b[1].0), (A, A, B, B));
    }

    // `Clone` impl which performs another allocation from the same arena.
    // The inner allocations must sit below the outer slice - no overlap.
    // Each element records its inner allocation's address.
    // `Reentrant` is 16 bytes, so the outer slice is 32 bytes. Clones run interleaved with
    // the element writes, so the 1st inner allocation is 8 bytes below the slice, the 2nd 16 below.
    struct Reentrant<'a> {
        arena: &'a Arena,
        inner_addr: usize,
    }

    impl Clone for Reentrant<'_> {
        fn clone(&self) -> Self {
            let inner = self.arena.alloc(C);
            assert_eq!(*inner, C);
            Self { arena: self.arena, inner_addr: addr(inner) }
        }
    }

    fn check_reentrant(arena: &Arena) {
        let src = [Reentrant { arena, inner_addr: 0 }, Reentrant { arena, inner_addr: 0 }];
        let outer = arena.alloc_slice_clone(&src);
        assert_eq!(outer[0].inner_addr, addr(&outer[0]) - 8);
        assert_eq!(outer[1].inner_addr, addr(&outer[0]) - 16);
    }

    // Fast path
    check(&Arena::with_capacity(0x1000));
    check_reentrant(&Arena::with_capacity(0x1000));

    // Slow path
    check(&Arena::new());
    check_reentrant(&Arena::new());
}

#[test]
fn small_size_and_large_align() {
    let b = Arena::new();
    let layout = std::alloc::Layout::from_size_align(1, 0x1000).unwrap();
    b.alloc_layout(layout);
}

fn with_capacity_helper<I, T>(iter: I)
where
    T: Copy + Debug + Eq,
    I: Clone + Iterator<Item = T> + DoubleEndedIterator,
{
    for &initial_size in &[0, 1, 8, 11, 0x1000, 0x12345] {
        let mut b = Arena::<1>::with_min_align_and_capacity(initial_size);

        for v in iter.clone() {
            b.alloc(v);
        }

        let mut pushed_values = b.iter_allocated_chunks().flat_map(|c| {
            let (before, mid, after) = unsafe { c.align_to::<T>() };
            assert!(before.is_empty());
            assert!(after.is_empty());
            mid.iter().copied()
        });

        let mut iter = iter.clone().rev();
        for (expected, actual) in iter.by_ref().zip(pushed_values.by_ref()) {
            assert_eq!(expected, actual);
        }

        assert!(iter.next().is_none());
        assert!(pushed_values.next().is_none());
    }
}

#[test]
fn with_capacity_test() {
    with_capacity_helper(0u8..255);
    #[cfg(not(miri))] // Miri is very slow, disable most of the test cases when using it
    {
        with_capacity_helper(0u16..10000);
        with_capacity_helper(0u32..10000);
        with_capacity_helper(0u64..10000);
        with_capacity_helper(0u128..10000);
    }
}

#[test]
fn test_reset() {
    let mut b = Arena::new();

    for i in 0u64..10_000 {
        b.alloc(i);
    }

    assert!(b.iter_allocated_chunks().count() > 1);

    let last_chunk = b.iter_allocated_chunks().next().unwrap();
    let start = last_chunk.as_ptr() as usize;
    let end = start + last_chunk.len();
    b.reset();
    assert_eq!(end - mem::size_of::<u64>(), b.alloc(0u64) as *const u64 as usize);
    assert_eq!(b.iter_allocated_chunks().count(), 1);
}

#[test]
fn test_alignment() {
    for &alignment in &[2, 4, 8, 16, 32, 64] {
        let b = Arena::with_capacity(513);
        let layout = std::alloc::Layout::from_size_align(alignment, alignment).unwrap();

        for _ in 0..1024 {
            let ptr = b.alloc_layout(layout).as_ptr();
            assert_eq!(ptr as *const u8 as usize % alignment, 0);
        }
    }
}

#[test]
fn test_chunk_capacity() {
    let b = Arena::with_capacity(512);
    let orig_capacity = b.chunk_capacity();
    b.alloc(true);
    assert!(b.chunk_capacity() < orig_capacity);
}

// `Arena` uses `Cell`s internally, so sharing one between threads would be undefined behavior
#[test]
fn arena_is_send_but_not_sync() {
    assert!(implements!(Arena: Send));
    assert!(implements!(Arena: !Sync));
}

#[test]
fn test_debug_assert_ptr_align_pr_313() {
    let arena = Arena::<16>::with_min_align();
    arena.alloc(0u8);
}
