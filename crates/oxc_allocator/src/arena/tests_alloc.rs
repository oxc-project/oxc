//! Unit tests for `try_alloc_layout_fast` bounds check.
//!
//! Uses `TestArena` to construct arenas with controlled `cursor_ptr` and `start_ptr` values,
//! including addresses in the top half of address space (top bit set) which can't be tested
//! via integration tests on any 64-bit platform we run CI on.
//!
//! All addresses are derived from `isize::MAX` so the same tests work on both 32-bit and 64-bit.

use std::{alloc::Layout, cell::Cell, cmp, mem::ManuallyDrop, ptr::NonNull};

use super::{Arena, CHUNK_ALIGN, ChunkFooter};

/// First address in the top half of address space (top bit set).
/// `0x8000_0000` on 32-bit, `0x8000_0000_0000_0000` on 64-bit.
const TOP: usize = (isize::MAX as usize) + 1;

/// Test wrapper that creates an `Arena` with fake `cursor_ptr` and `start_ptr` values.
/// Wrapped in `ManuallyDrop` so the `Arena` is never dropped (which would try to dealloc the fake pointers).
struct TestArena<const MIN_ALIGN: usize = 1> {
    inner: ManuallyDrop<Arena<MIN_ALIGN>>,
}

impl<const MIN_ALIGN: usize> TestArena<MIN_ALIGN> {
    /// Create a `TestArena` with `cursor_ptr` and `start_ptr` set to the given addresses.
    /// `current_chunk_footer_ptr` is pointed at `cursor` (satisfying `cursor <= footer` debug assert).
    /// The footer pointer is never dereferenced in `try_alloc_layout_fast`.
    fn new(start: usize, cursor: usize) -> Self {
        // Check input is valid
        assert!(
            start.is_multiple_of(CHUNK_ALIGN),
            "start {start:#x} must be a multiple of CHUNK_ALIGN ({CHUNK_ALIGN})"
        );
        assert!(cursor >= start, "cursor {cursor:#x} must be >= start {start:#x}");

        let capacity = cursor - start;
        let chunk_align = cmp::max(MIN_ALIGN, CHUNK_ALIGN);
        assert!(
            Layout::from_size_align(capacity, chunk_align).is_ok(),
            "capacity {capacity} (cursor {cursor:#x} - start {start:#x}) with align {chunk_align} must be a valid `Layout`",
        );

        // Construct arena
        let cursor_ptr = NonNull::new(cursor as *mut u8).unwrap();
        let start_ptr = NonNull::new(start as *mut u8).unwrap();
        let arena = Arena {
            cursor_ptr: Cell::new(cursor_ptr),
            current_chunk_footer_ptr: Cell::new(Some(cursor_ptr.cast::<ChunkFooter>())),
            start_ptr: Cell::new(start_ptr),
            can_grow: false,
            #[cfg(all(feature = "track_allocations", not(feature = "disable_track_allocations")))]
            stats: crate::tracking::AllocationStats::default(),
        };
        Self { inner: ManuallyDrop::new(arena) }
    }

    fn try_alloc_layout_fast(&self, layout: Layout) -> Option<NonNull<u8>> {
        self.inner.try_alloc_layout_fast(layout)
    }
}

// --- Bottom half of address space ---

#[test]
fn bottom_half_enough_room() {
    let arena = TestArena::<1>::new(0x1000, 0x2000);
    let layout = Layout::from_size_align(32, 1).unwrap();
    let result = arena.try_alloc_layout_fast(layout);
    assert!(result.is_some());
    assert_eq!(result.unwrap().as_ptr().addr(), 0x2000 - 32);
}

#[test]
fn bottom_half_exact_fit() {
    let arena = TestArena::<1>::new(0x1000, 0x1020); // 32 bytes capacity
    let layout = Layout::from_size_align(32, 1).unwrap();
    let result = arena.try_alloc_layout_fast(layout);
    assert!(result.is_some());
    assert_eq!(result.unwrap().as_ptr().addr(), 0x1000);
}

#[test]
fn bottom_half_not_enough_room() {
    let arena = TestArena::<1>::new(0x1000, 0x1010); // 16 bytes capacity
    let layout = Layout::from_size_align(32, 1).unwrap();
    assert!(arena.try_alloc_layout_fast(layout).is_none());
}

