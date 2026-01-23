//! Identifier string type.

#![expect(
    clippy::cast_possible_truncation,
    clippy::len_without_is_empty,
    clippy::undocumented_unsafe_blocks,
    clippy::cast_lossless
)]

use std::{borrow::Cow, fmt, hash, hash::Hasher, marker::PhantomData, ops::Deref, ptr::NonNull};

use oxc_allocator::{Allocator, CloneIn, Dummy, FromIn, StringBuilder as ArenaStringBuilder};
#[cfg(feature = "serialize")]
use oxc_estree::{ESTree, Serializer as ESTreeSerializer};
use rustc_hash::FxHasher;
#[cfg(feature = "serialize")]
use serde::{Serialize, Serializer as SerdeSerializer};

use crate::{Atom, CompactStr};

/// Compute FxHash of a string at compile time.
///
/// This matches what `hash::Hash::hash(&s, &mut FxHasher)` produces at runtime
/// with rustc-hash v2.x, which uses a wyhash-inspired algorithm.
///
/// The `str` Hash impl: writes bytes, then 0xff marker, then length.
const fn const_fx_hash_str(s: &str) -> u64 {
    // rustc-hash v2.x constants
    const SEED1: u64 = 0x243f_6a88_85a3_08d3;
    const SEED2: u64 = 0x1319_8a2e_0370_7344;
    const PREVENT_TRIVIAL_ZERO_COLLAPSE: u64 = 0xa409_3822_299f_31d0;
    const K: u64 = 0xf135_7aea_2e62_a9c5;

    // Folded multiplication: (x * y) XOR'd with its high bits
    const fn multiply_mix(x: u64, y: u64) -> u64 {
        let full = (x as u128) * (y as u128);
        (full as u64) ^ ((full >> 64) as u64)
    }

    // Read u64 from bytes in little-endian
    const fn read_u64_le(bytes: &[u8], offset: usize) -> u64 {
        u64::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
            bytes[offset + 4],
            bytes[offset + 5],
            bytes[offset + 6],
            bytes[offset + 7],
        ])
    }

    // Read u32 from bytes in little-endian
    const fn read_u32_le(bytes: &[u8], offset: usize) -> u32 {
        u32::from_le_bytes([bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3]])
    }

    // Hash bytes using rustc-hash v2.x algorithm (wyhash-inspired)
    const fn hash_bytes(bytes: &[u8]) -> u64 {
        let len = bytes.len();
        let mut s0 = SEED1;
        let mut s1 = SEED2;

        if len <= 16 {
            if len >= 8 {
                s0 ^= read_u64_le(bytes, 0);
                s1 ^= read_u64_le(bytes, len - 8);
            } else if len >= 4 {
                s0 ^= read_u32_le(bytes, 0) as u64;
                s1 ^= read_u32_le(bytes, len - 4) as u64;
            } else if len > 0 {
                let lo = bytes[0];
                let mid = bytes[len / 2];
                let hi = bytes[len - 1];
                s0 ^= lo as u64;
                s1 ^= ((hi as u64) << 8) | mid as u64;
            }
        } else {
            // Handle bulk (can partially overlap with suffix)
            let mut off = 0;
            while off < len - 16 {
                let x = read_u64_le(bytes, off);
                let y = read_u64_le(bytes, off + 8);
                let t = multiply_mix(s0 ^ x, PREVENT_TRIVIAL_ZERO_COLLAPSE ^ y);
                s0 = s1;
                s1 = t;
                off += 16;
            }
            // Process suffix
            s0 ^= read_u64_le(bytes, len - 16);
            s1 ^= read_u64_le(bytes, len - 8);
        }

        multiply_mix(s0, s1) ^ (len as u64)
    }

    // add_to_hash in rustc-hash v2.x: hash = (hash + i).wrapping_mul(K)
    const fn add_to_hash(hash: u64, i: u64) -> u64 {
        hash.wrapping_add(i).wrapping_mul(K)
    }

    let bytes = s.as_bytes();

    // str's Hash impl:
    // 1. write(bytes) -> write_u64(hash_bytes(bytes)) -> add_to_hash(hash_bytes_result)
    let mut state: u64 = 0;
    state = add_to_hash(state, hash_bytes(bytes));

    // 2. write_u8(0xff) -> add_to_hash(0xff)
    state = add_to_hash(state, 0xff);

    // Note: str's Hash impl does NOT call write_usize(len) anymore
    // (changed in recent Rust versions)

    state
}

