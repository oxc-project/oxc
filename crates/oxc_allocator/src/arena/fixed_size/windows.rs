//! Windows fixed-size allocator.
//!
//! # Regions
//!
//! In descending order of size:
//!
//! 1. Reserved (`RESERVED_SIZE` = 6 GiB)
//!    The address range we obtain from `VirtualAlloc(MEM_RESERVE)`.
//!    Address space only, no physical memory committed.
//!
//! 2. Container (`CONTAINER_SIZE` = 2 GiB, `CONTAINER_ALIGN`-aligned = 4 GiB-aligned)
//!    The 4 GiB-aligned region within Reserved where the Arena lives.
//!    Container holds the Committed region, which holds the Chunk.
//!
//! 3. Block (`BLOCK_SIZE` = `CONTAINER_SIZE - 16`)
//!    The cross-platform notion of the Arena's region (sized by the `BLOCK_SIZE` constant).
//!    We avoid using this term here - this Windows implementation works with the page-aligned Container instead,
//!    which is 16 bytes larger than Block. Those 16 trailing bytes are committed but never touched by
//!    the allocator (the cursor never goes above `ChunkFooter`).
//!
//! 4. Committed (page-aligned, dynamic)
//!    Pages within Container backed by physical memory (via `VirtualAlloc(MEM_COMMIT)`).
//!    `start_ptr..container_end_ptr`.
//!    The start expands downwards as the Arena grows.
//!    End is fixed at end of Container's region.
//!
//! 5. Chunk (`start_ptr..footer_ptr + CHUNK_FOOTER_SIZE`)
//!    What the Arena considers its chunk.
//!    Same start point as Committed (expands downwards as the Arena grows).
//!    End point is fixed at Block's end (16 bytes earlier than Container's end).
//!    Only relevant for `INITIAL_CHUNK_SIZE` (what we report to `Arena::from_raw_parts`).
//!
//! # Diagram
//!
//! ```txt
//!                                                           RESERVED (size 6 GiB, align 4 KiB)
//! <-------------------------------------------------------> Reserved address space (`RESERVED_SIZE` bytes)
//!
//!                                                           CONTAINER (size 2 GiB, align 4 GiB)
//!        <----------------------------->                    Container (`CONTAINER_SIZE` bytes)
//!
//!                                                           BLOCK (size 2 GiB - 16 bytes, align 4 GiB)
//!        <--------------------------->                      Block (`BLOCK_SIZE` bytes = `CONTAINER_SIZE - 16`)
//!                                     ^^                    16 trailing bytes (in Container, past Block)
//!
//!                                                           COMMITTED (page-aligned, dynamic)
//!                                 <---->                    Initially: `FIRST_ALLOCATION_GOAL` bytes
//!                    <----------------->                    After growth: Doubles, capped at `CONTAINER_SIZE`
//!                    ^                 ^
//!                    start_ptr         container_end_ptr (fixed)
//!
//!                                                           CHUNK (same start as Committed, same end as Block)
//!                                 <-->                      Initial: `INITIAL_CHUNK_SIZE` = `FIRST_ALLOCATION_GOAL - 16`
//!                    <--------------->                      After growth: `committed_size - 16`
//!                                     ^^                    16 trailing bytes (in Committed, past Chunk)
//! ```
//!
//! # Allocation strategy
//!
//! We use `VirtualAlloc(MEM_RESERVE)` to reserve `CONTAINER_SIZE + CONTAINER_ALIGN` bytes of address space,
//! then locate a `CONTAINER_ALIGN`-aligned Container inside it. Over-reservation gives us room to find
//! an aligned position.
//!
//! `VirtualAlloc` only guarantees alignment to the OS allocation granularity (typically 64 KiB),
//! so we have to align manually. The over-reservation only consumes address space, not physical memory,
//! so the cost is negligible on 64-bit Windows (~128 TB of user-mode address space).
//!
//! Initially we commit only the last `FIRST_ALLOCATION_GOAL` bytes of the Container (16 KiB).
//! This region will contain the `ChunkFooter` at its end, and space for the first batch of allocations.
//! 16 KiB covers the majority of real-world JS/TS files, so in many cases arenas never need to grow.
//! Additional pages are committed lazily by `grow_fixed_size_chunk` as the bump allocator needs them,
//! using a doubling strategy.
//!
//! Reserve + commit also avoids `std`'s workaround for high-alignment allocations on Windows, which over-allocates
//! and stores the real allocation pointer in a hidden slot, committing an extra page just for that pointer.
//! <https://github.com/rust-lang/rust/blob/556d20a834126d2d0ac20743b9792b8474d6d03c/library/std/src/sys/alloc/windows.rs#L120-L137>
//!
//! # Location of `ChunkFooter`
//!
//! There's no reason we couldn't position `ChunkFooter` at the very end of the Container region,
//! but other code relies on it being at a static offset 16 bytes earlier (which it has to be on other platforms).
//! So we do the same here. This is the reason for the 16 byte difference between `BLOCK_SIZE` and `CONTAINER_SIZE`.

use std::{
    alloc::Layout,
    cmp::{max, min},
    io,
    ptr::NonNull,
};

#[cfg(not(miri))]
use std::{ffi::c_void, ptr};

#[cfg(not(miri))]
use windows_sys::Win32::System::Memory::{
    MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_NOACCESS, PAGE_READWRITE, VirtualAlloc, VirtualFree,
};

use crate::generated::fixed_size_constants::{BLOCK_ALIGN, BLOCK_SIZE};

use super::super::{
    Arena, CHUNK_ALIGN, CHUNK_FOOTER_SIZE, ChunkFooter,
    create::FIRST_ALLOCATION_GOAL,
    utils::{is_pointer_aligned_to, round_down_to, round_mut_ptr_down_to, round_up_to},
};

