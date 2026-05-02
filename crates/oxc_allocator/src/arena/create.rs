//! Methods to create an `Arena`.

use std::{
    alloc::{self, Layout},
    cell::Cell,
    cmp::max,
    ptr::NonNull,
};

use super::bumpalo_alloc::AllocErr;

#[cfg(all(feature = "track_allocations", not(feature = "disable_track_allocations")))]
use crate::tracking::AllocationStats;

use super::{
    Arena, CHUNK_ALIGN, CHUNK_FOOTER_SIZE, ChunkFooter, EMPTY_ARENA_DATA_PTR,
    utils::{
        is_pointer_aligned_to, layout_from_size_align, oom, round_up_to, round_up_to_unchecked,
    },
};

// IMPORTANT:
// The const assertions here are relied on by `Arena::new_chunk`.
// If we change any constants, and these assertions fail, we'd need to alter the logic
// in `Arena::new_chunk` as well as the assertions, to account for the change.

/// The typical page size these days.
///
/// Note that we don't need to exactly match page size for correctness, and it is okay if this is smaller than
/// the real page size in practice. It isn't worth the portability concerns and lack of const propagation that
/// dynamically looking up the actual page size implies.
pub const TYPICAL_PAGE_SIZE: usize = 0x1000;

const _: () = {
    assert!(TYPICAL_PAGE_SIZE.is_power_of_two());
    assert!(TYPICAL_PAGE_SIZE <= (usize::MAX / 8) + 1);
};

// Check the hard-coded value in `ast_tools` raw transfer generator is accurate.
// We can only do this check if we're on a 64-bit little-endian platform with the `fixed_size` feature enabled,
// because the `fixed_size_constants` module is only compiled under those conditions.
// That's good enough, as the size of `ChunkFooter` only matters in that case anyway (Oxlint JS plugins).
#[cfg(all(feature = "fixed_size", target_pointer_width = "64", target_endian = "little"))]
const _: () = {
    use crate::generated::fixed_size_constants::CHUNK_FOOTER_SIZE as EXPECTED_CHUNK_FOOTER_SIZE;
    assert!(CHUNK_FOOTER_SIZE == EXPECTED_CHUNK_FOOTER_SIZE);
};

/// Maximum typical overhead per allocation imposed by allocators.
const MALLOC_OVERHEAD: usize = 16;

const _: () = {
    assert!(MALLOC_OVERHEAD == CHUNK_ALIGN);
    assert!(MALLOC_OVERHEAD < TYPICAL_PAGE_SIZE);
};

/// The overhead from malloc, footer and alignment.
///
/// For instance, if we want to request a chunk of memory that has at least X bytes usable for allocations
/// (where X is aligned to `CHUNK_ALIGN`), then we expect that after adding a footer, `malloc` overhead,
/// and alignment, the chunk of memory the allocator actually sets aside for us is `X + OVERHEAD`,
/// rounded up to the nearest suitable size boundary.
pub const OVERHEAD: usize = match round_up_to(MALLOC_OVERHEAD + CHUNK_FOOTER_SIZE, CHUNK_ALIGN) {
    Some(x) => x,
    None => panic!(),
};

const _: () = {
    assert!(OVERHEAD > CHUNK_FOOTER_SIZE);
    assert!(OVERHEAD < TYPICAL_PAGE_SIZE); // Implies `OVERHEAD < (usize::MAX / 8) + 1`
};

/// The target size of our first allocation, including our overhead.
///
/// The available capacity will be slightly smaller.
/// 16 KiB covers the majority of real-world JS/TS files.
const FIRST_ALLOCATION_GOAL: usize = 16 * 1024;

const _: () = {
    assert!(FIRST_ALLOCATION_GOAL.is_power_of_two());
    assert!(FIRST_ALLOCATION_GOAL <= (usize::MAX / 8) + 1);
    assert!(FIRST_ALLOCATION_GOAL > OVERHEAD);
};

/// Default chunk size for a new `Arena`, minus the size of the footer.
///
/// The actual size of the first allocation is going to be a bit smaller than the goal.
/// We need to make room for the footer, and we also need take the alignment into account.
/// We're trying to avoid this kind of situation:
/// <https://blog.mozilla.org/nnethercote/2011/08/05/clownshoes-available-in-sizes-2101-and-up/>
pub const DEFAULT_CHUNK_SIZE_WITHOUT_FOOTER: usize = FIRST_ALLOCATION_GOAL - OVERHEAD;

