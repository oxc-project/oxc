//! Core WTF-8 string types: [`Wtf8`] and [`Wtf8Buf`].
//!
//! [WTF-8](https://simonsapin.github.io/wtf-8/) is a superset of UTF-8 that can represent
//! lone surrogates. It is useful for representing JavaScript strings, which may contain
//! lone surrogate code units (U+D800..=U+DFFF).
//!
//! In memory, a WTF-8 string is a byte sequence that follows the same encoding rules as UTF-8,
//! but additionally allows lone surrogate code points (which are forbidden in valid UTF-8).
//! Lone surrogates are encoded as 3-byte sequences: `0xED 0xA0..=0xBF 0x80..=0xBF`.
//!
//! Well-formed WTF-8 additionally requires that surrogate pairs (a lead surrogate immediately
//! followed by a trail surrogate) must be encoded as a single 4-byte supplementary code point,
//! not as two separate 3-byte surrogate sequences.
//!
//! This implementation is adapted from the [rust-wtf8 crate] and [SWC's hstr crate],
//! both originally derived from Rust's standard library.
//!
//! [rust-wtf8 crate]: https://github.com/SimonSapin/rust-wtf8
//! [SWC's hstr crate]: https://github.com/swc-project/swc/tree/main/crates/hstr

use std::{
    borrow::Cow,
    fmt::{self, Write},
    hash::{self, Hash},
    mem, ops,
    ops::Deref,
    slice, str,
};

// ──────────────────────────────────────────────────────────────────────────────
// Constants
// ──────────────────────────────────────────────────────────────────────────────

/// The UTF-8 replacement character `U+FFFD` as bytes.
const REPLACEMENT_CHAR: char = '\u{FFFD}';
const REPLACEMENT_CHAR_UTF8: [u8; 3] = {
    let mut buf = [0u8; 3];
    // encode_utf8 is not const, so do it manually
    // U+FFFD = 0b1110_1111 0b1011_1111 0b1011_1101
    buf[0] = 0xEF;
    buf[1] = 0xBF;
    buf[2] = 0xBD;
    buf
};

/// First byte of any lone surrogate in WTF-8 encoding.
/// Lone surrogates U+D800..=U+DFFF are encoded as 3-byte sequences starting with 0xED.
const SURROGATE_FIRST_BYTE: u8 = 0xED;
/// Minimum second byte for a surrogate (lead surrogates start at 0xA0 which is U+D800).
const SURROGATE_SECOND_BYTE_MIN: u8 = 0xA0;
/// Maximum second byte for a surrogate (trail surrogates end at 0xBF which is U+DFFF).
const SURROGATE_SECOND_BYTE_MAX: u8 = 0xBF;

// ──────────────────────────────────────────────────────────────────────────────
// Wtf8 (borrowed slice)
// ──────────────────────────────────────────────────────────────────────────────

/// A borrowed WTF-8 string slice.
///
/// Analogous to `str` for UTF-8, but can additionally contain lone surrogate code points
/// (U+D800..=U+DFFF).
///
/// # Encoding
///
/// WTF-8 is encoded exactly like UTF-8, except that lone surrogate code points are allowed.
/// In memory they appear as 3-byte sequences `0xED 0xA0..=0xBF 0x80..=0xBF`.
///
/// Well-formed WTF-8 does **not** allow a lead surrogate followed immediately by a trail
/// surrogate — such pairs must be encoded as a single 4-byte supplementary code point.
#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Wtf8 {
    bytes: [u8],
}

impl Wtf8 {
    // ─── Construction ────────────────────────────────────────────────────────

    /// Create a `&Wtf8` from a UTF-8 string slice.
    ///
    /// This is always valid because WTF-8 is a strict superset of UTF-8.
    #[inline]
    pub const fn from_str(s: &str) -> &Self {
        // SAFETY: `str` bytes are valid UTF-8 which is a subset of WTF-8.
        unsafe { Self::from_bytes_unchecked(s.as_bytes()) }
    }

    /// Create a `&Wtf8` from a byte slice, returning `None` if the bytes are not valid WTF-8.
    #[inline]
    pub fn from_bytes(bytes: &[u8]) -> Option<&Self> {
        if validate_wtf8(bytes) {
            // SAFETY: bytes is valid WTF-8 per the check above.
            Some(unsafe { Self::from_bytes_unchecked(bytes) })
        } else {
            None
        }
    }

    /// Create a `&Wtf8` from a byte slice without validation.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `bytes` is valid well-formed WTF-8.
    #[inline]
    pub const unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &Self {
        // SAFETY: `Wtf8` is `#[repr(transparent)]` over `[u8]`, so this transmute is sound.
        unsafe { mem::transmute(bytes) }
    }

    // ─── Properties ──────────────────────────────────────────────────────────

    /// Return the length in bytes.
    #[inline]
    pub const fn len(&self) -> usize {
        self.bytes.len()
    }

    /// Return `true` if the string contains no bytes.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    /// Return `true` if the string contains only ASCII characters.
    #[inline]
    pub fn is_ascii(&self) -> bool {
        self.bytes.is_ascii()
    }

    /// Return `true` if the string is valid UTF-8 (contains no lone surrogates).
    ///
    /// Equivalent to `!self.contains_lone_surrogates()`.
    #[inline]
    pub fn is_well_formed(&self) -> bool {
        !self.contains_lone_surrogates()
    }

