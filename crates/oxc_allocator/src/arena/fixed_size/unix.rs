use std::{
    alloc::{GlobalAlloc, Layout, System},
    cmp::{max, min},
    io,
    ptr::NonNull,
};

#[cfg(miri)]
use std::alloc::{alloc, dealloc};
#[cfg(not(miri))]
use std::ptr;

use crate::generated::fixed_size_constants::{BLOCK_ALIGN, BLOCK_SIZE};

use super::super::{
    Arena, CHUNK_ALIGN, CHUNK_FOOTER_SIZE, ChunkFooter,
    create::FIRST_ALLOCATION_GOAL,
    utils::{is_pointer_aligned_to, round_down_to, round_mut_ptr_down_to},
};

// A 2 GiB container, aligned on 4 GiB, fits inside a 6 GiB reservation regardless of
// where the OS places the mapping. The trailing 16 bytes are inside the mapping but outside
// the fixed-size block; this keeps the ChunkFooter at the cross-platform offset.
const CONTAINER_SIZE: usize = BLOCK_SIZE + CHUNK_ALIGN;
const RESERVED_SIZE: usize = CONTAINER_SIZE + BLOCK_ALIGN;
const RESERVED_LAYOUT: Layout = match Layout::from_size_align(RESERVED_SIZE, 4096) {
    Ok(layout) => layout,
    Err(_) => unreachable!(),
};

const _: () = {
    assert!(BLOCK_ALIGN.is_power_of_two());
    assert!(CONTAINER_SIZE == 1 << 31);
    assert!(RESERVED_LAYOUT.size() > 0);
    assert!(FIRST_ALLOCATION_GOAL.is_multiple_of(CHUNK_ALIGN));
};

/// Return the OS page size, which can be larger than 4 KiB (for example on arm64).
fn page_size() -> Option<usize> {
    #[cfg(miri)]
    let size = 4096;
    #[cfg(not(miri))]
    let size = usize::try_from(unsafe { libc::sysconf(libc::_SC_PAGESIZE) }).ok()?;

    (size >= CHUNK_ALIGN && size.is_power_of_two() && CONTAINER_SIZE.is_multiple_of(size))
        .then_some(size)
}

impl<const MIN_ALIGN: usize> Arena<MIN_ALIGN> {
    /// Construct a fixed-size arena backed by a protected virtual-memory reservation.
    ///
    /// Only the last pages of the container are writable initially. As allocations need more
    /// space, `grow_fixed_size_chunk` makes preceding pages writable within the same reservation.
    /// This avoids charging the entire multi-gigabyte region against strict overcommit limits.
    ///
    /// # Panics
    /// Panics if releasing the reservation after a failed protection change fails.
    pub fn new_fixed_size() -> Option<Self> {
        let page_size = page_size()?;
        let initial_size = max(FIRST_ALLOCATION_GOAL, page_size);
        let reservation_ptr = Mmap::reserve(RESERVED_SIZE).ok()?;

        // The reservation has room for a full container after the next 4 GiB boundary.
        // Pointer arithmetic stays within the original mapping, retaining its provenance.
        let offset = reservation_ptr.addr().get().wrapping_neg() % BLOCK_ALIGN;
        // SAFETY: `offset < BLOCK_ALIGN` and `RESERVED_SIZE = CONTAINER_SIZE + BLOCK_ALIGN`.
        let container_ptr = unsafe { reservation_ptr.add(offset) };
        debug_assert!(is_pointer_aligned_to(container_ptr, BLOCK_ALIGN));

        let initial_start_offset = CONTAINER_SIZE - initial_size;
        // SAFETY: The offset and initial region lie within the reserved container.
        let initial_ptr = unsafe { container_ptr.add(initial_start_offset) };
        debug_assert!(is_pointer_aligned_to(initial_ptr, page_size));

        // SAFETY: `initial_ptr` and `initial_size` are page-aligned, non-zero, and inside the mapping.
        if unsafe { Mmap::protect(initial_ptr, initial_size) }.is_err() {
            // SAFETY: `reservation_ptr` is the start of the mapping just created above.
            unsafe { Mmap::free(reservation_ptr, RESERVED_SIZE) }
                .unwrap_or_else(|err| panic!("`munmap` failed during cleanup: {err}"));
            return None;
        }

        let initial_chunk_size = BLOCK_SIZE - initial_start_offset;
        debug_assert!(initial_chunk_size >= CHUNK_FOOTER_SIZE);
        debug_assert!(initial_chunk_size.is_multiple_of(CHUNK_ALIGN));

        // SAFETY: The initial chunk and its footer lie in the writable part of the reservation.
        // The private constructor records mmap ownership before returning the arena.
        let arena = unsafe {
            Self::from_mapped_parts(
                initial_ptr,
                initial_chunk_size,
                reservation_ptr,
                RESERVED_LAYOUT,
            )
        };

        Some(arena)
    }

