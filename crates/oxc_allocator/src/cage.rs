//! PROTOTYPE: Process-global virtual-address "cage" for pointer compression.
//!
//! All arena chunk memory is carved out of a single reserved virtual-address range
//! (the "cage"). Because every arena allocation lives inside this one range,
//! a pointer to an arena object can be represented as a 32-bit scaled offset from
//! the cage base instead of a full 64-bit pointer. See [`crate::Box`].
//!
//! # Design
//!
//! * One-time reservation of [`CAGE_SIZE`] (32 GiB) of lazily-committed anonymous memory
//!   (`mmap` with `MAP_NORESERVE` on unix). Pages are committed on first touch, so the
//!   reservation itself consumes (almost) no physical memory.
//! * [`CAGE_BASE`] holds the base pointer (null until initialized).
//! * [`CAGE_CURSOR`] is a monotonic bump offset for handing out chunks.
//!   Chunk memory is *never* returned to the cage - deallocation is a no-op
//!   (plus `madvise(MADV_FREE)` for RSS hygiene). Virtual address space is burned,
//!   but allocator reuse (reset / pool) keeps chunks alive as today, so the burn is
//!   bounded in practice. This is a prototype limitation - a production version needs
//!   a free-list of cage regions.
//! * The first 64 bytes of the cage are never handed out, so no allocation can sit at
//!   offset 0. This gives compressed pointers a `NonZeroU32` niche, making
//!   `Option<Box<T>>` 4 bytes.
//!
//! # Compression scheme
//!
//! Compressed pointer = `(address - cage_base) >> 3` stored as `NonZeroU32`.
//! Scale of 8 means the u32 offset range covers exactly `2^32 * 8` = 32 GiB = `CAGE_SIZE`.
//! All compressed allocations must therefore be 8-byte-aligned ([`crate::Box::new_in`]
//! over-aligns to 8).

use std::{
    alloc::Layout,
    ptr::NonNull,
    sync::{
        Once,
        atomic::{AtomicPtr, AtomicUsize, Ordering},
    },
};

use super::arena::CHUNK_ALIGN;

/// Size of the cage reservation: 32 GiB.
///
/// Must equal `2^32 << COMPRESSED_SCALE_SHIFT`, so that "offset fits in `u32`"
/// is exactly equivalent to "address is within the cage".
pub const CAGE_SIZE: usize = 32 * 1024 * 1024 * 1024;

/// Scale shift for compressed pointers. Compressed = `(addr - base) >> 3`.
pub const COMPRESSED_SCALE_SHIFT: u32 = 3;

const _: () = {
    assert!(CAGE_SIZE == (u32::MAX as usize + 1) << COMPRESSED_SCALE_SHIFT);
};

/// Number of bytes burned at the start of the cage, so that no allocation ever has offset 0.
/// Must be `>= 1 << COMPRESSED_SCALE_SHIFT` (so scaled offsets are non-zero)
/// and a multiple of `CHUNK_ALIGN`.
const BURNED_PREFIX: usize = 64;

const _: () = {
    assert!(BURNED_PREFIX >= 1 << COMPRESSED_SCALE_SHIFT);
    assert!(BURNED_PREFIX % CHUNK_ALIGN == 0);
};

/// Base pointer of the cage. Null until the cage is reserved.
///
/// `AtomicPtr` rather than `AtomicUsize` so that reconstructed pointers derive their
/// provenance from the original `mmap` pointer.
static CAGE_BASE: AtomicPtr<u8> = AtomicPtr::new(std::ptr::null_mut());

/// Bump cursor: offset (in bytes, relative to the cage base) of the next free byte.
static CAGE_CURSOR: AtomicUsize = AtomicUsize::new(BURNED_PREFIX);

/// One-time initialization guard for the cage reservation.
static CAGE_INIT: Once = Once::new();

