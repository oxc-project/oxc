//! String interner for arena-allocated identifier strings.
//!
//! Provides a 2-tier cache (L1 direct-mapped + L2 hash table) for deduplicating
//! strings allocated in the bump arena. Each interned string is stored with a
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

use crate::bump::Bump;

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

    /// Reconstruct a `&str` from the raw pointer and length.
    ///
    /// # Safety
    /// The pointer must be valid and point to `len` bytes of valid UTF-8.
    #[inline]
    unsafe fn as_str(&self) -> &str {
        // SAFETY: Caller guarantees `ptr` is valid for `len` bytes of UTF-8.
        let slice = unsafe { std::slice::from_raw_parts(self.ptr, self.len as usize) };
        // SAFETY: The string was originally valid UTF-8 when interned.
        unsafe { std::str::from_utf8_unchecked(slice) }
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
        Self { l1: [RawStr::NULL; L1_SIZE], l2: HashTable::new() }
    }

    /// Intern a string, returning a [`NonNull<u8>`] pointing to the string bytes
    /// in the arena (with the header at a negative offset).
    ///
    /// If the string was previously interned, returns the existing pointer.
    /// Otherwise allocates `[header][bytes]` in the bump arena.
    #[expect(clippy::cast_possible_truncation)]
    pub(crate) fn intern(&mut self, s: &str, bump: &Bump) -> NonNull<u8> {
        let hash = fx_hash(s);
        let idx = (hash as usize) % L1_SIZE;
        let s_len = s.len() as u32;

        // L1 check (fast path)
        let cached = self.l1[idx];
        if !cached.ptr.is_null() && cached.len == s_len {
            // SAFETY: `cached.ptr` was stored from a valid arena-allocated interned string.
            let cached_str = unsafe { cached.as_str() };
            if cached_str == s {
                // SAFETY: `cached.ptr` was allocated in the arena and is non-null.
                return unsafe { NonNull::new_unchecked(cached.ptr.cast_mut()) };
            }
        }

        // L2 check
        let l2_result = self.l2.find(hash, |raw| {
            // SAFETY: All `RawStr` entries in L2 were stored from valid arena-allocated strings.
            raw.len == s_len && unsafe { raw.as_str() == s }
        });
        if let Some(entry) = l2_result {
            let entry = *entry;
            self.l1[idx] = entry;
            // SAFETY: `entry.ptr` was allocated in the arena and is non-null.
            return unsafe { NonNull::new_unchecked(entry.ptr.cast_mut()) };
        }

        // Cache miss — allocate in arena
        let bytes_ptr = alloc_interned_str(bump, s, hash);
        let raw = RawStr { ptr: bytes_ptr, len: s_len };
        self.l2.insert_unique(hash, raw, |r| {
            // SAFETY: `r.ptr` points to string bytes with a valid header at negative offset.
            // The pointer was originally aligned to `InternedStrHeader` alignment.
            unsafe { read_header(r.ptr).hash }
        });
        self.l1[idx] = raw;
        // SAFETY: `bytes_ptr` was just allocated in the arena and is non-null.
        unsafe { NonNull::new_unchecked(bytes_ptr.cast_mut()) }
    }

    /// Reset the interner, clearing both caches.
    pub(crate) fn reset(&mut self) {
        self.l1 = [RawStr::NULL; L1_SIZE];
        self.l2.clear();
    }
}

/// Compute FxHash of a byte string.
#[inline]
fn fx_hash(s: &str) -> u64 {
    let mut hasher = FxBuildHasher.build_hasher();
    hasher.write(s.as_bytes());
    hasher.finish()
}

/// Read the [`InternedStrHeader`] from a pointer to interned string bytes.
///
/// # Safety
/// `bytes_ptr` must point to string bytes that were allocated via [`alloc_interned_str`],
/// with a valid [`InternedStrHeader`] at `bytes_ptr - HEADER_SIZE`. The original allocation
/// must have been aligned to `align_of::<InternedStrHeader>()`.
#[inline]
#[expect(clippy::cast_ptr_alignment)]
unsafe fn read_header(bytes_ptr: *const u8) -> &'static InternedStrHeader {
    // SAFETY: The header is at `bytes_ptr - HEADER_SIZE`. The original allocation was aligned
    // to `align_of::<InternedStrHeader>()`, so the header pointer is properly aligned.
    unsafe { &*bytes_ptr.sub(HEADER_SIZE).cast::<InternedStrHeader>() }
}

/// Allocate `[InternedStrHeader][string bytes]` in the bump arena.
/// Returns a pointer to the first byte of the string data (past the header).
#[expect(clippy::cast_possible_truncation, clippy::cast_ptr_alignment)]
fn alloc_interned_str(bump: &Bump, s: &str, hash: u64) -> *const u8 {
    let total = HEADER_SIZE + s.len();
    let layout = Layout::from_size_align(total, align_of::<InternedStrHeader>()).unwrap();
    let ptr = bump.alloc_layout(layout);

    // SAFETY: `ptr` is freshly allocated with sufficient size for header + string bytes,
    // and aligned to `align_of::<InternedStrHeader>()`.
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
#[inline]
pub unsafe fn interned_str_from_ptr<'a>(ptr: NonNull<u8>) -> &'a str {
    // SAFETY: Caller guarantees `ptr` was returned by `intern_str`, so the header is valid.
    let header = unsafe { read_header(ptr.as_ptr()) };
    // SAFETY: The string bytes are valid UTF-8, allocated contiguously after the header.
    let slice = unsafe { std::slice::from_raw_parts(ptr.as_ptr(), header.len as usize) };
    // SAFETY: The string was valid UTF-8 when it was interned.
    unsafe { std::str::from_utf8_unchecked(slice) }
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
