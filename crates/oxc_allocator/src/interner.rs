//! String interner for arena-allocated identifier strings.
//!
//! Provides a 2-tier cache (L1 direct-mapped + L2 hash table) for deduplicating
//! strings in a dedicated append-only arena. Each interned string is stored with a
//! header containing a precomputed hash and length.
//!
//! # Arena layout
//!
//! ```text
//! [hash: u64][len: u32][_pad: u32][string bytes...]
//!                                  ↑ returned pointer
//! ```

use std::{
    alloc::Layout,
    hash::{BuildHasher, Hasher},
    mem::{align_of, size_of},
    ptr::NonNull,
};

use hashbrown::HashTable;
use rustc_hash::FxBuildHasher;

/// Header prepended to every interned string in the arena.
///
/// Layout: `[hash: u64][len: u32][_pad: u32]` — 16 bytes, aligned to 8.
#[repr(C)]
pub struct InternedStrHeader {
    /// Precomputed FxHash of the string bytes.
    pub hash: u64,
    /// Length of the string in bytes.
    pub len: u32,
    /// Padding for alignment (unused).
    _pad: u32,
}

impl InternedStrHeader {
    /// Create a new header with the given hash and length.
    pub const fn new(hash: u64, len: u32) -> Self {
        Self { hash, len, _pad: 0 }
    }
}

const _: () = assert!(size_of::<InternedStrHeader>() == 16);
const _: () = assert!(align_of::<InternedStrHeader>() == 8);

/// Size of [`InternedStrHeader`] in bytes.
pub const HEADER_SIZE: usize = size_of::<InternedStrHeader>();

/// Alignment for arena chunks — matches [`InternedStrHeader`] alignment.
const ARENA_ALIGN: usize = align_of::<InternedStrHeader>();

/// Initial capacity for the first arena chunk (4 KiB).
const ARENA_INITIAL_CAPACITY: usize = 4096;

/// Simple append-only arena for interned string storage.
///
/// Allocates 8-byte-aligned chunks via the global allocator.
/// When the current chunk is full, it is retired and a new chunk
/// of double the size is allocated. Retired chunks are never freed
/// until [`reset`] or [`Drop`], so all pointers remain stable.
///
/// [`reset`]: StringArena::reset
pub(crate) struct StringArena {
    /// Start of the current (active) chunk.
    ptr: NonNull<u8>,
    /// Bytes used in the current chunk.
    offset: usize,
    /// Total capacity of the current chunk.
    cap: usize,
    /// Retired chunks kept alive for pointer stability: `(pointer, capacity, used_bytes)`.
    prev: std::vec::Vec<(NonNull<u8>, usize, usize)>,
}

impl StringArena {
    pub(crate) const fn new() -> Self {
        Self { ptr: NonNull::dangling(), offset: 0, cap: 0, prev: std::vec::Vec::new() }
    }

    /// Allocate `size` bytes aligned to [`ARENA_ALIGN`].
    /// Returns a pointer to the start of the allocation.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub(crate) fn alloc(&mut self, size: usize) -> NonNull<u8> {
        let aligned = (self.offset + (ARENA_ALIGN - 1)) & !(ARENA_ALIGN - 1);
        if aligned + size <= self.cap {
            self.offset = aligned + size;
            // SAFETY: `aligned` is within `[0, self.cap)` and `self.ptr` points to
            // a valid allocation of at least `self.cap` bytes.
            unsafe { NonNull::new_unchecked(self.ptr.as_ptr().add(aligned)) }
        } else {
            self.grow(size)
        }
    }

    #[cold]
    #[inline(never)]
    fn grow(&mut self, size: usize) -> NonNull<u8> {
        // Retire current chunk if it has any capacity.
        if self.cap > 0 {
            self.prev.push((self.ptr, self.cap, self.offset));
        }

        // New chunk: at least double previous, at least ARENA_INITIAL_CAPACITY, at least `size`.
        let new_cap = (self.cap * 2).max(ARENA_INITIAL_CAPACITY).max(size);
        // SAFETY: `ARENA_ALIGN` is 8 (a valid power of 2) and `new_cap > 0`.
        let layout = unsafe { Layout::from_size_align_unchecked(new_cap, ARENA_ALIGN) };
        // SAFETY: `layout` has non-zero size.
        let ptr = unsafe { std::alloc::alloc(layout) };
        let Some(ptr) = NonNull::new(ptr) else {
            std::alloc::handle_alloc_error(layout);
        };

        self.ptr = ptr;
        self.cap = new_cap;
        self.offset = size;

        ptr
    }

    /// Reset the arena, freeing all retired chunks.
    /// Keeps the current (largest) chunk for reuse.
    pub(crate) fn reset(&mut self) {
        for &(ptr, cap, _) in &self.prev {
            // SAFETY: Each chunk was allocated with `Layout::from_size_align(cap, ARENA_ALIGN)`.
            unsafe {
                let layout = Layout::from_size_align_unchecked(cap, ARENA_ALIGN);
                std::alloc::dealloc(ptr.as_ptr(), layout);
            }
        }
        self.prev.clear();
        self.offset = 0;
    }

    /// Total capacity across all chunks (current + retired).
    pub(crate) fn capacity(&self) -> usize {
        self.prev.iter().map(|&(_, cap, _)| cap).sum::<usize>() + self.cap
    }

    /// Total bytes used across all chunks (current + retired).
    pub(crate) fn used_bytes(&self) -> usize {
        self.prev.iter().map(|&(_, _, used)| used).sum::<usize>() + self.offset
    }
}