// Note: We don't have constructors as methods on `impl<N> Arena<N>` that return `Self` because then `rustc`
// can't infer the `N` if it isn't explicitly provided, even though it has a default value.
// There doesn't seem to be a good workaround, other than putting constructors on the `Arena<DEFAULT>`.
// Even `std` does this same thing with `HashMap`, for example.
impl Arena<1> {
    /// Construct a new arena to allocate into.
    ///
    /// # Example
    ///
    /// ```
    /// # use oxc_allocator::arena::Arena;
    /// let arena = Arena::new();
    /// ```
    #[inline] // Because it's very cheap
    pub fn new() -> Self {
        Self::with_min_align()
    }

    /// Construct a new arena with the specified byte capacity to allocate into.
    ///
    /// # Example
    ///
    /// ```
    /// # use oxc_allocator::arena::Arena;
    /// let arena = Arena::with_capacity(100);
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if allocating the initial capacity fails.
    #[inline] // Because it just delegates
    pub fn with_capacity(capacity: usize) -> Self {
        Self::with_min_align_and_capacity(capacity)
    }

    /// Attempt to construct a new arena with the specified byte capacity to allocate into.
    ///
    /// Propagates errors when allocating the initial capacity.
    ///
    /// # Example
    ///
    /// ```
    /// # use oxc_allocator::arena::{AllocErr, Arena};
    /// # fn _foo() -> Result<(), AllocErr> {
    /// let arena = Arena::try_with_capacity(100)?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `Err(AllocErr)` if any of:
    ///
    /// 1. A [`Layout`] cannot be constructed from `capacity` and `MIN_ALIGN`
    ///    (for example because the rounded-up size overflows `isize`).
    /// 2. Computing the new chunk's memory details overflows.
    /// 3. The underlying global allocator fails to allocate the initial chunk.
    ///
    /// When `capacity` is `0`, no allocation is performed, and `Ok` is always returned.
    #[inline] // Because it just delegates
    pub fn try_with_capacity(capacity: usize) -> Result<Self, AllocErr> {
        Self::try_with_min_align_and_capacity(capacity)
    }
}

impl<const MIN_ALIGN: usize> Arena<MIN_ALIGN> {
    /// Create a new `Arena` that enforces a minimum alignment.
    ///
    /// The minimum alignment must be a power of 2, and no larger than 16.
    ///
    /// Enforcing a minimum alignment can speed up allocation of objects with alignment less than or equal to
    /// the minimum alignment. This comes at the cost of introducing otherwise-unnecessary padding between
    /// allocations of objects with alignment less than the minimum.
    ///
    /// # Example
    ///
    /// ```
    /// # use oxc_allocator::arena::Arena;
    ///
    /// type ArenaAlign8 = Arena<8>;
    /// let arena = ArenaAlign8::with_min_align();
    /// for x in 0..u8::MAX {
    ///     let x = arena.alloc(x);
    ///     assert!(std::ptr::from_ref(x).addr().is_multiple_of(8), "x is aligned to 8");
    /// }
    /// ```
    //
    // Because of `rustc`'s poor type inference for default type/const parameters (see the comment above
    // the `impl Arena` block with no const `MIN_ALIGN` parameter), and because we don't want to force everyone
    // to specify a minimum alignment with `Arena::new()` et al, we have a separate constructor
    // for specifying the minimum alignment.
    #[inline] // Because it's very cheap
    pub fn with_min_align() -> Self {
        Self::new_impl(EMPTY_ARENA_DATA_PTR, EMPTY_ARENA_DATA_PTR, None)
    }

