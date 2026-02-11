//! Identifier string type with precomputed hash.

use std::{borrow::Cow, fmt, hash, marker::PhantomData, ops::Deref, ptr::NonNull, slice, str};

use oxc_allocator::{
    Allocator, CloneIn, Dummy, FromIn, IdentBuildHasher, StringBuilder as ArenaStringBuilder,
    ident_hash, pack_len_hash,
};
#[cfg(feature = "serialize")]
use oxc_estree::{ESTree, Serializer as ESTreeSerializer};
#[cfg(feature = "serialize")]
use serde::{Serialize, Serializer as SerdeSerializer};

use crate::{Atom, CompactStr};

/// An identifier string for oxc_allocator with a precomputed hash.
///
/// Stores a pointer to string data, length, and a precomputed hash for fast
/// HashMap lookups and equality comparisons.
///
/// On 64-bit platforms, length and hash are packed into a single `u64` for
/// maximum performance (16 bytes total, same as `&str`).
/// On 32-bit platforms, they are stored as separate `u32` fields (12 bytes total).
///
/// Use [CompactStr] with [Ident::to_compact_str] or [Ident::into_compact_str] for
/// the lifetimeless form.
#[repr(C)]
pub struct Ident<'a> {
    ptr: NonNull<u8>,
    #[cfg(target_pointer_width = "64")]
    len_and_hash: u64,
    #[cfg(not(target_pointer_width = "64"))]
    len: u32,
    #[cfg(not(target_pointer_width = "64"))]
    hash: u32,
    _marker: PhantomData<&'a str>,
}

// SAFETY: Ident is conceptually equivalent to &str, which is Send + Sync.
// NonNull is !Send/!Sync, but Ident only stores a pointer to borrowed data.
unsafe impl Send for Ident<'_> {}
// SAFETY: See above.
unsafe impl Sync for Ident<'_> {}

// We can't derive Clone/Copy because NonNull prevents it.
// The explicit impl is needed for Copy to work.
#[expect(clippy::expl_impl_clone_on_copy)]
impl Clone for Ident<'_> {
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn clone(&self) -> Self {
        *self
    }
}

impl Copy for Ident<'_> {}

impl<'a> Ident<'a> {
    /// Create an [`Ident`] from raw components.
    #[cfg(target_pointer_width = "64")]
    #[inline]
    const fn from_raw(ptr: NonNull<u8>, len: u32, hash: u32) -> Self {
        Self { ptr, len_and_hash: pack_len_hash(len, hash), _marker: PhantomData }
    }

    /// Create an [`Ident`] from raw components.
    #[cfg(not(target_pointer_width = "64"))]
    #[inline]
    const fn from_raw(ptr: NonNull<u8>, len: u32, hash: u32) -> Self {
        Self { ptr, len, hash, _marker: PhantomData }
    }

    /// Get the length of the identifier string.
    #[cfg(target_pointer_width = "64")]
    #[inline]
    const fn ident_len(&self) -> u32 {
        (self.len_and_hash & 0xFFFF_FFFF) as u32
    }

    /// Get the length of the identifier string.
    #[cfg(not(target_pointer_width = "64"))]
    #[inline]
    const fn ident_len(&self) -> u32 {
        self.len
    }

    /// Get the precomputed hash value.
    #[cfg(target_pointer_width = "64")]
    #[inline]
    const fn ident_hash_value(&self) -> u32 {
        (self.len_and_hash >> 32) as u32
    }

    /// Get the precomputed hash value.
    #[cfg(not(target_pointer_width = "64"))]
    #[inline]
    const fn ident_hash_value(&self) -> u32 {
        self.hash
    }

    /// Create a new [`Ident`] from a string slice.
    ///
    /// This is a const fn that computes the hash at compile time when possible.
    /// Use this for strings that already have the correct lifetime
    /// (e.g. arena-allocated strings, or `'static` string literals).
    #[expect(clippy::inline_always, clippy::cast_possible_truncation)]
    #[inline(always)]
    pub const fn new_const(s: &'a str) -> Self {
        let bytes = s.as_bytes();
        let len = bytes.len() as u32;
        let hash = ident_hash(bytes);
        // SAFETY: A &str's pointer is always non-null.
        let ptr = unsafe { NonNull::new_unchecked(bytes.as_ptr().cast_mut()) };
        Self::from_raw(ptr, len, hash)
    }

    /// Get an [`Ident`] containing the empty string (`""`).
    #[inline]
    pub const fn empty() -> Self {
        Self::from_raw(NonNull::dangling(), 0, ident_hash(b""))
    }

