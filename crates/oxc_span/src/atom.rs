use std::{
    borrow::{Borrow, Cow},
    fmt, hash,
    ops::Deref,
};

use oxc_allocator::{Allocator, CloneIn, FromIn};
#[cfg(feature = "serialize")]
use serde::Serialize;

use crate::{cmp::ContentEq, hash::ContentHash, CompactStr};

/// An inlinable string for oxc_allocator.
///
/// Use [CompactStr] with [Atom::to_compact_str] or [Atom::into_compact_str] for
/// the lifetimeless form.
#[derive(Clone, Eq)]
#[cfg_attr(feature = "serialize", derive(Serialize))]
#[cfg_attr(feature = "serialize", serde(transparent))]
pub struct Atom<'a>(&'a str);

impl Atom<'static> {
    /// Get an [`Atom`] containing the empty string (`""`).
    #[inline]
    pub const fn empty() -> Self {
        Atom("")
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
    pub fn to_compact_str(&self) -> CompactStr {
        CompactStr::new(self.as_str())
    }
}

impl<'old_alloc, 'new_alloc> CloneIn<'new_alloc> for Atom<'old_alloc> {
    type Cloned = Atom<'new_alloc>;

    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        Atom::from_in(self.as_str(), allocator)
    }
}

impl<'alloc> FromIn<'alloc, &Atom<'alloc>> for Atom<'alloc> {
    fn from_in(s: &Atom<'alloc>, _: &'alloc Allocator) -> Self {
        Self::from(s.0)
    }
}

impl<'alloc> FromIn<'alloc, &str> for Atom<'alloc> {
    fn from_in(s: &str, allocator: &'alloc Allocator) -> Self {
        Self::from(oxc_allocator::String::from_str_in(s, allocator).into_bump_str())
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

impl<'a> From<Atom<'a>> for &'a str {
    fn from(s: Atom<'a>) -> Self {
        s.as_str()
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

impl<'a> ContentEq for Atom<'a> {
    fn content_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl<'a> ContentHash for Atom<'a> {
    fn content_hash<H: hash::Hasher>(&self, state: &mut H) {
        hash::Hash::hash(self, state);
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
