// Copyright (c) 2014 Simon Sapin
// Licensed under the MIT License
// Original source: https://github.com/SimonSapin/rust-wtf8

//! Core WTF-8 string types.
//!
//! This module is adapted for Oxc from `rust-wtf8` and SWC's maintained fork.
//! WTF-8 is an internal encoding and must not be emitted as interchange text.

use std::{
    borrow::{Borrow, Cow},
    error::Error,
    fmt,
    hash::{Hash, Hasher},
    mem::transmute,
    ops::{Deref, Range},
    str,
};

/// A Unicode code point, including surrogate code points.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CodePoint(u32);

impl CodePoint {
    /// Create a code point if `value` is in the Unicode code point range.
    #[inline]
    pub const fn from_u32(value: u32) -> Option<Self> {
        if value <= 0x10_FFFF { Some(Self(value)) } else { None }
    }

    /// Create a code point without checking its range.
    ///
    /// # Safety
    ///
    /// `value` must be no greater than `0x10_FFFF`.
    #[inline]
    pub const unsafe fn from_u32_unchecked(value: u32) -> Self {
        Self(value)
    }

    /// Create a code point from a Unicode scalar value.
    #[inline]
    pub const fn from_char(value: char) -> Self {
        Self(value as u32)
    }

    /// Return the numeric code point value.
    #[inline]
    pub const fn to_u32(self) -> u32 {
        self.0
    }

    /// Convert this code point to a Unicode scalar value.
    ///
    /// Returns `None` for surrogate code points.
    #[inline]
    pub const fn to_char(self) -> Option<char> {
        char::from_u32(self.0)
    }

    /// Convert to a Unicode scalar value, replacing surrogates with `U+FFFD`.
    #[inline]
    pub const fn to_char_lossy(self) -> char {
        match self.to_char() {
            Some(value) => value,
            None => char::REPLACEMENT_CHARACTER,
        }
    }
}

impl From<char> for CodePoint {
    #[inline]
    fn from(value: char) -> Self {
        Self::from_char(value)
    }
}

/// Error returned when bytes are not canonical, well-formed WTF-8.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Wtf8Error {
    valid_up_to: usize,
    kind: Wtf8ErrorKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Wtf8ErrorKind {
    InvalidEncoding,
    NonCanonicalSurrogatePair,
}

impl Wtf8Error {
    /// Byte offset before which the input was valid WTF-8.
    #[inline]
    pub const fn valid_up_to(self) -> usize {
        self.valid_up_to
    }

    /// Whether the input encoded an adjacent surrogate pair as two code points.
    #[inline]
    pub const fn is_non_canonical_surrogate_pair(self) -> bool {
        matches!(self.kind, Wtf8ErrorKind::NonCanonicalSurrogatePair)
    }
}

impl fmt::Display for Wtf8Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            Wtf8ErrorKind::InvalidEncoding => {
                write!(formatter, "invalid WTF-8 at byte {}", self.valid_up_to)
            }
            Wtf8ErrorKind::NonCanonicalSurrogatePair => {
                write!(formatter, "non-canonical WTF-8 surrogate pair at byte {}", self.valid_up_to)
            }
        }
    }
}

impl Error for Wtf8Error {}

/// A borrowed slice of canonical, well-formed WTF-8 data.
///
/// This is analogous to `str`, but may additionally contain unpaired surrogate
/// code points. It is unsized and normally used as `&Wtf8`.
#[repr(transparent)]
#[derive(Eq, Ord, PartialEq, PartialOrd)]
pub struct Wtf8([u8]);

impl Wtf8 {
    /// Reinterpret a UTF-8 string as WTF-8.
    ///
    /// Valid UTF-8 is always canonical WTF-8.
    #[inline]
    pub const fn from_str(value: &str) -> &Self {
        // SAFETY: `Wtf8` is transparent over `[u8]`, and UTF-8 is a subset of WTF-8.
        unsafe { transmute(value.as_bytes()) }
    }