    /// Borrow a string slice.
    #[expect(clippy::inline_always)]
    #[inline(always)] // Hot path — must be inlined
    pub fn as_str(&self) -> &'a str {
        let len = self.ident_len() as usize;
        // SAFETY: The pointer and length are valid because they came from a valid &str.
        unsafe { str::from_utf8_unchecked(slice::from_raw_parts(self.ptr.as_ptr(), len)) }
    }

    /// Convert this [`Ident`] into an [`Atom`].
    #[expect(clippy::inline_always)]
    #[inline(always)] // Because this is a no-op
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

    /// Clone the identifier into a new allocator, preserving the precomputed hash.
    #[inline]
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        let s = allocator.alloc_str(self.as_str());
        // SAFETY: `alloc_str` returns a valid `&str` whose pointer is non-null.
        let ptr = unsafe { NonNull::new_unchecked(s.as_ptr().cast_mut()) };
        Ident::from_raw(ptr, self.ident_len(), self.ident_hash_value())
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
        Self::from_in(s.as_str(), allocator)
    }
}

impl<'alloc> FromIn<'alloc, &String> for Ident<'alloc> {
    #[inline]
    fn from_in(s: &String, allocator: &'alloc Allocator) -> Self {
        Self::from_in(s.as_str(), allocator)
    }
}

impl<'alloc> FromIn<'alloc, Cow<'_, str>> for Ident<'alloc> {
    #[inline]
    fn from_in(s: Cow<'_, str>, allocator: &'alloc Allocator) -> Self {
        Self::from_in(&*s, allocator)
    }
}

impl<'a> From<&'a str> for Ident<'a> {
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn from(s: &'a str) -> Self {
        Self::new_const(s)
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
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn from(s: Ident<'a>) -> Self {
        s.as_atom()
    }
}

impl<'a> From<Atom<'a>> for Ident<'a> {
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn from(s: Atom<'a>) -> Self {
        Self::from(s.as_str())
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

/// Allows looking up an `Ident`-keyed hashbrown map with a `&str` key,
/// without requiring `Ident: Borrow<str>`.
impl oxc_allocator::hash_map::Equivalent<Ident<'_>> for str {
    #[inline]
    fn equivalent(&self, key: &Ident<'_>) -> bool {
        self == key.as_str()
    }
}

impl Eq for Ident<'_> {}

impl PartialEq for Ident<'_> {
    /// Fast-reject equality: compare packed len+hash first, then bytes.
    #[cfg(target_pointer_width = "64")]
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.len_and_hash == other.len_and_hash && self.as_str() == other.as_str()
    }

    /// Fast-reject equality: compare len and hash first, then bytes.
    #[cfg(not(target_pointer_width = "64"))]
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.len == other.len && self.hash == other.hash && self.as_str() == other.as_str()
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

impl PartialEq<&Ident<'_>> for Ident<'_> {
    #[inline]
    fn eq(&self, other: &&Ident<'_>) -> bool {
        self == *other
    }
}

impl PartialEq<Atom<'_>> for Ident<'_> {
    #[inline]
    fn eq(&self, other: &Atom<'_>) -> bool {
        self.as_str() == other.as_str()
    }
}