impl Drop for StringArena {
    fn drop(&mut self) {
        for &(ptr, cap, _) in &self.prev {
            // SAFETY: Each chunk was allocated with `Layout::from_size_align(cap, ARENA_ALIGN)`.
            unsafe {
                let layout = Layout::from_size_align_unchecked(cap, ARENA_ALIGN);
                std::alloc::dealloc(ptr.as_ptr(), layout);
            }
        }
        if self.cap > 0 {
            // SAFETY: Current chunk was allocated with `Layout::from_size_align(self.cap, ARENA_ALIGN)`.
            unsafe {
                let layout = Layout::from_size_align_unchecked(self.cap, ARENA_ALIGN);
                std::alloc::dealloc(self.ptr.as_ptr(), layout);
            }
        }
    }
}

/// Raw pointer + length pair stored in the interner's caches.
/// Points to the first byte of the string data (past the header).
#[derive(Clone, Copy)]
struct RawStr {
    /// Pointer to the first byte of the string (after the header).
    ptr: *const u8,
    /// Length of the string in bytes.
    len: u32,
}

impl RawStr {
    const NULL: Self = Self { ptr: std::ptr::null(), len: 0 };

    /// Reconstruct a `&[u8]` from the raw pointer and length.
    ///
    /// # Safety
    /// The pointer must be valid and point to `len` bytes of valid UTF-8.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    unsafe fn as_bytes(&self) -> &[u8] {
        // SAFETY: Caller guarantees `ptr` is valid for `len` bytes.
        unsafe { std::slice::from_raw_parts(self.ptr, self.len as usize) }
    }
}

/// Number of entries in the L1 direct-mapped cache.
const L1_SIZE: usize = 256;

/// 2-tier string interner.
///
/// **L1**: Direct-mapped cache with 256 entries, indexed by `hash % 256`.
/// Lossy — collisions silently overwrite previous entries.
///
/// **L2**: `hashbrown::HashTable` storing all interned strings. Authoritative
/// source of truth for deduplication.
pub(crate) struct StringInterner {
    l1: [RawStr; L1_SIZE],
    l2: HashTable<RawStr>,
    arena: StringArena,
}

// SAFETY: `StringInterner` is only accessed via `&self` on `Allocator` through `UnsafeCell`.
// The raw pointers in `RawStr` point into the arena owned by the same `Allocator`,
// so they're valid as long as the allocator is alive. The `Allocator` is already `!Sync`,
// so `Send` is safe (only one thread accesses the interner at a time when the allocator is moved).
unsafe impl Send for StringInterner {}

impl Default for StringInterner {
    fn default() -> Self {
        Self::new()
    }
}

impl StringInterner {
    pub(crate) fn new() -> Self {
        Self { l1: [RawStr::NULL; L1_SIZE], l2: HashTable::new(), arena: StringArena::new() }
    }

