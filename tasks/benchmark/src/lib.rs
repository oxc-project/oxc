use std::alloc::{GlobalAlloc, Layout, System};

pub use criterion::*;

#[global_allocator]
static GLOBAL: NeverGrowInPlaceAllocator = NeverGrowInPlaceAllocator;

/// Global allocator for use in benchmarks.
///
/// A thin wrapper around Rust's default [`System`] allocator. It passes through `alloc`
/// and `dealloc` methods to [`System`], but does not implement [`GlobalAlloc::realloc`].
///
/// Rationale for this is:
///
/// `realloc` for default [`System`] allocator calls `libc::realloc`, which may either:
/// 1. allow the allocation to grow in place. or
/// 2. create a new allocation, and copy memory from old allocation to the new one.
///
/// Whether allocations can grow in place or not depends on the state of the operating system's
/// memory tables, and so is inherently non-deterministic. Using default `System` allocator
/// therefore produces large and unpredictable variance in benchmarks.
///
/// By not providing a `realloc` method, this custom allocator delegates to the default
/// [`GlobalAlloc::realloc`] implementation which *never* grows in place.
/// It therefore represents the "worse case scenario" for memory allocation performance.
/// This behavior is consistent and predictable, and therefore stabilizes benchmark results.
struct NeverGrowInPlaceAllocator;

// SAFETY: Methods simply delegate to `System` allocator
#[expect(unsafe_code, clippy::undocumented_unsafe_blocks)]
unsafe impl GlobalAlloc for NeverGrowInPlaceAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
    }
}
