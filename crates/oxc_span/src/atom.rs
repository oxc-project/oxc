use std::{
    borrow::{Borrow, Cow},
    fmt, hash,
    ops::Deref,
};

use oxc_allocator::{Allocator, CloneIn, Dummy, FromIn, StringBuilder as ArenaStringBuilder};
#[cfg(feature = "serialize")]
use oxc_estree::{ESTree, Serializer as ESTreeSerializer};
#[cfg(feature = "serialize")]
use serde::{Serialize, Serializer as SerdeSerializer};

use crate::{CompactStr, ContentEq};

/// An inlinable string for oxc_allocator.
///
/// Use [CompactStr] with [Atom::to_compact_str] or [Atom::into_compact_str] for
/// the lifetimeless form.
#[derive(Clone, Copy, Eq)]
pub struct Atom<'a>(&'a str);

impl Atom<'static> {
    /// Get an [`Atom`] containing a static string.
    #[inline]
    pub const fn new_const(s: &'static str) -> Self {
        Atom(s)
    }

    /// Get an [`Atom`] containing the empty string (`""`).
    #[inline]
    pub const fn empty() -> Self {
        Self::new_const("")
    }
}

impl<'a> Atom<'a> {
    /// Borrow a string slice.
    #[inline]
    pub fn as_str(&self) -> &'a str {
        self.0
    }

    /// Convert this [`Atom`] into a [`String`].
    ///
    /// This is the explicit form of [`Into<String>`], which [`Atom`] also implements.
    #[inline]
    pub fn into_string(self) -> String {
        String::from(self.as_str())
    }

    /// Convert this [`Atom`] into a [`CompactStr`].
    ///
    /// This is the explicit form of [`Into<CompactStr>`], which [`Atom`] also implements.
    #[inline]
    pub fn into_compact_str(self) -> CompactStr {
        CompactStr::new(self.as_str())
    }

    /// Convert this [`Atom`] into a [`CompactStr`] without consuming `self`.
    #[inline]
    pub fn to_compact_str(self) -> CompactStr {
        CompactStr::new(self.as_str())
    }

    /// Create new [`Atom`] from a fixed-size array of `&str`s concatenated together,
    /// allocated in the given `allocator`.
    ///
    /// # Panics
    ///
    /// Panics if the sum of length of all strings exceeds `isize::MAX`.
    ///
    /// # Example
    /// ```
    /// use oxc_allocator::Allocator;
    /// use oxc_span::Atom;
    ///
    /// let allocator = Allocator::new();
    /// let s = Atom::from_strs_array_in(["hello", " ", "world", "!"], &allocator);
    /// assert_eq!(s.as_str(), "hello world!");
    /// ```
    // `#[inline(always)]` because want compiler to be able to optimize where some of `strings`
    // are statically known. See `Allocator::alloc_concat_strs_array`.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub fn from_strs_array_in<const N: usize>(
        strings: [&str; N],
        allocator: &'a Allocator,
    ) -> Atom<'a> {
        Self::from(allocator.alloc_concat_strs_array(strings))
    }

    /// Convert a [`Cow<'a, str>`] to an [`Atom<'a>`].
    ///
    /// If the `Cow` borrows a string from arena, returns an `Atom` which references that same string,
    /// without allocating a new one.
    ///
    /// If the `Cow` is owned, allocates the string into arena to generate a new `Atom`.
    #[inline]
    pub fn from_cow_in(value: &Cow<'a, str>, allocator: &'a Allocator) -> Atom<'a> {
        match value {
            Cow::Borrowed(s) => Atom::from(*s),
            Cow::Owned(s) => Atom::from_in(s, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for Atom<'_> {
    type Cloned = Atom<'new_alloc>;

    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        Atom::from_in(self.as_str(), allocator)
    }
}

impl<'a> Dummy<'a> for Atom<'a> {
    /// Create a dummy [`Atom`].
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn dummy(_allocator: &'a Allocator) -> Self {
        Atom::empty()
    }
}

impl<'alloc> FromIn<'alloc, &Atom<'alloc>> for Atom<'alloc> {
    fn from_in(s: &Atom<'alloc>, _: &'alloc Allocator) -> Self {
        *s
    }
}

