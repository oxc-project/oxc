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
//!   "Deallocated" chunks go onto a global exact-size free-list ([`FREE_CHUNKS`]) and are
//!   reused by later allocations of the same size (plus `madvise(MADV_FREE)` for RSS
//!   hygiene while they sit unused). The address range is never unmapped, and chunks are
//!   never split or coalesced, so virtual address use only grows to the high-water mark of
//!   live + same-size-recyclable chunks. A production version needs real VA reuse.
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
    cell::UnsafeCell,
    ptr::NonNull,
    sync::{
        Once,
        atomic::{AtomicBool, AtomicPtr, AtomicUsize, Ordering},
    },
};

use rustc_hash::{FxBuildHasher, FxHashMap};

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

/// Size actually reserved from the OS, which the bump cursor is capped at.
///
/// Equals [`CAGE_SIZE`] in normal builds. Under Miri — which cannot `mmap` — the reservation
/// is a bounded heap allocation instead, so the cursor stops well before 32 GiB. Handed-out
/// offsets stay `< CAGE_SIZE`, so the "offset fits in `u32`" compression invariant still holds.
#[cfg(not(miri))]
const RESERVED_SIZE: usize = CAGE_SIZE;
#[cfg(miri)]
const RESERVED_SIZE: usize = 128 * 1024 * 1024;

/// Number of bytes burned at the start of the cage, so that no allocation ever has offset 0.
/// Must be `>= 1 << COMPRESSED_SCALE_SHIFT` (so scaled offsets are non-zero)
/// and a multiple of `CHUNK_ALIGN`.
const BURNED_PREFIX: usize = 64;

