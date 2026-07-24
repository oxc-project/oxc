use std::{
    borrow::{Borrow, Cow},
    fmt, hash,
    ops::Deref,
};

use oxc_allocator::{
    Allocator, ArenaStringBuilder, CloneIn, CloneInSemanticIds, Dummy, FromIn, GetAllocator,
};
#[cfg(feature = "serialize")]
use oxc_estree::{ESTree, Serializer as ESTreeSerializer};
#[cfg(feature = "serialize")]
use serde::{Serialize, Serializer as SerdeSerializer};

use crate::CompactStr;

/// An inlinable string for oxc_allocator.
///
/// Use [CompactStr] with [Str::to_compact_str] or [Str::into_compact_str] for
/// the lifetimeless form.
#[repr(transparent)]
#[derive(Clone, Copy, Eq)]
pub struct Str<'a>(&'a str);

impl Str<'static> {
    /// Get a [`Str`] containing a static string.
    #[expect(clippy::inline_always)]
    #[inline(always)] // Because this is a no-op
    pub const fn new_const(s: &'static str) -> Self {
        Str(s)
    }

    /// Get a [`Str`] containing the empty string (`""`).
    #[inline]
    pub const fn empty() -> Self {
        Self::new_const("")
    }
}

impl<'a> Str<'a> {
    /// Allocate provided `&str` into arena, and return a [`Str<'a>`].
    #[inline]
    pub fn from_str_in<A: GetAllocator<'a>>(s: &str, allocator: &A) -> Self {
        Self(allocator.allocator().alloc_str(s))
    }

    /// Borrow a string slice.
    #[expect(clippy::inline_always)]
    #[inline(always)] // Because this is a no-op
    pub fn as_str(&self) -> &'a str {
        self.0
    }

    /// Convert this [`Str`] into a [`String`].
    ///
    /// This is the explicit form of [`Into<String>`], which [`Str`] also implements.
    #[inline]
    pub fn into_string(self) -> String {
        String::from(self.as_str())
    }

    /// Convert this [`Str`] into a [`CompactStr`].
    ///
    /// This is the explicit form of [`Into<CompactStr>`], which [`Str`] also implements.
    #[inline]
    pub fn into_compact_str(self) -> CompactStr {
        CompactStr::new(self.as_str())
    }

    /// Convert this [`Str`] into a [`CompactStr`] without consuming `self`.
    #[inline]
    pub fn to_compact_str(self) -> CompactStr {
        CompactStr::new(self.as_str())
    }

    /// Create new [`Str`] from a fixed-size array of `&str`s concatenated together,
    /// allocated in the given `allocator`.
    ///
    /// # Panics
    ///
    /// Panics if the sum of length of all strings exceeds `isize::MAX`.
    ///
    /// # Example
    /// ```
    /// use oxc_allocator::Allocator;
    /// use oxc_str::Str;
    ///
    /// let allocator = Allocator::new();
    /// let s = Str::from_strs_array_in(["hello", " ", "world", "!"], &&allocator);
    /// assert_eq!(s.as_str(), "hello world!");
    /// ```
    // `#[inline(always)]` because want compiler to be able to optimize where some of `strings`
    // are statically known. See `Allocator::alloc_concat_strs_array`.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub fn from_strs_array_in<const N: usize, A: GetAllocator<'a>>(
        strings: [&str; N],
        allocator: &A,
    ) -> Str<'a> {
        Self::from(allocator.allocator().alloc_concat_strs_array(strings))
    }

    /// Convert a [`Cow<'a, str>`] to a [`Str<'a>`].
    ///
    /// If the `Cow` borrows a string from arena, returns a `Str` which references that same string,
    /// without allocating a new one.
    ///
    /// If the `Cow` is owned, allocates the string into arena to generate a new `Str`.
    #[inline]
    pub fn from_cow_in<A: GetAllocator<'a>>(value: &Cow<'a, str>, allocator: &A) -> Str<'a> {
        match value {
            Cow::Borrowed(s) => Str::from(*s),
            Cow::Owned(s) => Str::from_str_in(s, allocator),
        }
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for Str<'_> {
    type Cloned = Str<'new_alloc>;

    #[inline]
    fn clone_in_impl(
        &self,
        _with_semantic_ids: CloneInSemanticIds,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        Str::from_in(self.as_str(), allocator)
    }
}

impl<'a> Dummy<'a> for Str<'a> {
    /// Create a dummy [`Str`].
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn dummy(_allocator: &'a Allocator) -> Self {
        Str::empty()
    }
}

impl<'alloc> FromIn<'alloc, &Str<'alloc>> for Str<'alloc> {
    #[expect(clippy::inline_always)]
    #[inline(always)] // Because this is a no-op
    fn from_in(s: &Str<'alloc>, _: &'alloc Allocator) -> Self {
        *s
    }
}