    /// Return `true` if the string contains at least one lone surrogate code point.
    #[inline]
    pub fn contains_lone_surrogates(&self) -> bool {
        self.next_lone_surrogate(0).is_some()
    }

    // ─── Access ───────────────────────────────────────────────────────────────

    /// Return the underlying bytes.
    #[inline]
    pub const fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Try to convert to a `&str`.
    ///
    /// Returns `None` if the string contains lone surrogates.
    #[inline]
    pub fn as_str(&self) -> Option<&str> {
        str::from_utf8(&self.bytes).ok()
    }

    /// Convert to a UTF-8 string, replacing any lone surrogates with `U+FFFD`.
    ///
    /// Returns `Cow::Borrowed` if the string contains no lone surrogates.
    pub fn to_str_lossy(&self) -> Cow<'_, str> {
        if let Ok(s) = str::from_utf8(&self.bytes) {
            return Cow::Borrowed(s);
        }

        match self.next_lone_surrogate(0) {
            None => Cow::Owned(String::from_utf8_lossy(&self.bytes).into_owned()),
            Some((first_surrogate, _)) => {
                let mut result = String::with_capacity(self.bytes.len());
                // SAFETY: bytes up to the first surrogate are valid UTF-8.
                result
                    .push_str(unsafe { str::from_utf8_unchecked(&self.bytes[..first_surrogate]) });
                let mut pos = first_surrogate;
                loop {
                    result.push(REPLACEMENT_CHAR);
                    pos += 3; // surrogate sequences are always 3 bytes
                    let rest = &self.bytes[pos..];
                    match next_lone_surrogate_in(rest, 0) {
                        None => {
                            // SAFETY: rest after surrogates is valid UTF-8.
                            result.push_str(unsafe { str::from_utf8_unchecked(rest) });
                            break;
                        }
                        Some((surrogate_offset, _)) => {
                            // SAFETY: bytes between surrogates are valid UTF-8.
                            result.push_str(unsafe {
                                str::from_utf8_unchecked(&rest[..surrogate_offset])
                            });
                            pos += surrogate_offset;
                        }
                    }
                }
                Cow::Owned(result)
            }
        }
    }

    // ─── Surrogates ───────────────────────────────────────────────────────────

    /// Return the position and code unit of the first lone surrogate at or after `start`.
    ///
    /// The code unit is in the range `0xD800..=0xDFFF`.
    /// The returned position is the byte offset of the first byte of the 3-byte WTF-8 sequence.
    #[inline]
    pub fn next_lone_surrogate(&self, start: usize) -> Option<(usize, u16)> {
        next_lone_surrogate_in(&self.bytes, start)
    }

    /// Return the code unit of a lone surrogate whose 3-byte sequence starts at `pos`.
    ///
    /// # Safety
    ///
    /// `pos` must be the start of a valid lone surrogate sequence within this string.
    #[inline]
    pub unsafe fn surrogate_at_unchecked(&self, pos: usize) -> u16 {
        debug_assert!(pos + 3 <= self.bytes.len());
        let b1 = self.bytes[pos + 1];
        let b2 = self.bytes[pos + 2];
        decode_surrogate(b1, b2)
    }

    /// Return an iterator over code points in the string.
    ///
    /// Each item is either a `char` or a lone surrogate code unit (`u16`).
    #[inline]
    pub fn code_points(&self) -> CodePoints<'_> {
        CodePoints { bytes: self.bytes.iter() }
    }

    /// Return an iterator that encodes this string as ill-formed UTF-16.
    ///
    /// Lone surrogates appear as their individual `u16` code units.
    /// Supplementary code points (>= U+10000) appear as surrogate pairs.
    #[inline]
    pub fn to_ill_formed_utf16(&self) -> IllFormedUtf16<'_> {
        IllFormedUtf16 { code_points: self.code_points(), extra: 0 }
    }

    // ─── Internal helpers ─────────────────────────────────────────────────────

    /// Find the last lead surrogate in the string (used for `push_wtf8` combining).
    pub(crate) fn final_lead_surrogate(&self) -> Option<u16> {
        let len = self.bytes.len();
        if len < 3 {
            return None;
        }
        let b0 = self.bytes[len - 3];
        let b1 = self.bytes[len - 2];
        let b2 = self.bytes[len - 1];
        if is_lead_surrogate_bytes(b0, b1, b2) { Some(decode_surrogate(b1, b2)) } else { None }
    }

    /// Find the first trail surrogate in the string (used for `push_wtf8` combining).
    pub(crate) fn initial_trail_surrogate(&self) -> Option<u16> {
        let len = self.bytes.len();
        if len < 3 {
            return None;
        }
        let b0 = self.bytes[0];
        let b1 = self.bytes[1];
        let b2 = self.bytes[2];
        if is_trail_surrogate_bytes(b0, b1, b2) { Some(decode_surrogate(b1, b2)) } else { None }
    }
}

impl fmt::Debug for Wtf8 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("\"")?;
        let mut pos = 0;
        loop {
            match self.next_lone_surrogate(pos) {
                None => break,
                Some((surrogate_pos, surrogate)) => {
                    // SAFETY: bytes before the surrogate are valid UTF-8.
                    f.write_str(unsafe {
                        str::from_utf8_unchecked(&self.bytes[pos..surrogate_pos])
                    })?;
                    write!(f, "\\u{{{surrogate:X}}}")?;
                    pos = surrogate_pos + 3;
                }
            }
        }
        // SAFETY: remaining bytes after the last surrogate are valid UTF-8.
        f.write_str(unsafe { str::from_utf8_unchecked(&self.bytes[pos..]) })?;
        f.write_str("\"")
    }
}