    /// Validate a byte slice as canonical WTF-8.
    ///
    /// # Errors
    ///
    /// Returns [`Wtf8Error`] if the bytes are not well-formed and canonical.
    #[inline]
    pub fn from_bytes(bytes: &[u8]) -> Result<&Self, Wtf8Error> {
        validate_wtf8(bytes)?;
        // SAFETY: The bytes were validated above.
        Ok(unsafe { Self::from_bytes_unchecked(bytes) })
    }

    /// Reinterpret bytes as WTF-8 without validation.
    ///
    /// # Safety
    ///
    /// `bytes` must contain canonical, well-formed WTF-8 for the entire
    /// lifetime of the returned reference.
    #[inline]
    pub(crate) unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &Self {
        // SAFETY: `Wtf8` is transparent over `[u8]`; validity is the caller's contract.
        unsafe { &*(std::ptr::from_ref(bytes) as *const Self) }
    }

    /// Return the underlying WTF-8 bytes.
    ///
    /// WTF-8 is an internal encoding; these bytes must not be emitted directly
    /// as UTF-8 interchange text.
    #[inline]
    pub const fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Return the length in WTF-8 bytes.
    #[inline]
    pub const fn len(&self) -> usize {
        self.0.len()
    }

    /// Return whether this string is empty.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Return whether this string contains only ASCII.
    #[inline]
    pub const fn is_ascii(&self) -> bool {
        self.0.is_ascii()
    }

    /// Try to view this value as ordinary UTF-8.
    ///
    /// Returns `None` if the value contains any lone surrogate.
    #[inline]
    pub fn as_str(&self) -> Option<&str> {
        str::from_utf8(&self.0).ok()
    }

    /// Return whether this is a well-formed Unicode string with no lone surrogates.
    #[inline]
    pub fn is_well_formed_unicode(&self) -> bool {
        self.as_str().is_some()
    }

    /// Iterate over Unicode code points, including lone surrogates.
    #[inline]
    pub fn code_points(&self) -> Wtf8CodePoints<'_> {
        Wtf8CodePoints { remaining: &self.0 }
    }

    /// Iterate over maximal UTF-8 substrings and individual lone surrogates.
    ///
    /// This is the preferred interface for code generation and serialization:
    /// UTF-8 substrings can be copied in bulk, while lone surrogates can be
    /// escaped without ever treating the underlying WTF-8 bytes as UTF-8.
    #[inline]
    pub fn chunks(&self) -> Wtf8Chunks<'_> {
        Wtf8Chunks { remaining: &self.0 }
    }

    /// Iterate over the corresponding potentially ill-formed UTF-16 code units.
    #[inline]
    pub fn code_units(&self) -> Wtf8CodeUnits<'_> {
        Wtf8CodeUnits { code_points: self.code_points(), pending_trail: None }
    }

    /// Return the number of UTF-16 code units observed by JavaScript.
    #[inline]
    pub fn utf16_len(&self) -> usize {
        self.code_units().count()
    }

    /// Return the UTF-16 code unit at `index`.
    #[inline]
    pub fn code_unit_at(&self, index: usize) -> Option<u16> {
        self.code_units().nth(index)
    }

    /// Copy a UTF-16 code-unit range into a new canonical WTF-8 buffer.
    ///
    /// Returns `None` if the range is reversed or outside the string. This may
    /// allocate even when the source is ordinary UTF-8 because a range may
    /// split a supplementary code point into a lone surrogate.
    pub fn slice_code_units(&self, range: Range<usize>) -> Option<Wtf8Buf> {
        if range.start > range.end {
            return None;
        }

        let mut units = self.code_units();
        for _ in 0..range.start {
            units.next()?;
        }

        let mut result = Wtf8Buf::with_capacity(range.end - range.start);
        for _ in range {
            let unit = units.next()?;
            // SAFETY: Every u16 is a Unicode code point, including surrogates.
            result.push(unsafe { CodePoint::from_u32_unchecked(u32::from(unit)) });
        }
        Some(result)
    }

    /// Convert to UTF-8, replacing lone surrogates with `U+FFFD`.
    pub fn to_string_lossy(&self) -> Cow<'_, str> {
        if let Some(value) = self.as_str() {
            return Cow::Borrowed(value);
        }

        let mut value = String::with_capacity(self.len());
        for code_point in self.code_points() {
            value.push(code_point.to_char_lossy());
        }
        Cow::Owned(value)
    }

    #[inline]
    #[expect(clippy::cast_possible_truncation, reason = "a three-byte sequence is at most U+FFFF")]
    fn initial_trail_surrogate(&self) -> Option<u16> {
        let bytes = self.as_bytes();
        if bytes.len() < 3 || bytes[0] != 0xED || !(0xB0..=0xBF).contains(&bytes[1]) {
            return None;
        }
        Some(decode_three_byte_sequence(bytes[0], bytes[1], bytes[2]) as u16)
    }
}