    /// Intern a string, returning a [`NonNull<u8>`] pointing to the string bytes
    /// in the arena (with the header at a negative offset).
    ///
    /// If the string was previously interned, returns the existing pointer.
    /// Otherwise allocates `[header][bytes]` in the interner's own arena.
    ///
    /// The L1 cache check (hot path) is inlined at every call site and uses a cheap
    /// byte-based index (no hashing). `fx_hash` is only computed on L1 miss inside
    /// the cold `intern_slow` function.
    #[expect(clippy::inline_always, clippy::cast_possible_truncation)]
    #[inline(always)]
    pub(crate) fn intern(&mut self, s: &str) -> NonNull<u8> {
        let s_len = s.len() as u32;
        let idx = l1_index(s);

        // L1 check (hot path — no hashing, inlined at every call site)
        let cached = self.l1[idx];
        if !cached.ptr.is_null() && cached.len == s_len {
            // SAFETY: `cached.ptr` was stored from a valid arena-allocated interned string.
            if unsafe { cached.as_bytes() } == s.as_bytes() {
                // SAFETY: `cached.ptr` was allocated in the arena and is non-null.
                return unsafe { NonNull::new_unchecked(cached.ptr.cast_mut()) };
            }
        }

        // Hash computed inside intern_slow (keeps inlined fast path minimal)
        self.intern_slow(s, idx, s_len)
    }

    /// Slow path for interning: hash computation + L2 lookup + arena allocation.
    ///
    /// Separated from [`intern`] so that the L1 fast path can be inlined
    /// without bloating call sites with the L2 and allocation code.
    /// `fx_hash` is computed here rather than in `intern()` to keep the
    /// always-inlined fast path as small as possible.
    #[cold]
    #[inline(never)]
    fn intern_slow(&mut self, s: &str, idx: usize, s_len: u32) -> NonNull<u8> {
        let hash = fx_hash(s);

        // L2 check
        let l2_result = self.l2.find(hash, |raw| {
            // SAFETY: All `RawStr` entries in L2 were stored from valid arena-allocated strings.
            raw.len == s_len && unsafe { raw.as_bytes() } == s.as_bytes()
        });
        if let Some(entry) = l2_result {
            let entry = *entry;
            self.l1[idx] = entry;
            // SAFETY: `entry.ptr` was allocated in the arena and is non-null.
            return unsafe { NonNull::new_unchecked(entry.ptr.cast_mut()) };
        }

        // Cache miss — allocate in arena
        let bytes_ptr = alloc_interned_str(&mut self.arena, s, hash);
        let raw = RawStr { ptr: bytes_ptr, len: s_len };
        #[expect(clippy::cast_ptr_alignment)]
        self.l2.insert_unique(hash, raw, |r| {
            // SAFETY: `r.ptr` points to string bytes with a valid header at negative offset.
            // Hash is first field (u64) at ptr - HEADER_SIZE. Alignment is guaranteed
            // because the header was allocated at 8-byte alignment.
            unsafe { r.ptr.sub(HEADER_SIZE).cast::<u64>().read() }
        });
        self.l1[idx] = raw;
        // SAFETY: `bytes_ptr` was just allocated in the arena and is non-null.
        unsafe { NonNull::new_unchecked(bytes_ptr.cast_mut()) }
    }

    /// Reset the interner, clearing both caches and the string arena.
    pub(crate) fn reset(&mut self) {
        self.l1 = [RawStr::NULL; L1_SIZE];
        self.l2.clear();
        self.arena.reset();
    }

    /// Total capacity of the string arena.
    pub(crate) fn arena_capacity(&self) -> usize {
        self.arena.capacity()
    }

    /// Total bytes used in the string arena.
    pub(crate) fn arena_used_bytes(&self) -> usize {
        self.arena.used_bytes()
    }
}

/// Cheap L1 index from string bytes — no full hash computation.
/// Uses first byte, last byte, and length for reasonable distribution
/// across 256 slots.
#[expect(clippy::inline_always)]
#[inline(always)]
fn l1_index(s: &str) -> usize {
    let bytes = s.as_bytes();
    let len = bytes.len();
    let first = if len > 0 { bytes[0] as usize } else { 0 };
    let last = if len > 1 { bytes[len - 1] as usize } else { first };
    (first.wrapping_mul(31) ^ last ^ len) % L1_SIZE
}

/// Compute FxHash of a byte string.
#[inline]
pub fn fx_hash(s: &str) -> u64 {
    let mut hasher = FxBuildHasher.build_hasher();
    hasher.write(s.as_bytes());
    hasher.finish()
}

