use std::{
    borrow::{Borrow, Cow},
    fmt,
    mem::size_of,
    ops::Deref,
};

use oxc_allocator::{Allocator, CloneIn, CloneInSemanticIds, Dummy, FromIn, GetAllocator};

use crate::{Str, Wtf8, Wtf8Buf, Wtf8Chunks, Wtf8CodePoints, Wtf8CodeUnits, Wtf8Error};

/// An arena-backed, inlinable reference to a canonical WTF-8 string.
///
/// This is the WTF-8 counterpart to [`Str`]. Unlike `Str`, it deliberately
/// does not dereference to `str`, because lone surrogates are not valid UTF-8.
#[repr(transparent)]
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Wtf8Str<'a>(&'a Wtf8);

impl Wtf8Str<'static> {
    /// Return an empty static WTF-8 string.
    #[inline]
    pub const fn empty() -> Self {
        Self(Wtf8::from_str(""))
    }
}

impl<'a> Wtf8Str<'a> {
    /// Borrow ordinary UTF-8 as WTF-8 without allocating.
    #[inline]
    pub const fn from_str(value: &'a str) -> Self {
        Self(Wtf8::from_str(value))
    }

    /// Validate and borrow canonical WTF-8 bytes without allocating.
    ///
    /// # Errors
    ///
    /// Returns [`Wtf8Error`] if the bytes are not well-formed and canonical.
    #[inline]
    pub fn from_bytes(bytes: &'a [u8]) -> Result<Self, Wtf8Error> {
        Wtf8::from_bytes(bytes).map(Self)
    }

    /// Wrap an already validated WTF-8 slice.
    #[inline]
    pub const fn from_wtf8(value: &'a Wtf8) -> Self {
        Self(value)
    }

    /// Allocate ordinary UTF-8 into an arena as WTF-8.
    #[inline]
    pub fn from_str_in(value: &str, allocator: &impl GetAllocator<'a>) -> Self {
        Self::from_str(allocator.allocator().alloc_str(value))
    }

    /// Copy WTF-8 into an arena.
    #[inline]
    pub fn from_wtf8_in(value: &Wtf8, allocator: &impl GetAllocator<'a>) -> Self {
        let bytes = allocator.allocator().alloc_slice_copy(value.as_bytes());
        // SAFETY: `value` is canonical WTF-8 and the bytes were copied unchanged.
        Self(unsafe { Wtf8::from_bytes_unchecked(bytes) })
    }

    /// Validate bytes, copy them into an arena, and return an arena-backed value.
    ///
    /// # Errors
    ///
    /// Returns [`Wtf8Error`] without allocating if the bytes are invalid.
    pub fn from_bytes_in(
        bytes: &[u8],
        allocator: &impl GetAllocator<'a>,
    ) -> Result<Self, Wtf8Error> {
        let value = Wtf8::from_bytes(bytes)?;
        Ok(Self::from_wtf8_in(value, allocator))
    }

    /// Convert potentially ill-formed UTF-16 into arena-backed canonical WTF-8.
    pub fn from_ill_formed_utf16_in(units: &[u16], allocator: &impl GetAllocator<'a>) -> Self {
        let value = Wtf8Buf::from_ill_formed_utf16(units);
        Self::from_wtf8_in(&value, allocator)
    }

    /// Convert borrowed or owned WTF-8 into an arena-backed value.
    ///
    /// Borrowed data is reused directly; owned data is copied into the arena.
    #[inline]
    pub fn from_cow_in(value: &Cow<'a, Wtf8>, allocator: &impl GetAllocator<'a>) -> Self {
        match value {
            Cow::Borrowed(value) => Self::from_wtf8(value),
            Cow::Owned(value) => Self::from_wtf8_in(value, allocator),
        }
    }

    /// Return the underlying borrowed WTF-8 slice.
    #[inline]
    pub const fn as_wtf8(self) -> &'a Wtf8 {
        self.0
    }

    /// Return the underlying WTF-8 bytes.
    #[inline]
    pub const fn as_bytes(self) -> &'a [u8] {
        self.0.as_bytes()
    }

    /// Try to view this value as ordinary UTF-8.
    #[inline]
    pub fn as_str(self) -> Option<&'a str> {
        self.0.as_str()
    }

    /// Return whether this is a well-formed Unicode string with no lone surrogates.
    #[inline]
    pub fn is_well_formed_unicode(self) -> bool {
        self.0.is_well_formed_unicode()
    }

    /// Iterate over code points, including lone surrogates.
    #[inline]
    pub fn code_points(self) -> Wtf8CodePoints<'a> {
        self.0.code_points()
    }

    /// Iterate over maximal UTF-8 substrings and individual lone surrogates.
    #[inline]
    pub fn chunks(self) -> Wtf8Chunks<'a> {
        self.0.chunks()
    }

    /// Iterate over potentially ill-formed UTF-16 code units.
    #[inline]
    pub fn code_units(self) -> Wtf8CodeUnits<'a> {
        self.0.code_units()
    }

    /// Return the number of UTF-16 code units observed by JavaScript.
    #[inline]
    pub fn utf16_len(self) -> usize {
        self.0.utf16_len()
    }

    /// Return the UTF-16 code unit at `index`.
    #[inline]
    pub fn code_unit_at(self, index: usize) -> Option<u16> {
        self.0.code_unit_at(index)
    }
}

impl<'a> From<&'a str> for Wtf8Str<'a> {
    #[inline]
    fn from(value: &'a str) -> Self {
        Self::from_str(value)
    }
}