impl fmt::Debug for Wtf8 {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("\"")?;
        for code_point in self.code_points() {
            if let Some(value) = code_point.to_char() {
                for escaped in value.escape_debug() {
                    formatter.write_fmt(format_args!("{escaped}"))?;
                }
            } else {
                write!(formatter, "\\u{{{:04X}}}", code_point.to_u32())?;
            }
        }
        formatter.write_str("\"")
    }
}

impl Hash for Wtf8 {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl ToOwned for Wtf8 {
    type Owned = Wtf8Buf;

    #[inline]
    fn to_owned(&self) -> Self::Owned {
        Wtf8Buf { bytes: self.0.to_vec() }
    }
}

/// An owned, growable canonical WTF-8 string.
#[derive(Clone, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct Wtf8Buf {
    bytes: Vec<u8>,
}

impl Wtf8Buf {
    /// Create an empty buffer.
    #[inline]
    pub const fn new() -> Self {
        Self { bytes: Vec::new() }
    }

    /// Create an empty buffer with byte capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self { bytes: Vec::with_capacity(capacity) }
    }

    /// Convert an owned UTF-8 string without copying.
    #[inline]
    pub fn from_string(value: String) -> Self {
        Self { bytes: value.into_bytes() }
    }

    /// Validate owned bytes as canonical WTF-8.
    ///
    /// # Errors
    ///
    /// Returns the original bytes and a [`Wtf8Error`] if validation fails.
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, (Vec<u8>, Wtf8Error)> {
        match validate_wtf8(&bytes) {
            Ok(()) => Ok(Self { bytes }),
            Err(error) => Err((bytes, error)),
        }
    }

    /// Losslessly convert potentially ill-formed UTF-16 to canonical WTF-8.
    pub fn from_ill_formed_utf16(units: &[u16]) -> Self {
        let mut result = Self::with_capacity(units.len());
        for decoded in char::decode_utf16(units.iter().copied()) {
            match decoded {
                Ok(value) => result.push_char(value),
                Err(error) => {
                    // SAFETY: `unpaired_surrogate` is a u16 and therefore a valid code point.
                    let value = unsafe {
                        CodePoint::from_u32_unchecked(u32::from(error.unpaired_surrogate()))
                    };
                    // `decode_utf16` has already combined every adjacent valid pair.
                    result.push_code_point_without_boundary_check(value);
                }
            }
        }
        result
    }

    /// Reserve space for additional WTF-8 bytes.
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.bytes.reserve(additional);
    }

    /// Return the buffer capacity in bytes.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.bytes.capacity()
    }

    /// Empty the buffer without changing capacity.
    #[inline]
    pub fn clear(&mut self) {
        self.bytes.clear();
    }

    /// Append ordinary UTF-8.
    #[inline]
    pub fn push_str(&mut self, value: &str) {
        self.bytes.extend_from_slice(value.as_bytes());
    }

    /// Append a Unicode scalar value.
    #[inline]
    pub fn push_char(&mut self, value: char) {
        self.push_code_point_without_boundary_check(CodePoint::from_char(value));
    }

    /// Append one code point and canonicalize a new surrogate pair boundary.
    #[expect(clippy::cast_possible_truncation, reason = "the guarded value is a trail surrogate")]
    pub fn push(&mut self, value: CodePoint) {
        if is_trail_surrogate(value.to_u32())
            && let Some(lead) = self.final_lead_surrogate()
        {
            self.bytes.truncate(self.bytes.len() - 3);
            self.push_char(decode_surrogate_pair(lead, value.to_u32() as u16));
            return;
        }
        self.push_code_point_without_boundary_check(value);
    }

    /// Append WTF-8 and canonicalize a new surrogate pair at the boundary.
    pub fn push_wtf8(&mut self, value: &Wtf8) {
        match (self.final_lead_surrogate(), value.initial_trail_surrogate()) {
            (Some(lead), Some(trail)) => {
                self.bytes.truncate(self.bytes.len() - 3);
                self.bytes.reserve(value.len() + 1);
                self.push_char(decode_surrogate_pair(lead, trail));
                self.bytes.extend_from_slice(&value.as_bytes()[3..]);
            }
            _ => self.bytes.extend_from_slice(value.as_bytes()),
        }
    }

    /// Consume this buffer and return its bytes.
    ///
    /// These bytes are for internal storage and must not be emitted directly
    /// as UTF-8 interchange text.
    #[inline]
    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }

    /// Try to convert this buffer into ordinary UTF-8 without copying.
    ///
    /// # Errors
    ///
    /// Returns the original buffer when it contains one or more lone surrogates.
    pub fn into_string(self) -> Result<String, Self> {
        if self.as_str().is_some() {
            // SAFETY: `as_str` above verified that the buffer is UTF-8.
            Ok(unsafe { String::from_utf8_unchecked(self.bytes) })
        } else {
            Err(self)
        }
    }

    #[inline]
    #[expect(clippy::cast_possible_truncation, reason = "a three-byte sequence is at most U+FFFF")]
    fn final_lead_surrogate(&self) -> Option<u16> {
        let len = self.bytes.len();
        if len < 3 || self.bytes[len - 3] != 0xED || !(0xA0..=0xAF).contains(&self.bytes[len - 2]) {
            return None;
        }
        Some(decode_three_byte_sequence(
            self.bytes[len - 3],
            self.bytes[len - 2],
            self.bytes[len - 1],
        ) as u16)
    }

    #[inline]
    #[expect(
        clippy::cast_possible_truncation,
        reason = "each branch bounds the encoded byte fragments"
    )]
    fn push_code_point_without_boundary_check(&mut self, value: CodePoint) {
        let value = value.to_u32();
        if value <= 0x7F {
            self.bytes.push(value as u8);
        } else if value <= 0x7FF {
            self.bytes.push((0xC0 | (value >> 6)) as u8);
            self.bytes.push((0x80 | (value & 0x3F)) as u8);
        } else if value <= 0xFFFF {
            self.bytes.push((0xE0 | (value >> 12)) as u8);
            self.bytes.push((0x80 | ((value >> 6) & 0x3F)) as u8);
            self.bytes.push((0x80 | (value & 0x3F)) as u8);
        } else {
            debug_assert!(value <= 0x10_FFFF);
            self.bytes.push((0xF0 | (value >> 18)) as u8);
            self.bytes.push((0x80 | ((value >> 12) & 0x3F)) as u8);
            self.bytes.push((0x80 | ((value >> 6) & 0x3F)) as u8);
            self.bytes.push((0x80 | (value & 0x3F)) as u8);
        }
    }
}