/// Allocate `[InternedStrHeader][string bytes]` in the string arena.
/// Returns a pointer to the first byte of the string data (past the header).
#[expect(clippy::cast_possible_truncation, clippy::cast_ptr_alignment)]
fn alloc_interned_str(arena: &mut StringArena, s: &str, hash: u64) -> *const u8 {
    let total = HEADER_SIZE + s.len();
    let ptr = arena.alloc(total);

    // SAFETY: `ptr` is freshly allocated with sufficient size for header + string bytes,
    // and aligned to 8 bytes (ARENA_ALIGN == align_of::<InternedStrHeader>()).
    unsafe {
        let header_ptr = ptr.as_ptr().cast::<InternedStrHeader>();
        header_ptr.write(InternedStrHeader::new(hash, s.len() as u32));

        let bytes_ptr = ptr.as_ptr().add(HEADER_SIZE);
        std::ptr::copy_nonoverlapping(s.as_ptr(), bytes_ptr, s.len());

        bytes_ptr
    }
}

/// Read a `&str` from an interned string pointer.
///
/// # Safety
/// `ptr` must be a pointer returned by [`StringInterner::intern`] or [`Allocator::intern_str`],
/// pointing to valid interned string bytes with a valid [`InternedStrHeader`] at `ptr - HEADER_SIZE`.
///
/// [`Allocator::intern_str`]: crate::Allocator::intern_str
#[expect(clippy::inline_always, clippy::cast_ptr_alignment)]
#[inline(always)]
pub unsafe fn interned_str_from_ptr<'a>(ptr: NonNull<u8>) -> &'a str {
    // Read `len` directly from header. Layout: [hash:u64 @ -16][len:u32 @ -8][_pad:u32 @ -4][string @ 0]
    // SAFETY: Caller guarantees `ptr` was returned by `intern_str`. The `len` field is at
    // `ptr - 8` (offset 8 in the 16-byte header). The pointer is 8-byte aligned, so `ptr - 8`
    // is 4-byte aligned (valid for u32 read).
    let len = unsafe { ptr.as_ptr().sub(8).cast::<u32>().read() as usize };
    // SAFETY: The string bytes are valid UTF-8, allocated contiguously after the header.
    unsafe { std::str::from_utf8_unchecked(std::slice::from_raw_parts(ptr.as_ptr(), len)) }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::Allocator;

    /// Helper to read `&str` from interned `NonNull<u8>`.
    unsafe fn as_str<'a>(ptr: NonNull<u8>) -> &'a str {
        // SAFETY: Caller guarantees `ptr` was returned by `intern_str`.
        unsafe { interned_str_from_ptr(ptr) }
    }

    #[test]
    fn intern_returns_same_pointer() {
        let allocator = Allocator::new();
        let a = allocator.intern_str("hello");
        let b = allocator.intern_str("hello");
        // Same string should return the same pointer
        // SAFETY: `a` and `b` were returned by `intern_str`.
        assert_eq!(unsafe { as_str(a) }, unsafe { as_str(b) });
        assert_eq!(a.as_ptr(), b.as_ptr());
    }

    #[test]
    fn intern_different_strings() {
        let allocator = Allocator::new();
        let a = allocator.intern_str("hello");
        let b = allocator.intern_str("world");
        // SAFETY: `a` and `b` were returned by `intern_str`.
        assert_eq!(unsafe { as_str(a) }, "hello");
        // SAFETY: `b` was returned by `intern_str`.
        assert_eq!(unsafe { as_str(b) }, "world");
        assert_ne!(a.as_ptr(), b.as_ptr());
    }

    #[test]
    fn intern_empty_string() {
        let allocator = Allocator::new();
        let a = allocator.intern_str("");
        // SAFETY: `a` was returned by `intern_str`.
        assert_eq!(unsafe { as_str(a) }, "");
    }

    #[test]
    fn intern_deduplicates() {
        let allocator = Allocator::new();
        // Intern many strings, then re-intern them
        let strings = ["foo", "bar", "baz", "qux", "hello", "world"];
        let first_ptrs: std::vec::Vec<*mut u8> =
            strings.iter().map(|s| allocator.intern_str(s).as_ptr()).collect();
        let second_ptrs: std::vec::Vec<*mut u8> =
            strings.iter().map(|s| allocator.intern_str(s).as_ptr()).collect();
        assert_eq!(first_ptrs, second_ptrs);
    }
}