#[test]
fn bottom_half_zero_capacity() {
    let arena = TestArena::<1>::new(0x1000, 0x1000);
    let layout = Layout::from_size_align(1, 1).unwrap();
    assert!(arena.try_alloc_layout_fast(layout).is_none());
}

// --- Top half of address space (top bit set) ---

#[test]
fn top_half_enough_room() {
    let start = TOP + 0x1000;
    let cursor = start + 0x1000;
    let arena = TestArena::<1>::new(start, cursor);
    let layout = Layout::from_size_align(32, 1).unwrap();
    let result = arena.try_alloc_layout_fast(layout);
    assert!(result.is_some());
    assert_eq!(result.unwrap().as_ptr().addr(), cursor - 32);
}

#[test]
fn top_half_exact_fit() {
    let start = TOP + 0x1000;
    let cursor = start + 32;
    let arena = TestArena::<1>::new(start, cursor);
    let layout = Layout::from_size_align(32, 1).unwrap();
    let result = arena.try_alloc_layout_fast(layout);
    assert!(result.is_some());
    assert_eq!(result.unwrap().as_ptr().addr(), start);
}

#[test]
fn top_half_not_enough_room() {
    let start = TOP + 0x1000;
    let cursor = start + 16;
    let arena = TestArena::<1>::new(start, cursor);
    let layout = Layout::from_size_align(32, 1).unwrap();
    assert!(arena.try_alloc_layout_fast(layout).is_none());
}

#[test]
fn top_half_zero_capacity() {
    let start = TOP + 0x1000;
    let arena = TestArena::<1>::new(start, start);
    let layout = Layout::from_size_align(1, 1).unwrap();
    assert!(arena.try_alloc_layout_fast(layout).is_none());
}

// --- Wrapping (allocation size larger than cursor address) ---

#[test]
fn wrapping_sub_wraps_bottom_half() {
    let arena = TestArena::<1>::new(0x100, 0x200);
    let layout = Layout::from_size_align(0x1000, 1).unwrap();
    assert!(arena.try_alloc_layout_fast(layout).is_none());
}

#[test]
fn wrapping_sub_wraps_across_midpoint() {
    // Cursor is just past the top-bit boundary; subtracting pushes into the bottom half
    let start = TOP;
    let cursor = TOP + 0x10;
    let arena = TestArena::<1>::new(start, cursor);
    let layout = Layout::from_size_align(0x20, 1).unwrap();
    assert!(arena.try_alloc_layout_fast(layout).is_none());
}

// --- ZST ---

#[test]
fn zst_zero_capacity() {
    let arena = TestArena::<1>::new(0x1000, 0x1000);
    let layout = Layout::from_size_align(0, 1).unwrap();
    let result = arena.try_alloc_layout_fast(layout);
    assert!(result.is_some());
    assert_eq!(result.unwrap().as_ptr().addr(), 0x1000);
}

#[test]
fn zst_top_half() {
    let addr = TOP + 0x1000;
    let arena = TestArena::<1>::new(addr, addr);
    let layout = Layout::from_size_align(0, 1).unwrap();
    let result = arena.try_alloc_layout_fast(layout);
    assert!(result.is_some());
    assert_eq!(result.unwrap().as_ptr().addr(), addr);
}

// --- Greater path: align > MIN_ALIGN ---

#[test]
fn greater_enough_room() {
    // Cursor at 0x1037, rounds down to 0x1030. 0x30 bytes from start. Fits 16 bytes.
    let arena = TestArena::<1>::new(0x1000, 0x1037);
    let layout = Layout::from_size_align(16, 8).unwrap();
    let result = arena.try_alloc_layout_fast(layout);
    assert!(result.is_some());
    assert_eq!(result.unwrap().as_ptr().addr(), 0x1030 - 16);
    assert!(result.unwrap().as_ptr().addr().is_multiple_of(8));
}

#[test]
fn greater_not_enough_room() {
    // Cursor at 0x1017, rounds down to 0x1010. Only 0x10 bytes from start. 32 doesn't fit.
    let arena = TestArena::<1>::new(0x1000, 0x1017);
    let layout = Layout::from_size_align(32, 8).unwrap();
    assert!(arena.try_alloc_layout_fast(layout).is_none());
}