/// Get the cage base address.
///
/// Returns 0 if the cage has not been initialized yet. Callers which hold a pointer into
/// the cage (e.g. a compressed `Box`) are guaranteed the cage is initialized, because the
/// memory their pointer points into came from the cage.
//
// `#[inline(always)]` because this is a single relaxed atomic load (a plain `mov`/`ldr`
// on x86-64 and aarch64), and it's on the hot path of every compressed-pointer deref.
#[expect(clippy::inline_always)]
#[inline(always)]
pub fn cage_base() -> usize {
    CAGE_BASE.load(Ordering::Relaxed).addr()
}

/// Get the cage base pointer.
///
/// Same as [`cage_base`], but returns a pointer (with the provenance of the whole cage),
/// for reconstructing pointers from compressed offsets.
//
// `#[inline(always)]` because this is a single relaxed atomic load.
#[expect(clippy::inline_always)]
#[inline(always)]
pub(crate) fn cage_base_ptr() -> *mut u8 {
    CAGE_BASE.load(Ordering::Relaxed)
}

/// Reserve the cage, if it hasn't been reserved already.
///
/// # Panics
/// Panics if the OS refuses the reservation.
#[inline]
fn ensure_cage_init() {
    CAGE_INIT.call_once(|| {
        let base = reserve_cage();
        CAGE_BASE.store(base.as_ptr(), Ordering::Release);
    });
}

/// Reserve `CAGE_SIZE` bytes of lazily-committed virtual address space.
#[cfg(unix)]
fn reserve_cage() -> NonNull<u8> {
    // MAP_NORESERVE: don't reserve swap for the whole range. Pages are committed lazily
    // on first touch. On macOS, anonymous mappings behave this way in any case.
    //
    // SAFETY: Requesting a fresh anonymous private mapping; no invariants to uphold.
    let ptr = unsafe {
        libc::mmap(
            std::ptr::null_mut(),
            CAGE_SIZE,
            libc::PROT_READ | libc::PROT_WRITE,
            libc::MAP_PRIVATE | libc::MAP_ANON | libc::MAP_NORESERVE,
            -1,
            0,
        )
    };
    assert!(
        ptr != libc::MAP_FAILED && !ptr.is_null(),
        "pointer-compression cage: failed to reserve {CAGE_SIZE} bytes of virtual address space"
    );
    let ptr = ptr.cast::<u8>();
    // `mmap` returns page-aligned memory; chunk alignment (16) is far below page size.
    debug_assert!(ptr.addr().is_multiple_of(CHUNK_ALIGN));
    // SAFETY: Checked non-null above
    unsafe { NonNull::new_unchecked(ptr) }
}

/// Fallback reservation for non-unix platforms (compiles, but will likely fail at runtime
/// for a reservation this large on Windows without `MEM_RESERVE` semantics).
///
/// PROTOTYPE: Only macOS is exercised. A production version would use `VirtualAlloc`
/// with `MEM_RESERVE` on Windows.
#[cfg(not(unix))]
fn reserve_cage() -> NonNull<u8> {
    use std::alloc::{GlobalAlloc, System};
    let layout = Layout::from_size_align(CAGE_SIZE, CHUNK_ALIGN).unwrap();
    // SAFETY: `layout` has non-zero size
    let ptr = unsafe { System.alloc(layout) };
    NonNull::new(ptr).expect("pointer-compression cage: failed to reserve address space")
}

