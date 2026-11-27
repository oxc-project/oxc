//! Tests for `Allocator`s and `Arena`s which allocate their chunks from a custom parent allocator.

use std::{
    alloc::{GlobalAlloc, Layout, System},
    sync::Mutex,
    thread,
};

use oxc_allocator::{Allocator, arena::Arena};

/// Parent allocator which forwards to `System`, and records every allocation and deallocation.
///
/// Each test uses its own `static` instance, because tests run in parallel.
struct CountingAlloc {
    events: Mutex<Events>,
}

#[derive(Default, Clone)]
struct Events {
    /// Each allocation, in order
    allocs: Vec<Event>,
    /// Each deallocation, in order
    deallocs: Vec<Event>,
}

#[derive(Default, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct Event {
    address: usize,
    size: usize,
    align: usize,
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
            self.events.lock().unwrap().allocs.push(Event {
                address: ptr.addr(),
                size: layout.size(),
                align: layout.align(),
            });
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.events.lock().unwrap().deallocs.push(Event {
            address: ptr.addr(),
            size: layout.size(),
            align: layout.align(),
        });
        unsafe { System.dealloc(ptr, layout) };
    }
}

/// Make an allocation which does not fit in the current chunk of `arena`, so `arena` must allocate a new chunk.
fn alloc_in_new_chunk(arena: &Arena) {
    let layout = Layout::array::<u8>(arena.chunk_capacity() + 1).unwrap();
    arena.alloc_layout(layout);
}

/// Make an allocation which does not fit in the current chunk of `allocator`, so `allocator` must allocate
/// a new chunk.
///
/// `capacity` is the total capacity of all chunks, so it is at least the free space in the current chunk.
fn alloc_in_new_chunk_of_allocator(allocator: &Allocator) {
    let layout = Layout::array::<u8>(allocator.capacity() + 1).unwrap();
    allocator.alloc_layout(layout);
}

#[test]
fn new_in_parent_is_lazy() {
    static PARENT: CountingAlloc = CountingAlloc::new();

    let allocator = Allocator::new_in_parent(&PARENT);
    assert_eq!(PARENT.num_allocs(), 0);
    assert_eq!(allocator.capacity(), 0);

    let x = allocator.alloc(123u64);
    assert_eq!(*x, 123);
    assert_eq!(PARENT.num_allocs(), 1);

    drop(allocator);
    assert_eq!(PARENT.num_deallocs(), 1);
    PARENT.assert_all_freed();
}

#[test]
fn with_capacity_in_parent_zero_is_lazy() {
    static PARENT: CountingAlloc = CountingAlloc::new();

    let allocator = Allocator::with_capacity_in_parent(0, &PARENT);
    assert_eq!(PARENT.num_allocs(), 0);
    assert_eq!(allocator.capacity(), 0);

    allocator.alloc(1u8);
    assert_eq!(PARENT.num_allocs(), 1);

    drop(allocator);
    PARENT.assert_all_freed();
}

#[test]
fn with_capacity_in_parent_allocates_first_chunk() {
    static PARENT: CountingAlloc = CountingAlloc::new();

    let allocator = Allocator::with_capacity_in_parent(100_000, &PARENT);
    assert_eq!(PARENT.num_allocs(), 1);
    assert!(allocator.capacity() >= 100_000);

    // Fits in the first chunk, so no new chunk is allocated
    allocator.alloc_slice_copy(&[0u8; 50_000]);
    assert_eq!(PARENT.num_allocs(), 1);

    drop(allocator);
    PARENT.assert_all_freed();
}

#[test]
fn all_chunks_come_from_parent() {
    static PARENT: CountingAlloc = CountingAlloc::new();

    let num_chunks = 3;

    let mut arena = Arena::new_in_parent(&PARENT);
    for _ in 0..num_chunks {
        alloc_in_new_chunk(&arena);
    }

    assert_eq!(arena.iter_allocated_chunks().count(), num_chunks);
    assert_eq!(PARENT.num_allocs(), num_chunks);
    assert_eq!(PARENT.num_deallocs(), 0);

    drop(arena);
    assert_eq!(PARENT.num_deallocs(), num_chunks);
    PARENT.assert_all_freed();
}