/// Page size on Windows. 4 KiB on every shipping Windows target (x86, x86_64, aarch64, etc).
///
/// Strictly the page size is queryable at runtime via `GetSystemInfo` / `SYSTEM_INFO::dwPageSize`
/// rather than statically guaranteed, but Microsoft has used 4 KiB on these architectures
/// for the entire history of Windows NT and changing it would break ABI for the existing software
/// ecosystem - so it's stable in practice.
///
/// Source: Raymond Chen, "What are the page sizes used by Windows on various processors?":
/// <https://devblogs.microsoft.com/oldnewthing/20210510-00/?p=105200>
///
/// # If `PAGE_SIZE` is wrong
///
/// Hardcoding 4 KiB does not introduce undefined behaviour even if the actual page size is larger
/// (e.g. 16 KiB or 64 KiB).
/// When `VirtualAlloc(MEM_COMMIT)` is called with an address or size that isn't aligned to the actual page,
/// Windows rounds the address down and the size up to the actual page boundary, so the range that gets committed
/// is always a *superset* of what we requested. Three properties keep this sound:
///
/// 1. The Container is aligned for any plausible page size.
///    `CONTAINER_ALIGN` (4 GiB) and `CONTAINER_SIZE` (2 GiB) are multiples of any page size up to 2 GiB,
///    so the rounded range never exits the Container at either end.
///
/// 2. We never write outside our tracked committed region (`start_ptr..container_end_ptr`).
///    The tracked region is always a subset of what the OS actually committed, so we never touch uncommitted memory.
///
/// 3. Subsequent grows may re-commit pages that an earlier rounding-up already committed.
///    Microsoft documents this as guaranteed success, not an error:
///
///    > An attempt to commit a page that is already committed does not cause the function to fail.
///    > This means that you can commit pages without first determining the current commitment state of each page.
///
///    > `VirtualAlloc` cannot reserve a reserved page. It can commit a page that is already committed.
///    > This means you can commit a range of pages, regardless of whether they have already been committed,
///    > and the function will not fail.
///
///    Source: <https://learn.microsoft.com/en-us/windows/win32/api/memoryapi/nf-memoryapi-virtualalloc>
///
/// The cost of being wrong: A few extra pages committed beyond what we track, and possibly some
/// redundant `VirtualAlloc(MEM_COMMIT)` calls. No UB.
const PAGE_SIZE: usize = 4096;

/// Alignment of the Container region.
/// Same as the cross-platform `BLOCK_ALIGN`.
const CONTAINER_ALIGN: usize = BLOCK_ALIGN;

/// Size of Container region = `BLOCK_SIZE` rounded up to a multiple of `PAGE_SIZE` (= 2 GiB).
///
/// The `-16` baked into `BLOCK_SIZE` is there to leave room for a `malloc`-style allocator's per-allocation metadata
/// (see comments in `arena/create.rs`).
/// This doesn't apply to `VirtualAlloc` - the OS tracks reservations in kernel structures, not in user space.
/// So on Windows we work in terms of the page-aligned size, which makes the committed-region tracking cleaner.
/// The committed size is always a multiple of `PAGE_SIZE`.
///
/// The 16 trailing bytes (Container minus Block) are inside our reservation but never used by the `Arena`.
/// Cursor never goes above `ChunkFooter`, which sits at offset `BLOCK_SIZE - CHUNK_FOOTER_SIZE` within the Container.
const CONTAINER_SIZE: usize = match round_up_to(BLOCK_SIZE, PAGE_SIZE) {
    Some(x) => x,
    None => panic!(),
};

/// Total size of address space we reserve.
/// Over-reserve by `CONTAINER_ALIGN` so we can find a `CONTAINER_ALIGN`-aligned region for Container within it.
/// `CONTAINER_SIZE + CONTAINER_ALIGN` is page-aligned (both terms are).
const RESERVED_SIZE: usize = CONTAINER_SIZE + CONTAINER_ALIGN;

/// Layout of Reserved region. Stored in `ChunkFooter::layout` for record-keeping.
/// `VirtualFree(MEM_RELEASE)` only needs the start address, so the layout's contents don't matter
/// for deallocation. However our `#[cfg(miri)]` stub does need the actual reserved size.
const RESERVED_LAYOUT: Layout = match Layout::from_size_align(RESERVED_SIZE, PAGE_SIZE) {
    Ok(layout) => layout,
    Err(_) => unreachable!(),
};

const _: () = {
    assert!(PAGE_SIZE.is_power_of_two());
    assert!(CONTAINER_ALIGN.is_power_of_two());
    assert!(CONTAINER_SIZE <= CONTAINER_ALIGN);
    // Initial commit must fit inside the Block, and be a multiple of `PAGE_SIZE`
    // so we can pass it directly to `VirtualAlloc(MEM_COMMIT)`
    assert!(FIRST_ALLOCATION_GOAL <= BLOCK_SIZE);
    assert!(FIRST_ALLOCATION_GOAL.is_multiple_of(PAGE_SIZE));
    // Container alignment must be a multiple of page size, so the Container is itself page-aligned
    assert!(CONTAINER_ALIGN.is_multiple_of(PAGE_SIZE));
    // `RESERVED_SIZE` must be a multiple of `PAGE_SIZE`. `VirtualAlloc(MEM_RESERVE)` would round it up anyway.
    // We make it explicit so we don't depend on that behaviour.
    assert!(RESERVED_SIZE.is_multiple_of(PAGE_SIZE));
};

const _: () = assert!(RESERVED_LAYOUT.size() > 0);

/// Offset within the Container of the start of the initially committed region (which contains the `ChunkFooter`).
const INITIAL_COMMITTED_START_OFFSET: usize = CONTAINER_SIZE - FIRST_ALLOCATION_GOAL;

/// Initial Chunk size = bytes from start of initially committed region to Block's end.
/// This is the `size` we pass to `Arena::from_raw_parts` in [`Arena::new_fixed_size`].
const INITIAL_CHUNK_SIZE: usize = BLOCK_SIZE - INITIAL_COMMITTED_START_OFFSET;

const _: () = {
    // `from_raw_parts` requires `size >= CHUNK_FOOTER_SIZE` and `size` to be a multiple of `CHUNK_ALIGN` (16)
    assert!(INITIAL_CHUNK_SIZE >= CHUNK_FOOTER_SIZE);
    assert!(INITIAL_CHUNK_SIZE.is_multiple_of(CHUNK_ALIGN));
    // The initially committed region must be page-aligned. `container_ptr` is `CONTAINER_ALIGN`-aligned
    // (which is a multiple of `PAGE_SIZE`), and `INITIAL_COMMITTED_START_OFFSET` is page-aligned by construction.
    assert!(INITIAL_COMMITTED_START_OFFSET.is_multiple_of(PAGE_SIZE));
    // The `ChunkFooter` must live entirely within the initially committed region.
    // Footer occupies `BLOCK_SIZE - CHUNK_FOOTER_SIZE..BLOCK_SIZE` within the Container.
    // Initial commit covers `CONTAINER_SIZE - FIRST_ALLOCATION_GOAL..CONTAINER_SIZE`.
    // So footer start must be `>= initial commit start`.
    assert!(BLOCK_SIZE - CHUNK_FOOTER_SIZE >= INITIAL_COMMITTED_START_OFFSET);
};

