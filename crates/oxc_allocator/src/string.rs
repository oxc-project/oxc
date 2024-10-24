use crate::{Allocator, StringImpl};

#[cfg(any(feature = "serialize", test))]
use serde::{Serialize, Serializer};

/// A bump-allocated string.
pub struct String<'a>(StringImpl<'a>);

impl<'a> String<'a> {
    /// Constructs a new empty `String`.
    #[inline]
    pub fn new_in(allocator: &'a Allocator) -> Self {
        Self(StringImpl::new_in(&allocator.bump))
    }

    /// Constructs a `String` from a `&str`.
    #[inline]
    pub fn from_str_in(string: &str, allocator: &'a Allocator) -> Self {
        Self(StringImpl::from_str_in(string, &allocator.bump))
    }

    /// Constructs a new empty `String` with the specified capacity.
    #[inline]
    pub fn with_capacity_in(capacity: usize, allocator: &'a Allocator) -> Self {
        Self(StringImpl::with_capacity_in(capacity, &allocator.bump))
    }

    /// Converts a `String` into a `&str`.
    #[inline]
    pub fn into_str(self) -> &'a str {
        // First converts it to a `FixedBumpString` to suppress it trying to shrink its allocation.
        self.0.into_fixed_string().into_str()
    }

    /// Appends a given string slice to the end of this string.
    #[inline]
    pub fn push_str(&mut self, string: &str) {
        self.0.push_str(string);
    }

    /// Appends a given `char` to the end of this string.
    #[inline]
    pub fn push(&mut self, c: char) {
        self.0.push(c);
    }

    /// Extracts a string slice containing the entire `String`.
    #[inline]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[cfg(any(feature = "serialize", test))]
impl<'alloc> Serialize for String<'alloc> {
    #[inline]
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(s)
    }
}