impl fmt::Display for Wtf8 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Replace lone surrogates with the replacement character.
        let mut pos = 0;
        loop {
            match self.next_lone_surrogate(pos) {
                None => break,
                Some((surrogate_pos, _)) => {
                    // SAFETY: bytes before surrogate are valid UTF-8.
                    f.write_str(unsafe {
                        str::from_utf8_unchecked(&self.bytes[pos..surrogate_pos])
                    })?;
                    f.write_char(REPLACEMENT_CHAR)?;
                    pos = surrogate_pos + 3;
                }
            }
        }
        // SAFETY: remaining bytes are valid UTF-8.
        f.write_str(unsafe { str::from_utf8_unchecked(&self.bytes[pos..]) })
    }
}

impl Hash for Wtf8 {
    fn hash<H: hash::Hasher>(&self, hasher: &mut H) {
        self.bytes.hash(hasher);
    }
}

impl PartialEq<str> for Wtf8 {
    fn eq(&self, other: &str) -> bool {
        self.bytes == *other.as_bytes()
    }
}

impl PartialEq<Wtf8> for str {
    fn eq(&self, other: &Wtf8) -> bool {
        *self.as_bytes() == other.bytes
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Wtf8Buf (owned string)
// ──────────────────────────────────────────────────────────────────────────────

/// An owned, growable WTF-8 string.
///
/// Analogous to `String` for UTF-8, but can additionally contain lone surrogate code points.
pub struct Wtf8Buf {
    bytes: Vec<u8>,
}

impl Wtf8Buf {
    // ─── Construction ─────────────────────────────────────────────────────────

    /// Create a new, empty WTF-8 string.
    #[inline]
    pub fn new() -> Self {
        Self { bytes: Vec::new() }
    }

    /// Create a new, empty WTF-8 string with pre-allocated capacity for `n` bytes.
    #[inline]
    pub fn with_capacity(n: usize) -> Self {
        Self { bytes: Vec::with_capacity(n) }
    }

    /// Create a WTF-8 string from a UTF-8 `String` without copying.
    ///
    /// Since WTF-8 is a superset of UTF-8, this always succeeds.
    #[inline]
    pub fn from_string(s: String) -> Self {
        Self { bytes: s.into_bytes() }
    }

    /// Create a WTF-8 string from a UTF-8 `&str` by copying.
    #[inline]
    #[expect(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Self {
        Self { bytes: s.as_bytes().to_vec() }
    }

    /// Create a WTF-8 string from a potentially ill-formed UTF-16 slice.
    ///
    /// Surrogate pairs are combined into supplementary code points.
    /// Lone surrogates are preserved as WTF-8 lone surrogate sequences.
    pub fn from_ill_formed_utf16(v: &[u16]) -> Self {
        let mut buf = Self::with_capacity(v.len());
        let mut iter = v.iter().copied();
        while let Some(code_unit) = iter.next() {
            if (0xD800..=0xDBFF).contains(&code_unit) {
                // Lead surrogate — look ahead for a trail surrogate.
                if let Some(next) = iter.next() {
                    if (0xDC00..=0xDFFF).contains(&next) {
                        // Valid surrogate pair → supplementary code point.
                        let code_point = decode_surrogate_pair(code_unit, next);
                        // SAFETY: a valid surrogate pair always yields a valid supplementary code point.
                        buf.push_char(unsafe { char::from_u32_unchecked(code_point) });
                        continue;
                    }
                    // `next` is not a trail surrogate.  Push both as lone surrogates / chars.
                    buf.push_lone_surrogate_unchecked(code_unit);
                    if (0xD800..=0xDFFF).contains(&next) {
                        buf.push_lone_surrogate_unchecked(next);
                    } else {
                        // SAFETY: `next` is not a surrogate and not > 0xFFFF (it's u16), so valid char.
                        buf.push_char(unsafe { char::from_u32_unchecked(u32::from(next)) });
                    }
                } else {
                    // End of input — lone lead surrogate.
                    buf.push_lone_surrogate_unchecked(code_unit);
                }
            } else if (0xDC00..=0xDFFF).contains(&code_unit) {
                // Lone trail surrogate.
                buf.push_lone_surrogate_unchecked(code_unit);
            } else {
                // Regular BMP code point.
                // SAFETY: not a surrogate and within BMP (u16 range), so valid char.
                buf.push_char(unsafe { char::from_u32_unchecked(u32::from(code_unit)) });
            }
        }
        buf
    }

    // ─── Mutating ─────────────────────────────────────────────────────────────

    /// Append a UTF-8 string slice.
    #[inline]
    pub fn push_str(&mut self, s: &str) {
        self.bytes.extend_from_slice(s.as_bytes());
    }

    /// Append a WTF-8 slice, combining surrogate pairs at the boundary.
    ///
    /// If `self` ends with a lead surrogate and `other` starts with a trail surrogate,
    /// they are combined into a single supplementary code point.
    pub fn push_wtf8(&mut self, other: &Wtf8) {
        match (self.final_lead_surrogate(), other.initial_trail_surrogate()) {
            (Some(lead), Some(trail)) => {
                // Combine into a supplementary code point.
                let len_without_lead = self.bytes.len() - 3;
                self.bytes.truncate(len_without_lead);
                let code_point = decode_surrogate_pair(lead, trail);
                // SAFETY: lead+trail pair always yields a valid supplementary code point.
                self.push_char(unsafe { char::from_u32_unchecked(code_point) });
                // Append the rest of `other` (skip the 3-byte trail surrogate).
                self.bytes.extend_from_slice(&other.bytes[3..]);
            }
            _ => {
                self.bytes.extend_from_slice(&other.bytes);
            }
        }
    }

    /// Append a Unicode scalar value (regular `char`).
    #[inline]
    pub fn push_char(&mut self, c: char) {
        let mut buf = [0u8; 4];
        let encoded = c.encode_utf8(&mut buf);
        self.bytes.extend_from_slice(encoded.as_bytes());
    }

    /// Append a code point, which may be a lone surrogate.
    ///
    /// If the code point is a valid Unicode scalar value, it is encoded as UTF-8.
    /// If it is a surrogate (U+D800..=U+DFFF), it is encoded as a WTF-8 lone surrogate.
    ///
    /// If `self` ends with a lead surrogate and `code_point` is a trail surrogate, they
    /// are combined into a single supplementary code point.
    pub fn push_code_point(&mut self, code_point: u32) {
        if (0xDC00..=0xDFFF).contains(&code_point) {
            // Trail surrogate — check if the previous character was a lead surrogate.
            if let Some(lead) = self.final_lead_surrogate() {
                #[expect(clippy::cast_possible_truncation)]
                let trail = code_point as u16; // safe: 0xDC00..=0xDFFF fits in u16
                let len_without_lead = self.bytes.len() - 3;
                self.bytes.truncate(len_without_lead);
                let supplementary = decode_surrogate_pair(lead, trail);
                // SAFETY: a valid lead+trail surrogate pair always yields a valid supplementary code point.
                self.push_char(unsafe { char::from_u32_unchecked(supplementary) });
                return;
            }
        }
        if let Some(c) = char::from_u32(code_point) {
            self.push_char(c);
        } else if (0xD800..=0xDFFF).contains(&code_point) {
            // Lone surrogate.
            #[expect(clippy::cast_possible_truncation)]
            self.push_lone_surrogate_unchecked(code_point as u16); // safe: 0xD800..=0xDFFF fits in u16
        }
        // Silently ignore invalid code points outside the valid range.
    }

    /// Reserve capacity for at least `additional` more bytes.
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.bytes.reserve(additional);
    }

