// Copyright (c) 2014 Simon Sapin
// Licensed under the MIT License
// Encoding routines adapted from https://github.com/SimonSapin/rust-wtf8.

//! JavaScript string types.
//!
//! JavaScript strings are sequences of UTF-16 code units and may contain lone
//! surrogates. [`JSStr`] stores them as canonical WTF-8 while keeping WTF-8 an
//! implementation detail of this crate.

use std::{
    fmt,
    hash::{Hash, Hasher},
    marker::PhantomData,
    ptr::NonNull,
    slice, str,
};

use oxc_allocator::{Allocator, CloneIn, CloneInSemanticIds, Dummy, FromIn, GetAllocator};

use crate::{Str, ident_hasher::ident_hash};

const HAS_LONE_SURROGATE: u32 = 1 << 31;
const HASH_MASK: u32 = !HAS_LONE_SURROGATE;

/// A decoded JavaScript string value.
///
/// Unlike [`char`], this type can represent a lone surrogate. It represents a
/// code point in the WTF-8 encoding, not one UTF-16 code unit.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct JSChar(u32);

impl JSChar {
    /// Create a JavaScript character from a value in the Unicode code point range.
    #[inline]
    pub const fn from_u32(value: u32) -> Option<Self> {
        if value <= 0x10_FFFF { Some(Self(value)) } else { None }
    }

    /// Create a JavaScript character from a Unicode scalar value.
    #[inline]
    pub const fn from_char(value: char) -> Self {
        Self(value as u32)
    }

    /// Return the numeric code point value.
    #[inline]
    pub const fn as_u32(self) -> u32 {
        self.0
    }

    /// Convert this value to a Unicode scalar value.
    ///
    /// Returns `None` for a lone surrogate.
    #[inline]
    pub const fn as_char(self) -> Option<char> {
        char::from_u32(self.0)
    }

    /// Return whether this value is a surrogate code point.
    #[inline]
    pub const fn is_surrogate(self) -> bool {
        is_surrogate(self.0)
    }

    /// Create a value without checking its range.
    ///
    /// # Safety
    ///
    /// `value` must not be greater than `U+10FFFF`.
    #[inline]
    const unsafe fn from_u32_unchecked(value: u32) -> Self {
        Self(value)
    }
}

impl From<char> for JSChar {
    #[inline]
    fn from(value: char) -> Self {
        Self::from_char(value)
    }
}

/// An immutable JavaScript string with a precomputed hash.
///
/// The string data is stored as canonical WTF-8. The final `u32` packs a
/// 31-bit hash with a flag recording whether the string contains a lone
/// surrogate.
///
/// On 64-bit platforms this type is 16 bytes, the same size as [`Str`] and
/// `&str`. On 32-bit platforms it is 12 bytes.
#[repr(C)]
pub struct JSStr<'a> {
    ptr: NonNull<u8>,
    len: u32,
    hash_and_flags: u32,
    _marker: PhantomData<&'a [u8]>,
}

impl JSStr<'static> {
    /// Return an empty JavaScript string.
    #[inline]
    pub const fn empty() -> Self {
        // SAFETY: `NonNull::dangling()` is valid for reads of zero bytes, the
        // empty hash is 0, and an empty string has no lone surrogate.
        unsafe { Self::from_raw_parts(NonNull::dangling(), 0, 0) }
    }
}

impl<'a> JSStr<'a> {
    /// Allocate a UTF-8 string in an arena.
    #[inline]
    pub fn from_str_in(value: &str, allocator: &impl GetAllocator<'a>) -> Self {
        Self::from(allocator.allocator().alloc_str(value))
    }

    /// Convert potentially ill-formed UTF-16 into an arena-backed JavaScript string.
    pub fn from_utf16_in(units: &[u16], allocator: &impl GetAllocator<'a>) -> Self {
        let (bytes, has_lone_surrogate) = encode_utf16(units);
        Self::copy_bytes_in(&bytes, has_lone_surrogate, allocator)
    }