impl<'alloc> FromIn<'alloc, &str> for Atom<'alloc> {
    fn from_in(s: &str, allocator: &'alloc Allocator) -> Self {
        Self::from(allocator.alloc_str(s))
    }
}

impl<'alloc> FromIn<'alloc, String> for Atom<'alloc> {
    fn from_in(s: String, allocator: &'alloc Allocator) -> Self {
        Self::from_in(s.as_str(), allocator)
    }
}

impl<'alloc> FromIn<'alloc, &String> for Atom<'alloc> {
    fn from_in(s: &String, allocator: &'alloc Allocator) -> Self {
        Self::from_in(s.as_str(), allocator)
    }
}

impl<'alloc> FromIn<'alloc, Cow<'_, str>> for Atom<'alloc> {
    fn from_in(s: Cow<'_, str>, allocator: &'alloc Allocator) -> Self {
        Self::from_in(&*s, allocator)
    }
}

impl<'a> From<&'a str> for Atom<'a> {
    fn from(s: &'a str) -> Self {
        Self(s)
    }
}

impl<'alloc> From<ArenaStringBuilder<'alloc>> for Atom<'alloc> {
    fn from(s: ArenaStringBuilder<'alloc>) -> Self {
        Self::from(s.into_str())
    }
}

impl<'a> From<Atom<'a>> for &'a str {
    fn from(s: Atom<'a>) -> Self {
        s.as_str()
    }
}

impl From<Atom<'_>> for CompactStr {
    #[inline]
    fn from(val: Atom<'_>) -> Self {
        val.into_compact_str()
    }
}

impl From<Atom<'_>> for String {
    #[inline]
    fn from(val: Atom<'_>) -> Self {
        val.into_string()
    }
}

impl<'a> From<Atom<'a>> for Cow<'a, str> {
    #[inline]
    fn from(value: Atom<'a>) -> Self {
        Cow::Borrowed(value.as_str())
    }
}

impl Deref for Atom<'_> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl AsRef<str> for Atom<'_> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Borrow<str> for Atom<'_> {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl<T: AsRef<str>> PartialEq<T> for Atom<'_> {
    fn eq(&self, other: &T) -> bool {
        self.as_str() == other.as_ref()
    }
}

impl PartialEq<Atom<'_>> for &str {
    fn eq(&self, other: &Atom<'_>) -> bool {
        *self == other.as_str()
    }
}

impl PartialEq<str> for Atom<'_> {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<Atom<'_>> for Cow<'_, str> {
    fn eq(&self, other: &Atom<'_>) -> bool {
        self.as_ref() == other.as_str()
    }
}

impl ContentEq for Atom<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl hash::Hash for Atom<'_> {
    fn hash<H: hash::Hasher>(&self, hasher: &mut H) {
        self.as_str().hash(hasher);
    }
}

impl fmt::Debug for Atom<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

impl fmt::Display for Atom<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}

#[cfg(feature = "serialize")]
impl Serialize for Atom<'_> {
    fn serialize<S: SerdeSerializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        Serialize::serialize(self.as_str(), serializer)
    }
}

#[cfg(feature = "serialize")]
impl ESTree for Atom<'_> {
    fn serialize<S: ESTreeSerializer>(&self, serializer: S) {
        ESTree::serialize(self.as_str(), serializer);
    }
}

/// Creates an [`Atom`] using interpolation of runtime expressions.
///
/// Identical to [`std`'s `format!` macro](std::format), except:
///
/// * First argument is the allocator.
/// * Produces an [`Atom`] instead of a [`String`].
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
/// use oxc_span::format_atom;
/// let allocator = Allocator::new();
///
/// let s1 = "foo";
/// let s2 = "bar";
/// let formatted = format_atom!(&allocator, "{s1}.{s2}");
/// assert_eq!(formatted, "foo.bar");
/// ```
#[macro_export]
macro_rules! format_atom {
    ($alloc:expr, $($arg:tt)*) => {{
        use ::std::{write, fmt::Write};
        use $crate::{Atom, __internal::ArenaStringBuilder};

        let mut s = ArenaStringBuilder::new_in($alloc);
        write!(s, $($arg)*).unwrap();
        Atom::from(s)
    }}
}