impl Deref for Wtf8Buf {
    type Target = Wtf8;

    #[inline]
    fn deref(&self) -> &Self::Target {
        // SAFETY: All public mutation methods preserve canonical WTF-8.
        unsafe { Wtf8::from_bytes_unchecked(&self.bytes) }
    }
}

impl AsRef<Wtf8> for Wtf8Buf {
    #[inline]
    fn as_ref(&self) -> &Wtf8 {
        self
    }
}

impl Borrow<Wtf8> for Wtf8Buf {
    #[inline]
    fn borrow(&self) -> &Wtf8 {
        self
    }
}

impl fmt::Debug for Wtf8Buf {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        Wtf8::fmt(self, formatter)
    }
}

impl Hash for Wtf8Buf {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        Wtf8::hash(self, state);
    }
}

impl From<&str> for Wtf8Buf {
    #[inline]
    fn from(value: &str) -> Self {
        Self { bytes: value.as_bytes().to_vec() }
    }
}

impl From<String> for Wtf8Buf {
    #[inline]
    fn from(value: String) -> Self {
        Self::from_string(value)
    }
}

impl From<&Wtf8> for Wtf8Buf {
    #[inline]
    fn from(value: &Wtf8) -> Self {
        value.to_owned()
    }
}

impl FromIterator<CodePoint> for Wtf8Buf {
    fn from_iter<T: IntoIterator<Item = CodePoint>>(iter: T) -> Self {
        let mut result = Self::new();
        result.extend(iter);
        result
    }
}