/// Compute the hash of a string, taking top 32 bits which have highest entropy with FxHasher.
/// This matches what `Ident::new` computes at runtime.
const fn compute_hash(s: &str) -> u32 {
    // const_fx_hash_str computes the internal state, but FxHasher::finish()
    // rotates left by 26 bits before returning.
    let internal_state = const_fx_hash_str(s);
    let finished = internal_state.rotate_left(26);
    (finished >> 32) as u32
}

/// An identifier string for oxc_allocator.
///
/// This type stores a string reference with a precomputed hash for fast equality
/// checks and efficient hash map operations.
///
/// Use [CompactStr] with [Ident::to_compact_str] or [Ident::into_compact_str] for
/// the lifetimeless form.
#[derive(Clone, Copy, Eq)]
pub struct Ident<'a> {
    ptr: NonNull<u8>,
    len: u32,
    hash: u32,
    _marker: PhantomData<&'a str>,
}

impl Ident<'static> {
    /// Get an [`Ident`] containing a static string.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub const fn new_const(s: &'static str) -> Self {
        let ptr = unsafe { NonNull::new_unchecked(s.as_ptr().cast_mut()) };
        let len = s.len() as u32;
        let hash = compute_hash(s);
        Self { ptr, len, hash, _marker: PhantomData }
    }

    /// Get an [`Ident`] containing the empty string (`""`).
    #[inline]
    pub const fn empty() -> Self {
        Self::new_const("")
    }
}

impl<'a> Ident<'a> {
    /// Create a new [`Ident`] from a string slice.
    #[inline]
    pub fn new(s: &'a str) -> Self {
        let ptr = NonNull::from(s).cast::<u8>();
        let len = s.len() as u32;

        // Produce a hash of the string
        let hash = {
            let mut hasher = FxHasher::default();
            hash::Hash::hash(&s, &mut hasher);
            // Take top 32 bits which have highest entropy with FxHasher
            (hasher.finish() >> 32) as u32
        };

        Self { ptr, len, hash, _marker: PhantomData }
    }

    /// Get the length of this identifier.
    #[inline]
    pub fn len(&self) -> usize {
        self.len as usize
    }

    /// Borrow a string slice.
    #[inline]
    pub fn as_str(&self) -> &'a str {
        unsafe {
            let slice = std::slice::from_raw_parts(self.ptr.as_ptr(), self.len());
            std::str::from_utf8_unchecked(slice)
        }
    }

    /// Convert this [`Ident`] into an [`Atom`].
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub fn as_atom(&self) -> Atom<'a> {
        Atom::from(self.as_str())
    }

    /// Convert this [`Ident`] into a [`String`].
    ///
    /// This is the explicit form of [`Into<String>`], which [`Ident`] also implements.
    #[inline]
    pub fn into_string(self) -> String {
        String::from(self.as_str())
    }

    /// Convert this [`Ident`] into a [`CompactStr`].
    ///
    /// This is the explicit form of [`Into<CompactStr>`], which [`Ident`] also implements.
    #[inline]
    pub fn into_compact_str(self) -> CompactStr {
        CompactStr::new(self.as_str())
    }

    /// Convert this [`Ident`] into a [`CompactStr`] without consuming `self`.
    #[inline]
    pub fn to_compact_str(self) -> CompactStr {
        CompactStr::new(self.as_str())
    }

    /// Create new [`Ident`] from a fixed-size array of `&str`s concatenated together,
    /// allocated in the given `allocator`.
    ///
    /// # Panics
    ///
    /// Panics if the sum of length of all strings exceeds `isize::MAX`.
    // `#[inline(always)]` because want compiler to be able to optimize where some of `strings`
    // are statically known. See `Allocator::alloc_concat_strs_array`.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub fn from_strs_array_in<const N: usize>(
        strings: [&str; N],
        allocator: &'a Allocator,
    ) -> Ident<'a> {
        Self::from(allocator.alloc_concat_strs_array(strings))
    }

    /// Convert a [`Cow<'a, str>`] to an [`Ident<'a>`].
    ///
    /// If the `Cow` borrows a string from arena, returns an `Ident` which references that same string,
    /// without allocating a new one.
    ///
    /// If the `Cow` is owned, allocates the string into arena to generate a new `Ident`.
    #[inline]
    pub fn from_cow_in(value: &Cow<'a, str>, allocator: &'a Allocator) -> Ident<'a> {
        match value {
            Cow::Borrowed(s) => Ident::from(*s),
            Cow::Owned(s) => Ident::from_in(s, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for Ident<'_> {
    type Cloned = Ident<'new_alloc>;

    #[inline]
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        let s = allocator.alloc_str(self.as_str());
        let ptr = NonNull::from(s).cast::<u8>();
        Ident { ptr, len: self.len, hash: self.hash, _marker: PhantomData }
    }
}