    /// Create a new `Arena` that enforces a minimum alignment, and starts with room for at least `capacity` bytes.
    ///
    /// The minimum alignment must be a power of 2, and no larger than 16.
    ///
    /// Enforcing a minimum alignment can speed up allocation of objects with alignment less than or equal to
    /// the minimum alignment. This comes at the cost of introducing otherwise-unnecessary padding between
    /// allocations of objects with alignment less than the minimum.
    ///
    /// # Example
    ///
    /// ```
    /// # use oxc_allocator::arena::Arena;
    ///
    /// type ArenaAlign8 = Arena<8>;
    /// let mut arena = ArenaAlign8::with_min_align_and_capacity(8 * 100);
    /// for x in 0..100_u64 {
    ///     let x = arena.alloc(x);
    ///     assert!(std::ptr::from_ref(x).addr().is_multiple_of(8), "x is aligned to 8");
    /// }
    /// assert_eq!(
    ///     arena.iter_allocated_chunks().count(), 1,
    ///     "initial chunk had capacity for all allocations",
    /// );
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if allocating the initial capacity fails.
    pub fn with_min_align_and_capacity(capacity: usize) -> Self {
        Self::try_with_min_align_and_capacity(capacity).unwrap_or_else(|_| oom())
    }

    /// Create a new `Arena` that enforces a minimum alignment, and starts with room for at least `capacity` bytes.
    ///
    /// The minimum alignment must be a power of 2, and no larger than 16.
    ///
    /// Enforcing a minimum alignment can speed up allocation of objects with alignment less than or equal to
    /// the minimum alignment. This comes at the cost of introducing otherwise-unnecessary padding between
    /// allocations of objects with alignment less than the minimum.
    ///
    /// # Example
    ///
    /// ```
    /// # use oxc_allocator::arena::{AllocErr, Arena};
    /// # fn _foo() -> Result<(), AllocErr> {
    /// type ArenaAlign8 = Arena<8>;
    /// let mut arena = ArenaAlign8::try_with_min_align_and_capacity(8 * 100)?;
    /// for x in 0..100_u64 {
    ///     let x = arena.alloc(x);
    ///     assert!(std::ptr::from_ref(x).addr().is_multiple_of(8), "x is aligned to 8");
    /// }
    /// assert_eq!(
    ///     arena.iter_allocated_chunks().count(), 1,
    ///     "initial chunk had capacity for all allocations",
    /// );
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns `Err(AllocErr)` if any of:
    ///
    /// 1. A [`Layout`] cannot be constructed from `capacity` and `MIN_ALIGN`
    ///    (for example because the rounded-up size overflows `isize`).
    /// 2. Computing the new chunk's memory details overflows.
    /// 3. The underlying global allocator fails to allocate the initial chunk.
    ///
    /// When `capacity` is 0, no allocation is performed, and `Ok` is always returned.
    pub fn try_with_min_align_and_capacity(capacity: usize) -> Result<Self, AllocErr> {
        if capacity == 0 {
            return Ok(Self::with_min_align());
        }

        let layout = layout_from_size_align(capacity, MIN_ALIGN)?;

        // In original `bumpalo` code, `new_chunk` took `Option<usize>` as `new_size_without_footer`,
        // and converted `None` to `DEFAULT_CHUNK_SIZE_WITHOUT_FOOTER`. Here it was always called with
        // `None`, so chunk size was always increased to `DEFAULT_CHUNK_SIZE_WITHOUT_FOOTER` at minimum.
        //
        // We changed this behavior when we increased `FIRST_ALLOCATION_GOAL` to 16 KiB, to avoid
        // `Arena::with_capacity` allocating 16 KiB even when requested `capacity` is much smaller.
        //
        // SAFETY: We pass `None` as `previous_chunk_footer_ptr` (because we're creating the first chunk).
        let (start_ptr, chunk_footer_ptr) =
            unsafe { Self::new_chunk(capacity, layout, None) }.ok_or(AllocErr)?;

        // Initial cursor sits at the footer, which is the end of the allocatable region.
        // The footer is aligned on `CHUNK_ALIGN`, which is `>= MIN_ALIGN`, so this is already aligned to `MIN_ALIGN`.
        let cursor_ptr = chunk_footer_ptr.cast::<u8>();

        Ok(Self::new_impl(start_ptr, cursor_ptr, Some(chunk_footer_ptr)))
    }

