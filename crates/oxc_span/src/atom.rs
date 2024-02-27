use std::{
    borrow::{Borrow, Cow},
    fmt, hash,
    ops::Deref,
};

#[cfg(feature = "serde")]
use serde::{Serialize, Serializer};

use compact_str::CompactString;

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
#[derive(Clone, Eq)]
pub enum Atom<'a> {
    Arena(&'a str),
    Compact(CompactString),
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
    #[inline]
    pub fn as_str(&self) -> &str {
        match self {
            Self::Arena(s) => s,
            Self::Compact(s) => s.as_ref(),
        }
    }

    #[inline]
    pub fn into_string(self) -> String {
        match self {
            Self::Arena(s) => String::from(s),
            Self::Compact(s) => s.to_string(),
        }
    }

    #[inline]
    pub fn into_compact_string(self) -> CompactString {
        match self {
            Self::Arena(s) => CompactString::new(s),
            Self::Compact(s) => s,
        }
    }

    #[inline]
    pub fn to_compact_string(&self) -> CompactString {
        match &self {
            Self::Arena(s) => CompactString::new(s),
            Self::Compact(s) => s.clone(),
        }
    }
}

impl<'a> From<&'a str> for Atom<'a> {
    fn from(s: &'a str) -> Self {
        Self::Arena(s)
    }
}

impl<'a> From<String> for Atom<'a> {
    fn from(s: String) -> Self {
        Self::Compact(CompactString::from(s))
    }
}

impl<'a> From<Cow<'_, str>> for Atom<'a> {
    fn from(s: Cow<'_, str>) -> Self {
        Self::Compact(CompactString::from(s))
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

impl<'a> hash::Hash for Atom<'a> {
    fn hash<H: hash::Hasher>(&self, hasher: &mut H) {
        match self {
            Self::Arena(s) => s.hash(hasher),
            Self::Compact(s) => s.hash(hasher),
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
