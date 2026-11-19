//! Tests for `Allocator`s and `Arena`s which allocate their chunks from a custom chunk allocator.

use std::{
    alloc::{GlobalAlloc, Layout, System},
    sync::Mutex,
    thread,
};

use oxc_allocator::{Allocator, arena::Arena};

/// Chunk allocator which forwards to `System`, and records every allocation and deallocation.
///
/// Each test uses its own `static` instance, because tests run in parallel.
struct CountingAlloc {
    events: Mutex<Events>,
}

#[derive(Default, Clone)]
struct Events {
    /// `(address, size, align)` of each allocation, in order
    allocs: Vec<(usize, usize, usize)>,
    /// `(address, size, align)` of each deallocation, in order
    deallocs: Vec<(usize, usize, usize)>,
}

impl CountingAlloc {
    const fn new() -> Self {
        Self { events: Mutex::new(Events { allocs: Vec::new(), deallocs: Vec::new() }) }
    }

    fn events(&self) -> Events {
        self.events.lock().unwrap().clone()
    }

    fn num_allocs(&self) -> usize {
        self.events.lock().unwrap().allocs.len()
    }

    fn num_deallocs(&self) -> usize {
        self.events.lock().unwrap().deallocs.len()
    }

    /// Assert that every allocation was deallocated exactly once, with the same layout.
    fn assert_all_freed(&self) {
        let Events { mut allocs, mut deallocs } = self.events();
        allocs.sort_unstable();
        deallocs.sort_unstable();
        assert_eq!(allocs, deallocs, "every chunk must be deallocated once, with the same layout");
    }
}

unsafe impl GlobalAlloc for CountingAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() {
            self.events.lock().unwrap().allocs.push((ptr.addr(), layout.size(), layout.align()));
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.events.lock().unwrap().deallocs.push((ptr.addr(), layout.size(), layout.align()));
        unsafe { System.dealloc(ptr, layout) };
    }
}

#[test]
fn new_in_is_lazy() {
    static CHUNK_ALLOC: CountingAlloc = CountingAlloc::new();

    let allocator = Allocator::new_in(&CHUNK_ALLOC);
    assert_eq!(CHUNK_ALLOC.num_allocs(), 0);
    assert_eq!(allocator.capacity(), 0);

    let x = allocator.alloc(123u64);
    assert_eq!(*x, 123);
    assert_eq!(CHUNK_ALLOC.num_allocs(), 1);

    drop(allocator);
    assert_eq!(CHUNK_ALLOC.num_deallocs(), 1);
    CHUNK_ALLOC.assert_all_freed();
}

#[test]
fn with_capacity_in_zero_is_lazy() {
    static CHUNK_ALLOC: CountingAlloc = CountingAlloc::new();

    let allocator = Allocator::with_capacity_in(0, &CHUNK_ALLOC);
    assert_eq!(CHUNK_ALLOC.num_allocs(), 0);
    assert_eq!(allocator.capacity(), 0);

    allocator.alloc(1u8);
    assert_eq!(CHUNK_ALLOC.num_allocs(), 1);

    drop(allocator);
    CHUNK_ALLOC.assert_all_freed();
}

#[test]
fn with_capacity_in_allocates_first_chunk() {
    static CHUNK_ALLOC: CountingAlloc = CountingAlloc::new();

    let allocator = Allocator::with_capacity_in(100_000, &CHUNK_ALLOC);
    assert_eq!(CHUNK_ALLOC.num_allocs(), 1);
    assert!(allocator.capacity() >= 100_000);

    // Fits in the first chunk, so no new chunk is allocated
    allocator.alloc_slice_copy(&[0u8; 50_000]);
    assert_eq!(CHUNK_ALLOC.num_allocs(), 1);

    drop(allocator);
    CHUNK_ALLOC.assert_all_freed();
}

#[test]
fn all_chunks_come_from_chunk_alloc() {
    static CHUNK_ALLOC: CountingAlloc = CountingAlloc::new();

    let mut arena = Arena::new_in(&CHUNK_ALLOC);
    for i in 0..100_000u64 {
        arena.alloc(i);
    }

    let num_chunks = arena.iter_allocated_chunks().count();
    assert!(num_chunks >= 3, "test needs at least 3 chunks, got {num_chunks}");
    assert_eq!(CHUNK_ALLOC.num_allocs(), num_chunks);
    assert_eq!(CHUNK_ALLOC.num_deallocs(), 0);

    drop(arena);
    assert_eq!(CHUNK_ALLOC.num_deallocs(), num_chunks);
    CHUNK_ALLOC.assert_all_freed();
}