    /// Create a new `Arena`.
    ///
    /// `chunk_footer_ptr` is `None` if no initial chunk has been allocated (the empty `Arena` case).
    ///
    /// This is a helper function for all code paths which create an `Arena`.
    /// All code paths which create an `Arena` must go through this method in order to validate `MIN_ALIGN`.
    #[inline(always)]
    pub(super) fn new_impl(
        start_ptr: NonNull<u8>,
        cursor_ptr: NonNull<u8>,
        chunk_footer_ptr: Option<NonNull<ChunkFooter>>,
    ) -> Self {
        // Const assert that `MIN_ALIGN` is valid.
        // This line must be present - the validation assertions don't run unless the const is referenced
        // in active code paths. This method is called by all other methods which create an `Arena`,
        // so we only need it here to ensure that it's impossible to create an `Arena` with an invalid `MIN_ALIGN`.
        const { Self::MIN_ALIGN };

        debug_assert!(is_pointer_aligned_to(cursor_ptr, MIN_ALIGN));
        debug_assert!(start_ptr <= cursor_ptr);
        debug_assert!(
            chunk_footer_ptr.is_none_or(|footer_ptr| cursor_ptr <= footer_ptr.cast::<u8>())
        );

        Self {
            cursor_ptr: Cell::new(cursor_ptr),
            current_chunk_footer_ptr: Cell::new(chunk_footer_ptr),
            start_ptr: Cell::new(start_ptr),
            can_grow: true,
            #[cfg(all(feature = "track_allocations", not(feature = "disable_track_allocations")))]
            stats: AllocationStats::default(),
        }
    }

