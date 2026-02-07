//! Identifier string type.

use std::{borrow::Cow, fmt, hash, marker::PhantomData, ops::Deref, ptr::NonNull};

use oxc_allocator::{
    Allocator, CloneIn, Dummy, FromIn, HEADER_SIZE, InternedStrHeader, PassthroughBuildHasher,
    fx_hash, interned_str_from_ptr,
};
#[cfg(feature = "serialize")]
use oxc_estree::{ESTree, Serializer as ESTreeSerializer};
#[cfg(feature = "serialize")]
use serde::{Serialize, Serializer as SerdeSerializer};

use crate::{Atom, CompactStr};

/// An identifier string backed by an interned arena pointer.
///
/// Each `Ident` is 8 bytes (a single `NonNull<u8>` pointer). The pointer points
/// to the string bytes in the arena, with an [`InternedStrHeader`] at negative
/// offset (`ptr - HEADER_SIZE`) containing the precomputed hash and length.
///
/// Use [`CompactStr`] with [`Ident::to_compact_str`] or [`Ident::into_compact_str`] for
/// the lifetimeless form.
#[derive(Clone, Copy, Eq)]
pub struct Ident<'a> {
    /// Pointer to string bytes in arena.
    /// Header `[hash:u64][len:u32][_pad:u32]` at `ptr - HEADER_SIZE`.
    ptr: NonNull<u8>,
    _marker: PhantomData<&'a str>,
}

const _: () = assert!(size_of::<Ident<'_>>() == 8);

// SAFETY: `Ident` is just a pointer into arena memory. It's safe to send between threads
// as long as the arena outlives the `Ident`, which is guaranteed by the lifetime parameter.
unsafe impl Send for Ident<'_> {}
// SAFETY: `Ident` is immutable (shared reference semantics via the lifetime),
// so it's safe to share between threads.
unsafe impl Sync for Ident<'_> {}

/// Static empty interned string — used for `Ident::empty()`.
#[repr(C, align(8))]
struct StaticEmptyInternedStr {
    header: InternedStrHeader,
    // No bytes follow — the string is empty.
}

/// Precomputed FxHash of the empty string.
/// FxHash of "" is 0 (the hasher state starts at 0, and writing 0 bytes doesn't change it).
const EMPTY_FX_HASH: u64 = {
    // FxHasher initial state is 0. Hashing an empty byte slice produces 0.
    0
};

static EMPTY_INTERNED: StaticEmptyInternedStr =
    StaticEmptyInternedStr { header: InternedStrHeader::new(EMPTY_FX_HASH, 0) };

impl<'a> Ident<'a> {
    /// Create a new [`Ident`] from a raw interned pointer.
    ///
    /// # Safety
    /// `ptr` must point to valid interned string bytes with a valid
    /// [`InternedStrHeader`] at `ptr - HEADER_SIZE`.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub unsafe fn from_interned_ptr(ptr: NonNull<u8>) -> Self {
        Self { ptr, _marker: PhantomData }
    }

    /// Get an [`Ident`] containing the empty string (`""`).
    #[inline]
    pub fn empty() -> Self {
        // SAFETY: `EMPTY_INTERNED` has a valid header followed by zero bytes of string data.
        // The pointer points to the byte just after the header, which is the start of (empty) string data.
        unsafe {
            let base = (&raw const EMPTY_INTERNED).cast::<u8>();
            let str_ptr = base.add(HEADER_SIZE);
            Self::from_interned_ptr(NonNull::new_unchecked(str_ptr.cast_mut()))
        }
    }

    /// Borrow a string slice.
    #[expect(clippy::inline_always, clippy::trivially_copy_pass_by_ref)]
    #[inline(always)]
    pub fn as_str(&self) -> &'a str {
        // SAFETY: `self.ptr` points to a valid interned string with a valid header at negative offset.
        unsafe { interned_str_from_ptr(self.ptr) }
    }

    /// Convert this [`Ident`] into an [`Atom`].
    #[expect(clippy::trivially_copy_pass_by_ref)]
    #[inline]
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

    /// Get the precomputed FxHash stored in the interned string header.
    #[expect(clippy::cast_ptr_alignment)]
    #[inline]
    pub fn precomputed_hash(&self) -> u64 {
        // SAFETY: `self.ptr` was created by `intern_str`. The header is at `ptr - HEADER_SIZE`,
        // and the original allocation was aligned to `align_of::<InternedStrHeader>()`.
        unsafe { (*self.ptr.as_ptr().sub(HEADER_SIZE).cast::<InternedStrHeader>()).hash }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for Ident<'_> {
    type Cloned = Ident<'new_alloc>;

    #[inline]
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        Ident::from_in(self.as_str(), allocator)
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
        // SAFETY: `intern_str` returns a valid interned pointer.
        unsafe { Self::from_interned_ptr(allocator.intern_str(s)) }
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

/// Wrapper for `&str` lookups in [`PassthroughBuildHasher`]-based `Ident` maps.
///
/// Computes `fx_hash` on the fly so the lookup hash matches the precomputed
/// hash stored in [`Ident`].
#[repr(transparent)]
pub struct IdentStr<'a>(pub &'a str);

impl hash::Hash for IdentStr<'_> {
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        state.write_u64(fx_hash(self.0));
    }
}

impl oxc_allocator::hash_map::Equivalent<Ident<'_>> for IdentStr<'_> {
    #[inline]
    fn equivalent(&self, key: &Ident<'_>) -> bool {
        self.0 == key.as_str()
    }
}

impl<T: AsRef<str>> PartialEq<T> for Ident<'_> {
    #[inline]
    fn eq(&self, other: &T) -> bool {
        self.as_str() == other.as_ref()
    }
}

impl PartialEq<Ident<'_>> for &str {
    #[inline]
    fn eq(&self, other: &Ident<'_>) -> bool {
        *self == other.as_str()
    }
}

impl PartialEq<str> for Ident<'_> {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
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
        hasher.write_u64(self.precomputed_hash());
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

/// Hash map keyed by [`Ident`], using [`PassthroughBuildHasher`] to skip re-hashing.
pub type IdentHashMap<'a, V> = hashbrown::HashMap<Ident<'a>, V, PassthroughBuildHasher>;

/// Arena-allocated hash map keyed by [`Ident`], using [`PassthroughBuildHasher`].
pub type ArenaIdentHashMap<'alloc, V> =
    oxc_allocator::HashMap<'alloc, Ident<'alloc>, V, PassthroughBuildHasher>;

/// Hash set of [`Ident`], using [`PassthroughBuildHasher`] to skip re-hashing.
pub type IdentHashSet<'a> = hashbrown::HashSet<Ident<'a>, PassthroughBuildHasher>;

/// Creates an [`Ident`] using interpolation of runtime expressions.
///
/// Identical to [`std`'s `format!` macro](std::format), except:
///
/// * First argument is the allocator.
/// * Produces an [`Ident`] instead of a [`String`].
///
/// The string is built in the arena via a `StringBuilder`, then interned.
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
        use $crate::__internal::{ArenaStringBuilder, FromIn};

        let alloc = $alloc;
        let mut s = ArenaStringBuilder::new_in(alloc);
        write!(s, $($arg)*).unwrap();
        let str = s.into_str();
        <$crate::Ident as FromIn<&str>>::from_in(str, alloc)
    }}
}