#[test]
fn greater_rounds_below_start() {
    // Start at 0x1010, cursor at 0x1013. Subtracting size 4 gives 0x100F, then rounding down
    // to align 16 gives 0x1000 < start. Must reject.
    let arena = TestArena::<1>::new(0x1010, 0x1013);
    let layout = Layout::from_size_align(4, 16).unwrap();
    assert!(arena.try_alloc_layout_fast(layout).is_none());
}

#[test]
fn greater_top_half() {
    // Use an address in top half that's aligned to 8
    let start = TOP + 0x1000;
    let cursor = start + 0x100;
    let arena = TestArena::<1>::new(start, cursor);
    let layout = Layout::from_size_align(32, 8).unwrap();
    let result = arena.try_alloc_layout_fast(layout);
    assert!(result.is_some());
    assert_eq!(result.unwrap().as_ptr().addr(), cursor - 32);
    assert!(result.unwrap().as_ptr().addr().is_multiple_of(8));
}

#[test]
fn greater_top_half_rounds_below_start() {
    // Start at TOP+0x10, cursor at TOP+0x13. Subtracting size 4 gives TOP+0xF, then rounding
    // down to align 16 gives TOP+0 < start. Must reject.
    let start = TOP + 0x10;
    let cursor = TOP + 0x13;
    let arena = TestArena::<1>::new(start, cursor);
    let layout = Layout::from_size_align(4, 16).unwrap();
    assert!(arena.try_alloc_layout_fast(layout).is_none());
}

// --- Maximum allocation sizes ---

/// Equal path: aligned_size = isize::MAX (the maximum valid Layout size with align 1).
/// Always rejected — no chunk can hold isize::MAX bytes.
#[test]
fn max_size_equal_path() {
    let layout = Layout::from_size_align(isize::MAX as usize, 1).unwrap();

    // Bottom half
    let arena = TestArena::<1>::new(0x1000, 0x2000);
    assert!(arena.try_alloc_layout_fast(layout).is_none());

    // Top half
    let arena = TestArena::<1>::new(TOP + 0x1000, TOP + 0x2000);
    assert!(arena.try_alloc_layout_fast(layout).is_none());
}

/// Less path: layout.size() = isize::MAX with align 1 in Arena<8>.
/// aligned_size = round_up(isize::MAX, 8) = isize::MAX + 1. This is the maximum possible
/// aligned_size. Must be rejected.
#[test]
fn max_size_less_path_rounds_above_isize_max() {
    let layout = Layout::from_size_align(isize::MAX as usize, 1).unwrap();

    // Bottom half
    let arena = TestArena::<8>::new(0x1000, 0x2000);
    assert!(arena.try_alloc_layout_fast(layout).is_none());

    // Top half
    let arena = TestArena::<8>::new(TOP + 0x1000, TOP + 0x2000);
    assert!(arena.try_alloc_layout_fast(layout).is_none());
}

/// Less path: layout.size() = isize::MAX with align 1 in Arena<16>.
/// aligned_size = round_up(isize::MAX, 16) = isize::MAX + 1. Must be rejected.
#[test]
fn max_size_less_path_min_align_16() {
    let layout = Layout::from_size_align(isize::MAX as usize, 1).unwrap();

    let arena = TestArena::<16>::new(0x1000, 0x2000);
    assert!(arena.try_alloc_layout_fast(layout).is_none());

    let arena = TestArena::<16>::new(TOP + 0x1000, TOP + 0x2000);
    assert!(arena.try_alloc_layout_fast(layout).is_none());
}

/// Greater path: max Layout size for align 8 is isize::MAX - 7 (so rounded size fits in isize::MAX).
/// Must be rejected since no chunk is that big.
#[test]
fn max_size_greater_path() {
    // isize::MAX - 7 is the largest size where Layout::from_size_align(size, 8) succeeds
    let layout = Layout::from_size_align((isize::MAX as usize) - 7, 8).unwrap();

    let arena = TestArena::<1>::new(0x1000, 0x2000);
    assert!(arena.try_alloc_layout_fast(layout).is_none());

    let arena = TestArena::<1>::new(TOP + 0x1000, TOP + 0x2000);
    assert!(arena.try_alloc_layout_fast(layout).is_none());
}