impl<'alloc> FromIn<'alloc, &str> for Str<'alloc> {
    #[inline]
    fn from_in(s: &str, allocator: &'alloc Allocator) -> Self {
        Self::from(allocator.alloc_str(s))
    }
}

impl<'alloc> FromIn<'alloc, String> for Str<'alloc> {
    #[inline]
    fn from_in(s: String, allocator: &'alloc Allocator) -> Self {
        Self::from_in(s.as_str(), allocator)
    }
}

impl<'alloc> FromIn<'alloc, &String> for Str<'alloc> {
    #[inline]
    fn from_in(s: &String, allocator: &'alloc Allocator) -> Self {
        Self::from_in(s.as_str(), allocator)
    }
}

impl<'alloc> FromIn<'alloc, Cow<'_, str>> for Str<'alloc> {
    #[inline]
    fn from_in(s: Cow<'_, str>, allocator: &'alloc Allocator) -> Self {
        Self::from_in(&*s, allocator)
    }
}

impl<'a> From<&'a str> for Str<'a> {
    #[expect(clippy::inline_always)]
    #[inline(always)] // Because this is a no-op
    fn from(s: &'a str) -> Self {
        Self(s)
    }
}

impl<'alloc> From<ArenaStringBuilder<'alloc>> for Str<'alloc> {
    #[inline]
    fn from(s: ArenaStringBuilder<'alloc>) -> Self {
        Self::from(s.into_str())
    }
}

impl<'a> From<Str<'a>> for &'a str {
    #[expect(clippy::inline_always)]
    #[inline(always)] // Because this is a no-op
    fn from(s: Str<'a>) -> Self {
        s.as_str()
    }
}

impl From<Str<'_>> for CompactStr {
    #[inline]
    fn from(val: Str<'_>) -> Self {
        val.into_compact_str()
    }
}

impl From<Str<'_>> for String {
    #[inline]
    fn from(val: Str<'_>) -> Self {
        val.into_string()
    }
}

impl<'a> From<Str<'a>> for Cow<'a, str> {
    #[inline]
    fn from(value: Str<'a>) -> Self {
        Cow::Borrowed(value.as_str())
    }
}

impl Deref for Str<'_> {
    type Target = str;

    #[expect(clippy::inline_always)]
    #[inline(always)] // Because this is a no-op
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl AsRef<str> for Str<'_> {
    #[expect(clippy::inline_always)]
    #[inline(always)] // Because this is a no-op
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Borrow<str> for Str<'_> {
    #[expect(clippy::inline_always)]
    #[inline(always)] // Because this is a no-op
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl<T: AsRef<str>> PartialEq<T> for Str<'_> {
    #[inline]
    fn eq(&self, other: &T) -> bool {
        self.as_str() == other.as_ref()
    }
}

impl PartialEq<Str<'_>> for &str {
    #[inline]
    fn eq(&self, other: &Str<'_>) -> bool {
        *self == other.as_str()
    }
}

impl PartialEq<str> for Str<'_> {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<Str<'_>> for Cow<'_, str> {
    #[inline]
    fn eq(&self, other: &Str<'_>) -> bool {
        self.as_ref() == other.as_str()
    }
}

impl hash::Hash for Str<'_> {
    #[inline]
    fn hash<H: hash::Hasher>(&self, hasher: &mut H) {
        self.as_str().hash(hasher);
    }
}

impl fmt::Debug for Str<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

impl fmt::Display for Str<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}

#[cfg(feature = "serialize")]
impl Serialize for Str<'_> {
    #[inline] // Because it just delegates
    fn serialize<S: SerdeSerializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        Serialize::serialize(self.as_str(), serializer)
    }
}

#[cfg(feature = "serialize")]
impl ESTree for Str<'_> {
    #[inline] // Because it just delegates
    fn serialize<S: ESTreeSerializer>(&self, serializer: S) {
        ESTree::serialize(self.as_str(), serializer);
    }
}

/// Create a [`Str<'static>`] for a string literal, evaluated at compile time.
///
/// Why this macro? [`Str`] in time will likely evolve to have more features and constraints
/// than it currently does. e.g. constrain max length to `u32::MAX`, add a flag for "is all ASCII".
/// [`Str::from`] will likely gain runtime checks, whereas this macro will perform any checks
/// or calculations at compile time. So using this macro in preference to [`Str::from`]
/// is future-proof.
///
/// ```
/// use oxc_str::static_str;
///
/// let str = static_str!("undefined");
/// assert_eq!(str.as_str(), "undefined");
/// ```
///
/// Can also be used in const context:
///
/// ```
/// use oxc_str::{Str, static_str};
///
/// const UNDEFINED: Str<'static> = static_str!("undefined");
/// assert_eq!(UNDEFINED.as_str(), "undefined");
/// ```
///
/// Only accepts string literals, not variables:
///
/// ```compile_fail
/// use oxc_str::static_str;
///
/// let s = "hello";
/// let str = static_str!(s);
/// ```
#[macro_export]
macro_rules! static_str {
    ($s:literal) => {
        $crate::Str::new_const($s)
    };
}