impl Extend<CodePoint> for Wtf8Buf {
    fn extend<T: IntoIterator<Item = CodePoint>>(&mut self, iter: T) {
        let iter = iter.into_iter();
        self.reserve(iter.size_hint().0);
        for value in iter {
            self.push(value);
        }
    }
}

/// A maximal UTF-8 substring or one lone surrogate from a [`Wtf8`] string.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Wtf8Chunk<'a> {
    /// A non-empty substring containing only valid UTF-8.
    Utf8(&'a str),
    /// One unpaired UTF-16 surrogate code unit.
    Surrogate(u16),
}

/// Iterator over maximal UTF-8 substrings and individual lone surrogates.
#[derive(Clone, Debug)]
pub struct Wtf8Chunks<'a> {
    remaining: &'a [u8],
}

impl<'a> Iterator for Wtf8Chunks<'a> {
    type Item = Wtf8Chunk<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining.is_empty() {
            return None;
        }

        let mut position = 0;
        while position < self.remaining.len() {
            if self.remaining[position] == 0xED
                && matches!(self.remaining[position + 1], 0xA0..=0xBF)
            {
                if position == 0 {
                    let bytes = self.remaining;
                    self.remaining = &bytes[3..];
                    // A surrogate is encoded by one three-byte sequence.
                    #[expect(
                        clippy::cast_possible_truncation,
                        reason = "a three-byte sequence is at most U+FFFF"
                    )]
                    let surrogate = decode_three_byte_sequence(bytes[0], bytes[1], bytes[2]) as u16;
                    return Some(Wtf8Chunk::Surrogate(surrogate));
                }

                let (utf8, remaining) = self.remaining.split_at(position);
                self.remaining = remaining;
                // SAFETY: Valid WTF-8 differs from UTF-8 only at surrogate
                // sequences, and this prefix ends before the first one.
                return Some(Wtf8Chunk::Utf8(unsafe { str::from_utf8_unchecked(utf8) }));
            }

            let (_, len) = decode_next_valid_wtf8(&self.remaining[position..]);
            position += len;
        }

        let utf8 = self.remaining;
        self.remaining = &[];
        // SAFETY: The scan above found no surrogate sequences.
        Some(Wtf8Chunk::Utf8(unsafe { str::from_utf8_unchecked(utf8) }))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (usize::from(!self.remaining.is_empty()), Some(self.remaining.len()))
    }
}

/// Iterator over WTF-8 code points, including lone surrogates.
#[derive(Clone, Debug)]
pub struct Wtf8CodePoints<'a> {
    remaining: &'a [u8],
}

impl Iterator for Wtf8CodePoints<'_> {
    type Item = CodePoint;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining.is_empty() {
            return None;
        }
        let (value, len) = decode_next_valid_wtf8(self.remaining);
        self.remaining = &self.remaining[len..];
        // SAFETY: A validated WTF-8 sequence never exceeds U+10FFFF.
        Some(unsafe { CodePoint::from_u32_unchecked(value) })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let min = self.remaining.len().div_ceil(4);
        (min, Some(self.remaining.len()))
    }
}

/// Iterator over the potentially ill-formed UTF-16 code units represented by WTF-8.
#[derive(Clone, Debug)]
pub struct Wtf8CodeUnits<'a> {
    code_points: Wtf8CodePoints<'a>,
    pending_trail: Option<u16>,
}