/// Less path with size just below the rounding boundary.
/// size = isize::MAX - 7 with MIN_ALIGN=8 rounds to isize::MAX - 7 (already a multiple of 8).
/// Still too large for any real chunk.
#[test]
fn near_max_size_less_path_already_aligned() {
    let layout = Layout::from_size_align((isize::MAX as usize) - 7, 1).unwrap();

    let arena = TestArena::<8>::new(0x1000, 0x2000);
    assert!(arena.try_alloc_layout_fast(layout).is_none());
}

/// Boundary case: Greater path with the largest valid Layout for align=256.
/// Layout::from_size_align(isize::MAX - 255, 256) gives the maximum size whose round-up to 256
/// fits in isize::MAX. Combined with worst-case padding (cursor positioned at `start mod 256 = 240`,
/// the maximum cursor mod 256 achievable when start is CHUNK_ALIGN(16)-aligned), total subtraction
/// reaches isize::MAX - 15. The bounds check must reject this.
#[test]
fn boundary_greater_path_max_layout_with_max_padding() {
    let layout = Layout::from_size_align((isize::MAX as usize) - 255, 256).unwrap();

    // Empty chunk with start chosen so cursor mod 256 = 240 (max for CHUNK_ALIGN-aligned start)
    let arena = TestArena::<1>::new(0xF0, 0xF0);
    assert!(arena.try_alloc_layout_fast(layout).is_none());

    // Same in top half
    let arena = TestArena::<1>::new(TOP + 0xF0, TOP + 0xF0);
    assert!(arena.try_alloc_layout_fast(layout).is_none());
}

/// Boundary case: total subtraction (size + padding) equals exactly `isize::MAX + 1`.
/// This is the worst case for the bounds check — uses the `>` (not `>=`) discrimination.
/// With MIN_ALIGN=16 and size=isize::MAX-14, padding=15 → total = 2^63 = isize::MAX + 1.
#[test]
fn boundary_total_subtraction_equals_isize_max_plus_one() {
    let layout = Layout::from_size_align((isize::MAX as usize) - 14, 1).unwrap();

    // Empty chunk — cursor == start. Worst case for the bounds check.
    let arena = TestArena::<16>::new(0x1000, 0x1000);
    assert!(arena.try_alloc_layout_fast(layout).is_none());

    // Same in top half.
    let arena = TestArena::<16>::new(TOP + 0x1000, TOP + 0x1000);
    assert!(arena.try_alloc_layout_fast(layout).is_none());
}

/// Less path with size that rounds to exactly isize::MAX.
/// size = isize::MAX - 8 with MIN_ALIGN=8: round_up(isize::MAX - 8, 8) = isize::MAX - 7.
/// (isize::MAX = 0x7FFF...FFFF, isize::MAX - 8 = 0x7FFF...FFF7, rounded = 0x7FFF...FFF8 = isize::MAX - 7)
#[test]
fn near_max_size_less_path_rounds_to_near_max() {
    let layout = Layout::from_size_align((isize::MAX as usize) - 8, 1).unwrap();

    let arena = TestArena::<8>::new(0x1000, 0x2000);
    assert!(arena.try_alloc_layout_fast(layout).is_none());
}

// --- Multiple consecutive allocations ---

/// Verify cursor moves correctly across multiple allocations.
#[test]
fn consecutive_allocations() {
    let arena = TestArena::<1>::new(0x1000, 0x1100); // 256 bytes
    let layout = Layout::from_size_align(32, 1).unwrap();

    let p1 = arena.try_alloc_layout_fast(layout).unwrap();
    assert_eq!(p1.as_ptr().addr(), 0x1100 - 32);

    let p2 = arena.try_alloc_layout_fast(layout).unwrap();
    assert_eq!(p2.as_ptr().addr(), 0x1100 - 64);

    let p3 = arena.try_alloc_layout_fast(layout).unwrap();
    assert_eq!(p3.as_ptr().addr(), 0x1100 - 96);
}