    /// Make more pages writable within the existing reservation to fit `layout`.
    /// Returns the allocation pointer and updates `start_ptr` on success.
    /// The caller updates `cursor_ptr`.
    ///
    /// # SAFETY
    ///
    /// * `Arena` must have a fixed-size chunk.
    /// * The current chunk must not fit `layout` without growing.
    pub(in super::super) unsafe fn grow_fixed_size_chunk(
        &self,
        layout: Layout,
    ) -> Option<NonNull<u8>> {
        let footer_ptr = self.current_chunk_footer_ptr.get()?;
        // SAFETY: The arena's current footer is valid for its lifetime.
        let footer = unsafe { footer_ptr.as_ref() };
        debug_assert!(footer.is_fixed_size);
        if !footer.is_mapped {
            return None;
        }

        let chunk_start_ptr = self.start_ptr.get();
        let chunk_start_addr = chunk_start_ptr.addr().get();
        let cursor_ptr = self.cursor_ptr.get().as_ptr();
        let page_size = page_size()?;
        debug_assert!(is_pointer_aligned_to(chunk_start_ptr, page_size));

        // The footer remains at the fixed offset even as `start_ptr` moves downward.
        let container_start_addr = round_down_to(footer_ptr.addr().get(), BLOCK_ALIGN);
        let container_end_addr = container_start_addr + CONTAINER_SIZE;

        // Match the fast path's downward bump and alignment calculation.
        let new_ptr = cursor_ptr.wrapping_sub(layout.size());
        let new_ptr = round_mut_ptr_down_to(new_ptr, max(layout.align(), MIN_ALIGN));
        if new_ptr.addr().wrapping_sub(container_start_addr) > isize::MAX as usize {
            return None;
        }
        // SAFETY: The bounds check keeps `new_ptr` within the non-null container.
        let new_ptr = unsafe { NonNull::new_unchecked(new_ptr) };

        let committed_size = container_end_addr - chunk_start_addr;
        let doubled_start_addr = max(chunk_start_addr - committed_size, container_start_addr);
        let needed_start_addr = round_down_to(new_ptr.addr().get(), page_size);
        let new_start_addr = min(doubled_start_addr, needed_start_addr);
        debug_assert!(new_start_addr < chunk_start_addr);
        debug_assert!(new_start_addr >= container_start_addr);

        let delta = chunk_start_addr - new_start_addr;
        // SAFETY: `new_start_addr` is within the original reservation and below `chunk_start_ptr`.
        let new_start_ptr = unsafe { chunk_start_ptr.sub(delta) };
        // SAFETY: Both bounds are page-aligned and the range lies inside the mapping.
        unsafe { Mmap::protect(new_start_ptr, delta) }.ok()?;
        self.start_ptr.set(new_start_ptr);

        Some(new_ptr)
    }
}