impl Iterator for Wtf8CodeUnits<'_> {
    type Item = u16;

    #[expect(
        clippy::cast_possible_truncation,
        reason = "the BMP branch and surrogate calculations are bounded to u16"
    )]
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(trail) = self.pending_trail.take() {
            return Some(trail);
        }

        let value = self.code_points.next()?.to_u32();
        if value <= 0xFFFF {
            return Some(value as u16);
        }

        let supplementary = value - 0x1_0000;
        let lead = 0xD800 | (supplementary >> 10) as u16;
        let trail = 0xDC00 | (supplementary & 0x3FF) as u16;
        self.pending_trail = Some(trail);
        Some(lead)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let pending = usize::from(self.pending_trail.is_some());
        let (min, max) = self.code_points.size_hint();
        (min + pending, max.and_then(|max| max.checked_mul(2)).map(|max| max + pending))
    }
}

fn validate_wtf8(bytes: &[u8]) -> Result<(), Wtf8Error> {
    let mut position = 0;
    let mut previous_was_lead = false;
    while position < bytes.len() {
        let (value, len) = decode_next_checked(bytes, position)?;
        if previous_was_lead && is_trail_surrogate(value) {
            return Err(Wtf8Error {
                valid_up_to: position,
                kind: Wtf8ErrorKind::NonCanonicalSurrogatePair,
            });
        }
        previous_was_lead = is_lead_surrogate(value);
        position += len;
    }
    Ok(())
}