    /// Concatenate two JavaScript strings into an arena allocation.
    ///
    /// This operation is boundary-aware. A high surrogate at the end of
    /// `self` and a low surrogate at the start of `other` are combined into
    /// the canonical UTF-8 encoding of the resulting scalar value.
    ///
    /// # Panics
    ///
    /// Panics if the resulting byte length overflows `usize` or exceeds the
    /// maximum length supported by [`JSStr`].
    pub fn concat_in<'new_alloc>(
        self,
        other: JSStr<'_>,
        allocator: &impl GetAllocator<'new_alloc>,
    ) -> JSStr<'new_alloc> {
        let boundary_pair = self.final_lead_surrogate().zip(other.initial_trail_surrogate());
        let pair_saving = usize::from(boundary_pair.is_some()) * 2;
        let capacity = self
            .len()
            .checked_add(other.len())
            .and_then(|len| len.checked_sub(pair_saving))
            .expect("JavaScript string length overflow");
        let mut bytes = Vec::with_capacity(capacity);

        if let Some((lead, trail)) = boundary_pair {
            bytes.extend_from_slice(&self.as_bytes()[..self.len() - 3]);
            push_scalar(&mut bytes, decode_surrogate_pair(lead, trail));
            bytes.extend_from_slice(&other.as_bytes()[3..]);
        } else {
            bytes.extend_from_slice(self.as_bytes());
            bytes.extend_from_slice(other.as_bytes());
        }

        debug_assert_eq!(bytes.len(), capacity);
        let has_lone_surrogate = if boundary_pair.is_some() {
            contains_lone_surrogate(&bytes)
        } else {
            self.has_lone_surrogate() || other.has_lone_surrogate()
        };
        Self::copy_bytes_in(&bytes, has_lone_surrogate, allocator)
    }

    /// Return the canonical WTF-8 bytes.
    ///
    /// These bytes are an internal representation and must not be emitted as
    /// UTF-8 interchange text without handling lone surrogates.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub fn as_bytes(self) -> &'a [u8] {
        // SAFETY: All constructors guarantee that `ptr` points to `len` bytes
        // of immutable storage which remains valid for `'a`.
        unsafe { slice::from_raw_parts(self.ptr.as_ptr(), self.len as usize) }
    }

    /// Return this value as UTF-8 if it does not contain a lone surrogate.
    #[inline]
    pub fn as_str(self) -> Option<&'a str> {
        if self.has_lone_surrogate() {
            None
        } else {
            // SAFETY: Canonical WTF-8 without lone surrogates is UTF-8.
            Some(unsafe { str::from_utf8_unchecked(self.as_bytes()) })
        }
    }

    /// Return the number of bytes in the WTF-8 representation.
    #[inline]
    pub const fn len(self) -> usize {
        self.len as usize
    }

    /// Return whether this string is empty.
    #[inline]
    pub const fn is_empty(self) -> bool {
        self.len == 0
    }

    /// Return the cached 31-bit content hash.
    #[inline]
    pub const fn hash(self) -> u32 {
        self.hash_and_flags & HASH_MASK
    }

    /// Return whether this string contains at least one lone surrogate.
    #[inline]
    pub const fn has_lone_surrogate(self) -> bool {
        self.hash_and_flags & HAS_LONE_SURROGATE != 0
    }

    /// Iterate over decoded values, including lone surrogates.
    #[inline]
    pub fn code_points(self) -> impl Iterator<Item = JSChar> + 'a {
        JSCodePoints { remaining: self.as_bytes() }
    }

    #[inline]
    fn from_valid_bytes(bytes: &'a [u8], has_lone_surrogate: bool) -> Self {
        let len = u32::try_from(bytes.len()).expect("JavaScript strings cannot exceed 4 GiB");
        let hash = ident_hash(bytes) & HASH_MASK;
        let hash_and_flags = hash | if has_lone_surrogate { HAS_LONE_SURROGATE } else { 0 };
        let ptr = NonNull::from(bytes).cast::<u8>();

        debug_assert_eq!(contains_lone_surrogate(bytes), has_lone_surrogate);

        // SAFETY: `ptr` and `len` came from `bytes`, which remains immutable
        // and valid for `'a`. Hash and flag were computed above.
        unsafe { Self::from_raw_parts(ptr, len, hash_and_flags) }
    }

    #[inline]
    fn copy_bytes_in<'new_alloc>(
        bytes: &[u8],
        has_lone_surrogate: bool,
        allocator: &impl GetAllocator<'new_alloc>,
    ) -> JSStr<'new_alloc> {
        let bytes = allocator.allocator().alloc_slice_copy(bytes);
        JSStr::from_valid_bytes(bytes, has_lone_surrogate)
    }

    /// Create a string from raw components.
    ///
    /// # Safety
    ///
    /// - `ptr` must point to `len` bytes of canonical WTF-8 which are valid
    ///   and immutable for `'a`.
    /// - The hash and lone-surrogate flag must match those bytes.
    #[inline]
    const unsafe fn from_raw_parts(ptr: NonNull<u8>, len: u32, hash_and_flags: u32) -> Self {
        Self { ptr, len, hash_and_flags, _marker: PhantomData }
    }

    #[inline]
    #[expect(clippy::cast_possible_truncation, reason = "a three-byte sequence is at most U+FFFF")]
    fn initial_trail_surrogate(self) -> Option<u16> {
        let bytes = self.as_bytes();
        if bytes.len() < 3 || bytes[0] != 0xED || !(0xB0..=0xBF).contains(&bytes[1]) {
            return None;
        }
        Some(decode_three_byte_sequence(bytes[0], bytes[1], bytes[2]) as u16)
    }

    #[inline]
    #[expect(clippy::cast_possible_truncation, reason = "a three-byte sequence is at most U+FFFF")]
    fn final_lead_surrogate(self) -> Option<u16> {
        let bytes = self.as_bytes();
        if bytes.len() < 3 {
            return None;
        }
        let bytes = &bytes[bytes.len() - 3..];
        if bytes[0] != 0xED || !(0xA0..=0xAF).contains(&bytes[1]) {
            return None;
        }
        Some(decode_three_byte_sequence(bytes[0], bytes[1], bytes[2]) as u16)
    }
}

