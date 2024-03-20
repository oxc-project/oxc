use std::{borrow::Borrow, fmt, hash, ops::Deref};

use oxc_macros::ast_node;
#[cfg(feature = "serialize")]
use serde::{Serialize, Serializer};

use compact_str::CompactString;

#[cfg(feature = "serialize")]
#[wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
export type Atom = string;
export type CompactStr = string;
"#;

/// Maximum length for inline string, which can be created with `CompactStr::new_const`.
pub const MAX_INLINE_LEN: usize = 16;

/// An inlinable string for oxc_allocator.
///
/// Use [CompactStr] with [Atom::to_compact_str] or [Atom::into_compact_str] for the
/// lifetimeless form.
#[ast_node]
#[derive(Clone, Eq)]
pub struct Atom<'a>(&'a str);

#[cfg(feature = "serialize")]
impl<'a> Serialize for Atom<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'a> Atom<'a> {
    #[inline]
    pub fn as_str(&self) -> &str {
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

impl<'a> From<&'a str> for Atom<'a> {
    fn from(s: &'a str) -> Self {
        Self(s)
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

/// Lifetimeless version of `Atom<'_>` which owns its own string data allocation.
///
/// `CompactStr` is immutable. Use `CompactStr::into_string` for a mutable `String`.
///
/// Currently implemented as just a wrapper around `compact_str::CompactString`,
/// but will be reduced in size with a custom implementation later.
#[derive(Clone, Eq)]
pub struct CompactStr(CompactString);

impl CompactStr {
    /// Create a new `CompactStr`.
    ///
    /// If `&str` is `'static` and no more than `MAX_INLINE_LEN` bytes,
    /// prefer `CompactStr::new_const` which creates the `CompactStr` at compile time.
    ///
    /// # Examples
    /// ```
    /// let s = CompactStr::new("long string which can't use new_const for");
    /// ```
    #[inline]
    pub fn new(s: &str) -> Self {
        Self(CompactString::new(s))
    }

    /// Create a `CompactStr` at compile time.
    ///
    /// String must be no longer than `MAX_INLINE_LEN` bytes.
    ///
    /// Prefer this over `CompactStr::new` or `CompactStr::from` where string
    /// is `'static` and not longer than `MAX_INLINE_LEN` bytes.
    ///
    /// # Panics
    /// Panics if string is longer than `MAX_INLINE_LEN` bytes.
    ///
    /// # Examples
    /// ```
    /// const S: CompactStr = CompactStr::new_const("short");
    /// ```
    #[inline]
    pub const fn new_const(s: &'static str) -> Self {
        assert!(s.len() <= MAX_INLINE_LEN);
        Self(CompactString::new_inline(s))
    }

    /// Get string content as a `&str` slice.
    #[inline]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Convert a `CompactStr` into a `String`.
    #[inline]
    pub fn into_string(self) -> String {
        self.0.into_string()
    }

    /// Get length of `CompactStr`.
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Check if a `CompactStr` is empty (0 length).
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