/// Free a fixed-size chunk through the allocator that owns its backing memory.
///
/// # SAFETY
/// `footer_ptr` must point to a fixed-size chunk's valid footer.
pub unsafe fn dealloc_fixed_size_arena_chunk(footer_ptr: NonNull<ChunkFooter>) {
    let (backing_alloc_ptr, layout, is_fixed_size, is_mapped) = {
        // SAFETY: The caller provides a valid footer.
        let footer = unsafe { footer_ptr.as_ref() };
        (footer.backing_alloc_ptr, footer.layout, footer.is_fixed_size, footer.is_mapped)
    };
    debug_assert!(is_fixed_size);

    if is_mapped {
        // SAFETY: `new_fixed_size` created this mapping with the recorded pointer and size.
        unsafe { Mmap::free(backing_alloc_ptr, layout.size()) }
            .unwrap_or_else(|err| panic!("`munmap` failed: {err}"));
    } else {
        // SAFETY: `from_raw_parts` requires the imported backing memory to use `System`.
        unsafe { System.dealloc(backing_alloc_ptr.as_ptr(), layout) };
    }
}

/// OS operations on one reservation. Protection changes keep the original mapping intact.
struct Mmap;

#[cfg(not(miri))]
impl Mmap {
    fn reserve(size: usize) -> io::Result<NonNull<u8>> {
        // SAFETY: Anonymous private mapping with no accessible pages requires no input pointer.
        let ptr = unsafe {
            libc::mmap(
                ptr::null_mut(),
                size,
                libc::PROT_NONE,
                libc::MAP_PRIVATE | libc::MAP_ANON,
                -1,
                0,
            )
        };
        if ptr == libc::MAP_FAILED {
            Err(io::Error::last_os_error())
        } else {
            NonNull::new(ptr.cast::<u8>()).ok_or_else(|| {
                // A mapping at address zero cannot be represented by `NonNull`.
                // SAFETY: `mmap` succeeded and returned this mapping with `size` bytes.
                unsafe { libc::munmap(ptr, size) };
                io::Error::other("mmap returned a null address")
            })
        }
    }

    /// # SAFETY
    /// `ptr..ptr + size` must be page-aligned and within a live mapping made by `reserve`.
    unsafe fn protect(ptr: NonNull<u8>, size: usize) -> io::Result<()> {
        // SAFETY: The caller guarantees page alignment and mapping bounds.
        let result = unsafe {
            libc::mprotect(ptr.as_ptr().cast(), size, libc::PROT_READ | libc::PROT_WRITE)
        };
        if result == 0 { Ok(()) } else { Err(io::Error::last_os_error()) }
    }

    /// # SAFETY
    /// `ptr` must be the start of a live mapping made by `reserve` with the same `size`.
    unsafe fn free(ptr: NonNull<u8>, size: usize) -> io::Result<()> {
        // SAFETY: The caller guarantees the pointer and size identify the original mapping.
        let result = unsafe { libc::munmap(ptr.as_ptr().cast(), size) };
        if result == 0 { Ok(()) } else { Err(io::Error::last_os_error()) }
    }
}

#[cfg(test)]
mod tests {
    use std::alloc::{GlobalAlloc, Layout, System};

    use super::*;

    #[test]
    fn reserve_and_grow_in_place() {
        let arena = Arena::<1>::new_fixed_size().unwrap();
        let initial_start = arena.start_ptr.get();
        let footer = arena.data_end_ptr();
        let container_start = footer.addr().get() - (BLOCK_SIZE - CHUNK_FOOTER_SIZE);
        assert!(container_start.is_multiple_of(BLOCK_ALIGN));

        let first = arena.alloc(7_u8);
        let initial_size = max(FIRST_ALLOCATION_GOAL, page_size().unwrap());
        arena.alloc_layout(Layout::from_size_align(initial_size, 1).unwrap());
        assert!(arena.start_ptr.get() < initial_start);
        assert_eq!(arena.data_end_ptr(), footer);
        assert_eq!(*first, 7);

        let second = arena.alloc(9_u8);
        assert_eq!(*second, 9);
    }