/// Creates a [`Str`] using interpolation of runtime expressions.
///
/// Identical to [`std`'s `format!` macro](std::format), except:
///
/// * First argument is the allocator.
/// * Produces a [`Str`] instead of a [`String`].
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
/// use oxc_str::format_str;
/// let allocator = Allocator::new();
///
/// let s1 = "foo";
/// let s2 = "bar";
/// let formatted = format_str!(&allocator, "{s1}.{s2}");
/// assert_eq!(formatted, "foo.bar");
/// ```
#[macro_export]
macro_rules! format_str {
    ($alloc:expr, $($arg:tt)*) => {{
        let mut s = $crate::__internal::ArenaStringBuilder::new_in($alloc);
        ::std::fmt::Write::write_fmt(&mut s, ::std::format_args!($($arg)*)).unwrap();
        $crate::Str::from(s)
    }}
}

#[cfg(test)]
mod test {
    use oxc_allocator::Allocator;

    use super::*;

    #[test]
    #[expect(clippy::items_after_statements)]
    fn str_from_str_in() {
        let allocator = Allocator::new();
        let allocator = &allocator;

        // Pass an actual `Allocator`
        let s = Str::from_str_in("world", &allocator);
        assert_eq!(s.as_str(), "world");
        assert_eq!(s, Str::from("world"));

        // Pass a struct which implements `GetAllocator`
        struct Wrapper<'a>(&'a Allocator);

        impl<'a> GetAllocator<'a> for Wrapper<'a> {
            fn allocator(&self) -> &'a Allocator {
                self.0
            }
        }

        let wrapper = Wrapper(allocator);
        let s = Str::from_str_in("hello", &wrapper);
        assert_eq!(s.as_str(), "hello");
        assert_eq!(s, Str::from("hello"));
    }

    #[test]
    #[expect(clippy::items_after_statements)]
    fn str_from_strs_array_in() {
        let allocator = Allocator::new();
        let allocator = &allocator;

        // Pass an actual `Allocator`
        let s = Str::from_strs_array_in(["hello", " ", "world", "!"], &allocator);
        assert_eq!(s.as_str(), "hello world!");
        assert_eq!(s, Str::from("hello world!"));

        // Pass a struct which implements `GetAllocator`
        struct Wrapper<'a>(&'a Allocator);

        impl<'a> GetAllocator<'a> for Wrapper<'a> {
            fn allocator(&self) -> &'a Allocator {
                self.0
            }
        }

        let wrapper = Wrapper(allocator);
        let s = Str::from_strs_array_in(["foo", "_", "bar"], &wrapper);
        assert_eq!(s.as_str(), "foo_bar");
        assert_eq!(s, Str::from("foo_bar"));
    }

    #[test]
    #[expect(clippy::items_after_statements)]
    fn str_from_cow_in() {
        let allocator = Allocator::new();
        let allocator = &allocator;

        // `Cow::Borrowed` references the same string, without allocating in arena
        let borrowed = "world";
        let used_before = allocator.used_bytes();
        let s = Str::from_cow_in(&Cow::Borrowed(borrowed), &allocator);
        assert_eq!(s.as_str(), "world");
        assert_eq!(s, Str::from("world"));
        assert_eq!(s.as_str().as_ptr(), borrowed.as_ptr());
        assert_eq!(allocator.used_bytes(), used_before);

        // `Cow::Owned` allocates a new string in arena
        let owned = "owned".to_string();
        let owned_ptr = owned.as_ptr();
        let s = Str::from_cow_in(&Cow::Owned(owned), &allocator);
        assert_eq!(s.as_str(), "owned");
        assert_eq!(s, Str::from("owned"));
        assert_ne!(s.as_str().as_ptr(), owned_ptr);
        assert!(allocator.used_bytes() > used_before);

        // Pass a struct which implements `GetAllocator`
        struct Wrapper<'a>(&'a Allocator);

        impl<'a> GetAllocator<'a> for Wrapper<'a> {
            fn allocator(&self) -> &'a Allocator {
                self.0
            }
        }

        let wrapper = Wrapper(allocator);
        let s = Str::from_cow_in(&Cow::Borrowed("hello"), &wrapper);
        assert_eq!(s.as_str(), "hello");
        assert_eq!(s, Str::from("hello"));
    }
}