// SAFETY: `JSStr` is conceptually an immutable `&[u8]`, which is Send + Sync.
// `NonNull` is !Send/!Sync, but this type only stores a pointer to borrowed data.
unsafe impl Send for JSStr<'_> {}
// SAFETY: See above.
unsafe impl Sync for JSStr<'_> {}

// We cannot derive `Clone` or `Copy` because the pointer is stored as `NonNull`.
#[expect(clippy::expl_impl_clone_on_copy)]
impl Clone for JSStr<'_> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl Copy for JSStr<'_> {}

impl PartialEq for JSStr<'_> {
    /// Fast-reject equality using length, hash, and flag before comparing bytes.
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.len == other.len
            && self.hash_and_flags == other.hash_and_flags
            && self.as_bytes() == other.as_bytes()
    }
}

impl Eq for JSStr<'_> {}

impl PartialEq<str> for JSStr<'_> {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.as_str().is_some_and(|value| value == other)
    }
}

impl PartialEq<&str> for JSStr<'_> {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        self == *other
    }
}

impl PartialEq<JSStr<'_>> for str {
    #[inline]
    fn eq(&self, other: &JSStr<'_>) -> bool {
        other == self
    }
}

impl PartialEq<JSStr<'_>> for &str {
    #[inline]
    fn eq(&self, other: &JSStr<'_>) -> bool {
        other == *self
    }
}

impl Hash for JSStr<'_> {
    #[inline]
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        let packed = u64::from(self.len) | (u64::from((*self).hash()) << 32);
        hasher.write_u64(packed);
    }
}

impl fmt::Debug for JSStr<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("\"")?;
        for value in self.code_points() {
            if let Some(value) = value.as_char() {
                for escaped in value.escape_debug() {
                    write!(formatter, "{escaped}")?;
                }
            } else {
                write!(formatter, "\\u{{{:04X}}}", value.as_u32())?;
            }
        }
        formatter.write_str("\"")
    }
}

impl<'a> From<&'a str> for JSStr<'a> {
    #[inline]
    fn from(value: &'a str) -> Self {
        Self::from_valid_bytes(value.as_bytes(), false)
    }
}

impl<'a> From<Str<'a>> for JSStr<'a> {
    #[inline]
    fn from(value: Str<'a>) -> Self {
        Self::from(value.as_str())
    }
}

impl<'new_alloc> CloneIn<'new_alloc> for JSStr<'_> {
    type Cloned = JSStr<'new_alloc>;

    #[inline]
    fn clone_in_impl(
        &self,
        _with_semantic_ids: CloneInSemanticIds,
        allocator: &'new_alloc Allocator,
    ) -> Self::Cloned {
        JSStr::copy_bytes_in(self.as_bytes(), self.has_lone_surrogate(), &allocator)
    }
}

impl<'a> Dummy<'a> for JSStr<'a> {
    #[inline]
    fn dummy(_allocator: &'a Allocator) -> Self {
        JSStr::empty()
    }
}

impl<'alloc> FromIn<'alloc, &JSStr<'alloc>> for JSStr<'alloc> {
    #[inline]
    fn from_in(value: &JSStr<'alloc>, _allocator: &'alloc Allocator) -> Self {
        *value
    }
}