    #[test]
    fn page_boundary_growth_and_reset() {
        let page = page_size().unwrap();
        let initial_size = max(FIRST_ALLOCATION_GOAL, page);
        let overhead = CONTAINER_SIZE - BLOCK_SIZE + CHUNK_FOOTER_SIZE;

        // End the allocation exactly on a page boundary below the doubling target.
        let mut arena = Arena::<1>::new_fixed_size().unwrap();
        let container_end = arena.start_ptr.get().addr().get() + initial_size;
        let size = 3 * initial_size - overhead;
        let ptr = arena.try_alloc_layout(Layout::from_size_align(size, 1).unwrap()).unwrap();
        assert_eq!(ptr.addr().get(), container_end - 3 * initial_size);
        assert_eq!(arena.start_ptr.get().addr().get(), ptr.addr().get());
        // SAFETY: Both writes are within the newly allocated, writable range.
        unsafe {
            ptr.as_ptr().write(1);
            ptr.as_ptr().add(size - 1).write(2);
        }

        let grown_start = arena.start_ptr.get();
        arena.reset();
        assert_eq!(arena.start_ptr.get(), grown_start);
        assert_eq!(*arena.alloc(5_u8), 5);
        drop(arena);

        // One byte further down needs one more page of protection.
        let arena = Arena::<1>::new_fixed_size().unwrap();
        let container_end = arena.start_ptr.get().addr().get() + initial_size;
        let size = 3 * initial_size - overhead + 1;
        let ptr = arena.try_alloc_layout(Layout::from_size_align(size, 1).unwrap()).unwrap();
        assert_eq!(ptr.addr().get(), container_end - 3 * initial_size - 1);
        assert_eq!(arena.start_ptr.get().addr().get(), container_end - 3 * initial_size - page);
    }

    #[test]
    fn aligned_allocation_and_out_of_bounds_rejection() {
        let arena = Arena::<1>::new_fixed_size().unwrap();
        let layout = Layout::from_size_align(64 * 1024, 64 * 1024).unwrap();
        let ptr = arena.try_alloc_layout(layout).unwrap();
        assert!(ptr.addr().get().is_multiple_of(layout.align()));

        let start = arena.start_ptr.get();
        let cursor = arena.cursor_ptr.get();
        let largest = BLOCK_SIZE - CHUNK_FOOTER_SIZE;
        assert!(arena.try_alloc_layout(Layout::from_size_align(largest + 1, 1).unwrap()).is_err());
        assert!(
            arena
                .try_alloc_layout(Layout::from_size_align(isize::MAX as usize, 1).unwrap())
                .is_err()
        );
        assert_eq!(arena.start_ptr.get(), start);
        assert_eq!(arena.cursor_ptr.get(), cursor);
    }

    #[test]
    fn imported_system_chunk_does_not_grow_or_unmap() {
        let layout = Layout::from_size_align(CHUNK_FOOTER_SIZE + 64, CHUNK_ALIGN).unwrap();
        // SAFETY: The layout has non-zero size.
        let ptr = NonNull::new(unsafe { System.alloc(layout) }).unwrap();
        // SAFETY: The entire chunk is inside this System allocation.
        let arena = unsafe { Arena::<1>::from_raw_parts(ptr, layout.size(), ptr, layout) };

        let capacity = arena.chunk_capacity();
        assert!(arena.try_alloc_layout(Layout::from_size_align(capacity, 1).unwrap()).is_ok());
        assert!(arena.try_alloc_layout(Layout::from_size_align(1, 1).unwrap()).is_err());
        drop(arena);
    }
}

// Miri models the reservation as one Rust allocation. Its memory model does not support
// `mmap` or page protection, but this preserves provenance checks on all arena pointers.
#[cfg(miri)]
impl Mmap {
    fn reserve(_size: usize) -> io::Result<NonNull<u8>> {
        // SAFETY: `RESERVED_LAYOUT` has non-zero size and valid alignment.
        NonNull::new(unsafe { alloc(RESERVED_LAYOUT) })
            .ok_or_else(|| io::Error::other("failed to reserve memory"))
    }

    unsafe fn protect(_ptr: NonNull<u8>, _size: usize) -> io::Result<()> {
        Ok(())
    }

    unsafe fn free(ptr: NonNull<u8>, _size: usize) -> io::Result<()> {
        // SAFETY: `ptr` came from `reserve` with `RESERVED_LAYOUT`.
        unsafe { dealloc(ptr.as_ptr(), RESERVED_LAYOUT) };
        Ok(())
    }
}