impl<'a> From<&'a Wtf8> for Wtf8Str<'a> {
    #[inline]
    fn from(value: &'a Wtf8) -> Self {
        Self::from_wtf8(value)
    }
}

impl<'a> From<Str<'a>> for Wtf8Str<'a> {
    #[inline]
    fn from(value: Str<'a>) -> Self {
        Self::from_str(value.as_str())
    }
}

impl PartialEq<str> for Wtf8Str<'_> {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl PartialEq<&str> for Wtf8Str<'_> {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl PartialEq<Wtf8Str<'_>> for str {
    #[inline]
    fn eq(&self, other: &Wtf8Str<'_>) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl PartialEq<Wtf8Str<'_>> for &str {
    #[inline]
    fn eq(&self, other: &Wtf8Str<'_>) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for Wtf8Str<'_> {
    type Cloned = Wtf8Str<'new_alloc>;

    #[inline]
    fn clone_in_impl(
        &self,
        _with_semantic_ids: CloneInSemanticIds,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        Wtf8Str::from_wtf8_in(self.0, &allocator)
    }
}

impl<'a> Dummy<'a> for Wtf8Str<'a> {
    #[inline]
    fn dummy(_allocator: &'a Allocator) -> Self {
        Wtf8Str::from_str("")
    }
}

impl<'alloc> FromIn<'alloc, &Wtf8Str<'alloc>> for Wtf8Str<'alloc> {
    #[inline]
    fn from_in(value: &Wtf8Str<'alloc>, _: &'alloc Allocator) -> Self {
        *value
    }
}

impl<'alloc> FromIn<'alloc, &str> for Wtf8Str<'alloc> {
    #[inline]
    fn from_in(value: &str, allocator: &'alloc Allocator) -> Self {
        Self::from_str_in(value, &allocator)
    }
}

impl<'alloc> FromIn<'alloc, String> for Wtf8Str<'alloc> {
    #[inline]
    fn from_in(value: String, allocator: &'alloc Allocator) -> Self {
        Self::from_str_in(&value, &allocator)
    }
}

impl<'alloc> FromIn<'alloc, Wtf8Buf> for Wtf8Str<'alloc> {
    #[inline]
    fn from_in(value: Wtf8Buf, allocator: &'alloc Allocator) -> Self {
        Self::from_wtf8_in(&value, &allocator)
    }
}

impl<'a> From<Wtf8Str<'a>> for &'a Wtf8 {
    #[inline]
    fn from(value: Wtf8Str<'a>) -> Self {
        value.as_wtf8()
    }
}

impl<'a> From<Wtf8Str<'a>> for Cow<'a, Wtf8> {
    #[inline]
    fn from(value: Wtf8Str<'a>) -> Self {
        Cow::Borrowed(value.as_wtf8())
    }
}

impl Deref for Wtf8Str<'_> {
    type Target = Wtf8;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl AsRef<Wtf8> for Wtf8Str<'_> {
    #[inline]
    fn as_ref(&self) -> &Wtf8 {
        self.0
    }
}

impl Borrow<Wtf8> for Wtf8Str<'_> {
    #[inline]
    fn borrow(&self) -> &Wtf8 {
        self.0
    }
}

impl fmt::Debug for Wtf8Str<'_> {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.0, formatter)
    }
}

const _: () = assert!(size_of::<Wtf8Str<'_>>() == size_of::<Str<'_>>());

#[cfg(test)]
mod tests {
    use oxc_allocator::Allocator;

    use super::*;

    #[test]
    fn has_same_size_as_str() {
        assert_eq!(size_of::<Wtf8Str<'_>>(), size_of::<Str<'_>>());
        assert_eq!(size_of::<Option<Wtf8Str<'_>>>(), size_of::<Option<Str<'_>>>());
    }

    #[test]
    fn allocates_utf8_and_lone_surrogates_in_arena() {
        let allocator = Allocator::new();
        let utf8 = Wtf8Str::from_str_in("hello 🔥", &&allocator);
        assert_eq!(utf8.as_str(), Some("hello 🔥"));

        let surrogate = Wtf8Str::from_ill_formed_utf16_in(&[0xD800], &&allocator);
        assert_eq!(surrogate.as_str(), None);
        assert_eq!(surrogate.code_units().collect::<Vec<_>>(), [0xD800]);
    }

    #[test]
    fn clone_in_copies_wtf8_bytes() {
        let source_allocator = Allocator::new();
        let target_allocator = Allocator::new();
        let source = Wtf8Str::from_ill_formed_utf16_in(&[0x61, 0xD800], &&source_allocator);
        let cloned = source.clone_in(&target_allocator);

        assert_eq!(source, cloned);
        assert_ne!(source.as_bytes().as_ptr(), cloned.as_bytes().as_ptr());
    }

    #[test]
    fn compares_with_utf8_strings() {
        let value = Wtf8Str::from_str("hello");
        assert_eq!(value, "hello");
        assert_eq!("hello", value);
        assert!(value != "world");
    }

    #[test]
    fn converts_cow_into_arena() {
        let allocator = Allocator::new();
        let source = Wtf8Buf::from_ill_formed_utf16(&[0xD800]);
        let value = Wtf8Str::from_cow_in(&Cow::Owned(source), &&allocator);
        assert_eq!(value.code_units().collect::<Vec<_>>(), [0xD800]);

        let borrowed_wtf8 = Wtf8::from_str("borrowed");
        let borrowed = Cow::Borrowed(borrowed_wtf8);
        let value = Wtf8Str::from_cow_in(&borrowed, &&allocator);
        assert_eq!(value.as_bytes().as_ptr(), borrowed_wtf8.as_bytes().as_ptr());
    }
}