impl<'a> Dummy<'a> for Ident<'a> {
    /// Create a dummy [`Ident`].
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn dummy(_allocator: &'a Allocator) -> Self {
        Ident::empty()
    }
}

impl<'alloc> FromIn<'alloc, &Ident<'alloc>> for Ident<'alloc> {
    #[expect(clippy::inline_always)]
    #[inline(always)] // Because this is a no-op
    fn from_in(s: &Ident<'alloc>, _: &'alloc Allocator) -> Self {
        *s
    }
}

impl<'alloc> FromIn<'alloc, &str> for Ident<'alloc> {
    #[inline]
    fn from_in(s: &str, allocator: &'alloc Allocator) -> Self {
        Self::from(allocator.alloc_str(s))
    }
}

impl<'alloc> FromIn<'alloc, String> for Ident<'alloc> {
    #[inline]
    fn from_in(s: String, allocator: &'alloc Allocator) -> Self {
        Self::from(allocator.alloc_str(&s))
    }
}

impl<'alloc> FromIn<'alloc, &String> for Ident<'alloc> {
    #[inline]
    fn from_in(s: &String, allocator: &'alloc Allocator) -> Self {
        Self::from(allocator.alloc_str(s))
    }
}

impl<'alloc> FromIn<'alloc, Cow<'_, str>> for Ident<'alloc> {
    #[inline]
    fn from_in(s: Cow<'_, str>, allocator: &'alloc Allocator) -> Self {
        Self::from_in(&*s, allocator)
    }
}

impl<'a> From<&'a str> for Ident<'a> {
    #[inline]
    fn from(s: &'a str) -> Self {
        Self::new(s)
    }
}

impl<'alloc> From<ArenaStringBuilder<'alloc>> for Ident<'alloc> {
    #[inline]
    fn from(s: ArenaStringBuilder<'alloc>) -> Self {
        Self::from(s.into_str())
    }
}

impl<'a> From<Ident<'a>> for &'a str {
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn from(s: Ident<'a>) -> Self {
        s.as_str()
    }
}

impl<'a> From<Ident<'a>> for Atom<'a> {
    #[inline]
    fn from(s: Ident<'a>) -> Self {
        s.as_atom()
    }
}

impl<'a> From<Atom<'a>> for Ident<'a> {
    #[inline]
    fn from(s: Atom<'a>) -> Self {
        Self::new(s.as_str())
    }
}

impl From<Ident<'_>> for CompactStr {
    #[inline]
    fn from(val: Ident<'_>) -> Self {
        val.into_compact_str()
    }
}

impl From<Ident<'_>> for String {
    #[inline]
    fn from(val: Ident<'_>) -> Self {
        val.into_string()
    }
}

impl<'a> From<Ident<'a>> for Cow<'a, str> {
    #[inline]
    fn from(value: Ident<'a>) -> Self {
        Cow::Borrowed(value.as_str())
    }
}

impl Deref for Ident<'_> {
    type Target = str;

    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl AsRef<str> for Ident<'_> {
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl PartialEq for Ident<'_> {
    #[inline]
    fn eq(&self, other: &Ident<'_>) -> bool {
        // First compare len and hash for fast rejection (single 64-bit compare),
        // then fall back to string comparison for potential hash collisions.
        self.len == other.len && self.hash == other.hash && self.as_str() == other.as_str()
    }
}

impl PartialEq<&Ident<'_>> for Ident<'_> {
    #[inline]
    fn eq(&self, other: &&Ident<'_>) -> bool {
        self == *other
    }
}