/// Allocate a chunk of `layout.size()` bytes, aligned to `max(layout.align(), CHUNK_ALIGN)`,
/// out of the cage.
///
/// Returns `None` if the cage is exhausted (callers convert this to an OOM panic),
/// or if `layout` is degenerate.
///
/// The returned memory is zero-initialized (fresh anonymous pages), read-write,
/// and valid until the process exits. It is never reused after "deallocation"
/// ([`dealloc_chunk`] is a no-op). The address is always `>= cage_base() + 64`,
/// so scaled offsets into the chunk are always non-zero.
pub(crate) fn alloc_chunk(layout: Layout) -> Option<NonNull<u8>> {
    ensure_cage_init();

    let size = layout.size();
    let align = layout.align().max(CHUNK_ALIGN);

    // Bump `CAGE_CURSOR` atomically. Relaxed ordering is sufficient: the cursor is just
    // an offset allotter; the memory itself is already mapped, and any cross-thread handoff
    // of the allocated memory synchronizes through other means.
    let result = CAGE_CURSOR.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |cursor| {
        let start = cursor.checked_next_multiple_of(align)?;
        let end = start.checked_add(size)?;
        (end <= CAGE_SIZE).then_some(end)
    });

    match result {
        Ok(prev_cursor) => {
            // Recompute the aligned start that the closure computed for the successful update
            let start = prev_cursor.next_multiple_of(align);
            let base = cage_base_ptr();
            debug_assert!(!base.is_null());
            // SAFETY: `start + size <= CAGE_SIZE` (checked in `fetch_update` closure),
            // so `base + start` is within the cage reservation. `base` is non-null,
            // and `start >= BURNED_PREFIX > 0`, so `base + start` is non-null.
            unsafe { Some(NonNull::new_unchecked(base.add(start))) }
        }
        Err(_) => cage_exhausted(size),
    }
}

#[cold]
#[inline(never)]
fn cage_exhausted(size: usize) -> Option<NonNull<u8>> {
    panic!(
        "pointer-compression cage exhausted: cannot allocate {size} bytes \
         (cage size: {CAGE_SIZE} bytes, used: {} bytes). \
         The prototype cage never reuses chunk memory - long-running processes \
         which create and drop many allocators will exhaust it.",
        CAGE_CURSOR.load(Ordering::Relaxed)
    );
}

/// "Deallocate" a chunk previously returned by [`alloc_chunk`].
///
/// The virtual address range is leaked (never reused). To keep resident memory (RSS) in check,
/// the page-aligned interior of the chunk is passed to `madvise(MADV_FREE)`, which lets the OS
/// reclaim the physical pages lazily.
pub(crate) fn dealloc_chunk(ptr: NonNull<u8>, layout: Layout) {
    #[cfg(unix)]
    {
        // Round inward to page boundaries; `madvise` requires page-aligned addresses.
        const PAGE_SIZE: usize = 0x1000;
        let start = ptr.addr().get().next_multiple_of(PAGE_SIZE);
        let end = (ptr.addr().get() + layout.size()) & !(PAGE_SIZE - 1);
        if end > start {
            let offset = start - ptr.addr().get();
            // SAFETY: `[start, end)` is within the chunk, which is within the cage mapping.
            // `MADV_FREE` is advisory; the range remains mapped and accessible.
            unsafe {
                libc::madvise(ptr.as_ptr().add(offset).cast(), end - start, libc::MADV_FREE);
            }
        }
    }
    #[cfg(not(unix))]
    {
        let _ = (ptr, layout);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cage_alloc_basic() {
        let layout = Layout::from_size_align(1024, 16).unwrap();
        let p1 = alloc_chunk(layout).unwrap();
        let p2 = alloc_chunk(layout).unwrap();
        let base = cage_base();
        assert_ne!(base, 0);
        // Both chunks are inside the cage, beyond the burned prefix
        assert!(p1.addr().get() >= base + BURNED_PREFIX);
        assert!(p2.addr().get() + 1024 <= base + CAGE_SIZE);
        // Chunks don't overlap
        assert!(
            p2.addr().get() >= p1.addr().get() + 1024 || p1.addr().get() >= p2.addr().get() + 1024
        );
        // Aligned to chunk alignment
        assert!(p1.addr().get().is_multiple_of(CHUNK_ALIGN));

        // Memory is writable and readable
        unsafe {
            p1.write_bytes(0xAB, 1024);
            assert_eq!(p1.read(), 0xAB);
        }

        // Dealloc is a no-op (must not crash), memory stays mapped
        dealloc_chunk(p1, layout);
        unsafe {
            p1.write_bytes(0xCD, 1024);
            assert_eq!(p1.read(), 0xCD);
        }
    }
}