impl<'alloc> FromIn<'alloc, &str> for JSStr<'alloc> {
    #[inline]
    fn from_in(value: &str, allocator: &'alloc Allocator) -> Self {
        Self::from_str_in(value, &allocator)
    }
}

impl<'alloc> FromIn<'alloc, String> for JSStr<'alloc> {
    #[inline]
    fn from_in(value: String, allocator: &'alloc Allocator) -> Self {
        Self::from_str_in(&value, &allocator)
    }
}

struct JSCodePoints<'a> {
    remaining: &'a [u8],
}

impl Iterator for JSCodePoints<'_> {
    type Item = JSChar;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining.is_empty() {
            return None;
        }
        let (value, len) = decode_next_valid_wtf8(self.remaining);
        self.remaining = &self.remaining[len..];
        // SAFETY: Valid WTF-8 only contains values up to U+10FFFF.
        Some(unsafe { JSChar::from_u32_unchecked(value) })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining.len().div_ceil(4), Some(self.remaining.len()))
    }
}

fn encode_utf16(units: &[u16]) -> (Vec<u8>, bool) {
    let mut bytes = Vec::with_capacity(units.len());
    let mut has_lone_surrogate = false;
    let mut index = 0;

    while index < units.len() {
        let unit = units[index];
        if is_lead_surrogate(u32::from(unit)) {
            if let Some(&trail) = units.get(index + 1)
                && is_trail_surrogate(u32::from(trail))
            {
                push_scalar(&mut bytes, decode_surrogate_pair(unit, trail));
                index += 2;
                continue;
            }
            push_surrogate(&mut bytes, unit);
            has_lone_surrogate = true;
        } else if is_trail_surrogate(u32::from(unit)) {
            push_surrogate(&mut bytes, unit);
            has_lone_surrogate = true;
        } else {
            // SAFETY: A non-surrogate `u16` is a Unicode scalar value.
            push_scalar(&mut bytes, unsafe { char::from_u32_unchecked(u32::from(unit)) });
        }
        index += 1;
    }

    (bytes, has_lone_surrogate)
}

#[inline]
fn push_scalar(bytes: &mut Vec<u8>, value: char) {
    let mut encoded = [0; 4];
    bytes.extend_from_slice(value.encode_utf8(&mut encoded).as_bytes());
}

#[inline]
#[expect(clippy::cast_possible_truncation, reason = "surrogate encodings only use the low 16 bits")]
fn push_surrogate(bytes: &mut Vec<u8>, value: u16) {
    debug_assert!(is_surrogate(u32::from(value)));
    let value = u32::from(value);
    bytes.push(0xE0 | (value >> 12) as u8);
    bytes.push(0x80 | ((value >> 6) & 0x3F) as u8);
    bytes.push(0x80 | (value & 0x3F) as u8);
}

#[inline]
fn contains_lone_surrogate(mut bytes: &[u8]) -> bool {
    while !bytes.is_empty() {
        let (value, len) = decode_next_valid_wtf8(bytes);
        if is_surrogate(value) {
            return true;
        }
        bytes = &bytes[len..];
    }
    false
}

#[inline]
fn decode_next_valid_wtf8(bytes: &[u8]) -> (u32, usize) {
    let first = bytes[0];
    match first {
        0x00..=0x7F => (u32::from(first), 1),
        0xC2..=0xDF => ((u32::from(first & 0x1F) << 6) | u32::from(bytes[1] & 0x3F), 2),
        0xE0..=0xEF => (decode_three_byte_sequence(first, bytes[1], bytes[2]), 3),
        _ => (
            (u32::from(first & 0x07) << 18)
                | (u32::from(bytes[1] & 0x3F) << 12)
                | (u32::from(bytes[2] & 0x3F) << 6)
                | u32::from(bytes[3] & 0x3F),
            4,
        ),
    }
}

#[inline]
const fn decode_three_byte_sequence(first: u8, second: u8, third: u8) -> u32 {
    ((first as u32 & 0x0F) << 12) | ((second as u32 & 0x3F) << 6) | (third as u32 & 0x3F)
}

#[inline]
const fn is_surrogate(value: u32) -> bool {
    matches!(value, 0xD800..=0xDFFF)
}

#[inline]
const fn is_lead_surrogate(value: u32) -> bool {
    matches!(value, 0xD800..=0xDBFF)
}

