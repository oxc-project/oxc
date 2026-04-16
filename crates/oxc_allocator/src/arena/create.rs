//! Methods to create an `Arena`.

use std::{
    alloc::{self, Layout},
    cell::Cell,
    ptr::{self, NonNull},
};

use super::bumpalo_alloc::AllocErr;

#[cfg(all(feature = "track_allocations", not(feature = "disable_track_allocations")))]
use crate::tracking::AllocationStats;

use super::{
    Arena, CHUNK_ALIGN, CHUNK_FOOTER_SIZE, ChunkFooter, EMPTY_CHUNK,
    utils::{layout_from_size_align, oom, round_up_to},
};

/// The typical page size these days.
///
/// Note that we don't need to exactly match page size for correctness, and it is okay if this is smaller than
/// the real page size in practice. It isn't worth the portability concerns and lack of const propagation that
/// dynamically looking up the actual page size implies.
const TYPICAL_PAGE_SIZE: usize = 0x1000;

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

/// The overhead from malloc, footer and alignment.
///
/// For instance, if we want to request a chunk of memory that has at least X bytes usable for allocations
/// (where X is aligned to `CHUNK_ALIGN`), then we expect that after adding a footer, `malloc` overhead,
/// and alignment, the chunk of memory the allocator actually sets aside for us is `X + OVERHEAD`,
/// rounded up to the nearest suitable size boundary.
const OVERHEAD: usize = match round_up_to(MALLOC_OVERHEAD + CHUNK_FOOTER_SIZE, CHUNK_ALIGN) {
    Some(x) => x,
    None => panic!(),
};

/// The target size of our first allocation, including our overhead.
///
/// The available capacity will be slightly smaller.
/// 16 KiB covers the majority of real-world JS/TS files.
const FIRST_ALLOCATION_GOAL: usize = 16 * 1024;

/// Default chunk size for a new `Arena`, minus the size of the footer.
///
/// The actual size of the first allocation is going to be a bit smaller than the goal.
/// We need to make room for the footer, and we also need take the alignment into account.
/// We're trying to avoid this kind of situation:
/// <https://blog.mozilla.org/nnethercote/2011/08/05/clownshoes-available-in-sizes-2101-and-up/>
pub const DEFAULT_CHUNK_SIZE_WITHOUT_FOOTER: usize = FIRST_ALLOCATION_GOAL - OVERHEAD;