    /// Clear the string.
    #[inline]
    pub fn clear(&mut self) {
        self.bytes.clear();
    }

    // ─── Conversion ───────────────────────────────────────────────────────────

    /// Convert to a UTF-8 `String`.
    ///
    /// Returns `Err(self)` if the string contains lone surrogates.
    ///
    /// # Errors
    ///
    /// Returns `Err(self)` when the string contains lone surrogates (WTF-8 sequences
    /// that are not valid UTF-8).
    pub fn into_string(self) -> Result<String, Self> {
        if self.is_well_formed() {
            // SAFETY: well-formed WTF-8 without lone surrogates is valid UTF-8.
            Ok(unsafe { String::from_utf8_unchecked(self.bytes) })
        } else {
            Err(self)
        }
    }

    /// Convert to a UTF-8 `String`, replacing lone surrogates with `U+FFFD`.
    pub fn into_string_lossy(mut self) -> String {
        let mut pos = 0;
        while let Some((surrogate_pos, _)) = self.next_lone_surrogate(pos) {
            self.bytes[surrogate_pos..surrogate_pos + 3].copy_from_slice(&REPLACEMENT_CHAR_UTF8);
            pos = surrogate_pos + 3;
        }
        // SAFETY: all surrogates replaced; remaining bytes are valid UTF-8.
        unsafe { String::from_utf8_unchecked(self.bytes) }
    }

    // ─── Internal ─────────────────────────────────────────────────────────────

    /// Encode and append a lone surrogate code unit.
    ///
    /// `surrogate` must be in the range `0xD800..=0xDFFF`.
    #[inline]
    fn push_lone_surrogate_unchecked(&mut self, surrogate: u16) {
        debug_assert!((0xD800..=0xDFFF).contains(&surrogate));
        let s = u32::from(surrogate);
        self.bytes.push(0xED);
        #[expect(clippy::cast_possible_truncation)]
        self.bytes.push(0x80 | ((s >> 6) as u8 & 0x3F)); // bits 11-6 fit in 6 bits
        #[expect(clippy::cast_possible_truncation)]
        self.bytes.push(0x80 | (s as u8 & 0x3F)); // bits 5-0 fit in 6 bits
    }
}

impl Default for Wtf8Buf {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for Wtf8Buf {
    type Target = Wtf8;