/// Multiple allocations in top half.
#[test]
fn consecutive_allocations_top_half() {
    let start = TOP + 0x1000;
    let cursor = start + 0x100;
    let arena = TestArena::<8>::new(start, cursor);
    let layout = Layout::from_size_align(32, 8).unwrap();

    let p1 = arena.try_alloc_layout_fast(layout).unwrap();
    assert_eq!(p1.as_ptr().addr(), cursor - 32);

    let p2 = arena.try_alloc_layout_fast(layout).unwrap();
    assert_eq!(p2.as_ptr().addr(), cursor - 64);
}

/// Fill chunk to exact capacity, then fail.
#[test]
fn fill_then_fail() {
    let arena = TestArena::<1>::new(0x1000, 0x1040); // 64 bytes
    let layout = Layout::from_size_align(16, 1).unwrap();

    assert!(arena.try_alloc_layout_fast(layout).is_some()); // 48 left
    assert!(arena.try_alloc_layout_fast(layout).is_some()); // 32 left
    assert!(arena.try_alloc_layout_fast(layout).is_some()); // 16 left
    assert!(arena.try_alloc_layout_fast(layout).is_some()); // 0 left
    assert!(arena.try_alloc_layout_fast(layout).is_none()); // fail
}

// --- Off-by-one at capacity boundary ---

/// Capacity = size - 1: one byte short, must fail.
#[test]
fn off_by_one_too_small() {
    let arena = TestArena::<1>::new(0x1000, 0x101F); // 31 bytes
    let layout = Layout::from_size_align(32, 1).unwrap();
    assert!(arena.try_alloc_layout_fast(layout).is_none());
}

/// Capacity = size: exact fit, must succeed.
#[test]
fn off_by_one_exact() {
    let arena = TestArena::<1>::new(0x1000, 0x1020); // 32 bytes
    let layout = Layout::from_size_align(32, 1).unwrap();
    assert!(arena.try_alloc_layout_fast(layout).is_some());
}

/// Capacity = size + 1: one byte spare, must succeed.
#[test]
fn off_by_one_just_enough() {
    let arena = TestArena::<1>::new(0x1000, 0x1021); // 33 bytes
    let layout = Layout::from_size_align(32, 1).unwrap();
    assert!(arena.try_alloc_layout_fast(layout).is_some());
}

/// Off-by-one in top half.
#[test]
fn off_by_one_top_half() {
    let start = TOP + 0x1000;
    let arena_small = TestArena::<1>::new(start, start + 31);
    let arena_exact = TestArena::<1>::new(start, start + 32);
    let layout = Layout::from_size_align(32, 1).unwrap();
    assert!(arena_small.try_alloc_layout_fast(layout).is_none());
    assert!(arena_exact.try_alloc_layout_fast(layout).is_some());
}

// --- Chunk spanning the midpoint (start in bottom half, cursor in top half) ---

/// Start near the bottom of the top-bit boundary, cursor just past it.
/// This is a pathological case that can't happen on real 64-bit platforms
/// (user space never spans the midpoint), but is valid on 32-bit.
#[test]
fn chunk_spans_midpoint() {
    let start = TOP - 0x100;
    let cursor = TOP + 0x100;
    let arena = TestArena::<1>::new(start, cursor);

    // Small allocation should succeed — cursor is in top half, result stays in range
    let layout = Layout::from_size_align(16, 1).unwrap();
    let result = arena.try_alloc_layout_fast(layout);
    assert!(result.is_some());
    assert_eq!(result.unwrap().as_ptr().addr(), cursor - 16);
}

/// Same as above but allocation is too big.
#[test]
fn chunk_spans_midpoint_not_enough_room() {
    let start = TOP - 0x10;
    let cursor = TOP + 0x10; // only 32 bytes capacity
    let arena = TestArena::<1>::new(start, cursor);

    let layout = Layout::from_size_align(64, 1).unwrap();
    assert!(arena.try_alloc_layout_fast(layout).is_none());
}

// --- Addresses near usize::MAX ---

