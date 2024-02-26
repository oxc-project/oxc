use std::{
    borrow::{Borrow, Cow},
    fmt, hash,
    ops::Deref,
};

#[cfg(feature = "serde")]
use serde::{Serialize, Serializer};

use compact_str::CompactString;
use inlinable_string::inline_string::{InlineString, INLINE_STRING_CAPACITY};

const BASE54_CHARS: &[u8; 64] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ$_0123456789";

#[cfg_attr(
    all(feature = "serde", feature = "wasm"),
    wasm_bindgen::prelude::wasm_bindgen(typescript_custom_section)
)]
#[allow(dead_code)]
const TS_APPEND_CONTENT: &'static str = r#"
export type Atom = string;
export type CompactString = string;
"#;

/// An inlinable string for oxc_allocator.
///
/// Use [CompactString] with [Atom::to_compact_string()] for the lifetimeless form.
#[derive(Clone, Eq, Hash)]
pub struct Atom<'a>(AtomImpl<'a>);

/// Immutable Inlinable String
///
/// https://github.com/fitzgen/inlinable_string/blob/master/src/lib.rs
#[derive(Clone, Eq, PartialEq)]
enum AtomImpl<'a> {
    /// A arena heap-allocated string.
    Arena(&'a str),
    /// A heap-allocated string.
    Heap(Box<str>),
    /// A small string stored inline.
    Inline(InlineString),
}

#[cfg(feature = "serde")]
impl<'a> Serialize for Atom<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'a> Atom<'a> {
    pub fn new_inline(s: &str) -> Self {
        Self(AtomImpl::Inline(InlineString::from(s)))
    }

    #[inline]
    pub fn as_str(&self) -> &str {
        match &self.0 {
            AtomImpl::Arena(s) => s,
            AtomImpl::Heap(s) => s,
            AtomImpl::Inline(s) => s.as_ref(),
        }
    }

    #[inline]
    pub fn into_string(self) -> String {
        match self.0 {
            AtomImpl::Arena(s) => String::from(s),
            AtomImpl::Heap(s) => s.to_string(),
            AtomImpl::Inline(s) => s.to_string(),
        }
    }

    pub fn into_compact_string(self) -> CompactString {
        match self.0 {
            AtomImpl::Arena(s) => CompactString::new(s),
            AtomImpl::Heap(s) => CompactString::new(s),
            AtomImpl::Inline(s) => CompactString::new(s),
        }
    }

    pub fn to_compact_string(&self) -> CompactString {
        match &self.0 {
            AtomImpl::Arena(s) => CompactString::new(s),
            AtomImpl::Heap(s) => CompactString::new(s),
            AtomImpl::Inline(s) => CompactString::new(s),
        }
    }

    /// Get the shortest mangled name for a given n.
    /// Code adapted from [terser](https://github.com/terser/terser/blob/8b966d687395ab493d2c6286cc9dd38650324c11/lib/scope.js#L1041-L1051)
    pub fn base54(n: usize) -> Self {
        let mut num = n;
        // Base 54 at first because these are the usable first characters in JavaScript identifiers
        // <https://tc39.es/ecma262/#prod-IdentifierStart>
        let base = 54usize;
        let mut ret = String::new();
        ret.push(BASE54_CHARS[num % base] as char);
        num /= base;
        // Base 64 for the rest because after the first character we can also use 0-9 too
        // <https://tc39.es/ecma262/#prod-IdentifierPart>
        let base = 64usize;
        while num > 0 {
            num -= 1;
            ret.push(BASE54_CHARS[num % base] as char);
            num /= base;
        }
        Self(AtomImpl::Heap(ret.into_boxed_str()))
    }
}

impl<'a> From<&'a str> for Atom<'a> {
    fn from(s: &'a str) -> Self {
        if s.len() <= INLINE_STRING_CAPACITY {
            Self(AtomImpl::Inline(InlineString::from(s)))
        } else {
            Self(AtomImpl::Arena(s))
        }
    }
}

impl<'a> From<String> for Atom<'a> {
    fn from(s: String) -> Self {
        if s.len() <= INLINE_STRING_CAPACITY {
            Self(AtomImpl::Inline(InlineString::from(s.as_str())))
        } else {
            Self(AtomImpl::Heap(s.into_boxed_str()))
        }
    }
}

impl<'a> From<Cow<'_, str>> for Atom<'a> {
    fn from(s: Cow<'_, str>) -> Self {
        if s.len() <= INLINE_STRING_CAPACITY {
            Self(AtomImpl::Inline(InlineString::from(s.borrow())))
        } else {
            Self(AtomImpl::Heap(s.into()))
        }
    }
}

impl<'a> Deref for Atom<'a> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl<'a> AsRef<str> for Atom<'a> {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl<'a> Borrow<str> for Atom<'a> {
    #[inline]
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

impl<'a> hash::Hash for AtomImpl<'a> {
    #[inline]
    fn hash<H: hash::Hasher>(&self, hasher: &mut H) {
        match self {
            Self::Arena(s) => s.hash(hasher),
            Self::Heap(s) => s.hash(hasher),
            Self::Inline(s) => s.hash(hasher),
        }
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