fn decode_next_checked(bytes: &[u8], position: usize) -> Result<(u32, usize), Wtf8Error> {
    let invalid = || Wtf8Error { valid_up_to: position, kind: Wtf8ErrorKind::InvalidEncoding };
    let remaining = &bytes[position..];
    let Some(&first) = remaining.first() else { return Err(invalid()) };

    match first {
        0x00..=0x7F => Ok((u32::from(first), 1)),
        0xC2..=0xDF => {
            let second = *remaining.get(1).ok_or_else(invalid)?;
            if !is_continuation(second) {
                return Err(invalid());
            }
            Ok(((u32::from(first & 0x1F) << 6) | u32::from(second & 0x3F), 2))
        }
        0xE0..=0xEF => {
            let second = *remaining.get(1).ok_or_else(invalid)?;
            let third = *remaining.get(2).ok_or_else(invalid)?;
            let valid_second = match first {
                0xE0 => (0xA0..=0xBF).contains(&second),
                // Unlike UTF-8, WTF-8 allows ED A0..BF to encode surrogates.
                0xE1..=0xEF => is_continuation(second),
                _ => unreachable!(),
            };
            if !valid_second || !is_continuation(third) {
                return Err(invalid());
            }
            Ok((decode_three_byte_sequence(first, second, third), 3))
        }
        0xF0..=0xF4 => {
            let second = *remaining.get(1).ok_or_else(invalid)?;
            let third = *remaining.get(2).ok_or_else(invalid)?;
            let fourth = *remaining.get(3).ok_or_else(invalid)?;
            let valid_second = match first {
                0xF0 => (0x90..=0xBF).contains(&second),
                0xF1..=0xF3 => is_continuation(second),
                0xF4 => (0x80..=0x8F).contains(&second),
                _ => unreachable!(),
            };
            if !valid_second || !is_continuation(third) || !is_continuation(fourth) {
                return Err(invalid());
            }
            Ok((
                (u32::from(first & 0x07) << 18)
                    | (u32::from(second & 0x3F) << 12)
                    | (u32::from(third & 0x3F) << 6)
                    | u32::from(fourth & 0x3F),
                4,
            ))
        }
        _ => Err(invalid()),
    }
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
const fn is_continuation(byte: u8) -> bool {
    byte & 0xC0 == 0x80
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
    debug_assert!((0xD800..=0xDBFF).contains(&lead));
    debug_assert!((0xDC00..=0xDFFF).contains(&trail));
    let value = 0x1_0000 + ((u32::from(lead) - 0xD800) << 10) + u32::from(trail) - 0xDC00;
    // SAFETY: A valid surrogate pair always produces a Unicode scalar value.
    unsafe { char::from_u32_unchecked(value) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_utf8_and_lone_surrogates() {
        assert_eq!(Wtf8::from_bytes("hello 🔥".as_bytes()).unwrap().as_str(), Some("hello 🔥"));

        let surrogate = Wtf8::from_bytes(&[0xED, 0xA0, 0x80]).unwrap();
        assert_eq!(surrogate.as_str(), None);
        assert_eq!(surrogate.code_points().map(CodePoint::to_u32).collect::<Vec<_>>(), [0xD800]);
        assert_eq!(surrogate.code_units().collect::<Vec<_>>(), [0xD800]);
    }

    #[test]
    fn rejects_invalid_and_non_canonical_bytes() {
        for invalid in [
            &[0x80][..],
            &[0xC0, 0x80],
            &[0xE0, 0x80, 0x80],
            &[0xF0, 0x80, 0x80, 0x80],
            &[0xF4, 0x90, 0x80, 0x80],
            &[0xED, 0xA0],
        ] {
            assert!(Wtf8::from_bytes(invalid).is_err(), "accepted {invalid:?}");
        }

        let separate_pair = [0xED, 0xA0, 0x80, 0xED, 0xB0, 0x80];
        let error = Wtf8::from_bytes(&separate_pair).unwrap_err();
        assert!(error.is_non_canonical_surrogate_pair());
        assert_eq!(error.valid_up_to(), 3);
    }

    #[test]
    fn round_trips_ill_formed_utf16() {
        let cases: &[&[u16]] = &[
            &[],
            &[0x61, 0x62, 0x63],
            &[0xD83D, 0xDD25],
            &[0xD800],
            &[0xDC00],
            &[0xD800, 0xD800, 0xDC00],
            &[0xDC00, 0xD800],
            &[0x0061, 0xD800, 0x0062, 0xDC00, 0x0063],
        ];

        for units in cases {
            let value = Wtf8Buf::from_ill_formed_utf16(units);
            assert_eq!(value.code_units().collect::<Vec<_>>(), *units);
            assert!(validate_wtf8(value.as_bytes()).is_ok());
        }
    }

    #[test]
    fn round_trips_every_lone_surrogate() {
        for unit in 0xD800..=0xDFFF {
            let value = Wtf8Buf::from_ill_formed_utf16(&[unit]);
            assert_eq!(value.code_units().collect::<Vec<_>>(), [unit]);
            assert_eq!(value.as_str(), None);
        }
    }

    #[test]
    fn round_trips_every_unicode_code_point() {
        for raw in 0..=0x10_FFFF {
            let code_point = CodePoint::from_u32(raw).unwrap();
            let mut value = Wtf8Buf::new();
            value.push(code_point);

            assert_eq!(value.code_points().collect::<Vec<_>>(), [code_point]);
            assert!(validate_wtf8(value.as_bytes()).is_ok());
            assert_eq!(value.as_str().is_some(), !(0xD800..=0xDFFF).contains(&raw));
        }
    }

    #[test]
    fn concatenation_canonicalizes_boundary_pair() {
        let mut lead = Wtf8Buf::from_ill_formed_utf16(&[0xD83D]);
        let trail = Wtf8Buf::from_ill_formed_utf16(&[0xDD25]);
        lead.push_wtf8(&trail);

        assert_eq!(lead.as_str(), Some("🔥"));
        assert_eq!(lead.as_bytes(), "🔥".as_bytes());
        assert_eq!(lead.code_units().collect::<Vec<_>>(), [0xD83D, 0xDD25]);
    }

    #[test]
    fn concatenation_canonicalizes_every_surrogate_pair() {
        let mut value = Wtf8Buf::with_capacity(6);
        for lead in 0xD800_u16..=0xDBFF {
            for trail in 0xDC00_u16..=0xDFFF {
                value.clear();
                // SAFETY: Every u16 value is a valid code point, including surrogates.
                value.push(unsafe { CodePoint::from_u32_unchecked(u32::from(lead)) });
                // SAFETY: Every u16 value is a valid code point, including surrogates.
                value.push(unsafe { CodePoint::from_u32_unchecked(u32::from(trail)) });

                assert_eq!(value.code_units().collect::<Vec<_>>(), [lead, trail]);
                assert!(value.as_str().is_some());
                assert_eq!(value.len(), 4);
            }
        }
    }

    #[test]
    fn concatenation_only_checks_the_boundary() {
        let mut lead = Wtf8Buf::from_ill_formed_utf16(&[0xD800]);
        lead.push_str("x");
        let trail = Wtf8Buf::from_ill_formed_utf16(&[0xDC00]);
        lead.push_wtf8(&trail);

        assert_eq!(lead.code_units().collect::<Vec<_>>(), [0xD800, u16::from(b'x'), 0xDC00]);
        assert!(validate_wtf8(lead.as_bytes()).is_ok());
    }

    #[test]
    fn slices_by_utf16_code_units() {
        let fire = Wtf8::from_str("🔥");
        assert_eq!(fire.utf16_len(), 2);
        assert_eq!(fire.code_unit_at(0), Some(0xD83D));
        assert_eq!(fire.code_unit_at(1), Some(0xDD25));

        let lead = fire.slice_code_units(0..1).unwrap();
        let trail = fire.slice_code_units(1..2).unwrap();
        assert_eq!(lead.code_units().collect::<Vec<_>>(), [0xD83D]);
        assert_eq!(trail.code_units().collect::<Vec<_>>(), [0xDD25]);
        assert_eq!(fire.slice_code_units(0..2).unwrap().as_str(), Some("🔥"));
        assert!(fire.slice_code_units(0..3).is_none());
    }

    #[test]
    fn replacement_character_has_no_special_meaning() {
        let value = Wtf8::from_str("\u{FFFD}d800\u{FFFD}fffd");
        assert_eq!(value.as_str(), Some("\u{FFFD}d800\u{FFFD}fffd"));
        assert_eq!(value.utf16_len(), 10);
    }

    #[test]
    fn deterministic_utf16_round_trip_stress() {
        let mut state = 0xC0DE_CAFE_u64;
        for len in 0..128 {
            let mut units = Vec::with_capacity(len);
            for _ in 0..len {
                state = state.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
                units.push(u16::try_from(state >> 48).unwrap());
            }
            let value = Wtf8Buf::from_ill_formed_utf16(&units);
            assert_eq!(value.code_units().collect::<Vec<_>>(), units);
        }
    }

    #[test]
    fn deterministic_concatenation_stress() {
        let mut state = 0x5EED_F00D_u64;
        for left_len in 0..32 {
            for right_len in 0..32 {
                let mut left_units = Vec::with_capacity(left_len);
                let mut right_units = Vec::with_capacity(right_len);
                for units in [&mut left_units, &mut right_units] {
                    while units.len() < units.capacity() {
                        state = state.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
                        units.push(u16::try_from(state >> 48).unwrap());
                    }
                }

                let mut actual = Wtf8Buf::from_ill_formed_utf16(&left_units);
                actual.push_wtf8(&Wtf8Buf::from_ill_formed_utf16(&right_units));
                let expected = left_units.iter().chain(&right_units).copied().collect::<Vec<_>>();
                assert_eq!(actual.code_units().collect::<Vec<_>>(), expected);
                assert!(validate_wtf8(actual.as_bytes()).is_ok());
            }
        }
    }

    #[test]
    fn chunks_separate_utf8_and_lone_surrogates() {
        let value = Wtf8Buf::from_ill_formed_utf16(&[
            u16::from(b'a'),
            0xD800,
            0xD83D,
            0xDD25,
            0xDC00,
            u16::from(b'b'),
        ]);
        assert_eq!(
            value.chunks().collect::<Vec<_>>(),
            [
                Wtf8Chunk::Utf8("a"),
                Wtf8Chunk::Surrogate(0xD800),
                Wtf8Chunk::Utf8("🔥"),
                Wtf8Chunk::Surrogate(0xDC00),
                Wtf8Chunk::Utf8("b"),
            ]
        );
    }

    #[test]
    fn supports_borrowed_or_owned_cow() {
        let borrowed: Cow<'_, Wtf8> = Cow::Borrowed(Wtf8::from_str("borrowed"));
        assert_eq!(borrowed.into_owned().as_str(), Some("borrowed"));

        let owned: Cow<'_, Wtf8> = Cow::Owned(Wtf8Buf::from_ill_formed_utf16(&[0xD800]));
        assert_eq!(owned.code_units().collect::<Vec<_>>(), [0xD800]);
    }
}