/// Chunk near the very top of address space.
#[test]
fn near_usize_max() {
    // Place chunk near the end of address space.
    // start must be CHUNK_ALIGN-aligned and capacity must be a valid Layout.
    let cursor = usize::MAX - (CHUNK_ALIGN - 1); // align down to CHUNK_ALIGN
    let start = cursor - 0x100;
    let arena = TestArena::<1>::new(start, cursor);

    let layout = Layout::from_size_align(16, 1).unwrap();
    let result = arena.try_alloc_layout_fast(layout);
    assert!(result.is_some());
    assert_eq!(result.unwrap().as_ptr().addr(), cursor - 16);
}

/// Near usize::MAX, allocation too big.
#[test]
fn near_usize_max_not_enough_room() {
    let cursor = usize::MAX - (CHUNK_ALIGN - 1);
    let start = cursor - 0x20; // 32 bytes
    let arena = TestArena::<1>::new(start, cursor);

    let layout = Layout::from_size_align(64, 1).unwrap();
    assert!(arena.try_alloc_layout_fast(layout).is_none());
}

// --- Minimum allocation (size 1) ---

#[test]
fn size_one_bottom_half() {
    let arena = TestArena::<1>::new(0x1000, 0x1001);
    let layout = Layout::from_size_align(1, 1).unwrap();
    let result = arena.try_alloc_layout_fast(layout);
    assert!(result.is_some());
    assert_eq!(result.unwrap().as_ptr().addr(), 0x1000);
}

#[test]
fn size_one_top_half() {
    let start = TOP + 0x1000;
    let arena = TestArena::<1>::new(start, start + 1);
    let layout = Layout::from_size_align(1, 1).unwrap();
    let result = arena.try_alloc_layout_fast(layout);
    assert!(result.is_some());
    assert_eq!(result.unwrap().as_ptr().addr(), start);
}

// --- ZST with large alignment (Greater path) ---

/// ZST with align > MIN_ALIGN. Cursor must round down to alignment but size is 0,
/// so new_ptr == aligned_cursor. Should succeed as long as aligned_cursor >= start.
#[test]
fn zst_greater_path() {
    // cursor at 0x1037, rounds down to 0x1030. Size 0, so new_ptr = 0x1030.
    // 0x1030 >= 0x1000 (start), so succeeds.
    let arena = TestArena::<1>::new(0x1000, 0x1037);
    let layout = Layout::from_size_align(0, 8).unwrap();
    let result = arena.try_alloc_layout_fast(layout);
    assert!(result.is_some());
    assert_eq!(result.unwrap().as_ptr().addr(), 0x1030);
}

/// ZST with align > MIN_ALIGN where rounding cursor down lands exactly on start.
#[test]
fn zst_greater_path_exact_start() {
    // cursor at 0x1003, rounds down to 0x1000 == start. Size 0, new_ptr = start. Succeeds.
    let arena = TestArena::<1>::new(0x1000, 0x1003);
    let layout = Layout::from_size_align(0, 16).unwrap();
    let result = arena.try_alloc_layout_fast(layout);
    assert!(result.is_some());
    assert_eq!(result.unwrap().as_ptr().addr(), 0x1000);
}

// --- Greater path: cursor already aligned ---

/// When cursor is already aligned to layout.align(), no padding is wasted.
#[test]
fn greater_cursor_already_aligned() {
    // cursor at 0x1040 (already aligned to 8). No rounding needed.
    let arena = TestArena::<1>::new(0x1000, 0x1040);
    let layout = Layout::from_size_align(16, 8).unwrap();
    let result = arena.try_alloc_layout_fast(layout);
    assert!(result.is_some());
    assert_eq!(result.unwrap().as_ptr().addr(), 0x1040 - 16);
}

// --- Arena<16> (maximum MIN_ALIGN) ---

#[test]
fn arena16_equal_path() {
    let arena = TestArena::<16>::new(0x1000, 0x1100);
    let layout = Layout::from_size_align(48, 16).unwrap();
    let result = arena.try_alloc_layout_fast(layout);
    assert!(result.is_some());
    assert_eq!(result.unwrap().as_ptr().addr(), 0x1100 - 48);
    assert!(result.unwrap().as_ptr().addr().is_multiple_of(16));
}

