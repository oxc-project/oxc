use std::{borrow::Borrow, fmt, ops::Deref};

use compact_str::CompactString;
#[cfg(feature = "serde")]
use serde::Serialize;

/// Newtype for [`CompactString`]
#[derive(Clone, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Atom(CompactString);

impl Atom {
    #[inline]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl Deref for Atom {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.as_str()
    }
}

impl<'a> From<&'a str> for Atom {
    fn from(s: &'a str) -> Self {
        Self(s.into())
    }
}

impl From<String> for Atom {
    fn from(s: String) -> Self {
        Self(s.into())
    }
}

impl AsRef<str> for Atom {
    #[inline]
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl Borrow<str> for Atom {
    #[inline]
    fn borrow(&self) -> &str {
        self.0.as_str()
    }
}

impl<T: AsRef<str>> PartialEq<T> for Atom {
    fn eq(&self, other: &T) -> bool {
        self.0.as_str() == other.as_ref()
    }
}

impl PartialEq<Atom> for String {
    fn eq(&self, other: &Atom) -> bool {
        self.as_str() == other.0.as_str()
    }
}

impl PartialEq<Atom> for &str {
    fn eq(&self, other: &Atom) -> bool {
        *self == other.0.as_str()
    }
}

impl fmt::Debug for Atom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.0.as_str(), f)
    }
}

impl fmt::Display for Atom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.0.as_str(), f)
    }
}