#[test]
fn reset_frees_all_chunks_except_current() {
    static PARENT: CountingAlloc = CountingAlloc::new();

    let num_chunks = 3;

    let mut allocator = Allocator::new_in_parent(&PARENT);
    for _ in 0..num_chunks {
        alloc_in_new_chunk_of_allocator(&allocator);
    }
    assert_eq!(PARENT.num_allocs(), num_chunks);

    allocator.reset();

    // All chunks except the current (last) one are freed
    let Events { allocs, deallocs } = PARENT.events();
    assert_eq!(deallocs.len(), num_chunks - 1);
    let current_chunk = allocs.last().unwrap();
    assert!(!deallocs.contains(current_chunk), "current chunk must not be freed by `reset`");
    for chunk in &allocs[..num_chunks - 1] {
        assert!(deallocs.contains(chunk), "retired chunk {chunk:?} must be freed by `reset`");
    }

    // Allocator still works after reset, and uses the chunk it kept
    allocator.alloc(1u64);
    assert_eq!(PARENT.num_allocs(), num_chunks);

    drop(allocator);
    assert_eq!(PARENT.num_deallocs(), num_chunks);
    PARENT.assert_all_freed();
}

#[test]
fn large_alignment() {
    static PARENT: CountingAlloc = CountingAlloc::new();

    const ALIGN: usize = 4096;

    let allocator = Allocator::new_in_parent(&PARENT);
    allocator.alloc(1u8);

    // Too large for the first chunk (16 KiB), so a new chunk is allocated with this alignment
    let layout = Layout::from_size_align(64 * 1024, ALIGN).unwrap();
    let ptr = allocator.alloc_layout(layout);
    assert!(ptr.addr().get().is_multiple_of(ALIGN));

    let Events { allocs, .. } = PARENT.events();
    assert_eq!(allocs.len(), 2);
    let last = &allocs[1];
    assert!(last.align >= ALIGN, "chunk for over-aligned allocation must have align >= {ALIGN}");
    assert!(last.address.is_multiple_of(ALIGN));

    drop(allocator);
    PARENT.assert_all_freed();
}

#[test]
fn large_alignment_in_first_chunk() {
    static PARENT: CountingAlloc = CountingAlloc::new();

    const ALIGN: usize = 64 * 1024;

    let allocator = Allocator::with_capacity_in_parent(0, &PARENT);
    let layout = Layout::from_size_align(ALIGN, ALIGN).unwrap();
    let ptr = allocator.alloc_layout(layout);
    assert!(ptr.addr().get().is_multiple_of(ALIGN));

    let Events { allocs, .. } = PARENT.events();
    assert_eq!(allocs.len(), 1);
    assert!(allocs[0].align >= ALIGN);

    drop(allocator);
    PARENT.assert_all_freed();
}

#[test]
fn drop_on_another_thread() {
    static PARENT: CountingAlloc = CountingAlloc::new();

    let num_chunks = 3;

    let allocator = thread::spawn(move || {
        let allocator = Allocator::new_in_parent(&PARENT);
        for _ in 0..num_chunks {
            alloc_in_new_chunk_of_allocator(&allocator);
        }
        allocator
    })
    .join()
    .unwrap();
    assert_eq!(PARENT.num_allocs(), num_chunks);

    thread::spawn(move || drop(allocator)).join().unwrap();
    PARENT.assert_all_freed();
}

#[test]
fn global_allocator_arena_does_not_use_parent() {
    static PARENT: CountingAlloc = CountingAlloc::new();

    // Make sure `PARENT` is used by one allocator, and not by others
    let allocator_with_parent = Allocator::new_in_parent(&PARENT);
    let allocator = Allocator::with_capacity(1000);
    for _ in 0..2 {
        alloc_in_new_chunk_of_allocator(&allocator);
    }
    assert_eq!(PARENT.num_allocs(), 0);

    allocator_with_parent.alloc(1u8);
    assert_eq!(PARENT.num_allocs(), 1);

    drop(allocator);
    drop(allocator_with_parent);
    PARENT.assert_all_freed();
}