/// The memory size and alignment details for a potential new chunk allocation.
#[derive(Debug, Clone, Copy)]
pub struct NewChunkMemoryDetails {
    new_size_without_footer: usize,
    align: usize,
    size: usize,
}

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
    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    /// Attempt to construct a new arena to allocate into.
    ///
    /// # Example
    ///
    /// ```
    /// # use oxc_allocator::arena::Arena;
    /// let arena = Arena::try_new();
    /// # let _ = arena.unwrap();
    /// ```
    #[expect(clippy::missing_errors_doc, reason = "`try_with_capacity(0)` always returns `Ok`")]
    pub fn try_new() -> Result<Self, AllocErr> {
        Arena::try_with_capacity(0)
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
    pub fn with_capacity(capacity: usize) -> Self {
        Self::try_with_capacity(capacity).unwrap_or_else(|_| oom())
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
    ///     assert_eq!((x as *mut _ as usize) % 8, 0, "x is aligned to 8");
    /// }
    /// ```
    //
    // Because of `rustc`'s poor type inference for default type/const parameters (see the comment above
    // the `impl Arena` block with no const `MIN_ALIGN` parameter), and because we don't want to force everyone
    // to specify a minimum alignment with `Arena::new()` et al, we have a separate constructor
    // for specifying the minimum alignment.
    pub fn with_min_align() -> Self {
        Self::new_impl(EMPTY_CHUNK.get())
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
    ///     assert_eq!((x as *mut _ as usize) % 8, 0, "x is aligned to 8");
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
    ///     assert_eq!((x as *mut _ as usize) % 8, 0, "x is aligned to 8");
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
            return Ok(Self::new_impl(EMPTY_CHUNK.get()));
        }

        let layout = layout_from_size_align(capacity, MIN_ALIGN)?;

        let chunk_footer = unsafe {
            Self::new_chunk(
                // `new_size_without_footer` here was `None` in original `bumpalo` code.
                // Changed to `Some(capacity)` when we increased `FIRST_ALLOCATION_GOAL` to 16 KiB,
                // to avoid `Arena::with_capacity` allocating 16 KiB even when requested `capacity` is much smaller.
                Self::new_chunk_memory_details(Some(capacity), layout).ok_or(AllocErr)?,
                layout,
                EMPTY_CHUNK.get(),
            )
            .ok_or(AllocErr)?
        };

        Ok(Self::new_impl(chunk_footer))
    }

    /// Create a new `Arena` from a chunk footer pointer.
    ///
    /// This is a helper function for all code paths which create an `Arena`.
    /// All code paths which create an `Arena` must go through this method in order to validate `MIN_ALIGN`.
    #[inline(always)]
    pub(super) fn new_impl(chunk_footer_ptr: NonNull<ChunkFooter>) -> Self {
        // Const assert that `MIN_ALIGN` is valid.
        // This line must be present - the validation assertions don't run unless the const is referenced
        // in active code paths. This method is called by all other methods which create an `Arena`,
        // so we only need it here to ensure that it's impossible to create an `Arena` with an invalid `MIN_ALIGN`.
        const { Self::MIN_ALIGN };

        Self {
            current_chunk_footer: Cell::new(chunk_footer_ptr),
            can_grow: true,
            #[cfg(all(feature = "track_allocations", not(feature = "disable_track_allocations")))]
            stats: AllocationStats::default(),
        }
    }

    /// Determine the memory details including final size, alignment, and final size without footer
    /// for a new chunk that would be allocated to fulfill an allocation request.
    pub(super) fn new_chunk_memory_details(
        new_size_without_footer: Option<usize>,
        requested_layout: Layout,
    ) -> Option<NewChunkMemoryDetails> {
        // We must have `CHUNK_ALIGN` or better alignment...
        let align = CHUNK_ALIGN
            // and we have to have at least our configured minimum alignment...
            .max(MIN_ALIGN)
            // and make sure we satisfy the requested allocation's alignment
            .max(requested_layout.align());

        let mut new_size_without_footer =
            new_size_without_footer.unwrap_or(DEFAULT_CHUNK_SIZE_WITHOUT_FOOTER);

        let requested_size =
            round_up_to(requested_layout.size(), align).unwrap_or_else(allocation_size_overflow);
        new_size_without_footer = new_size_without_footer.max(requested_size);

        // We want our allocations to play nice with the memory allocator, and waste as little memory as possible.
        // For small allocations, this means that the entire allocation including the chunk footer and `malloc`'s
        // internal overhead is as close to a power of two as we can go without going over.
        // For larger allocations, we only need to get close to a page boundary without going over.
        if new_size_without_footer < TYPICAL_PAGE_SIZE {
            new_size_without_footer =
                (new_size_without_footer + OVERHEAD).next_power_of_two() - OVERHEAD;
        } else {
            new_size_without_footer =
                round_up_to(new_size_without_footer + OVERHEAD, TYPICAL_PAGE_SIZE)? - OVERHEAD;
        }

        debug_assert_eq!(align % CHUNK_ALIGN, 0);
        debug_assert_eq!(new_size_without_footer % CHUNK_ALIGN, 0);
        let size = new_size_without_footer
            .checked_add(CHUNK_FOOTER_SIZE)
            .unwrap_or_else(allocation_size_overflow);

        Some(NewChunkMemoryDetails { new_size_without_footer, align, size })
    }

    /// Allocate a new chunk and return its initialized footer.
    pub(super) unsafe fn new_chunk(
        new_chunk_memory_details: NewChunkMemoryDetails,
        requested_layout: Layout,
        prev: NonNull<ChunkFooter>,
    ) -> Option<NonNull<ChunkFooter>> {
        unsafe {
            let NewChunkMemoryDetails { new_size_without_footer, align, size } =
                new_chunk_memory_details;

            let layout = layout_from_size_align(size, align).ok()?;

            debug_assert!(size >= requested_layout.size());

            let start_ptr = alloc::alloc(layout);
            let start_ptr = NonNull::new(start_ptr)?;

            // The `ChunkFooter` is at the end of the chunk
            let footer_ptr = start_ptr.as_ptr().add(new_size_without_footer);
            debug_assert_eq!((start_ptr.as_ptr() as usize) % align, 0);
            debug_assert_eq!(footer_ptr as usize % CHUNK_ALIGN, 0);
            #[expect(
                clippy::cast_ptr_alignment,
                reason = "footer_ptr is aligned to CHUNK_ALIGN, which is == align_of::<ChunkFooter>()"
            )]
            let footer_ptr = footer_ptr.cast::<ChunkFooter>();

            // Initial cursor sits at the footer, which is the end of the allocatable region.
            // The footer is aligned on `CHUNK_ALIGN`, which is `>= MIN_ALIGN`, so this is already aligned to `MIN_ALIGN`.
            let cursor_ptr = NonNull::new_unchecked(footer_ptr.cast::<u8>());
            debug_assert_eq!(cursor_ptr.as_ptr() as usize % MIN_ALIGN, 0);

            ptr::write(
                footer_ptr,
                ChunkFooter {
                    start_ptr,
                    layout,
                    previous_chunk_footer_ptr: Cell::new(prev),
                    cursor_ptr: Cell::new(cursor_ptr),
                },
            );

            Some(NonNull::new_unchecked(footer_ptr))
        }
    }
}

impl<const MIN_ALIGN: usize> Default for Arena<MIN_ALIGN> {
    fn default() -> Self {
        Self::with_min_align()
    }
}

#[cold]
#[inline(never)]
fn allocation_size_overflow<T>() -> T {
    panic!("requested allocation size overflowed")
}