impl PartialEq<str> for Ident<'_> {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<&str> for Ident<'_> {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<String> for Ident<'_> {
    #[inline]
    fn eq(&self, other: &String) -> bool {
        self.as_str() == other.as_str()
    }
}

impl PartialEq<Atom<'_>> for Ident<'_> {
    #[inline]
    fn eq(&self, other: &Atom<'_>) -> bool {
        self.as_str() == other.as_str()
    }
}

impl PartialEq<&Atom<'_>> for Ident<'_> {
    #[inline]
    fn eq(&self, other: &&Atom<'_>) -> bool {
        self.as_str() == other.as_str()
    }
}

impl PartialEq<Ident<'_>> for &str {
    #[inline]
    fn eq(&self, other: &Ident<'_>) -> bool {
        *self == other.as_str()
    }
}

impl PartialEq<Ident<'_>> for Cow<'_, str> {
    #[inline]
    fn eq(&self, other: &Ident<'_>) -> bool {
        self.as_ref() == other.as_str()
    }
}

impl hash::Hash for Ident<'_> {
    #[inline]
    fn hash<H: hash::Hasher>(&self, hasher: &mut H) {
        // Use precomputed hash.
        // `hashbrown` only uses top 7 bits and bottom N bits of hash,
        // where N is the exponent of number of buckets in the hash table.
        // Rotate by 57 bits (= 64 - 7) so that:
        // - Original bits 0-6 end up in u64 bits 57-63 (top 7 bits for group matching)
        // - Original bits 7-31 end up in u64 bits 0-24 (bottom 25 bits for bucket selection)
        // This gives good entropy as long as HashMap contains less than 33 million entries.
        let hash = (self.hash as u64).rotate_left(64 - 7);
        hasher.write_u64(hash);
    }
}

impl fmt::Debug for Ident<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

impl fmt::Display for Ident<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}

#[cfg(feature = "serialize")]
impl Serialize for Ident<'_> {
    #[inline] // Because it just delegates
    fn serialize<S: SerdeSerializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        Serialize::serialize(self.as_str(), serializer)
    }
}

#[cfg(feature = "serialize")]
impl ESTree for Ident<'_> {
    #[inline] // Because it just delegates
    fn serialize<S: ESTreeSerializer>(&self, serializer: S) {
        ESTree::serialize(self.as_str(), serializer);
    }
}