#[test]
fn arena16_less_path() {
    let arena = TestArena::<16>::new(0x1000, 0x1100);
    // 5 bytes with align 1, rounds up to 16
    let layout = Layout::from_size_align(5, 1).unwrap();
    let result = arena.try_alloc_layout_fast(layout);
    assert!(result.is_some());
    assert_eq!(result.unwrap().as_ptr().addr(), 0x1100 - 16);
}

#[test]
fn arena16_top_half() {
    let start = TOP + 0x1000;
    let cursor = start + 0x100;
    let arena = TestArena::<16>::new(start, cursor);
    let layout = Layout::from_size_align(32, 16).unwrap();
    let result = arena.try_alloc_layout_fast(layout);
    assert!(result.is_some());
    assert_eq!(result.unwrap().as_ptr().addr(), cursor - 32);
    assert!(result.unwrap().as_ptr().addr().is_multiple_of(16));
}

// --- Lowest valid start address ---

#[test]
fn lowest_valid_start() {
    // Start at CHUNK_ALIGN (the lowest non-zero aligned address)
    let cursor = CHUNK_ALIGN + 0x100;
    let arena = TestArena::<1>::new(CHUNK_ALIGN, cursor);
    let layout = Layout::from_size_align(32, 1).unwrap();
    let result = arena.try_alloc_layout_fast(layout);
    assert!(result.is_some());
    assert_eq!(result.unwrap().as_ptr().addr(), cursor - 32);
}

#[test]
fn lowest_valid_start_not_enough_room() {
    // Start at CHUNK_ALIGN (the lowest non-zero aligned address)
    let arena = TestArena::<1>::new(CHUNK_ALIGN, CHUNK_ALIGN + 16);
    let layout = Layout::from_size_align(32, 1).unwrap();
    let result = arena.try_alloc_layout_fast(layout);
    assert!(result.is_none());
}

// --- Less path: align < MIN_ALIGN ---

#[test]
fn less_rounds_size_up() {
    let arena = TestArena::<8>::new(0x1000, 0x1100);
    // 3 bytes with align 1, but MIN_ALIGN=8 rounds size up to 8
    let layout = Layout::from_size_align(3, 1).unwrap();
    let result = arena.try_alloc_layout_fast(layout);
    assert!(result.is_some());
    assert_eq!(result.unwrap().as_ptr().addr(), 0x1100 - 8);
}

#[test]
fn less_top_half() {
    let start = TOP + 0x1000;
    let cursor = start + 0x100;
    let arena = TestArena::<8>::new(start, cursor);
    let layout = Layout::from_size_align(3, 1).unwrap();
    let result = arena.try_alloc_layout_fast(layout);
    assert!(result.is_some());
    assert_eq!(result.unwrap().as_ptr().addr(), cursor - 8);
}

// --- Equal path: align == MIN_ALIGN ---

#[test]
fn equal_top_half() {
    let start = TOP + 0x1000;
    let cursor = start + 0x100;
    let arena = TestArena::<8>::new(start, cursor);
    let layout = Layout::from_size_align(32, 8).unwrap();
    let result = arena.try_alloc_layout_fast(layout);
    assert!(result.is_some());
    assert_eq!(result.unwrap().as_ptr().addr(), cursor - 32);
}

#[test]
fn equal_not_enough_room_top_half() {
    let start = TOP + 0x1000;
    let cursor = start + 16; // only 16 bytes, need 32
    let arena = TestArena::<8>::new(start, cursor);
    let layout = Layout::from_size_align(32, 8).unwrap();
    assert!(arena.try_alloc_layout_fast(layout).is_none());
}

// --- Over-aligned layout ---

#[test]
fn over_aligned() {
    let start = 0x1000;
    let cursor = start + 0x1008;
    let arena = TestArena::<8>::new(start, cursor);

    let layout = Layout::from_size_align(8, 16).unwrap();

    let result = arena.try_alloc_layout_fast(layout);
    assert!(result.is_some());
    assert_eq!(result.unwrap().as_ptr().addr(), cursor - 8);
    assert!(result.unwrap().as_ptr().addr().is_multiple_of(16));

    let result = arena.try_alloc_layout_fast(layout);
    assert!(result.is_some());
    assert_eq!(result.unwrap().as_ptr().addr(), cursor - 24);
    assert!(result.unwrap().as_ptr().addr().is_multiple_of(16));
}