impl<const MIN_ALIGN: usize> Arena<MIN_ALIGN> {
    /// Construct a static-sized [`Arena`] backed by a Windows virtual-memory reservation.
    ///
    /// Reserves `RESERVED_SIZE` bytes of address space.
    /// Within that, locates the Container region (`CONTAINER_SIZE` bytes (2 GiB), aligned on `CONTAINER_ALIGN` (4 GiB)).
    ///
    /// Commits the last `FIRST_ALLOCATION_GOAL` bytes of the Container region (where the `ChunkFooter` lives).
    /// Subsequent allocations can grow the committed region downwards on demand via `grow_fixed_size_chunk`.
    /// The `Arena` cannot grow beyond `CONTAINER_SIZE` bytes (2 GiB) - it cannot add further chunks.
    ///
    /// Returns `None` if the reservation or initial commit fails.
    ///
    /// See module-level docs for the rationale and platform-specific allocation strategy.
    ///
    /// # Panics
    ///
    /// Panics if `VirtualFree` fails while releasing the reservation after a failed commit (should never happen).
    pub fn new_fixed_size() -> Option<Self> {
        // Reserve `CONTAINER_SIZE + CONTAINER_ALIGN` bytes of address space, without committing any pages
        let reservation_ptr = Mmap::reserve(RESERVED_SIZE).ok()?;

        // Find `CONTAINER_ALIGN`-aligned `container_ptr` within the reservation, by rounding up.
        // If `reservation_ptr` is already 4 GiB-aligned, `container_ptr == reservation_ptr`.
        //
        // For `CONTAINER_ALIGN = 4 GiB`, this `wrapping_neg` form compiles to one fewer instruction
        // than `round_nonnull_ptr_up_to_unchecked`. The AND mask `CONTAINER_ALIGN - 1 = 0xFFFFFFFF`
        // exactly matches the 32-bit register width, so the AND folds into a 32-bit `mov`. For other
        // alignments the helper would be at least as good - this is a 4 GiB-specific micro-optimization.
        //
        // It's also safer. `round_nonnull_ptr_up_to_unchecked` internally computes `addr + (divisor - 1)`,
        // which would be UB if it overflowed. The helper would be sound here today because `reservation_ptr`
        // is the start of a `RESERVED_SIZE` = 6 GiB allocation, so it's at least 6 GiB below `usize::MAX + 1`.
        // Therefore `+ (4 GiB - 1)` cannot overflow. But a future change to constants could silently turn that
        // into a hazard. `wrapping_neg` cannot overflow regardless of inputs.
        //
        // SAFETY: `offset < CONTAINER_ALIGN` and we reserved `CONTAINER_SIZE + CONTAINER_ALIGN` bytes,
        // so `reservation_ptr + offset` is in bounds.
        let container_ptr = unsafe {
            let reservation_addr = reservation_ptr.addr().get();
            let offset = reservation_addr.wrapping_neg() % CONTAINER_ALIGN;
            reservation_ptr.add(offset)
        };
        debug_assert!(is_pointer_aligned_to(container_ptr, CONTAINER_ALIGN));

        // Get pointer to the start of the initial committed region within the Container
        // (the region that contains `ChunkFooter`).
        // SAFETY: `INITIAL_COMMITTED_START_OFFSET < CONTAINER_SIZE`, and the Container's `CONTAINER_SIZE` bytes
        // are within the reservation, so `container_ptr + INITIAL_COMMITTED_START_OFFSET` is in bounds.
        let committed_ptr = unsafe { container_ptr.add(INITIAL_COMMITTED_START_OFFSET) };
        debug_assert!(is_pointer_aligned_to(committed_ptr, PAGE_SIZE));

        // Commit the initial region - the last `FIRST_ALLOCATION_GOAL` bytes of the Container.
        // SAFETY: `committed_ptr` is page-aligned and within the reservation we just made.
        let commit_result = unsafe { Mmap::commit(committed_ptr, FIRST_ALLOCATION_GOAL) };
        if commit_result.is_err() {
            // Commit failed - release the reservation before returning.
            // SAFETY: `reservation_ptr` was just returned by `Mmap::reserve(RESERVED_SIZE)`.
            unsafe { Mmap::free(reservation_ptr, RESERVED_SIZE) }.unwrap_or_else(|err| {
                panic!("`VirtualFree` failed during cleanup: {err}");
            });
            return None;
        }

        // Construct the Arena via `from_raw_parts`.
        //
        // It writes the `ChunkFooter` at `start_ptr + INITIAL_CHUNK_SIZE - CHUNK_FOOTER_SIZE`
        // = `container_addr + BLOCK_SIZE - CHUNK_FOOTER_SIZE`, so `ChunkFooter` ends 16 bytes before end of Container.
        // This is the position other code expects it to be.
        //
        // SAFETY:
        // * `committed_ptr` is page-aligned (so `CHUNK_ALIGN`-aligned, since `PAGE_SIZE >= CHUNK_ALIGN`).
        // * `INITIAL_CHUNK_SIZE` is asserted statically to be `>= CHUNK_FOOTER_SIZE` and a multiple of `CHUNK_ALIGN`.
        // * `committed_ptr..committed_ptr + INITIAL_CHUNK_SIZE` lies within
        //   `reservation_ptr..reservation_ptr + RESERVED_SIZE`: The Container is within the reservation,
        //   and `committed_ptr + INITIAL_CHUNK_SIZE` = Block end <= Container end <= Reserved end.
        // * The reservation was made via `VirtualAlloc` with `MEM_RESERVE`. The Arena's `Drop` impl
        //   delegates to `dealloc_fixed_size_arena_chunk` (because `is_fixed_size` is `true`),
        //   which correctly calls `VirtualFree(MEM_RELEASE)` on the reservation.
        // * The initially committed region contains both `committed_ptr` and the `ChunkFooter` slot,
        //   so writes there are valid.
        let arena = unsafe {
            Self::from_raw_parts(
                committed_ptr,
                INITIAL_CHUNK_SIZE,
                reservation_ptr,
                RESERVED_LAYOUT,
            )
        };

        Some(arena)
    }