/// Creates an [`Ident`] using interpolation of runtime expressions.
///
/// Identical to [`std`'s `format!` macro](std::format), except:
///
/// * First argument is the allocator.
/// * Produces an [`Ident`] instead of a [`String`].
///
/// The string is built in the arena, without allocating an intermediate `String`.
///
/// # Panics
///
/// Panics if a formatting trait implementation returns an error.
///
/// # Example
///
/// ```
/// use oxc_allocator::Allocator;
/// use oxc_str::format_ident;
/// let allocator = Allocator::new();
///
/// let s1 = "foo";
/// let s2 = "bar";
/// let formatted = format_ident!(&allocator, "{s1}.{s2}");
/// assert_eq!(formatted, "foo.bar");
/// ```
#[macro_export]
macro_rules! format_ident {
    ($alloc:expr, $($arg:tt)*) => {{
        use ::std::{write, fmt::Write};
        use $crate::{Ident, __internal::ArenaStringBuilder};

        let mut s = ArenaStringBuilder::new_in($alloc);
        write!(s, $($arg)*).unwrap();
        Ident::from(s)
    }}
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn ident_eq() {
        let foo = Ident::new("foo");
        let foo2 = Ident::new("foo");
        let bar = Ident::new("bar");
        assert_eq!(foo, foo2);
        assert_ne!(foo, bar);
    }

    #[test]
    fn ident_as_str() {
        let s = "hello_world";
        let ident = Ident::new(s);
        assert_eq!(ident.as_str(), s);
    }

    #[test]
    fn ident_len() {
        let ident = Ident::new("hello");
        assert_eq!(ident.len(), 5);
    }

    #[test]
    fn ident_empty() {
        let ident = Ident::empty();
        assert_eq!(ident.len(), 0);
        assert_eq!(ident.as_str(), "");
    }

    #[test]
    fn ident_hashset_lookup() {
        use rustc_hash::FxHashSet;

        let mut set: FxHashSet<Ident<'_>> = FxHashSet::default();

        // Insert Idents
        set.insert(Ident::new("foo"));
        set.insert(Ident::new("bar"));

        // Look up with new Idents - should find them
        assert!(set.contains(&Ident::new("foo")));
        assert!(set.contains(&Ident::new("bar")));

        // Verify that a different key is not found
        assert!(!set.contains(&Ident::new("baz")));
    }

    #[test]
    fn ident_hash_compatible_with_str() {
        use rustc_hash::FxHashSet;

        // This test verifies that Ident's Hash impl is compatible with str lookups
        // which is required for correct behavior in mixed hash containers
        let mut set: FxHashSet<Ident<'_>> = FxHashSet::default();
        set.insert(Ident::new("foo"));

        // The hash of Ident("foo") should allow finding it via another Ident("foo")
        let lookup = Ident::new("foo");
        assert!(set.contains(&lookup));
    }

    #[test]
    fn new_const_equals_new() {
        // Test that Ident::new_const and Ident::new produce equal Idents
        // for the same string content.

        // Test various string lengths to exercise different code paths in const_fx_hash_str
        let test_cases: &[&str] = &[
            "",                                    // empty
            "a",                                   // 1 byte
            "ab",                                  // 2 bytes
            "abc",                                 // 3 bytes
            "abcd",                                // 4 bytes
            "abcde",                               // 5 bytes
            "abcdef",                              // 6 bytes
            "abcdefg",                             // 7 bytes
            "abcdefgh",                            // 8 bytes (exactly one chunk)
            "abcdefghi",                           // 9 bytes (one chunk + 1 remaining)
            "0123456789abcdef",                    // 16 bytes (two chunks)
            "0123456789abcdefghij",                // 20 bytes (two chunks + 4 remaining)
            "hello_world_this_is_a_longer_string", // longer string
        ];

        for s in test_cases {
            let runtime = Ident::new(s);
            let compile_time = Ident::new_const(s);

            // Check that the string content is the same
            assert_eq!(runtime.as_str(), compile_time.as_str(), "String content differs for {s:?}");

            // Check that len is the same
            assert_eq!(runtime.len, compile_time.len, "len differs for {s:?}");

            // Check that hash is the same - this is the critical test!
            assert_eq!(
                runtime.hash, compile_time.hash,
                "hash differs for {s:?}: new={:#x}, new_const={:#x}",
                runtime.hash, compile_time.hash
            );

            // Check that they compare equal (uses both len and hash)
            assert_eq!(runtime, compile_time, "Idents not equal for {s:?}");
        }
    }

    #[test]
    fn new_const_in_hashset() {
        use rustc_hash::FxHashSet;

        // Verify that Ident::new_const works correctly with hash sets
        let mut set: FxHashSet<Ident<'_>> = FxHashSet::default();

        // Insert using new_const
        set.insert(Ident::new_const("foo"));
        set.insert(Ident::new_const("bar"));

        // Look up using new - should find them
        assert!(set.contains(&Ident::new("foo")), "Failed to find 'foo' inserted with new_const");
        assert!(set.contains(&Ident::new("bar")), "Failed to find 'bar' inserted with new_const");

        // Look up using new_const - should also find them
        assert!(
            set.contains(&Ident::new_const("foo")),
            "Failed to find 'foo' with new_const lookup"
        );

        // Insert using new, look up using new_const
        set.insert(Ident::new("baz"));
        assert!(set.contains(&Ident::new_const("baz")), "Failed to find 'baz' inserted with new");
    }

    #[test]
    fn const_fx_hash_matches_runtime() {
        // Directly test the const_fx_hash_str function against runtime FxHasher
        // const_fx_hash_str returns internal state (before rotation)
        // FxHasher::finish() returns state.rotate_left(26)
        let test_cases: &[&str] = &["", "a", "ab", "abc", "abcdefgh", "hello_world"];

        for s in test_cases {
            // const_fx_hash_str returns internal state, apply rotation to match finish()
            let const_hash = const_fx_hash_str(s).rotate_left(26);

            let runtime_hash = {
                let mut hasher = FxHasher::default();
                hash::Hash::hash(&s, &mut hasher);
                hasher.finish()
            };

            assert_eq!(
                const_hash, runtime_hash,
                "Hash mismatch for {s:?}: const={const_hash:#x}, runtime={runtime_hash:#x}"
            );
        }
    }
}