const _: () = {
    assert!(BURNED_PREFIX >= 1 << COMPRESSED_SCALE_SHIFT);
    assert!(BURNED_PREFIX.is_multiple_of(CHUNK_ALIGN));
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

/// Free-list of "deallocated" chunks, keyed by exact chunk size (in bytes).
/// Values are cage-relative byte offsets of the chunk starts.
///
/// Without this, workloads which create and drop many `Allocator`s (e.g. conformance runs
/// parsing 100k+ files) burn through the whole 32 GiB reservation. Chunk sizes follow the
/// arena's doubling ladder, so exact-size matches recycle nearly everything.
///
/// This is deliberately dumb (no splitting, no coalescing): chunks are only ever reused for
/// an allocation of *exactly* the same size, so alignment (>= `CHUNK_ALIGN`) is preserved.
/// A production allocator would want something better.
///
/// Guarded by a spin lock, NOT `std::sync::Mutex`: on macOS, `Mutex` lazily boxes a pthread
/// mutex on first lock, and the cage's *allocation* path must never allocate from the global
/// heap (arena allocation is documented to work even when the global allocator fails).
/// Critical sections are a few instructions, so spinning is fine.
static FREE_CHUNKS: SpinLock<FxHashMap<usize, Vec<usize>>> =
    SpinLock::new(FxHashMap::with_hasher(FxBuildHasher));

/// A minimal allocation-free spin lock.
struct SpinLock<T> {
    locked: AtomicBool,
    value: UnsafeCell<T>,
}

// SAFETY: `SpinLock` provides mutual exclusion, so `&SpinLock<T>` can be shared across
// threads if `T` can be sent between them.
unsafe impl<T: Send> Sync for SpinLock<T> {}

impl<T> SpinLock<T> {
    const fn new(value: T) -> Self {
        Self { locked: AtomicBool::new(false), value: UnsafeCell::new(value) }
    }

    /// Run `f` with exclusive access to the value.
    ///
    /// If `f` panics, the lock is left held (all callers pass non-panicking closures).
    fn with<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        while self
            .locked
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            std::hint::spin_loop();
        }
        // SAFETY: The `locked` flag guarantees exclusive access.
        let result = f(unsafe { &mut *self.value.get() });
        self.locked.store(false, Ordering::Release);
        result
    }
}

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
pub fn cage_base_ptr() -> *mut u8 {
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

/// Reserve the cage as a bounded heap allocation under Miri, which cannot `mmap`.
///
/// `RESERVED_SIZE` (not `CAGE_SIZE`) bytes are allocated, and the base pointer carries
/// provenance over that whole region, so reconstructing chunk pointers via
/// `cage_base_ptr().add(offset)` stays valid under `-Zmiri-strict-provenance`.
#[cfg(miri)]
fn reserve_cage() -> NonNull<u8> {
    use std::alloc::{GlobalAlloc, System};
    let layout = Layout::from_size_align(RESERVED_SIZE, CHUNK_ALIGN).unwrap();
    // SAFETY: `layout` has non-zero size.
    let ptr = unsafe { System.alloc(layout) };
    NonNull::new(ptr).expect("pointer-compression cage: failed to reserve heap cage under Miri")
}

/// Reserve `CAGE_SIZE` bytes of lazily-committed virtual address space.
#[cfg(all(unix, not(miri)))]
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
#[cfg(all(not(unix), not(miri)))]
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
/// The returned memory is uninitialized (fresh anonymous pages read as zero; recycled
/// chunks may hold old data), read-write, and valid until "deallocated" via
/// [`dealloc_chunk`]. The address is always `>= cage_base() + 64`, so scaled offsets
/// into the chunk are always non-zero.
pub fn alloc_chunk(layout: Layout) -> Option<NonNull<u8>> {
    ensure_cage_init();

    let size = layout.size();
    let align = layout.align().max(CHUNK_ALIGN);

    // Try to reuse a previously-freed chunk of exactly this size.
    // Freed chunk offsets are always aligned to at least `CHUNK_ALIGN`; only reuse when
    // that's sufficient for the requested alignment.
    if align <= CHUNK_ALIGN {
        let offset = FREE_CHUNKS.with(|free_chunks| free_chunks.get_mut(&size).and_then(Vec::pop));
        if let Some(offset) = offset {
            let base = cage_base_ptr();
            debug_assert!(!base.is_null());
            debug_assert!(offset.is_multiple_of(CHUNK_ALIGN) && offset >= BURNED_PREFIX);
            // SAFETY: `offset` was previously handed out by `alloc_chunk` for a chunk of
            // `size` bytes, so `base + offset .. base + offset + size` is within the cage.
            // The chunk was freed (via `dealloc_chunk`), so nothing else references it.
            return Some(unsafe { NonNull::new_unchecked(base.add(offset)) });
        }
    }

    // Bump `CAGE_CURSOR` atomically. Relaxed ordering is sufficient: the cursor is just
    // an offset allotter; the memory itself is already mapped, and any cross-thread handoff
    // of the allocated memory synchronizes through other means.
    let result = CAGE_CURSOR.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |cursor| {
        let start = cursor.checked_next_multiple_of(align)?;
        let end = start.checked_add(size)?;
        (end <= RESERVED_SIZE).then_some(end)
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
         The prototype cage only reuses freed chunks of identical size - workloads \
         with highly varied allocator sizes can exhaust it.",
        CAGE_CURSOR.load(Ordering::Relaxed)
    );
}

/// "Deallocate" a chunk previously returned by [`alloc_chunk`].
///
/// The chunk is pushed onto [`FREE_CHUNKS`] for reuse by a future allocation of exactly the
/// same size. The virtual address range is never unmapped. To keep resident memory (RSS) in
/// check while a chunk sits on the free-list, the page-aligned interior of the chunk is passed
/// to `madvise(MADV_FREE)`, which lets the OS reclaim the physical pages lazily
/// (they read back as zeros, or the old contents - either is fine for uninitialized
/// chunk memory).
pub fn dealloc_chunk(ptr: NonNull<u8>, layout: Layout) {
    // Miri can't `madvise`, and the heap-backed Miri cage doesn't need RSS trimming anyway.
    #[cfg(all(unix, not(miri)))]
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

    let offset = ptr.addr().get().wrapping_sub(cage_base());
    debug_assert!(offset >= BURNED_PREFIX && offset + layout.size() <= CAGE_SIZE);
    // Free-list bookkeeping lives on the global heap. Deallocation must never fail or panic,
    // so under global-allocator memory pressure, fall back to leaking the chunk
    // (the cage's address range stays mapped either way).
    FREE_CHUNKS.with(|free_chunks| {
        if free_chunks.try_reserve(1).is_err() {
            return;
        }
        let list = free_chunks.entry(layout.size()).or_default();
        if list.try_reserve(1).is_ok() {
            list.push(offset);
        }
    });
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
        // SAFETY: `p1` points to a fresh 1024-byte chunk we own
        unsafe {
            p1.write_bytes(0xAB, 1024);
            assert_eq!(p1.read(), 0xAB);
        }

        // Dealloc pushes the chunk to the free-list; an allocation of the same size reuses it.
        // (Other tests in this process also hit the free-list, so accept any in-cage result.)
        dealloc_chunk(p1, layout);
        let p3 = alloc_chunk(layout).unwrap();
        assert!(p3.addr().get() >= base + BURNED_PREFIX);
        assert!(p3.addr().get() + 1024 <= base + CAGE_SIZE);
    }
}