    /// Attempt to grow the [`Arena`]'s current chunk in place to accommodate an allocation of `layout`.
    ///
    /// Commits more pages within the existing reservation, using a doubling strategy.
    /// Growth is bounded so the committed region grows at most to cover all of the Container region.
    ///
    /// If the chunk can be grown in place to accommodate the request:
    /// * Returns `Some(new_ptr)`, where `new_ptr` is the pointer to write the layout at.
    /// * Updates `start_ptr`.
    /// * Does NOT update `cursor_ptr` - that is left to the caller.
    ///
    /// Returns `None` if growth is not possible, due to any of:
    /// * All of Container region is already committed, so no further growth is possible.
    /// * There is insufficient space for `layout`, even with whole Container region committed.
    /// * `VirtualAlloc(MEM_COMMIT)` failed.
    ///
    /// # SAFETY
    ///
    /// * `Arena` must be fixed-size (created via `Arena::new_fixed_size`).
    /// * Arena must not be able to accommodate an allocation of `layout` within current chunk, prior to growing it.
    pub(in super::super) unsafe fn grow_fixed_size_chunk(
        &self,
        layout: Layout,
    ) -> Option<NonNull<u8>> {
        #[cfg(debug_assertions)]
        {
            let footer_ptr = self.current_chunk_footer_ptr.get().expect("Arena has no chunks");
            // SAFETY: `footer_ptr` always points to a valid `ChunkFooter`
            let footer = unsafe { footer_ptr.as_ref() };
            assert!(
                footer.is_fixed_size,
                "Only fixed-size allocators should be passed to `Arena::grow_fixed_size_chunk`"
            );
        }

        let chunk_start_ptr = self.start_ptr.get();
        let cursor_ptr = self.cursor_ptr.get().as_ptr();
        let chunk_start_addr = chunk_start_ptr.addr().get();

        debug_assert!(is_pointer_aligned_to(chunk_start_ptr, PAGE_SIZE));

        // Compute the Container's start, and the page-aligned upper bound of the Committed region.
        // `current_committed_size`, `new_committed_size`, and `new_committed_start_addr` are all
        // multiples of `PAGE_SIZE` by construction (since `container_addr` and `CONTAINER_SIZE` are
        // both page-aligned) - no separate page-alignment rounding step needed.
        let container_start_addr = round_down_to(chunk_start_addr, CONTAINER_ALIGN);
        let container_end_addr = container_start_addr + CONTAINER_SIZE;

        // Compute the new pointer the same way as allocation fast path (`Arena::try_alloc_layout_fast`).
        // See comments in that method for rationale and codegen notes.
        let new_ptr = cursor_ptr.wrapping_sub(layout.size());
        let align = max(layout.align(), MIN_ALIGN);
        let new_ptr = round_mut_ptr_down_to(new_ptr, align);

        // Bail if `new_ptr` is out of bounds of the Container.
        // The wrapping-sub trick (same as in `try_alloc_layout_fast`) detects this with a single branch.
        //
        // This handles three cases:
        // 1. The request is too large to fit, even with the whole Container fully committed.
        // 2. The Container is already fully committed (`chunk_start_addr == container_start_addr`).
        // 3. The arena was created via `Arena::from_raw_parts` from an externally-managed buffer
        //    (e.g. NAPI parser with raw transfer enabled), which creates Arena with `start_ptr` at start of Container.
        //    Same logic as the fully-committed case. We must not `VirtualAlloc(MEM_COMMIT)` on these -
        //    the backing memory wasn't reserved via `VirtualAlloc`.
        if new_ptr.addr().wrapping_sub(container_start_addr) > isize::MAX as usize {
            return None;
        }
        // SAFETY: The bounds check above ensures `new_ptr` is in bounds of the Container, so it cannot be null
        let new_ptr = unsafe { NonNull::new_unchecked(new_ptr) };

        // Compute the new Committed region start as whichever of the following results in a larger Committed region:
        // 1. Doubling: Extend committed region to twice its current size, capped at full Container size.
        // 2. Needed: Extend it down far enough to cover `new_ptr` (page-aligned).
        // The lower of the two starts wins, since lower start = more committed memory.
        //
        // `chunk_start_addr - current_committed_size` cannot underflow, since Container is 4 GiB aligned,
        // and `chunk_start_ptr` is within the Container. Therefore `chunk_start_addr >= 4 GiB`.
        // `current_committed_size` is at most `CONTAINER_SIZE` (2 GiB).
        let current_committed_size = container_end_addr - chunk_start_addr;
        let doubling_start_addr =
            max(chunk_start_addr - current_committed_size, container_start_addr);

        let needed_start_addr = round_down_to(new_ptr.addr().get(), PAGE_SIZE);

        let new_committed_start_addr = min(doubling_start_addr, needed_start_addr);
        debug_assert!(new_committed_start_addr.is_multiple_of(PAGE_SIZE));
        debug_assert!(new_committed_start_addr >= container_start_addr);

        // Caller guarantees `new_ptr < chunk_start_addr` (chunk could not service the allocation without growing).
        // So `new_committed_start_addr` must be before `chunk_start_addr`, and this subtraction cannot underflow,
        // or result in 0.
        let delta_committed = chunk_start_addr - new_committed_start_addr;
        debug_assert!(delta_committed > 0);

        // SAFETY: `new_committed_start_addr >= container_start_addr`, so subtracting `delta_committed`
        // from `chunk_start_ptr` stays in-bounds of the reservation
        let new_committed_start_ptr = unsafe { chunk_start_ptr.sub(delta_committed) };
        debug_assert!(is_pointer_aligned_to(new_committed_start_ptr, PAGE_SIZE));

        // Commit the new pages: `new_committed_start_ptr..chunk_start_ptr`.
        // SAFETY: `new_committed_start_ptr` is page-aligned and within the reservation. `delta_committed` is non-zero.
        let commit_result = unsafe { Mmap::commit(new_committed_start_ptr, delta_committed) };
        if commit_result.is_err() {
            return None;
        }

        self.start_ptr.set(new_committed_start_ptr);

        Some(new_ptr)
    }
}

