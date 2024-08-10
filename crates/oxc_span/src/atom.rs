use std::{
    borrow::{Borrow, Cow},
    fmt, hash,
    ops::{Deref, Index},
};

use compact_str::CompactString;
#[cfg(feature = "serialize")]
use serde::{Serialize, Serializer};

use crate::Span;
use oxc_allocator::{Allocator, CloneIn, FromIn};

#[cfg(feature = "serialize")]
#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
export type Atom = string;
export type CompactStr = string;
"#;

/// Maximum length for inline string, which can be created with [`CompactStr::new_const`].
pub const MAX_INLINE_LEN: usize = 16;

/// An inlinable string for oxc_allocator.
///
/// Use [CompactStr] with [Atom::to_compact_str] or [Atom::into_compact_str] for
/// the lifetimeless form.
#[derive(Clone, Eq)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "serialize", serde(transparent))]
pub struct Atom<'a>(&'a str);

impl Atom<'static> {
    #[inline]
    pub const fn empty() -> Self {
        Atom("")
    }
}

impl<'a> Atom<'a> {
    #[inline]
    pub fn as_str(&self) -> &'a str {
        self.0
    }

    #[inline]
    pub fn into_string(self) -> String {
        String::from(self.as_str())
    }

    #[inline]
    pub fn into_compact_str(self) -> CompactStr {
        CompactStr::new(self.as_str())
    }

    #[inline]
    pub fn to_compact_str(&self) -> CompactStr {
        CompactStr::new(self.as_str())
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for Atom<'old_alloc> {
    type Cloned = Atom<'new_alloc>;

    fn clone_in(&self, alloc: &'new_alloc Allocator) -> Self::Cloned {
        Atom::from_in(self.as_str(), alloc)
    }
}

impl<'a, 'b> FromIn<'a, &'b Atom<'a>> for Atom<'a> {
    fn from_in(s: &'b Atom<'a>, _: &'a Allocator) -> Self {
        Self::from(s.0)
    }
}

impl<'a, 'b> FromIn<'a, &'b str> for Atom<'a> {
    fn from_in(s: &'b str, alloc: &'a Allocator) -> Self {
        Self::from(oxc_allocator::String::from_str_in(s, alloc).into_bump_str())
    }
}

impl<'a> FromIn<'a, String> for Atom<'a> {
    fn from_in(s: String, alloc: &'a Allocator) -> Self {
        Self::from_in(s.as_str(), alloc)
    }
}

impl<'a> FromIn<'a, &String> for Atom<'a> {
    fn from_in(s: &String, alloc: &'a Allocator) -> Self {
        Self::from_in(s.as_str(), alloc)
    }
}

impl<'a, 'b> FromIn<'a, Cow<'b, str>> for Atom<'a> {
    fn from_in(s: Cow<'b, str>, alloc: &'a Allocator) -> Self {
        Self::from_in(&*s, alloc)
    }
}

impl<'a> From<&'a str> for Atom<'a> {
    fn from(s: &'a str) -> Self {
        Self(s)
    }
}

impl<'a> From<Atom<'a>> for CompactStr {
    #[inline]
    fn from(val: Atom<'a>) -> Self {
        val.into_compact_str()
    }
}

impl<'a> From<Atom<'a>> for String {
    #[inline]
    fn from(val: Atom<'a>) -> Self {
        val.into_string()
    }
}

impl<'a> From<Atom<'a>> for Cow<'a, str> {
    #[inline]
    fn from(value: Atom<'a>) -> Self {
        Cow::Borrowed(value.as_str())
    }
}

impl<'a> Deref for Atom<'a> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl<'a> AsRef<str> for Atom<'a> {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<'a> Borrow<str> for Atom<'a> {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl<'a, T: AsRef<str>> PartialEq<T> for Atom<'a> {
    fn eq(&self, other: &T) -> bool {
        self.as_str() == other.as_ref()
    }
}

impl<'a> PartialEq<Atom<'a>> for &str {
    fn eq(&self, other: &Atom<'a>) -> bool {
        *self == other.as_str()
    }
}

impl<'a> PartialEq<str> for Atom<'a> {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl<'a> PartialEq<Atom<'a>> for Cow<'_, str> {
    fn eq(&self, other: &Atom<'a>) -> bool {
        self.as_ref() == other.as_str()
    }
}
impl<'a> PartialEq<&Atom<'a>> for Cow<'_, str> {
    fn eq(&self, other: &&Atom<'a>) -> bool {
        self.as_ref() == other.as_str()
    }
}

impl<'a> hash::Hash for Atom<'a> {
    fn hash<H: hash::Hasher>(&self, hasher: &mut H) {
        self.as_str().hash(hasher);
    }
}

impl<'a> fmt::Debug for Atom<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

impl<'a> fmt::Display for Atom<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}

/// Lifetimeless version of [`Atom<'_>`] which owns its own string data allocation.
///
/// [`CompactStr`] is immutable. Use [`CompactStr::into_string`] for a mutable
/// [`String`].
///
/// Currently implemented as just a wrapper around [`compact_str::CompactString`],
/// but will be reduced in size with a custom implementation later.
#[derive(Clone, Eq)]
#[cfg_attr(feature = "serialize", derive(serde::Deserialize))]
pub struct CompactStr(CompactString);

impl CompactStr {
    /// Create a new [`CompactStr`].
    ///
    /// If `&str` is `'static` and no more than [`MAX_INLINE_LEN`] bytes,
    /// prefer [`CompactStr::new_const`] which creates the [`CompactStr`] at
    /// compile time.
    ///
    /// # Examples
    /// ```
    /// let s = CompactStr::new("long string which can't use new_const for");
    /// ```
    #[inline]
    pub fn new(s: &str) -> Self {
        Self(CompactString::new(s))
    }