    /// Allocate a new chunk and return pointers to its start and its initialized footer.
    ///
    /// The actual chunk size is derived from `new_size_without_footer` and `requested_layout`,
    /// rounded up to play nicely with the global allocator
    /// (power of two for small chunks, or a page boundary for larger ones).
    ///
    /// `previous_chunk_footer_ptr` is `None` when the new chunk will be the first chunk in the arena.
    ///
    /// Returns `None` if either:
    /// * The requested size/layout is too large for the global allocator to service.
    /// * The global allocator fails to allocate the requested size.
    ///
    /// # SAFETY
    ///
    /// `previous_chunk_footer_ptr` must point to the current `ChunkFooter` for this `Arena`,
    /// or `None` if this will be the first chunk in the arena.
    pub(super) unsafe fn new_chunk(
        new_size_without_footer: usize,
        requested_layout: Layout,
        previous_chunk_footer_ptr: Option<NonNull<ChunkFooter>>,
    ) -> Option<(
        // Pointer to start of allocatable region of the new chunk
        NonNull<u8>,
        // Pointer to the new chunk's footer
        NonNull<ChunkFooter>,
    )> {
        // Chunks must be aligned to at least `CHUNK_ALIGN` and `MIN_ALIGN`.
        // `MIN_ALIGN` is always `<= CHUNK_ALIGN`, so aligning to `CHUNK_ALIGN` satisfies both.
        let align = max(requested_layout.align(), CHUNK_ALIGN);

        // SAFETY: `Layout::size()` is always `<= isize::MAX`. `align` is a `usize` power of 2.
        // So rounding up can result in at maximum `isize::MAX + 1` - cannot overflow `usize`.
        let requested_size = unsafe { round_up_to_unchecked(requested_layout.size(), align) };
        let new_size_without_footer = max(new_size_without_footer, requested_size);

        // We want our allocations to play nice with the memory allocator, and waste as little memory as possible.
        // For small allocations, this means that the entire allocation including the chunk footer and `malloc`'s
        // internal overhead is as close to a power of two as we can go without going over.
        // For larger allocations, we only need to get close to a page boundary without going over.
        let new_size_with_overhead = if new_size_without_footer < TYPICAL_PAGE_SIZE {
            // `TYPICAL_PAGE_SIZE` and `OVERHEAD` are both `<= (usize::MAX / 8) + 1`.
            // `new_size_without_footer < TYPICAL_PAGE_SIZE`, so:
            // * `new_size_without_footer + OVERHEAD` is `<= usize::MAX / 4`, so cannot overflow `usize`.
            // * `.next_power_of_two()` result is `<= (usize::MAX / 4) + 1`, so also cannot overflow `usize`.
            (new_size_without_footer + OVERHEAD).next_power_of_two()

            // `new_size_with_overhead`:
            // * is always `>= OVERHEAD`.
            // * is always `< isize::MAX`.
            //
            // `size` (`new_size_with_overhead - OVERHEAD + CHUNK_FOOTER_SIZE`):
            // * is always a valid size for `Layout` for any value of `align` except `isize::MAX + 1`.
            // * will *not* be a valid size for `Layout` with alignment `align` if `align == isize::MAX + 1`.
        } else {
            // There is no guarantee of max value of `new_size_without_footer`, so we need to check it here,
            // to prevent overflow below.
            //
            // `MAX_SIZE_WITHOUT_FOOTER` is the maximum size without footer which can end up with a `size` value
            // which is a valid `Layout` with `align == CHUNK_ALIGN` (the minimum).
            // It does *not* guarantee that `size` can form a valid `Layout` for *any* value of `align`,
            // but it does rule out definitely invalid sizes, without ruling out any possibly valid sizes.
            // We could make this check looser, but we might as well rule out sizes that definitely won't work,
            // and will fail later anyway.
            const MAX_SIZE_WITHOUT_FOOTER: usize = isize::MAX as usize + 1 - OVERHEAD;
            if new_size_without_footer > MAX_SIZE_WITHOUT_FOOTER {
                return None;
            }

            // SAFETY: The check above guarantees that `new_size_without_footer + OVERHEAD` cannot overflow `usize`,
            // and rounding up to `TYPICAL_PAGE_SIZE` also cannot overflow `usize`.
            unsafe { round_up_to_unchecked(new_size_without_footer + OVERHEAD, TYPICAL_PAGE_SIZE) }

            // `new_size_with_overhead`:
            // * is always `>= OVERHEAD`.
            // * is always `<= isize::MAX + 1`.
            //
            // `size` (`new_size_with_overhead - OVERHEAD + CHUNK_FOOTER_SIZE`):
            // * is always a valid size for `Layout` with `align == CHUNK_ALIGN` (the minimum).
            // * may *not* be a valid size for `Layout` with alignment `align` if `align > CHUNK_ALIGN`.
        };

        debug_assert!(new_size_with_overhead <= isize::MAX as usize + 1);

        // Cannot underflow as `new_size_with_overhead` is `>= OVERHEAD`
        let new_size_without_footer = new_size_with_overhead - OVERHEAD;

        debug_assert!(align.is_multiple_of(CHUNK_ALIGN));
        debug_assert!(new_size_without_footer.is_multiple_of(CHUNK_ALIGN));
        // Cannot overflow because `CHUNK_FOOTER_SIZE < OVERHEAD`, and we just subtracted `OVERHEAD`
        let size = new_size_without_footer + CHUNK_FOOTER_SIZE;

        let layout = layout_from_size_align(size, align).ok()?;

        debug_assert!(size >= requested_layout.size());
        debug_assert!(size > 0);

        // Allocate memory for the chunk.
        // SAFETY: `layout` has non-zero size.
        let start_ptr = unsafe { alloc::alloc(layout) };
        let start_ptr = NonNull::new(start_ptr)?;

        // The `ChunkFooter` is at the end of the chunk.
        // SAFETY: We allocated `new_size_without_footer + CHUNK_FOOTER_SIZE` bytes, starting at `start_ptr`,
        // so `start_ptr + new_size_without_footer` is within that allocation.
        let footer_ptr = unsafe { start_ptr.add(new_size_without_footer) }.cast::<ChunkFooter>();
        debug_assert!(is_pointer_aligned_to(start_ptr, align));
        debug_assert!(is_pointer_aligned_to(footer_ptr, CHUNK_ALIGN));

        // Initial cursor sits at the footer, which is the end of the allocatable region.
        // The footer is aligned on `CHUNK_ALIGN`, which is `>= MIN_ALIGN`, so this is already aligned to `MIN_ALIGN`.
        let cursor_ptr = footer_ptr.cast::<u8>();
        debug_assert!(is_pointer_aligned_to(cursor_ptr, MIN_ALIGN));

        // SAFETY: `footer_ptr + size_of::<ChunkFooter>()` is the end of the allocation we just made,
        // and `footer_ptr` is aligned for `ChunkFooter`
        unsafe {
            footer_ptr.write(ChunkFooter {
                backing_alloc_ptr: start_ptr,
                layout,
                previous_chunk_footer_ptr: Cell::new(previous_chunk_footer_ptr),
                cursor_ptr: Cell::new(cursor_ptr),
            });
        }

        Some((start_ptr, footer_ptr))
    }
}

impl<const MIN_ALIGN: usize> Default for Arena<MIN_ALIGN> {
    #[inline] // Because it just delegates
    fn default() -> Self {
        Self::with_min_align()
    }
}