/// Deallocate the chunk whose footer is pointed to by `footer_ptr`, when the chunk is fixed size,
/// created via `Arena::new_fixed_size`.
///
/// `dealloc_chunk` in `drop` module delegates to this function when chunk's `is_fixed_size` flag is set.
/// `free_fixed_size_allocator` in `pool/fixed_size.rs` also uses this function for deallocation.
///
/// This releases the entire reservation (committed and uncommitted parts) via `VirtualFree`.
///
/// # SAFETY
///
/// * `footer_ptr` must point to a valid `ChunkFooter`.
/// * `ChunkFooter` must be for a fixed size chunk.
/// * Chunk must have been created via `Arena::new_fixed_size`.
pub unsafe fn dealloc_fixed_size_arena_chunk(footer_ptr: NonNull<ChunkFooter>) {
    // Create `&ChunkFooter` reference in a block, to ensure it is not live when we deallocate the chunk's memory
    // (which contains the `ChunkFooter`)
    let (backing_alloc_ptr, layout, is_fixed_size) = {
        // SAFETY: Caller guarantees that `footer_ptr` points to a valid `ChunkFooter`
        let footer = unsafe { footer_ptr.as_ref() };
        (footer.backing_alloc_ptr, footer.layout, footer.is_fixed_size)
    };

    debug_assert!(
        is_fixed_size,
        "Only fixed-size allocators should be passed to `dealloc_fixed_size_arena_chunk` to deallocate"
    );

    // SAFETY: Each `ChunkFooter`'s `backing_alloc_ptr` and `layout` describe its backing allocation.
    // Caller guarantees chunk was created with `Arena::new_fixed_size`, so backing allocation was made
    // via `Mmap::reserve` with `layout.size()` bytes.
    unsafe { Mmap::free(backing_alloc_ptr, layout.size()) }.unwrap_or_else(|err| {
        panic!("`VirtualFree` failed: {err}");
    });
}

/// Windows-specific allocator wrapper around `VirtualAlloc` and `VirtualFree`.
///
/// Stateless namespace - all operations are associated functions, not methods.
/// The OS owns the bookkeeping, we just pass the reservation pointer back in for each operation.
///
/// # Usage flow
///
/// ```ignore
/// const RESERVED_SIZE: usize = 1 << 32; // 4 GiB
/// let start_ptr = Mmap::reserve(RESERVED_SIZE).unwrap();
///
/// // Offset pointers to anywhere within reservation using `add` and `sub`
/// // (not `wrapping_add` / `wrapping_sub`), even though this memory is not committed yet
/// let end_ptr = unsafe { start_ptr.add(RESERVED_SIZE) };
/// let mid_ptr = unsafe { end_ptr.sub(RESERVED_SIZE / 2) };
/// let start_ptr2 = unsafe { mid_ptr.sub(RESERVED_SIZE / 2) };
/// assert_eq!(start_ptr, start_ptr2);
///
/// // Commit a range of memory in the middle
/// const COMMIT_SIZE: usize = 4 * 1024; // 4 KiB
/// unsafe { Mmap::commit(mid_ptr, COMMIT_SIZE) }.unwrap();
///
/// // Write data to the committed region
/// unsafe { mid_ptr.write(123_u8) };
///
/// // Get reference to data, and/or read data back via a different pointer
/// // derived from original reservation pointer
/// let value = {
///     let value_ref: &u8 = unsafe { start_ptr.add(RESERVED_SIZE / 2).as_ref() };
///     *value_ref
/// };
/// assert_eq!(value, 123);
///
/// // Get distance between any two pointers pointing anywhere within reservation,
/// // both derived from the pointer returned by `Mmap::reserve`,
/// // even though this crosses through committed and uncommitted regions
/// let size = unsafe { end_ptr.offset_from_unsigned(start_ptr) };
/// assert_eq!(size, RESERVED_SIZE);
///
/// // Free the whole reservation, including both committed and uncommitted regions
/// unsafe { Mmap::free(start_ptr, RESERVED_SIZE) }.unwrap();
/// ```
///
/// # Pointer provenance
///
/// The pointer returned by [`reserve`] has provenance covering the entire reservation,
/// including pages that have not yet been committed. The whole reservation is considered a single
/// "allocation" in Rust's memory model. Any pointers derived from it via `.add()` / `.sub()`
/// inherit that provenance and remain valid for reading and writing once
/// [`commit`] has made the relevant pages accessible.
///
/// We do not need to use the pointer returned from `VirtualAlloc(MEM_COMMIT)` for that, and
/// in fact must not, because doing so would split the chunk across multiple provenances and
/// break pointer arithmetic that crosses commit boundaries (it would be UB).
///
/// ## Evidence for this interpretation
///
/// `MEM_COMMIT` is a page-state change within an existing reservation, not the creation of a
/// new Rust allocation. Ralf Jung (who maintains Rust's memory model) has explicitly endorsed
/// treating an `mmap` or `VirtualAlloc` reservation as a single Rust allocation:
/// <https://github.com/rust-lang/unsafe-code-guidelines/issues/430#issuecomment-1697423817>
/// <https://github.com/rust-lang/miri/issues/4187>
///
/// `wasmtime` and `mmap-rs` rely on the same pattern in production code.
/// Microsoft's own documentation describes `MEM_COMMIT` as "changing the state of a region of pages",
/// rather than allocating, and the canonical MS example code accesses committed pages
/// via the original `VirtualAlloc(MEM_RESERVE)` pointer.
///
/// The pattern here almost exactly matches the code in `wasmtime`:
/// <https://github.com/bytecodealliance/wasmtime/blob/386a3280dee61f5c4120ce7cde621c1039e383d5/crates/wasmtime/src/runtime/vm/sys/windows/mmap.rs>
///
/// # `windows-sys` crate
///
/// `Mmap` uses the battle-tested [`windows-sys`] crate for `VirtualAlloc` and `VirtualFree`.
///
/// Documentation for the underlying APIs:
/// <https://learn.microsoft.com/en-us/windows/win32/api/memoryapi/nf-memoryapi-virtualalloc>
/// <https://learn.microsoft.com/en-us/windows/win32/api/memoryapi/nf-memoryapi-virtualfree>
///
/// [`reserve`]: Self::reserve
/// [`commit`]: Self::commit
/// [`windows-sys`]: https://crates.io/crates/windows-sys
struct Mmap;