    /// Create a [`CompactStr`] at compile time.
    ///
    /// String must be no longer than [`MAX_INLINE_LEN`] bytes.
    ///
    /// Prefer this over [`CompactStr::new`] or [`CompactStr::from`] where
    /// string is `'static` and not longer than [`MAX_INLINE_LEN`] bytes.
    ///
    /// # Panics
    /// Panics if string is longer than [`MAX_INLINE_LEN`] bytes.
    ///
    /// # Examples
    /// ```
    /// const S: CompactStr = CompactStr::new_const("short");
    /// ```
    #[inline]
    pub const fn new_const(s: &'static str) -> Self {
        assert!(s.len() <= MAX_INLINE_LEN);
        Self(CompactString::const_new(s))
    }

    /// Get string content as a `&str` slice.
    #[inline]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Convert a [`CompactStr`] into a [`String`].
    #[inline]
    pub fn into_string(self) -> String {
        self.0.into_string()
    }

    /// Get length of [`CompactStr`].
    ///
    /// # Examples
    /// ```
    /// use oxc_span::CompactStr;
    ///
    /// assert_eq!(CompactStr::new("").len(), 0);
    /// assert_eq!(CompactStr::new_const("").len(), 0);
    /// assert_eq!(CompactStr::new("hello").len(), 5);
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Check if a [`CompactStr`] is empty (0 length).
    ///
    /// # Examples
    /// ```
    /// use oxc_span::CompactStr;
    ///
    /// assert!(CompactStr::new("").is_empty());
    /// assert!(!CompactStr::new("hello").is_empty());
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl From<&str> for CompactStr {
    fn from(s: &str) -> Self {
        Self(CompactString::from(s))
    }
}

impl From<String> for CompactStr {
    fn from(s: String) -> Self {
        Self(CompactString::from(s))
    }
}

impl<'s> From<&'s CompactStr> for Cow<'s, str> {
    fn from(value: &'s CompactStr) -> Self {
        Self::Borrowed(value.as_str())
    }
}

impl From<CompactStr> for Cow<'_, str> {
    fn from(value: CompactStr) -> Self {
        value.0.into()
    }
}
impl From<Cow<'_, str>> for CompactStr {
    fn from(value: Cow<'_, str>) -> Self {
        match value {
            Cow::Borrowed(s) => CompactStr::new(s),
            Cow::Owned(s) => CompactStr::from(s),
        }
    }
}

impl Deref for CompactStr {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl AsRef<str> for CompactStr {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Borrow<str> for CompactStr {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl<T: AsRef<str>> PartialEq<T> for CompactStr {
    fn eq(&self, other: &T) -> bool {
        self.as_str() == other.as_ref()
    }
}

impl PartialEq<CompactStr> for &str {
    fn eq(&self, other: &CompactStr) -> bool {
        *self == other.as_str()
    }
}

impl PartialEq<CompactStr> for str {
    fn eq(&self, other: &CompactStr) -> bool {
        self == other.as_str()
    }
}

impl PartialEq<str> for CompactStr {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<CompactStr> for Cow<'_, str> {
    fn eq(&self, other: &CompactStr) -> bool {
        self.as_ref() == other.as_str()
    }
}

impl Index<Span> for CompactStr {
    type Output = str;

    fn index(&self, index: Span) -> &Self::Output {
        &self.0[index]
    }
}

impl hash::Hash for CompactStr {
    fn hash<H: hash::Hasher>(&self, hasher: &mut H) {
        self.as_str().hash(hasher);
    }
}

impl fmt::Debug for CompactStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

impl fmt::Display for CompactStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}

#[cfg(feature = "serialize")]
impl Serialize for CompactStr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

#[cfg(feature = "schemars")]
impl schemars::JsonSchema for CompactStr {
    fn is_referenceable() -> bool {
        false
    }

    fn schema_name() -> std::string::String {
        "String".to_string()
    }

    fn schema_id() -> Cow<'static, str> {
        Cow::Borrowed("String")
    }

    fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        <&str>::json_schema(gen)
    }
}

#[cfg(test)]
mod test {
    use super::CompactStr;

    #[test]
    fn test_compactstr_eq() {
        let foo = CompactStr::new("foo");
        assert_eq!(foo, "foo");
        assert_eq!(&foo, "foo");
        assert_eq!("foo", foo);
        assert_eq!("foo", &foo);
    }
}