#[inline]
const fn is_trail_surrogate(value: u32) -> bool {
    matches!(value, 0xDC00..=0xDFFF)
}

#[inline]
fn decode_surrogate_pair(lead: u16, trail: u16) -> char {
    debug_assert!(is_lead_surrogate(u32::from(lead)));
    debug_assert!(is_trail_surrogate(u32::from(trail)));
    let value = 0x1_0000 + ((u32::from(lead) - 0xD800) << 10) + u32::from(trail) - 0xDC00;
    // SAFETY: A valid surrogate pair always produces a Unicode scalar value.
    unsafe { char::from_u32_unchecked(value) }
}

#[cfg(test)]
mod tests {
    use std::{
        fmt::Display,
        mem::{needs_drop, size_of},
        ops::Deref,
    };

    use oxc_allocator::{Allocator, CloneIn};
    use oxc_data_structures::types::implements;

    use super::*;

    #[test]
    fn layout() {
        #[cfg(target_pointer_width = "64")]
        {
            assert_eq!(size_of::<JSStr<'_>>(), 16);
            assert_eq!(size_of::<Option<JSStr<'_>>>(), 16);
            assert_eq!(size_of::<JSStr<'_>>(), size_of::<Str<'_>>());
        }
        #[cfg(target_pointer_width = "32")]
        assert_eq!(size_of::<JSStr<'_>>(), 12);
    }

    #[test]
    fn trait_contract() {
        assert!(implements!(JSStr: Copy));
        assert!(implements!(JSStr: Send));
        assert!(implements!(JSStr: Sync));
        assert!(implements!(JSStr: !Deref<Target = str>));
        assert!(implements!(JSStr: !AsRef<str>));
        assert!(implements!(JSStr: !Display));
        assert!(!needs_drop::<JSStr<'_>>());
    }

    #[test]
    fn js_char() {
        assert_eq!(JSChar::from_char('A').as_char(), Some('A'));
        assert_eq!(JSChar::from_u32(0x1F525).unwrap().as_char(), Some('🔥'));

        let surrogate = JSChar::from_u32(0xD800).unwrap();
        assert!(surrogate.is_surrogate());
        assert_eq!(surrogate.as_char(), None);
        assert!(JSChar::from_u32(0x11_0000).is_none());
    }

    #[test]
    fn utf8_fast_path() {
        let value = JSStr::from("hello 🔥");
        assert_eq!(value.as_str(), Some("hello 🔥"));
        assert!(!value.has_lone_surrogate());
        assert_eq!(
            value.code_points().map(JSChar::as_u32).collect::<Vec<_>>(),
            [0x68, 0x65, 0x6C, 0x6C, 0x6F, 0x20, 0x1F525,]
        );
    }

    #[test]
    fn ill_formed_utf16() {
        let allocator = Allocator::new();
        let value = JSStr::from_utf16_in(&[0x61, 0xD800, 0x62, 0xDC00], &&allocator);

        assert!(value.has_lone_surrogate());
        assert_eq!(value.as_str(), None);
        assert_eq!(
            value.code_points().map(JSChar::as_u32).collect::<Vec<_>>(),
            [0x61, 0xD800, 0x62, 0xDC00,]
        );
        assert_eq!(format!("{value:?}"), "\"a\\u{D800}b\\u{DC00}\"");
    }

    #[test]
    fn utf16_pair_becomes_scalar() {
        let allocator = Allocator::new();
        let value = JSStr::from_utf16_in(&[0xD83D, 0xDD25], &&allocator);

        assert_eq!(value.as_str(), Some("🔥"));
        assert!(!value.has_lone_surrogate());
        assert_eq!(value.code_points().map(JSChar::as_u32).collect::<Vec<_>>(), [0x1F525]);
    }

    #[test]
    fn concatenation_combines_boundary_pair() {
        let allocator = Allocator::new();
        let lead = JSStr::from_utf16_in(&[0xD83D], &&allocator);
        let trail = JSStr::from_utf16_in(&[0xDD25], &&allocator);
        let result = lead.concat_in(trail, &&allocator);

        assert_eq!(result.as_str(), Some("🔥"));
        assert!(!result.has_lone_surrogate());
        assert_eq!(result.as_bytes(), "🔥".as_bytes());
    }

    #[test]
    fn concatenation_preserves_other_lone_surrogates() {
        let allocator = Allocator::new();
        let left = JSStr::from_utf16_in(&[0xD800, 0xD83D], &&allocator);
        let right = JSStr::from_utf16_in(&[0xDD25, 0xDC00], &&allocator);
        let result = left.concat_in(right, &&allocator);

        assert!(result.has_lone_surrogate());
        assert_eq!(
            result.code_points().map(JSChar::as_u32).collect::<Vec<_>>(),
            [0xD800, 0x1F525, 0xDC00,]
        );
    }

    #[test]
    fn concatenation_without_boundary_pair_is_byte_copy() {
        let allocator = Allocator::new();
        let left = JSStr::from_utf16_in(&[0xD800, 0x61], &&allocator);
        let right = JSStr::from("🔥b");
        let result = left.concat_in(right, &&allocator);

        assert!(result.has_lone_surrogate());
        assert_eq!(
            result.code_points().map(JSChar::as_u32).collect::<Vec<_>>(),
            [0xD800, 0x61, 0x1F525, 0x62,]
        );
    }

    #[test]
    fn concatenation_matches_utf16_concatenation() {
        let allocator = Allocator::new();
        let cases = [
            vec![],
            vec![0x61],
            vec![0xD800],
            vec![0xDC00],
            vec![0xD83D, 0xDD25],
            vec![0xD800, 0xD83D, 0xDD25, 0xDC00],
            vec![0xD800, 0xD800, 0xDC00, 0xDC00],
        ];

        for units in cases {
            let expected = JSStr::from_utf16_in(&units, &&allocator);
            for split in 0..=units.len() {
                let left = JSStr::from_utf16_in(&units[..split], &&allocator);
                let right = JSStr::from_utf16_in(&units[split..], &&allocator);
                let actual = left.concat_in(right, &&allocator);

                assert_eq!(actual, expected, "failed at split {split} for {units:04X?}");
                assert_eq!(actual.has_lone_surrogate(), expected.has_lone_surrogate());
            }
        }
    }

    #[test]
    fn equality_uses_hash_then_bytes() {
        let left = JSStr::from("string-a");
        let same = JSStr::from("string-a");
        let mut collision = JSStr::from("string-b");

        assert_eq!(left, same);
        assert_eq!(left.hash(), same.hash());

        // Simulate a real hash collision. The final byte comparison must still
        // reject different strings of the same length.
        collision.hash_and_flags = left.hash_and_flags;
        assert_eq!(left.hash(), collision.hash());
        assert_ne!(left, collision);
    }

    #[test]
    fn clone_in_preserves_value_and_metadata() {
        let source_allocator = Allocator::new();
        let destination_allocator = Allocator::new();
        let source = JSStr::from_utf16_in(&[0x61, 0xD800], &&source_allocator);
        let cloned = source.clone_in(&destination_allocator);

        assert_eq!(source, cloned);
        assert_eq!(source.hash_and_flags, cloned.hash_and_flags);
        assert_ne!(source.as_bytes().as_ptr(), cloned.as_bytes().as_ptr());
    }

    #[test]
    fn every_utf16_code_unit_round_trips() {
        for unit in 0..=u16::MAX {
            let (bytes, has_lone_surrogate) = encode_utf16(&[unit]);
            let value = JSStr::from_valid_bytes(&bytes, has_lone_surrogate);
            assert_eq!(to_utf16(value), [unit]);
        }
    }

    #[test]
    fn representative_utf16_sequences_round_trip() {
        for units in [
            vec![],
            vec![0x61, 0x62],
            vec![0xD800],
            vec![0xDC00],
            vec![0xD83D, 0xDD25],
            vec![0xD800, 0xD83D, 0xDD25, 0xDC00],
            vec![0xD800, 0xD800, 0xDC00, 0xDC00],
        ] {
            let (bytes, has_lone_surrogate) = encode_utf16(&units);
            let value = JSStr::from_valid_bytes(&bytes, has_lone_surrogate);
            assert_eq!(to_utf16(value), units);
        }
    }

    #[expect(
        clippy::cast_possible_truncation,
        reason = "values are range-checked before conversion"
    )]
    fn to_utf16(value: JSStr<'_>) -> Vec<u16> {
        let mut units = Vec::new();
        for value in value.code_points() {
            let value = value.as_u32();
            if value <= 0xFFFF {
                units.push(value as u16);
            } else {
                let supplementary = value - 0x1_0000;
                units.push(0xD800 | (supplementary >> 10) as u16);
                units.push(0xDC00 | (supplementary & 0x3FF) as u16);
            }
        }
        units
    }
}