    #[inline]
    fn deref(&self) -> &Wtf8 {
        // SAFETY: `Wtf8Buf` always contains valid WTF-8.
        unsafe { Wtf8::from_bytes_unchecked(&self.bytes) }
    }
}

impl ops::DerefMut for Wtf8Buf {
    fn deref_mut(&mut self) -> &mut Wtf8 {
        // SAFETY: `Wtf8Buf` always contains valid WTF-8.
        #[expect(clippy::ref_as_ptr)]
        unsafe {
            &mut *(self.bytes.as_mut_slice() as *mut [u8] as *mut Wtf8)
        }
    }
}

impl fmt::Debug for Wtf8Buf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl fmt::Display for Wtf8Buf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl Clone for Wtf8Buf {
    fn clone(&self) -> Self {
        Self { bytes: self.bytes.clone() }
    }
}

impl PartialEq for Wtf8Buf {
    fn eq(&self, other: &Self) -> bool {
        self.bytes == other.bytes
    }
}

impl Eq for Wtf8Buf {}

impl PartialOrd for Wtf8Buf {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Wtf8Buf {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.bytes.cmp(&other.bytes)
    }
}

impl Hash for Wtf8Buf {
    fn hash<H: hash::Hasher>(&self, hasher: &mut H) {
        self.bytes.hash(hasher);
    }
}

impl From<String> for Wtf8Buf {
    #[inline]
    fn from(s: String) -> Self {
        Self::from_string(s)
    }
}

impl From<&str> for Wtf8Buf {
    #[inline]
    fn from(s: &str) -> Self {
        Self::from_str(s)
    }
}

impl From<Wtf8Buf> for String {
    /// Lossily convert a `Wtf8Buf` to a `String`, replacing lone surrogates with `U+FFFD`.
    ///
    /// If you want a panic on lone surrogates, use [`Wtf8Buf::into_string`] instead.
    fn from(buf: Wtf8Buf) -> Self {
        buf.into_string_lossy()
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Iterators
// ──────────────────────────────────────────────────────────────────────────────

/// A code point in a WTF-8 string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodePoint {
    /// A valid Unicode scalar value.
    Unicode(char),
    /// A lone surrogate code unit (U+D800..=U+DFFF).
    LoneSurrogate(u16),
}

impl CodePoint {
    /// Return the numeric value of the code point.
    #[inline]
    pub fn to_u32(self) -> u32 {
        match self {
            CodePoint::Unicode(c) => c as u32,
            CodePoint::LoneSurrogate(s) => u32::from(s),
        }
    }

    /// Try to convert to a `char`.
    ///
    /// Returns `None` for lone surrogates.
    #[inline]
    pub fn to_char(self) -> Option<char> {
        match self {
            CodePoint::Unicode(c) => Some(c),
            CodePoint::LoneSurrogate(_) => None,
        }
    }

    /// Convert to a `char`, replacing lone surrogates with `U+FFFD`.
    #[inline]
    pub fn to_char_lossy(self) -> char {
        self.to_char().unwrap_or(REPLACEMENT_CHAR)
    }
}

/// Iterator over the code points of a [`Wtf8`] string.
pub struct CodePoints<'a> {
    bytes: slice::Iter<'a, u8>,
}

impl Iterator for CodePoints<'_> {
    type Item = CodePoint;

    fn next(&mut self) -> Option<CodePoint> {
        let &first = self.bytes.next()?;
        let code_point = if first < 0x80 {
            u32::from(first)
        } else if first < 0xE0 {
            let cont = *self.bytes.next().unwrap_or(&0);
            (u32::from(first & 0x1F) << 6) | u32::from(cont & 0x3F)
        } else if first < 0xF0 {
            let cont1 = *self.bytes.next().unwrap_or(&0);
            let cont2 = *self.bytes.next().unwrap_or(&0);
            (u32::from(first & 0x0F) << 12)
                | (u32::from(cont1 & 0x3F) << 6)
                | u32::from(cont2 & 0x3F)
        } else {
            let cont1 = *self.bytes.next().unwrap_or(&0);
            let cont2 = *self.bytes.next().unwrap_or(&0);
            let cont3 = *self.bytes.next().unwrap_or(&0);
            (u32::from(first & 0x07) << 18)
                | (u32::from(cont1 & 0x3F) << 12)
                | (u32::from(cont2 & 0x3F) << 6)
                | u32::from(cont3 & 0x3F)
        };
        Some(match char::from_u32(code_point) {
            Some(c) => CodePoint::Unicode(c),
            #[expect(clippy::cast_possible_truncation)]
            None => CodePoint::LoneSurrogate(code_point as u16), // surrogates fit in u16
        })
    }
}

/// Iterator that encodes a WTF-8 string as ill-formed UTF-16.
pub struct IllFormedUtf16<'a> {
    code_points: CodePoints<'a>,
    /// Buffered low surrogate from a supplementary code point.
    extra: u16,
}