impl hash::Hash for Ident<'_> {
    /// Write the precomputed packed len+hash as a single u64.
    #[cfg(target_pointer_width = "64")]
    #[inline]
    fn hash<H: hash::Hasher>(&self, hasher: &mut H) {
        hasher.write_u64(self.len_and_hash);
    }

    /// Pack len and hash on the fly and write as u64.
    #[cfg(not(target_pointer_width = "64"))]
    #[inline]
    fn hash<H: hash::Hasher>(&self, hasher: &mut H) {
        hasher.write_u64(pack_len_hash(self.len, self.hash));
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

/// Hash map keyed by [`Ident`], using precomputed ident hash.
pub type IdentHashMap<'a, V> = hashbrown::HashMap<Ident<'a>, V, IdentBuildHasher>;

/// Arena-allocated hash map keyed by [`Ident`], using precomputed ident hash.
pub type ArenaIdentHashMap<'alloc, V> =
    oxc_allocator::HashMap<'alloc, Ident<'alloc>, V, IdentBuildHasher>;

/// Hash set of [`Ident`], using precomputed ident hash.
pub type IdentHashSet<'a> = hashbrown::HashSet<Ident<'a>, IdentBuildHasher>;

#[expect(missing_docs)]
pub const AGGREGATE_ERROR: Ident<'static> = Ident::new_const("AggregateError");
#[expect(missing_docs)]
pub const ARGUMENTS: Ident<'static> = Ident::new_const("arguments");
#[expect(missing_docs)]
pub const ARRAY: Ident<'static> = Ident::new_const("Array");
#[expect(missing_docs)]
pub const ERROR: Ident<'static> = Ident::new_const("Error");
#[expect(missing_docs)]
pub const EXPORTS: Ident<'static> = Ident::new_const("exports");
#[expect(missing_docs)]
pub const FUNCTION: Ident<'static> = Ident::new_const("Function");
#[expect(missing_docs)]
pub const GLOBAL_THIS: Ident<'static> = Ident::new_const("globalThis");
#[expect(missing_docs)]
pub const MATH: Ident<'static> = Ident::new_const("Math");
#[expect(missing_docs)]
pub const MODULE: Ident<'static> = Ident::new_const("module");
#[expect(missing_docs)]
pub const OBJECT: Ident<'static> = Ident::new_const("Object");
#[expect(missing_docs)]
pub const PROCESS: Ident<'static> = Ident::new_const("process");
#[expect(missing_docs)]
pub const REG_EXP: Ident<'static> = Ident::new_const("RegExp");
#[expect(missing_docs)]
pub const REQUIRE: Ident<'static> = Ident::new_const("require");
#[expect(missing_docs)]
pub const TYPE_ERROR: Ident<'static> = Ident::new_const("TypeError");

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
    use std::hash::BuildHasher;

    use oxc_allocator::Allocator;

    use super::*;

    #[test]
    fn ident_size() {
        #[cfg(target_pointer_width = "64")]
        assert_eq!(std::mem::size_of::<Ident<'_>>(), 16);
        #[cfg(target_pointer_width = "32")]
        assert_eq!(std::mem::size_of::<Ident<'_>>(), 12);
    }

    #[test]
    fn ident_send_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        assert_send::<Ident<'_>>();
        assert_sync::<Ident<'_>>();
    }

    #[test]
    fn ident_copy() {
        fn assert_copy<T: Copy>() {}
        assert_copy::<Ident<'_>>();
    }

    #[test]
    fn ident_from_str() {
        let s = "hello";
        let ident = Ident::from(s);
        assert_eq!(ident.as_str(), "hello");
    }

    #[test]
    fn ident_empty() {
        let ident = Ident::empty();
        assert_eq!(ident.as_str(), "");
        assert_eq!(ident.ident_len(), 0);
    }

    #[test]
    fn ident_new_const() {
        let ident = Ident::new_const("world");
        assert_eq!(ident.as_str(), "world");
    }

    #[test]
    fn ident_eq() {
        let a = Ident::from("foo");
        let b = Ident::from("foo");
        let c = Ident::from("bar");
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn ident_eq_str() {
        let ident = Ident::from("hello");
        assert_eq!(ident, "hello");
        assert_ne!(ident, "world");
    }

    #[test]
    fn ident_hash_consistency() {
        let build_hasher = IdentBuildHasher;

        // Hash an Ident via BuildHasher::hash_one
        let ident = Ident::from("fooBar");
        let ident_hash_val = build_hasher.hash_one(ident);

        // Hash a &str through the same hasher
        let str_hash = build_hasher.hash_one("fooBar");

        assert_eq!(ident_hash_val, str_hash);
    }

    #[test]
    fn ident_hashmap_str_lookup() {
        let mut map = IdentHashMap::default();
        map.insert(Ident::from("key1"), 1);
        map.insert(Ident::from("key2"), 2);

        // Lookup with &str via Equivalent
        assert_eq!(map.get("key1"), Some(&1));
        assert_eq!(map.get("key2"), Some(&2));
        assert_eq!(map.get("key3"), None);
    }

    #[test]
    fn ident_hashmap_ident_lookup() {
        let mut map = IdentHashMap::default();
        map.insert(Ident::from("key1"), 1);

        let key = Ident::from("key1");
        assert_eq!(map.get(&key), Some(&1));
    }

    #[test]
    fn ident_clone_in() {
        let allocator = Allocator::new();
        let original = Ident::from("test");
        let cloned = original.clone_in(&allocator);
        assert_eq!(original.as_str(), cloned.as_str());
        assert_eq!(original, cloned);
        // Verify hash is preserved (not recomputed from different pointer)
        assert_eq!(original.ident_hash_value(), cloned.ident_hash_value());
    }

    #[test]
    fn ident_deref() {
        let ident = Ident::from("hello");
        // Should be able to call str methods
        assert!(ident.starts_with("hel")); // spellchecker:disable-line
        assert_eq!(ident.len(), 5);
    }

    #[test]
    fn ident_display() {
        let ident = Ident::from("test");
        assert_eq!(format!("{ident}"), "test");
    }

    #[test]
    fn ident_debug() {
        let ident = Ident::from("test");
        assert_eq!(format!("{ident:?}"), "\"test\"");
    }

    #[test]
    fn arena_ident_hashmap() {
        let allocator = Allocator::new();
        let mut map = ArenaIdentHashMap::new_in(&allocator);
        let key = Ident::from_in("hello", &allocator);
        map.insert(key, 42);
        assert_eq!(map.get("hello"), Some(&42));
        assert_eq!(map.get(&Ident::from("hello")), Some(&42));
    }
}