#[test]
fn reset_frees_all_chunks_except_current() {
    static CHUNK_ALLOC: CountingAlloc = CountingAlloc::new();

    let mut allocator = Allocator::new_in(&CHUNK_ALLOC);
    for i in 0..100_000u64 {
        allocator.alloc(i);
    }
    let num_chunks = CHUNK_ALLOC.num_allocs();
    assert!(num_chunks >= 3, "test needs at least 3 chunks, got {num_chunks}");

    allocator.reset();

    // All chunks except the current (last) one are freed
    let Events { allocs, deallocs } = CHUNK_ALLOC.events();
    assert_eq!(deallocs.len(), num_chunks - 1);
    let current_chunk = *allocs.last().unwrap();
    assert!(!deallocs.contains(&current_chunk), "current chunk must not be freed by `reset`");
    for chunk in &allocs[..num_chunks - 1] {
        assert!(deallocs.contains(chunk), "retired chunk {chunk:?} must be freed by `reset`");
    }

    // Allocator still works after reset, and uses the chunk it kept
    allocator.alloc(1u64);
    assert_eq!(CHUNK_ALLOC.num_allocs(), num_chunks);

    drop(allocator);
    assert_eq!(CHUNK_ALLOC.num_deallocs(), num_chunks);
    CHUNK_ALLOC.assert_all_freed();
}

#[test]
fn large_alignment() {
    static CHUNK_ALLOC: CountingAlloc = CountingAlloc::new();

    const ALIGN: usize = 4096;

    let allocator = Allocator::new_in(&CHUNK_ALLOC);
    allocator.alloc(1u8);

    // Too large for the first chunk (16 KiB), so a new chunk is allocated with this alignment
    let layout = Layout::from_size_align(64 * 1024, ALIGN).unwrap();
    let ptr = allocator.alloc_layout(layout);
    assert!(ptr.addr().get().is_multiple_of(ALIGN));

    let Events { allocs, .. } = CHUNK_ALLOC.events();
    assert_eq!(allocs.len(), 2);
    let (addr, _size, align) = allocs[1];
    assert!(align >= ALIGN, "chunk for over-aligned allocation must have align >= {ALIGN}");
    assert!(addr.is_multiple_of(ALIGN));

    drop(allocator);
    CHUNK_ALLOC.assert_all_freed();
}

#[test]
fn large_alignment_in_first_chunk() {
    static CHUNK_ALLOC: CountingAlloc = CountingAlloc::new();

    const ALIGN: usize = 64 * 1024;

    let allocator = Allocator::with_capacity_in(0, &CHUNK_ALLOC);
    let layout = Layout::from_size_align(ALIGN, ALIGN).unwrap();
    let ptr = allocator.alloc_layout(layout);
    assert!(ptr.addr().get().is_multiple_of(ALIGN));

    let Events { allocs, .. } = CHUNK_ALLOC.events();
    assert_eq!(allocs.len(), 1);
    assert!(allocs[0].2 >= ALIGN);

    drop(allocator);
    CHUNK_ALLOC.assert_all_freed();
}

#[test]
fn drop_on_another_thread() {
    static CHUNK_ALLOC: CountingAlloc = CountingAlloc::new();

    let allocator = thread::spawn(|| {
        let allocator = Allocator::new_in(&CHUNK_ALLOC);
        for i in 0..10_000u64 {
            allocator.alloc(i);
        }
        allocator
    })
    .join()
    .unwrap();
    assert!(CHUNK_ALLOC.num_allocs() >= 2);

    thread::spawn(move || drop(allocator)).join().unwrap();
    CHUNK_ALLOC.assert_all_freed();
}

#[test]
fn global_allocator_arena_does_not_use_chunk_alloc() {
    static CHUNK_ALLOC: CountingAlloc = CountingAlloc::new();

    // Make sure `CHUNK_ALLOC` is used by one allocator, and not by others
    let allocator_in = Allocator::new_in(&CHUNK_ALLOC);
    let allocator = Allocator::with_capacity(1000);
    for i in 0..10_000u64 {
        allocator.alloc(i);
    }
    assert_eq!(CHUNK_ALLOC.num_allocs(), 0);

    allocator_in.alloc(1u8);
    assert_eq!(CHUNK_ALLOC.num_allocs(), 1);

    drop(allocator);
    drop(allocator_in);
    CHUNK_ALLOC.assert_all_freed();
}