impl Iterator for IllFormedUtf16<'_> {
    type Item = u16;

    fn next(&mut self) -> Option<u16> {
        if self.extra != 0 {
            return Some(mem::replace(&mut self.extra, 0));
        }
        let cp = self.code_points.next()?;
        match cp {
            CodePoint::LoneSurrogate(s) => Some(s),
            CodePoint::Unicode(c) => {
                let v = c as u32;
                if v < 0x10000 {
                    #[expect(clippy::cast_possible_truncation)]
                    return Some(v as u16); // BMP code points fit in u16
                }
                // Encode as surrogate pair.
                let v = v - 0x10000;
                #[expect(clippy::cast_possible_truncation)]
                {
                    self.extra = 0xDC00 | (v & 0x3FF) as u16; // 10-bit value fits in u16
                    Some(0xD800 | (v >> 10) as u16) // 10-bit value fits in u16
                }
            }
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Free functions (internal helpers)
// ──────────────────────────────────────────────────────────────────────────────

/// Decode the code unit of a lone surrogate from its 2nd and 3rd bytes.
///
/// The first byte is always `0xED` for surrogates, which encodes bits 15-12 as `0xD`.
/// The second byte encodes bits 11-6, and the third byte encodes bits 5-0.
/// We reconstruct the full 16-bit code unit by adding the implicit `0xD000` prefix.
#[inline]
fn decode_surrogate(b1: u8, b2: u8) -> u16 {
    0xD000 | ((u16::from(b1) & 0x3F) << 6) | (u16::from(b2) & 0x3F)
}

/// Combine a lead/trail surrogate pair into a supplementary code point (U+10000..=U+10FFFF).
#[inline]
fn decode_surrogate_pair(lead: u16, trail: u16) -> u32 {
    let lead = u32::from(lead) - 0xD800;
    let trail = u32::from(trail) - 0xDC00;
    (lead << 10) + trail + 0x10000
}

/// Return `true` if bytes `[b0, b1, b2]` encode a lead surrogate (U+D800..=U+DBFF).
#[inline]
fn is_lead_surrogate_bytes(b0: u8, b1: u8, b2: u8) -> bool {
    b0 == SURROGATE_FIRST_BYTE
        && (SURROGATE_SECOND_BYTE_MIN..=0xAF).contains(&b1)
        && (0x80..=0xBF).contains(&b2)
}

/// Return `true` if bytes `[b0, b1, b2]` encode a trail surrogate (U+DC00..=U+DFFF).
#[inline]
fn is_trail_surrogate_bytes(b0: u8, b1: u8, b2: u8) -> bool {
    b0 == SURROGATE_FIRST_BYTE
        && (0xB0..=SURROGATE_SECOND_BYTE_MAX).contains(&b1)
        && (0x80..=0xBF).contains(&b2)
}

/// Return `true` if bytes at position `i` encode any lone surrogate.
#[inline]
fn is_surrogate_at(bytes: &[u8], i: usize) -> bool {
    bytes.get(i) == Some(&SURROGATE_FIRST_BYTE)
        && bytes
            .get(i + 1)
            .is_some_and(|&b| (SURROGATE_SECOND_BYTE_MIN..=SURROGATE_SECOND_BYTE_MAX).contains(&b))
        && bytes.get(i + 2).is_some_and(|&b| (0x80..=0xBF).contains(&b))
}

/// Scan `bytes` from `start` for the first lone surrogate.
///
/// Returns `(byte_offset, code_unit)` where `byte_offset` is the index of the `0xED` byte.
fn next_lone_surrogate_in(bytes: &[u8], start: usize) -> Option<(usize, u16)> {
    let mut search_start = start;
    // `0xED` is the only possible first byte of a lone-surrogate 3-byte sequence.
    // Use a simple byte-search (LLVM/auto-vectorises this into a SIMD memchr) to skip
    // large stretches of ASCII or non-ED multi-byte sequences in one pass, then only
    // do the detailed three-byte validation when a candidate byte is found.
    loop {
        // Find next 0xED byte, skipping non-candidates efficiently.
        // Use `bytes.get(search_start..)` to avoid panicking when `search_start > bytes.len()`.
        let slice = bytes.get(search_start..)?;
        let rel = slice.iter().position(|&b| b == SURROGATE_FIRST_BYTE)?;
        let i = search_start + rel;
        if i + 2 < bytes.len() {
            let b1 = bytes[i + 1];
            if (SURROGATE_SECOND_BYTE_MIN..=SURROGATE_SECOND_BYTE_MAX).contains(&b1) {
                let b2 = bytes[i + 2];
                if (0x80..=0xBF).contains(&b2) {
                    return Some((i, decode_surrogate(b1, b2)));
                }
            }
        }
        // Not a surrogate sequence (or truncated), skip past this 0xED byte.
        search_start = i + 3;
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// WTF-8 validation
// ──────────────────────────────────────────────────────────────────────────────

/// Validate that `bytes` is well-formed WTF-8.
///
/// Well-formed WTF-8 is valid UTF-8 that additionally allows lone surrogate sequences
/// (`0xED 0xA0..=0xBF 0x80..=0xBF`), but **not** surrogate pairs encoded as two separate
/// 3-byte sequences (they must be combined into 4-byte sequences).
pub fn validate_wtf8(bytes: &[u8]) -> bool {
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];

        // 1-byte (ASCII)
        if b < 0x80 {
            i += 1;
            continue;
        }

        // 2-byte sequence: 0xC2..=0xDF
        if b < 0xE0 {
            if b < 0xC2 {
                return false; // Overlong or invalid lead byte
            }
            if i + 1 >= bytes.len() || bytes[i + 1] & 0xC0 != 0x80 {
                return false; // Truncated or invalid continuation
            }
            i += 2;
            continue;
        }

        // 3-byte sequence: 0xE0..=0xEF
        if b < 0xF0 {
            if i + 2 >= bytes.len() || bytes[i + 1] & 0xC0 != 0x80 || bytes[i + 2] & 0xC0 != 0x80 {
                return false; // Truncated or invalid continuation
            }

            let b1 = bytes[i + 1];

            // Overlong check for 0xE0
            if b == 0xE0 && b1 < 0xA0 {
                return false;
            }

            // Check for surrogate pair (lead + trail) — forbidden in WTF-8.
            // A lead surrogate (0xED 0xA0..=0xAF …) must not be immediately followed by
            // a trail surrogate (0xED 0xB0..=0xBF …).
            if is_surrogate_at(bytes, i) && i + 3 < bytes.len() && is_surrogate_at(bytes, i + 3) {
                let b1_self = bytes[i + 1];
                let b1_next = bytes[i + 4];
                // Lead: second byte 0xA0..=0xAF; trail: second byte 0xB0..=0xBF
                if (0xA0..=0xAF).contains(&b1_self) && (0xB0..=0xBF).contains(&b1_next) {
                    return false;
                }
            }

            i += 3;
            continue;
        }

        // 4-byte sequence: 0xF0..=0xF4
        if b <= 0xF4 {
            if i + 3 >= bytes.len()
                || bytes[i + 1] & 0xC0 != 0x80
                || bytes[i + 2] & 0xC0 != 0x80
                || bytes[i + 3] & 0xC0 != 0x80
            {
                return false;
            }
            let b1 = bytes[i + 1];
            // Range check: 0xF0 needs b1 >= 0x90, 0xF4 needs b1 <= 0x8F
            if (b == 0xF0 && b1 < 0x90) || (b == 0xF4 && b1 > 0x8F) {
                return false;
            }
            i += 4;
            continue;
        }

        // Invalid lead byte (0xF5..=0xFF)
        return false;
    }
    true
}

// ──────────────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ─── validate_wtf8 ────────────────────────────────────────────────────────

    #[test]
    fn validate_empty() {
        assert!(validate_wtf8(b""));
    }

    #[test]
    fn validate_ascii() {
        assert!(validate_wtf8(b"hello world"));
    }

    #[test]
    fn validate_utf8() {
        // U+00E9 (é) — 2 bytes
        assert!(validate_wtf8(&[0xC3, 0xA9]));
        // U+4E2D (中) — 3 bytes
        assert!(validate_wtf8(&[0xE4, 0xB8, 0xAD]));
        // U+1F600 (😀) — 4 bytes
        assert!(validate_wtf8(&[0xF0, 0x9F, 0x98, 0x80]));
    }

    #[test]
    fn validate_lone_lead_surrogate() {
        // U+D800 encoded as WTF-8: 0xED 0xA0 0x80
        assert!(validate_wtf8(&[0xED, 0xA0, 0x80]));
    }

    #[test]
    fn validate_lone_trail_surrogate() {
        // U+DC00 encoded as WTF-8: 0xED 0xB0 0x80
        assert!(validate_wtf8(&[0xED, 0xB0, 0x80]));
    }

    #[test]
    fn validate_surrogate_pair_invalid() {
        // Surrogate pair as two 3-byte sequences — invalid in WTF-8
        assert!(!validate_wtf8(&[0xED, 0xA0, 0x80, 0xED, 0xB0, 0x80]));
    }

    #[test]
    fn validate_string_with_lone_surrogate() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"hello");
        bytes.extend_from_slice(&[0xED, 0xA0, 0x80]); // U+D800
        bytes.extend_from_slice(b"world");
        assert!(validate_wtf8(&bytes));
    }

    #[test]
    fn validate_invalid_lead_byte() {
        assert!(!validate_wtf8(&[0xFF]));
        assert!(!validate_wtf8(&[0xC1, 0x80])); // Overlong 2-byte
    }

    #[test]
    fn validate_truncated() {
        assert!(!validate_wtf8(&[0xED, 0xA0])); // Truncated 3-byte
    }

    // ─── Wtf8 methods ────────────────────────────────────────────────────────

    #[test]
    fn from_str() {
        let s = Wtf8::from_str("hello");
        assert_eq!(s.as_bytes(), b"hello");
    }

    #[test]
    fn as_str_no_surrogates() {
        let s = Wtf8::from_str("hello");
        assert_eq!(s.as_str(), Some("hello"));
    }

    #[test]
    fn as_str_with_surrogate() {
        let bytes = [0xED, 0xA0, 0x80]; // U+D800
        // SAFETY: bytes are a valid lone-surrogate WTF-8 sequence (U+D800).
        let _s = unsafe { Wtf8::from_bytes_unchecked(&bytes) };
    }

    #[test]
    fn contains_lone_surrogates_false() {
        assert!(!Wtf8::from_str("hello").contains_lone_surrogates());
    }

    #[test]
    fn contains_lone_surrogates_true() {
        let bytes = [0xED, 0xA0, 0x80];
        // SAFETY: bytes are a valid lone-surrogate WTF-8 sequence (U+D800).
        let s = unsafe { Wtf8::from_bytes_unchecked(&bytes) };
        assert!(s.contains_lone_surrogates());
    }

    #[test]
    fn next_lone_surrogate_finds_it() {
        // "a" + U+D800 + "b"
        let bytes = [b'a', 0xED, 0xA0, 0x80, b'b'];
        // SAFETY: bytes are a valid WTF-8 sequence (ASCII 'a', lone surrogate U+D800, ASCII 'b').
        let s = unsafe { Wtf8::from_bytes_unchecked(&bytes) };
        let result = s.next_lone_surrogate(0);
        assert_eq!(result, Some((1, 0xD800)));
    }

    #[test]
    fn next_lone_surrogate_none() {
        let s = Wtf8::from_str("hello");
        assert_eq!(s.next_lone_surrogate(0), None);
    }

    #[test]
    fn to_str_lossy_no_surrogates() {
        let s = Wtf8::from_str("hello");
        assert!(matches!(s.to_str_lossy(), Cow::Borrowed("hello")));
    }

    #[test]
    fn to_str_lossy_with_surrogate() {
        // U+D800 alone
        let bytes = [0xED, 0xA0, 0x80];
        // SAFETY: bytes are a valid lone-surrogate WTF-8 sequence (U+D800).
        let s = unsafe { Wtf8::from_bytes_unchecked(&bytes) };
        let lossy = s.to_str_lossy();
        assert_eq!(&*lossy, "\u{FFFD}");
    }

    // ─── Wtf8Buf methods ─────────────────────────────────────────────────────

    #[test]
    fn wtf8buf_from_string() {
        let buf = Wtf8Buf::from_string(String::from("hello"));
        assert_eq!(buf.as_str(), Some("hello"));
    }

    #[test]
    fn wtf8buf_push_str() {
        let mut buf = Wtf8Buf::new();
        buf.push_str("hello");
        buf.push_str(" world");
        assert_eq!(buf.as_str(), Some("hello world"));
    }

    #[test]
    fn wtf8buf_push_code_point_surrogate() {
        let mut buf = Wtf8Buf::new();
        buf.push_code_point(0xD800);
        assert!(buf.contains_lone_surrogates());
        assert_eq!(buf.next_lone_surrogate(0), Some((0, 0xD800)));
    }

    #[test]
    fn wtf8buf_push_code_point_surrogate_pair_combines() {
        let mut buf = Wtf8Buf::new();
        buf.push_code_point(0xD83D); // lead surrogate
        buf.push_code_point(0xDE00); // trail surrogate
        // Should combine into U+1F600 (😀)
        assert_eq!(buf.as_str(), Some("\u{1F600}"));
    }

    #[test]
    fn wtf8buf_into_string_no_surrogates() {
        let buf = Wtf8Buf::from_str("hello");
        assert_eq!(buf.into_string(), Ok(String::from("hello")));
    }

    #[test]
    fn wtf8buf_into_string_with_surrogate() {
        let mut buf = Wtf8Buf::new();
        buf.push_code_point(0xD800);
        assert!(buf.into_string().is_err());
    }

    #[test]
    fn wtf8buf_into_string_lossy() {
        let mut buf = Wtf8Buf::new();
        buf.push_str("a");
        buf.push_code_point(0xD800);
        buf.push_str("b");
        assert_eq!(buf.into_string_lossy(), "a\u{FFFD}b");
    }

    #[test]
    fn wtf8buf_from_ill_formed_utf16_pair() {
        // Surrogate pair for U+1F600 (😀): [0xD83D, 0xDE00]
        let buf = Wtf8Buf::from_ill_formed_utf16(&[0xD83D, 0xDE00]);
        assert_eq!(buf.as_str(), Some("\u{1F600}"));
    }

    #[test]
    fn wtf8buf_from_ill_formed_utf16_lone() {
        // Lone lead surrogate
        let buf = Wtf8Buf::from_ill_formed_utf16(&[0xD800]);
        assert!(buf.contains_lone_surrogates());
        assert_eq!(buf.next_lone_surrogate(0), Some((0, 0xD800)));
    }

    // ─── CodePoints iterator ─────────────────────────────────────────────────

    #[test]
    fn code_points_all_unicode() {
        let s = Wtf8::from_str("ab");
        let cps: Vec<_> = s.code_points().collect();
        assert_eq!(cps, vec![CodePoint::Unicode('a'), CodePoint::Unicode('b')]);
    }

    #[test]
    fn code_points_with_surrogate() {
        let bytes = [0xED, 0xA0, 0x80]; // U+D800
        // SAFETY: bytes are a valid lone-surrogate WTF-8 sequence (U+D800).
        let s = unsafe { Wtf8::from_bytes_unchecked(&bytes) };
        let cps: Vec<_> = s.code_points().collect();
        assert_eq!(cps, vec![CodePoint::LoneSurrogate(0xD800)]);
    }

    // ─── IllFormedUtf16 iterator ─────────────────────────────────────────────

    #[test]
    fn ill_formed_utf16_ascii() {
        let s = Wtf8::from_str("AB");
        let units: Vec<u16> = s.to_ill_formed_utf16().collect();
        assert_eq!(units, vec![0x0041, 0x0042]);
    }

    #[test]
    fn ill_formed_utf16_supplementary() {
        let s = Wtf8::from_str("\u{1F600}");
        let units: Vec<u16> = s.to_ill_formed_utf16().collect();
        assert_eq!(units, vec![0xD83D, 0xDE00]);
    }

    #[test]
    fn ill_formed_utf16_lone_surrogate() {
        let bytes = [0xED, 0xB0, 0x80]; // U+DC00
        // SAFETY: bytes are a valid lone-surrogate WTF-8 sequence (U+DC00).
        let s = unsafe { Wtf8::from_bytes_unchecked(&bytes) };
        let units: Vec<u16> = s.to_ill_formed_utf16().collect();
        assert_eq!(units, vec![0xDC00]);
    }
}