#[cfg(not(miri))]
impl Mmap {
    /// Reserve `size` bytes of address space, without committing any physical memory.
    ///
    /// Returns `Ok` with the pointer to the start of the reserved region,
    /// or `Err` carrying the OS error if `VirtualAlloc(MEM_RESERVE)` fails.
    ///
    /// `size` should be a multiple of the page size.
    /// Windows rounds up internally if not, but passing a page-aligned value avoids depending on that behaviour.
    fn reserve(size: usize) -> io::Result<NonNull<u8>> {
        // SAFETY: `VirtualAlloc(null, _, MEM_RESERVE, _)` is always safe to call.
        // We're asking the OS to pick the address. Returns null on failure.
        let reservation_ptr =
            unsafe { VirtualAlloc(ptr::null(), size, MEM_RESERVE, PAGE_NOACCESS) };
        NonNull::new(reservation_ptr.cast::<u8>()).ok_or_else(io::Error::last_os_error)
    }

    /// Commit `size` bytes of physical memory at `ptr`, mapped read+write.
    ///
    /// Returns `Err` carrying the OS error if `VirtualAlloc(MEM_COMMIT)` fails.
    ///
    /// # SAFETY
    ///
    /// * `ptr..ptr + size` must lie within an existing reservation made via [`reserve`].
    /// * `ptr` must be page-aligned.
    /// * `size` must be greater than 0.
    ///
    /// [`reserve`]: Self::reserve
    unsafe fn commit(ptr: NonNull<u8>, size: usize) -> io::Result<()> {
        // SAFETY: Caller guarantees the range lies within an existing reservation.
        // On success, returns a non-null pointer to the start of the committed page range.
        // This equals `ptr` only when `ptr` is aligned to the OS page size. If not, it's rounded down.
        // We discard the pointer either way - see `Mmap` type-level doc comment for why.
        // Returns null on failure.
        let result = unsafe {
            VirtualAlloc(ptr.as_ptr().cast::<c_void>(), size, MEM_COMMIT, PAGE_READWRITE)
        };
        if result.is_null() { Err(io::Error::last_os_error()) } else { Ok(()) }
    }

    /// Free a reservation made via [`reserve`].
    ///
    /// Releases both committed and uncommitted pages within the reserved region in a single call.
    ///
    /// Returns `Err` carrying the OS error if `VirtualFree(MEM_RELEASE)` fails.
    ///
    /// `size` is unused here (`VirtualFree(MEM_RELEASE)` doesn't need it) but is part of the API
    /// so the `#[cfg(miri)]` stub can pass it to `std::alloc::dealloc`. In release builds,
    /// after inlining, the unused load of `layout.size()` at the call site will be optimized out.
    ///
    /// # SAFETY
    ///
    /// * `reservation_ptr` must be the start of a reservation made by [`reserve`]
    ///   (either the original pointer, or one derived from it, with the same address).
    /// * `size` must be the same value passed to [`reserve`].
    /// * This reservation must not have been already freed.
    ///
    /// [`reserve`]: Self::reserve
    #[expect(unused_variables)]
    unsafe fn free(reservation_ptr: NonNull<u8>, size: usize) -> io::Result<()> {
        // `MEM_RELEASE` requires `size = 0`. Docs state:
        // > If the `dwFreeType` parameter is `MEM_RELEASE`, this parameter must be 0 (zero).
        //
        // This releases both committed and uncommitted regions. Docs state:
        // > The function does not fail if you attempt to release pages that are in different states,
        // > some reserved and some committed. This means that you can release a range of pages
        // > without first determining the current commitment state.
        //
        // Returns non-zero on success. Docs state:
        // > If the function succeeds, the return value is nonzero.
        //
        // SAFETY: Caller guarantees `reservation_ptr` is the start of a live reservation.
        let result =
            unsafe { VirtualFree(reservation_ptr.as_ptr().cast::<c_void>(), 0, MEM_RELEASE) };
        if result == 0 { Err(io::Error::last_os_error()) } else { Ok(()) }
    }
}

/// Miri stub for [`Mmap`].
/// Models the reservation as a single `std::alloc::alloc` allocation and `commit` as a no-op
/// (under Miri, the whole region is already accessible from `alloc`).
///
/// This lets `cargo miri test` exercise the bump allocator and grow logic in this file under Miri's
/// pointer-aliasing checker, even though Miri doesn't yet support `VirtualAlloc` directly
/// (see <https://github.com/rust-lang/miri/issues/4187>).
/// Modeled on `wasmtime`'s approach of swapping in a pure-Rust stub for OS-specific syscalls under Miri.
#[cfg(miri)]
impl Mmap {
    fn reserve(size: usize) -> io::Result<NonNull<u8>> {
        if size == 0 {
            return Err(io::Error::other("cannot reserve zero bytes"));
        }
        let layout = Layout::from_size_align(size, PAGE_SIZE)
            .map_err(|_| io::Error::other("invalid layout"))?;
        // SAFETY: Checked above that `size` is non-zero
        let ptr = unsafe { std::alloc::alloc(layout) };
        NonNull::new(ptr).ok_or_else(|| io::Error::other("alloc failed"))
    }

    unsafe fn commit(_ptr: NonNull<u8>, _size: usize) -> io::Result<()> {
        // No-op. The whole region is already accessible memory from `alloc`.
        //
        // Note: `wasmtime` models this operation as zero-filling the region, but we don't want to do that.
        // 1. It would make tests under Miri slower.
        // 2. We want Miri to catch if we write to uninitialized memory.
        //    Actually, `VirtualAlloc(MEM_COMMIT)` zero-fills pages, so committed pages aren't uninitialized,
        //    but we don't want to rely on that.
        Ok(())
    }

