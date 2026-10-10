//! Thread-local heap allocation counters for opt-in native rule profiling.
//!
//! The executable must install [`TrackingAllocator`] as its global allocator.
//! Counters measure successful allocation requests, not live memory or RSS.

use std::{
    alloc::{GlobalAlloc, Layout},
    cell::Cell,
};

/// Cumulative heap allocation requests made on one thread.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct AllocationStats {
    pub allocations: u64,
    pub reallocations: u64,
    /// Allocated bytes plus positive growth from reallocations.
    pub allocated_bytes: u64,
}

thread_local! {
    // Const initialization and no destructor keep allocator hooks allocation-free,
    // including when allocations occur during thread-local initialization/teardown.
    static COUNTERS: Cell<AllocationStats> = const { Cell::new(AllocationStats {
        allocations: 0,
        reallocations: 0,
        allocated_bytes: 0,
    }) };
}

impl AllocationStats {
    pub(crate) fn current() -> Self {
        COUNTERS.get()
    }

    pub(crate) fn since(self, before: Self) -> Self {
        Self {
            allocations: self.allocations - before.allocations,
            reallocations: self.reallocations - before.reallocations,
            allocated_bytes: self.allocated_bytes - before.allocated_bytes,
        }
    }

    pub(crate) fn add(&mut self, other: Self) {
        self.allocations = self.allocations.saturating_add(other.allocations);
        self.reallocations = self.reallocations.saturating_add(other.reallocations);
        self.allocated_bytes = self.allocated_bytes.saturating_add(other.allocated_bytes);
    }
}

/// Wraps an allocator with allocation-free, thread-local counters.
///
/// Only profiling builds should install this wrapper: even when no rule is being
/// measured, every successful allocation updates the calling thread's counters.
pub struct TrackingAllocator<A>(pub A);

// SAFETY: All operations forward the original arguments to the wrapped allocator.
// Counter updates neither allocate nor unwind.
unsafe impl<A: GlobalAlloc> GlobalAlloc for TrackingAllocator<A> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: The caller satisfies the wrapped allocator's contract.
        let ptr = unsafe { self.0.alloc(layout) };
        if !ptr.is_null() {
            record(AllocationStats {
                allocations: 1,
                allocated_bytes: layout.size() as u64,
                ..AllocationStats::default()
            });
        }
        ptr
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        // SAFETY: The caller satisfies the wrapped allocator's contract.
        let ptr = unsafe { self.0.alloc_zeroed(layout) };
        if !ptr.is_null() {
            record(AllocationStats {
                allocations: 1,
                allocated_bytes: layout.size() as u64,
                ..AllocationStats::default()
            });
        }
        ptr
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // SAFETY: The caller satisfies the wrapped allocator's contract.
        let ptr = unsafe { self.0.realloc(ptr, layout, new_size) };
        if !ptr.is_null() {
            record(AllocationStats {
                reallocations: 1,
                allocated_bytes: new_size.saturating_sub(layout.size()) as u64,
                ..AllocationStats::default()
            });
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        // SAFETY: The caller satisfies the wrapped allocator's contract.
        unsafe { self.0.dealloc(ptr, layout) };
    }
}

fn record(stat: AllocationStats) {
    let mut counters = COUNTERS.get();
    counters.add(stat);
    COUNTERS.set(counters);
}
