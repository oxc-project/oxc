use std::{borrow::Borrow, fmt, ops::Deref};

use compact_str::CompactString;
#[cfg(feature = "serde")]
use serde::Serialize;

/// Newtype for [`CompactString`]
#[derive(Clone, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Atom(CompactString);

const BASE54_CHARS: &[u8; 64] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ$_0123456789";

impl Atom {
    #[inline]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Get the shortest mangled name for a given n.
    /// Code adapted from [terser](https://github.com/terser/terser/blob/8b966d687395ab493d2c6286cc9dd38650324c11/lib/scope.js#L1041-L1051)
    pub fn base54(n: usize) -> Self {
        let mut num = n;
        let base = 54usize;
        let mut ret = CompactString::default();
        ret.push(BASE54_CHARS[num % base] as char);
        num /= base;
        let base = 64usize;
        while num > 0 {
            num -= 1;
            ret.push(BASE54_CHARS[num % base] as char);
            num /= base;
        }
        Self(ret)
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