    unsafe fn free(reservation_ptr: NonNull<u8>, size: usize) -> io::Result<()> {
        let layout = Layout::from_size_align(size, PAGE_SIZE)
            .map_err(|_| io::Error::other("invalid layout"))?;
        // SAFETY: Caller guarantees `reservation_ptr` was returned by `reserve` with the same `size`,
        // so `layout` matches the original allocation
        unsafe { std::alloc::dealloc(reservation_ptr.as_ptr(), layout) };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::alloc::Layout;

    use super::*;

    /// Size of the chunk's data region (chunk size minus the `ChunkFooter`).
    fn data_capacity(arena: &Arena) -> usize {
        let start_addr = arena.start_ptr.get().addr().get();
        let data_end_addr = arena.data_end_ptr().addr().get();
        data_end_addr - start_addr
    }

    /// Bytes still available for allocation in the current chunk (cursor minus start).
    fn available(arena: &Arena) -> usize {
        let start_addr = arena.start_ptr.get().addr().get();
        let cursor_addr = arena.cursor_ptr().addr().get();
        cursor_addr - start_addr
    }

    /// Maximum single allocation that can fit. Block size minus the `ChunkFooter`.
    /// Block = `CONTAINER_SIZE - 16`. Footer occupies the last `CHUNK_FOOTER_SIZE` bytes of Block.
    const MAX_ALLOCATION: usize = BLOCK_SIZE - CHUNK_FOOTER_SIZE;
    const MALLOC_OVERHEAD: usize = CONTAINER_SIZE - BLOCK_SIZE;
    const OVERHEAD: usize = MALLOC_OVERHEAD + CHUNK_FOOTER_SIZE;

    #[test]
    fn growth_flow() {
        let arena: Arena = Arena::new_fixed_size().unwrap();
        let initial_start_addr = arena.start_ptr.get().addr().get();

        // 1. Initial chunk is sized for `FIRST_ALLOCATION_GOAL`
        let initial_capacity = FIRST_ALLOCATION_GOAL - OVERHEAD;
        assert_eq!(data_capacity(&arena), initial_capacity);
        assert_eq!(available(&arena), initial_capacity);

        // Sanity-check that real allocations work in the initial chunk
        let byte_a = arena.alloc(0xAAu8);
        assert_eq!(*byte_a, 0xAA);
        let mut consumed = 1;

        // 2. Bulk allocation that fits within initial capacity - chunk doesn't grow, `start_ptr` unchanged
        let alloc_size1 = 8 * 1024;
        arena.alloc_layout(Layout::from_size_align(alloc_size1, 1).unwrap());
        consumed += alloc_size1;
        assert_eq!(arena.start_ptr.get().addr().get(), initial_start_addr);
        assert_eq!(data_capacity(&arena), initial_capacity);
        assert_eq!(available(&arena), initial_capacity - consumed);

        // 3. Bulk allocation that overflows current chunk - chunk grows in place, committed region doubles.
        // `start_ptr` moves down by `FIRST_ALLOCATION_GOAL` (the previous committed size). If a new
        // chunk had been allocated instead of growing in place, `start_ptr` would be at an unrelated address.
        let alloc_size2 = 12 * 1024;
        arena.alloc_layout(Layout::from_size_align(alloc_size2, 1).unwrap());
        consumed += alloc_size2;
        let capacity_after_grow1 = 2 * FIRST_ALLOCATION_GOAL - OVERHEAD;
        assert_eq!(arena.start_ptr.get().addr().get(), initial_start_addr - FIRST_ALLOCATION_GOAL);
        assert_eq!(data_capacity(&arena), capacity_after_grow1);
        assert_eq!(available(&arena), capacity_after_grow1 - consumed);

        // Verify writes still work after the grow, and the earlier `byte_a` was not corrupted
        let byte_b = arena.alloc(0xBBu8);
        assert_eq!(*byte_b, 0xBB);
        assert_eq!(*byte_a, 0xAA);
        consumed += 1;

        // 4. Another overflow - chunk grows in place again. `start_ptr` moves down by another
        // `2 * FIRST_ALLOCATION_GOAL` (committed size before this grow), so `3 * FIRST_ALLOCATION_GOAL`
        // total below the initial position.
        let alloc_size3 = 20 * 1024;
        arena.alloc_layout(Layout::from_size_align(alloc_size3, 1).unwrap());
        consumed += alloc_size3;
        let capacity_after_grow2 = 4 * FIRST_ALLOCATION_GOAL - OVERHEAD;
        assert_eq!(
            arena.start_ptr.get().addr().get(),
            initial_start_addr - 3 * FIRST_ALLOCATION_GOAL
        );
        assert_eq!(data_capacity(&arena), capacity_after_grow2);
        assert_eq!(available(&arena), capacity_after_grow2 - consumed);

        // Both single-byte allocations still intact after the second grow
        assert_eq!(*byte_a, 0xAA);
        assert_eq!(*byte_b, 0xBB);

        // 5. Dropping releases the reservation via `Mmap::free` and doesn't panic
        drop(arena);
    }

    /// Verify the doubling-growth strategy is bounded at `CONTAINER_SIZE`.
    ///
    /// We push the committed region just past `CONTAINER_SIZE / 2`, then trigger another grow.
    /// The unclamped doubling target would exceed `CONTAINER_SIZE`, so the `max(_, container_start_ptr)`
    /// clamp in `grow_fixed_size_chunk` must kick in. The grow should still succeed, with committed
    /// pinned at exactly `CONTAINER_SIZE` (the whole Container).
    #[test]
    fn grow_capped_at_container_size() {
        let arena: Arena = Arena::new_fixed_size().unwrap();
        let initial_start_addr = arena.start_ptr.get().addr().get();
        let container_end_addr = initial_start_addr + FIRST_ALLOCATION_GOAL;

        // Step 1: A large allocation that pushes committed region just past `CONTAINER_SIZE / 2`.
        // Allocating `CONTAINER_SIZE / 2` bytes lands `new_ptr` 64 bytes inside a page
        // (the data region ends 64 bytes before the Container end, and `64 % PAGE_SIZE != 0`),
        // so the page-rounding pulls `needed_start` down by an extra page.
        // Result: committed = `CONTAINER_SIZE / 2 + PAGE_SIZE`.
        let alloc_size1 = CONTAINER_SIZE / 2;
        arena.alloc_layout(Layout::from_size_align(alloc_size1, 1).unwrap());
        let committed_after_1 = CONTAINER_SIZE / 2 + PAGE_SIZE;
        assert_eq!(arena.start_ptr.get().addr().get(), container_end_addr - committed_after_1);
        assert_eq!(data_capacity(&arena), committed_after_1 - OVERHEAD);
        assert_eq!(available(&arena), committed_after_1 - OVERHEAD - alloc_size1);
        // Confirm we're past the half-way point so doubling would overshoot
        assert!(committed_after_1 * 2 > CONTAINER_SIZE);

        // Step 2: A small allocation that triggers a further grow.
        // Doubling target would be `2 * (CONTAINER_SIZE / 2 + PAGE_SIZE)`, which would exceed `CONTAINER_SIZE`.
        // Check that committed region is clamped to `CONTAINER_SIZE` maximum.
        let alloc_size2 = 100 * 1024; // 100 KiB - much less than the remaining ~1 GiB
        arena.alloc_layout(Layout::from_size_align(alloc_size2, 1).unwrap());
        assert_eq!(arena.start_ptr.get().addr().get(), container_end_addr - CONTAINER_SIZE);
        assert_eq!(data_capacity(&arena), CONTAINER_SIZE - OVERHEAD);
        assert_eq!(available(&arena), CONTAINER_SIZE - OVERHEAD - alloc_size1 - alloc_size2);
    }

    #[test]
    fn alloc_max_capacity_succeeds() {
        let arena: Arena = Arena::new_fixed_size().unwrap();
        arena.alloc_layout(Layout::from_size_align(MAX_ALLOCATION, 1).unwrap());
    }

    #[test]
    #[should_panic(expected = "out of memory")]
    fn alloc_one_byte_beyond_max_capacity_panics() {
        let arena: Arena = Arena::new_fixed_size().unwrap();
        arena.alloc_layout(Layout::from_size_align(MAX_ALLOCATION + 1, 1).unwrap());
    }

    #[test]
    #[should_panic(expected = "out of memory")]
    fn alloc_isize_max_panics() {
        let arena: Arena = Arena::new_fixed_size().unwrap();
        arena.alloc_layout(Layout::from_size_align(isize::MAX as usize, 1).unwrap());
    }

    /// Boundary cases for `grow_fixed_size_chunk`'s page rounding.
    ///
    /// Both sub-cases trigger a grow where `needed_start` wins over `doubling_start`
    /// (so the rounding logic is what determines the committed region), and verify that:
    /// * Case A: `new_ptr` lands exactly on a page boundary - `round_down` is a no-op.
    /// * Case B: `new_ptr` lands 1 byte below a page boundary - `round_down` pulls back by
    ///   `PAGE_SIZE - 1` (max possible rounding).
    ///
    /// Both should produce the same committed region size, exercising `round_down` at its boundaries.
    #[test]
    fn grow_page_rounding_boundaries() {
        // Case A: `new_ptr.addr()` is page-aligned, so `round_down` is identity.
        // For this, `alloc_size = (k+1) * PAGE_SIZE - OVERHEAD`
        // (cursor starts `OVERHEAD` bytes below the container's top page boundary).
        // Smallest `k` with `alloc_size > 2 * FIRST_ALLOCATION_GOAL - OVERHEAD = 32704` (the doubling threshold)
        // is `k = 8`.
        {
            let arena: Arena = Arena::new_fixed_size().unwrap();
            let alloc_size = 9 * PAGE_SIZE - OVERHEAD;
            arena.alloc_layout(Layout::from_size_align(alloc_size, 1).unwrap());
            // Committed = 9 * PAGE_SIZE. Allocation exactly fills the chunk
            assert_eq!(data_capacity(&arena), 9 * PAGE_SIZE - OVERHEAD);
            assert_eq!(available(&arena), 0);
        }

        // Case B: `new_ptr.addr()` is 1 byte below a page boundary - max rounding pulls committed
        // up by another full page.
        {
            let arena: Arena = Arena::new_fixed_size().unwrap();
            // 1 byte more than an alloc_size that would give an 8-page commit
            let alloc_size = 8 * PAGE_SIZE - OVERHEAD + 1;
            arena.alloc_layout(Layout::from_size_align(alloc_size, 1).unwrap());
            // Same committed size as Case A: rounding stretched it to 9 pages
            assert_eq!(data_capacity(&arena), 9 * PAGE_SIZE - OVERHEAD);
            // `PAGE_SIZE - 1` bytes left over (the rounding distance)
            assert_eq!(available(&arena), PAGE_SIZE - 1);
        }
    }

    /// Verify that `Arena::reset` keeps the (possibly-grown) committed region intact.
    ///
    /// After reset, the cursor returns to the top of the chunk but the committed region is unchanged,
    /// so subsequent allocations reuse the previously-committed memory without triggering another grow.
    #[test]
    fn reset_preserves_grown_chunk() {
        let mut arena: Arena = Arena::new_fixed_size().unwrap();
        let initial_start_addr = arena.start_ptr.get().addr().get();

        // Trigger a grow by allocating just over initial capacity
        arena.alloc_layout(Layout::from_size_align(FIRST_ALLOCATION_GOAL, 1).unwrap());
        let capacity_after_grow = 2 * FIRST_ALLOCATION_GOAL - OVERHEAD;
        let start_after_grow = initial_start_addr - FIRST_ALLOCATION_GOAL;
        assert_eq!(arena.start_ptr.get().addr().get(), start_after_grow);
        assert_eq!(data_capacity(&arena), capacity_after_grow);
        assert_eq!(available(&arena), capacity_after_grow - FIRST_ALLOCATION_GOAL);

        // Reset: cursor returns to the top, but the grown chunk persists
        arena.reset();
        assert_eq!(arena.start_ptr.get().addr().get(), start_after_grow);
        assert_eq!(data_capacity(&arena), capacity_after_grow);
        assert_eq!(available(&arena), capacity_after_grow);

        // Sanity-check writes work after reset
        let v = arena.alloc(0xCDu8);
        assert_eq!(*v, 0xCD);

        // Allocate the rest of the (already-grown) chunk - should not trigger a new grow
        let remaining = available(&arena);
        arena.alloc_layout(Layout::from_size_align(remaining, 1).unwrap());
        assert_eq!(arena.start_ptr.get().addr().get(), start_after_grow);
        assert_eq!(data_capacity(&arena), capacity_after_grow);
        assert_eq!(available(&arena), 0);
    }
}
