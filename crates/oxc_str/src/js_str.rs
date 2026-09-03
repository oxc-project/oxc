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

use oxc_allocator::{
    Allocator, CloneIn, CloneInSemanticIds, Dummy, FromIn, GetAllocator, Vec as ArenaVec,
};
#[cfg(feature = "serialize")]
use oxc_estree::{ESTree, Serializer as ESTreeSerializer};

use crate::{Ident, Str};

/// A code point in a JavaScript string, including lone surrogate code points.
///
/// Unlike [`char`], `JSChar` can represent values in the surrogate range
/// `U+D800..=U+DFFF`. Every `JSChar` is at most `U+10FFFF`.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct JSChar(u32);

impl JSChar {
    /// Create a JavaScript character from a code point.
    #[inline]
    pub const fn from_code_point(value: u32) -> Option<Self> {
        if value <= 0x10_FFFF { Some(Self(value)) } else { None }
    }

    /// Create a JavaScript character from a Unicode scalar value.
    #[inline]
    pub const fn from_char(value: char) -> Self {
        Self(value as u32)
    }

    /// Return the numeric code point value.
    #[inline]
    pub const fn value(self) -> u32 {
        self.0
    }

    /// Convert this value to a Unicode scalar value.
    ///
    /// Returns `None` if this value is a lone surrogate.
    #[inline]
    pub const fn to_char(self) -> Option<char> {
        if self.is_lone_surrogate() {
            None
        } else {
            // SAFETY: `JSChar` guarantees that the value is at most
            // `U+10FFFF`, and the surrogate range was excluded above.
            Some(unsafe { char::from_u32_unchecked(self.0) })
        }
    }

    /// Return whether this value is a lone surrogate.
    #[inline]
    pub const fn is_lone_surrogate(self) -> bool {
        is_surrogate(self.0)
    }

    /// Return whether this value is an ASCII whitespace character.
    #[inline]
    pub const fn is_ascii_whitespace(self) -> bool {
        matches!(self.0, 0x09 | 0x0A | 0x0C | 0x0D | 0x20)
    }

    /// Return the number of UTF-16 code units required for this value.
    #[inline]
    pub const fn len_utf16(self) -> usize {
        if self.0 > 0xFFFF { 2 } else { 1 }
    }

    /// Create a JavaScript character without checking its range.
    ///
    /// # Safety
    ///
    /// `value` must not exceed `0x10FFFF`.
    #[inline]
    const unsafe fn from_code_point_unchecked(value: u32) -> Self {
        Self(value)
    }
}

impl From<char> for JSChar {
    #[inline]
    fn from(value: char) -> Self {
        Self::from_char(value)
    }
}

impl PartialEq<char> for JSChar {
    #[inline]
    fn eq(&self, other: &char) -> bool {
        self.0 == *other as u32
    }
}

impl PartialEq<JSChar> for char {
    #[inline]
    fn eq(&self, other: &JSChar) -> bool {
        other == self
    }
}

/// An immutable JavaScript string.
///
/// JavaScript strings are sequences of UTF-16 code units, so unlike [`prim@str`]
/// they may contain lone surrogates. `JSStr` stores the same value as canonical
/// WTF-8 in borrowed source or arena memory.
///
/// The bytes are always canonical WTF-8, and `has_lone_surrogate` is `true` if
/// and only if they contain a lone surrogate encoding. Therefore, when the flag
/// is `false`, the bytes are valid UTF-8.
///
/// The lone-surrogate flag makes [`JSStr::as_str`] constant time.
///
/// On 64-bit platforms this type is 16 bytes, the same size as [`Str`] and
/// `&str`. On 32-bit platforms it is 12 bytes.
#[repr(C)]
pub struct JSStr<'a> {
    ptr: NonNull<u8>,
    len: u32,
    has_lone_surrogate: bool,
    _marker: PhantomData<&'a [u8]>,
}

impl JSStr<'static> {
    /// Return an empty JavaScript string.
    #[inline]
    pub const fn empty() -> Self {
        // SAFETY: `NonNull::dangling()` is valid for reads of zero bytes, and
        // an empty string has no lone surrogate.
        unsafe { Self::from_raw_parts(NonNull::dangling(), 0, false) }
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
        let mut builder = JSStrBuilder::with_capacity_in(units.len(), allocator.allocator());
        builder.push_utf16(units);
        builder.finish()
    }

    /// Concatenate a fixed-size array of JavaScript strings in an arena.
    ///
    /// Surrogate pairs split across input boundaries are combined.
    pub fn from_js_strs_array_in<const N: usize>(
        values: [JSStr<'_>; N],
        allocator: &impl GetAllocator<'a>,
    ) -> Self {
        let capacity = values.iter().map(|value| value.len()).sum();
        let mut builder = JSStrBuilder::with_capacity_in(capacity, allocator.allocator());
        for value in values {
            builder.push_js_str(value);
        }
        builder.finish()
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
    ///
    /// This is not JavaScript's UTF-16 `String.prototype.length`.
    #[inline]
    pub const fn len(self) -> usize {
        self.len as usize
    }

    /// Return whether this string is empty.
    #[inline]
    pub const fn is_empty(self) -> bool {
        self.len == 0
    }

    /// Return the number of UTF-16 code units in this JavaScript string.
    #[inline]
    pub fn utf16_len(self) -> usize {
        // Every non-continuation byte starts one code point, and every 4-byte
        // sequence needs one additional UTF-16 code unit. A lone surrogate is
        // a 3-byte WTF-8 sequence, so it contributes one code unit.
        self.as_bytes()
            .iter()
            .map(|&byte| usize::from(byte.cast_signed() >= -0x40) + usize::from(byte >= 0xF0))
            .sum()
    }

    /// Iterate over the UTF-16 code units in this JavaScript string.
    #[inline]
    #[expect(
        clippy::cast_possible_truncation,
        reason = "code point ranges are checked before conversion"
    )]
    pub fn encode_utf16(self) -> impl Iterator<Item = u16> + 'a {
        self.chars().flat_map(|js_char| {
            let code_point = js_char.value();
            let (units, len) = if code_point <= 0xFFFF {
                ([code_point as u16, 0], 1)
            } else {
                let offset = code_point - 0x10000;
                ([0xD800 | (offset >> 10) as u16, 0xDC00 | (offset & 0x3FF) as u16], 2)
            };
            units.into_iter().take(len)
        })
    }

    /// Return whether this string contains at least one lone surrogate.
    #[inline]
    pub const fn has_lone_surrogate(self) -> bool {
        self.has_lone_surrogate
    }

    /// Iterate over JavaScript characters.
    ///
    /// Each item is either a Unicode scalar value or a lone surrogate. A valid
    /// UTF-16 surrogate pair is returned as one supplementary character.
    #[inline]
    pub fn chars(self) -> impl Iterator<Item = JSChar> + 'a {
        JSChars { remaining: self.as_bytes() }
    }

    /// Return whether this string starts with `prefix`.
    #[inline]
    pub fn starts_with(self, prefix: &str) -> bool {
        self.as_bytes().starts_with(prefix.as_bytes())
    }

    /// Return whether this string ends with `suffix`.
    #[inline]
    pub fn ends_with(self, suffix: &str) -> bool {
        self.as_bytes().ends_with(suffix.as_bytes())
    }

    /// Return whether this string contains `needle`.
    #[inline]
    pub fn contains(self, needle: &str) -> bool {
        if needle.is_empty() {
            return true;
        }
        self.as_bytes().windows(needle.len()).any(|window| window == needle.as_bytes())
    }

    /// Return the canonical WTF-8 bytes used internally.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn as_bytes(self) -> &'a [u8] {
        // SAFETY: All constructors guarantee that `ptr` points to `len` bytes
        // of immutable storage which remains valid for `'a`.
        unsafe { slice::from_raw_parts(self.ptr.as_ptr(), self.len as usize) }
    }

    /// Create a JavaScript string from unchecked WTF-8 bytes.
    ///
    /// # Safety
    ///
    /// - `bytes` must contain canonical WTF-8.
    /// - `has_lone_surrogate` must be `true` if and only if `bytes` contains a
    ///   lone surrogate encoding.
    #[inline]
    unsafe fn from_wtf8_bytes_unchecked(bytes: &'a [u8], has_lone_surrogate: bool) -> Self {
        let len = u32::try_from(bytes.len()).expect("JavaScript strings cannot exceed 4 GiB");
        let ptr = NonNull::from(bytes).cast::<u8>();

        debug_assert_eq!(contains_lone_surrogate(bytes), has_lone_surrogate);

        // SAFETY: `ptr` and `len` came from `bytes`, which remains immutable and
        // valid for `'a`. The remaining requirements are guaranteed by caller.
        unsafe { Self::from_raw_parts(ptr, len, has_lone_surrogate) }
    }

    /// Copy unchecked WTF-8 bytes into an arena.
    ///
    /// # Safety
    ///
    /// - `bytes` must contain canonical WTF-8.
    /// - `has_lone_surrogate` must be `true` if and only if `bytes` contains a
    ///   lone surrogate encoding.
    #[inline]
    unsafe fn copy_wtf8_bytes_in_unchecked<'new_alloc>(
        bytes: &[u8],
        has_lone_surrogate: bool,
        allocator: &impl GetAllocator<'new_alloc>,
    ) -> JSStr<'new_alloc> {
        let bytes = allocator.allocator().alloc_slice_copy(bytes);
        // SAFETY: The bytes were copied unchanged, so the caller's guarantees
        // still hold for the arena-backed copy.
        unsafe { JSStr::from_wtf8_bytes_unchecked(bytes, has_lone_surrogate) }
    }

    /// Create a string from raw components.
    ///
    /// # Safety
    ///
    /// - `ptr` must point to `len` bytes of canonical WTF-8 which are valid
    ///   and immutable for `'a`.
    /// - `has_lone_surrogate` must match those bytes.
    #[inline]
    const unsafe fn from_raw_parts(ptr: NonNull<u8>, len: u32, has_lone_surrogate: bool) -> Self {
        Self { ptr, len, has_lone_surrogate, _marker: PhantomData }
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
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.len == other.len
            && self.has_lone_surrogate == other.has_lone_surrogate
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
        if let Some(value) = self.as_str() {
            value.hash(hasher);
        } else {
            // Use the same prefix-free byte sequence as `str` for canonical
            // WTF-8 that cannot be passed to `str::hash`.
            hasher.write(self.as_bytes());
            hasher.write_u8(0xFF);
        }
    }
}

impl fmt::Debug for JSStr<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("\"")?;
        for js_char in self.chars() {
            if let Some(value) = js_char.to_char() {
                for escaped in value.escape_debug() {
                    write!(formatter, "{escaped}")?;
                }
            } else {
                let value = js_char.value();
                write!(formatter, "\\u{{{value:04X}}}")?;
            }
        }
        formatter.write_str("\"")
    }
}

#[cfg(feature = "serialize")]
impl ESTree for JSStr<'_> {
    fn serialize<S: ESTreeSerializer>(&self, mut serializer: S) {
        const HEX: &[u8; 16] = b"0123456789abcdef";

        if let Some(value) = self.as_str() {
            value.serialize(serializer);
            return;
        }

        let buffer = serializer.buffer_mut();
        buffer.print_ascii_byte(b'"');
        for js_char in self.chars() {
            let value = js_char.value();
            let short_escape = match value {
                0x08 => Some("\\b"),
                0x09 => Some("\\t"),
                0x0A => Some("\\n"),
                0x0C => Some("\\f"),
                0x0D => Some("\\r"),
                0x22 => Some("\\\""),
                0x5C => Some("\\\\"),
                _ => None,
            };
            if let Some(escaped) = short_escape {
                buffer.print_str(escaped);
            } else if value <= 0x1F {
                buffer.print_str("\\u00");
                buffer.print_ascii_byte(HEX[((value >> 4) & 0xF) as usize]);
                buffer.print_ascii_byte(HEX[(value & 0xF) as usize]);
            } else if js_char.is_lone_surrogate() {
                buffer.print_str("\\u");
                for shift in [12, 8, 4, 0] {
                    buffer.print_ascii_byte(HEX[((value >> shift) & 0xF) as usize]);
                }
            } else {
                let mut encoded = [0; 4];
                // `JSChar::to_char` only returns `None` for lone surrogates,
                // which were handled above.
                buffer.print_str(js_char.to_char().unwrap().encode_utf8(&mut encoded));
            }
        }
        buffer.print_ascii_byte(b'"');
    }
}

impl<'a> From<&'a str> for JSStr<'a> {
    #[inline]
    fn from(value: &'a str) -> Self {
        // SAFETY: Valid UTF-8 is canonical WTF-8 and cannot contain lone
        // surrogate encodings.
        unsafe { Self::from_wtf8_bytes_unchecked(value.as_bytes(), false) }
    }
}

impl<'a> From<Str<'a>> for JSStr<'a> {
    #[inline]
    fn from(value: Str<'a>) -> Self {
        Self::from(value.as_str())
    }
}

impl<'a> From<Ident<'a>> for JSStr<'a> {
    #[inline]
    fn from(value: Ident<'a>) -> Self {
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
        // SAFETY: `self` guarantees canonical WTF-8 and a matching flag. The
        // bytes are copied unchanged into `allocator`.
        unsafe {
            JSStr::copy_wtf8_bytes_in_unchecked(
                self.as_bytes(),
                self.has_lone_surrogate(),
                &allocator,
            )
        }
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

/// Arena-backed builder for [`JSStr`].
///
/// This is the JavaScript-string counterpart of `ArenaStringBuilder`. It
/// accepts UTF-8 text, UTF-16 code units, and existing `JSStr` values while
/// preserving canonical WTF-8 across append boundaries.
pub struct JSStrBuilder<'a> {
    bytes: ArenaVec<'a, u8>,
    pending_lead_surrogate: Option<u16>,
    has_lone_surrogate: bool,
}

impl<'a> JSStrBuilder<'a> {
    /// Create an empty builder in `allocator` without allocating.
    #[inline]
    pub fn new_in(allocator: &'a Allocator) -> Self {
        Self {
            bytes: ArenaVec::new_in(&allocator),
            pending_lead_surrogate: None,
            has_lone_surrogate: false,
        }
    }

    /// Create an empty builder with space for at least `capacity` WTF-8 bytes.
    #[inline]
    pub fn with_capacity_in(capacity: usize, allocator: &'a Allocator) -> Self {
        Self {
            bytes: ArenaVec::with_capacity_in(capacity, &allocator),
            pending_lead_surrogate: None,
            has_lone_surrogate: false,
        }
    }

    /// Append valid UTF-8 text.
    #[inline]
    pub fn push_str(&mut self, value: &str) {
        if value.is_empty() {
            return;
        }
        self.flush_pending_lead_surrogate();
        self.bytes.extend_from_slice(value.as_bytes());
    }

    /// Append a Unicode scalar value.
    #[inline]
    pub fn push_char(&mut self, value: char) {
        self.flush_pending_lead_surrogate();
        push_scalar(&mut self.bytes, value);
    }

    /// Append one JavaScript character.
    ///
    /// Lone surrogates are routed through the UTF-16 boundary logic so that a
    /// lead and trail surrogate supplied by consecutive pushes form one
    /// canonical supplementary character.
    #[inline]
    pub fn push_js_char(&mut self, value: JSChar) {
        if let Some(value) = value.to_char() {
            self.push_char(value);
        } else {
            #[expect(clippy::cast_possible_truncation, reason = "lone surrogates fit in u16")]
            self.push_code_unit(value.value() as u16);
        }
    }

    /// Append one UTF-16 code unit.
    ///
    /// A lead surrogate is held until the following append so that a trail
    /// surrogate arriving at a chunk boundary can be encoded canonically.
    pub fn push_code_unit(&mut self, unit: u16) {
        if let Some(lead) = self.pending_lead_surrogate.take() {
            if is_trail_surrogate(u32::from(unit)) {
                push_scalar(&mut self.bytes, decode_surrogate_pair(lead, unit));
                return;
            }
            self.push_lone_surrogate(lead);
        }

        if is_lead_surrogate(u32::from(unit)) {
            self.pending_lead_surrogate = Some(unit);
        } else if is_trail_surrogate(u32::from(unit)) {
            self.push_lone_surrogate(unit);
        } else {
            // SAFETY: A non-surrogate `u16` is a Unicode scalar value.
            push_scalar(&mut self.bytes, unsafe { char::from_u32_unchecked(u32::from(unit)) });
        }
    }

    /// Append potentially ill-formed UTF-16.
    #[inline]
    pub fn push_utf16(&mut self, units: &[u16]) {
        for &unit in units {
            self.push_code_unit(unit);
        }
    }

    /// Append another JavaScript string.
    ///
    /// This repairs the only boundary that cannot be copied byte-for-byte: a
    /// pending lead surrogate followed by an initial trail surrogate. A final
    /// lead surrogate is retained for the next append.
    pub fn push_js_str(&mut self, value: JSStr<'_>) {
        let bytes = value.as_bytes();
        if bytes.is_empty() {
            return;
        }

        let mut start = 0;
        if let Some(lead) = self.pending_lead_surrogate.take() {
            if let Some(trail) = initial_trail_surrogate(bytes) {
                push_scalar(&mut self.bytes, decode_surrogate_pair(lead, trail));
                start = 3;
            } else {
                self.push_lone_surrogate(lead);
            }
        }

        let mut end = bytes.len();
        let final_lead = final_lead_surrogate(&bytes[start..end]);
        if final_lead.is_some() {
            end -= 3;
        }

        let copied = &bytes[start..end];
        self.bytes.extend_from_slice(copied);
        if start == 0 && end == bytes.len() {
            self.has_lone_surrogate |= value.has_lone_surrogate();
        } else {
            // Boundary repair removed at least one surrogate. Scan only the
            // remaining bytes, which is a rare path.
            self.has_lone_surrogate |= contains_lone_surrogate(copied);
        }
        self.pending_lead_surrogate = final_lead;
    }

    /// Finish construction and return the arena-backed JavaScript string.
    pub fn finish(mut self) -> JSStr<'a> {
        self.flush_pending_lead_surrogate();
        let has_lone_surrogate = self.has_lone_surrogate;
        let bytes = self.bytes.into_arena_slice();
        // SAFETY: All builder entry points preserve canonical WTF-8, and the
        // builder updates the flag whenever it writes a lone surrogate.
        unsafe { JSStr::from_wtf8_bytes_unchecked(bytes, has_lone_surrogate) }
    }

    #[inline]
    fn push_lone_surrogate(&mut self, value: u16) {
        push_surrogate(&mut self.bytes, value);
        self.has_lone_surrogate = true;
    }

    #[inline]
    fn flush_pending_lead_surrogate(&mut self) {
        if let Some(value) = self.pending_lead_surrogate.take() {
            self.push_lone_surrogate(value);
        }
    }
}

struct JSChars<'a> {
    remaining: &'a [u8],
}

impl Iterator for JSChars<'_> {
    type Item = JSChar;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining.is_empty() {
            return None;
        }
        let (value, len) = decode_next_valid_wtf8(self.remaining);
        self.remaining = &self.remaining[len..];
        // SAFETY: Decoding canonical WTF-8 always produces a value in the
        // Unicode code point range.
        Some(unsafe { JSChar::from_code_point_unchecked(value) })
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining.len().div_ceil(4), Some(self.remaining.len()))
    }
}

#[inline]
#[expect(clippy::cast_possible_truncation, reason = "a three-byte sequence is at most U+FFFF")]
fn initial_trail_surrogate(bytes: &[u8]) -> Option<u16> {
    if bytes.len() < 3 || bytes[0] != 0xED || !(0xB0..=0xBF).contains(&bytes[1]) {
        return None;
    }
    Some(decode_three_byte_sequence(bytes[0], bytes[1], bytes[2]) as u16)
}

#[inline]
#[expect(clippy::cast_possible_truncation, reason = "a three-byte sequence is at most U+FFFF")]
fn final_lead_surrogate(bytes: &[u8]) -> Option<u16> {
    if bytes.len() < 3 {
        return None;
    }
    let bytes = &bytes[bytes.len() - 3..];
    if bytes[0] != 0xED || !(0xA0..=0xAF).contains(&bytes[1]) {
        return None;
    }
    Some(decode_three_byte_sequence(bytes[0], bytes[1], bytes[2]) as u16)
}

#[inline]
fn push_scalar(bytes: &mut ArenaVec<'_, u8>, value: char) {
    let mut encoded = [0; 4];
    bytes.extend_from_slice(value.encode_utf8(&mut encoded).as_bytes());
}

#[inline]
#[expect(clippy::cast_possible_truncation, reason = "surrogate encodings only use the low 16 bits")]
fn push_surrogate(bytes: &mut ArenaVec<'_, u8>, value: u16) {
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
        collections::hash_map::DefaultHasher,
        fmt::Display,
        mem::{needs_drop, offset_of, size_of},
        ops::Deref,
    };

    use oxc_allocator::{Allocator, CloneIn};
    use oxc_data_structures::types::implements;

    use super::*;

    fn code_points(value: JSStr<'_>) -> Vec<u32> {
        value.chars().map(JSChar::value).collect()
    }

    #[test]
    fn layout() {
        #[cfg(target_pointer_width = "64")]
        {
            assert_eq!(size_of::<JSStr<'_>>(), 16);
            assert_eq!(size_of::<Option<JSStr<'_>>>(), 16);
            assert_eq!(size_of::<JSStr<'_>>(), size_of::<Str<'_>>());
            assert_eq!(offset_of!(JSStr<'_>, ptr), 0);
            assert_eq!(offset_of!(JSStr<'_>, len), 8);
            assert_eq!(offset_of!(JSStr<'_>, has_lone_surrogate), 12);
        }
        #[cfg(target_pointer_width = "32")]
        {
            assert_eq!(size_of::<JSStr<'_>>(), 12);
            assert_eq!(size_of::<Option<JSStr<'_>>>(), 12);
            assert_eq!(offset_of!(JSStr<'_>, ptr), 0);
            assert_eq!(offset_of!(JSStr<'_>, len), 4);
            assert_eq!(offset_of!(JSStr<'_>, has_lone_surrogate), 8);
        }

        // Raw transfer reads this niche value to deserialize `Option<JSStr>`.
        let value = Option::<JSStr<'_>>::None;
        let flag_offset = offset_of!(JSStr<'_>, has_lone_surrogate);
        // SAFETY: The bool niche is the initialized discriminant byte of `None`.
        let discriminant =
            unsafe { std::ptr::from_ref(&value).cast::<u8>().add(flag_offset).read() };
        assert_eq!(discriminant, 2);
    }

    #[test]
    fn js_char_contract() {
        assert_eq!(size_of::<JSChar>(), size_of::<u32>());

        let ascii = JSChar::from_char('A');
        assert_eq!(ascii.value(), 0x41);
        assert_eq!(ascii.to_char(), Some('A'));
        assert!(!ascii.is_lone_surrogate());
        assert!(!ascii.is_ascii_whitespace());
        assert_eq!(ascii.len_utf16(), 1);
        assert_eq!(ascii, 'A');
        assert_eq!('A', ascii);

        let whitespace = JSChar::from_char('\n');
        assert!(whitespace.is_ascii_whitespace());
        let vertical_tab = JSChar::from_char('\u{0B}');
        assert!(!vertical_tab.is_ascii_whitespace());

        let supplementary = JSChar::from_char('🔥');
        assert_eq!(supplementary.value(), 0x1F525);
        assert_eq!(supplementary.to_char(), Some('🔥'));
        assert!(!supplementary.is_lone_surrogate());
        assert_eq!(supplementary.len_utf16(), 2);

        let surrogate = JSChar::from_code_point(0xD800).unwrap();
        assert_eq!(surrogate.to_char(), None);
        assert!(surrogate.is_lone_surrogate());
        assert_eq!(surrogate.len_utf16(), 1);

        assert!(JSChar::from_code_point(0x10_FFFF).is_some());
        assert!(JSChar::from_code_point(0x11_0000).is_none());
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
    fn utf8_fast_path() {
        let value = JSStr::from("hello 🔥");
        assert_eq!(value.as_str(), Some("hello 🔥"));
        assert!(!value.has_lone_surrogate());
        assert_eq!(code_points(value), [0x68, 0x65, 0x6C, 0x6C, 0x6F, 0x20, 0x1F525]);
    }

    #[test]
    fn utf8_queries_work_with_lone_surrogates() {
        let allocator = Allocator::new();
        let value = JSStr::from_utf16_in(
            &[0x61, 0x62, 0xD800, 0x63, 0x64, 0xDC00, 0x65, 0x66],
            &&allocator,
        );

        assert!(value.starts_with("ab"));
        assert!(value.ends_with("ef"));
        assert!(value.contains("cd"));
        assert!(value.contains(""));
        assert!(!value.starts_with("abc"));
        assert!(!value.ends_with("def"));
        assert!(!value.contains("bc"));
        assert!(!value.contains("de"));
    }

    #[test]
    fn ill_formed_utf16() {
        let allocator = Allocator::new();
        let value = JSStr::from_utf16_in(&[0x61, 0xD800, 0x62, 0xDC00], &&allocator);

        assert!(value.has_lone_surrogate());
        assert_eq!(value.as_str(), None);
        assert_eq!(code_points(value), [0x61, 0xD800, 0x62, 0xDC00]);
        assert_eq!(format!("{value:?}"), "\"a\\u{D800}b\\u{DC00}\"");
    }

    #[test]
    fn utf16_pair_becomes_scalar() {
        let allocator = Allocator::new();
        let value = JSStr::from_utf16_in(&[0xD83D, 0xDD25], &&allocator);

        assert_eq!(value.as_str(), Some("🔥"));
        assert!(!value.has_lone_surrogate());
        assert_eq!(code_points(value), [0x1F525]);
    }

    #[test]
    fn builder_combines_pair_across_code_unit_boundary() {
        let allocator = Allocator::new();
        let mut builder = JSStrBuilder::new_in(&allocator);
        builder.push_code_unit(0xD83D);
        builder.push_code_unit(0xDD25);
        let value = builder.finish();

        assert_eq!(value.as_str(), Some("🔥"));
        assert!(!value.has_lone_surrogate());
    }

    #[test]
    fn builder_combines_pair_across_js_char_boundary() {
        let allocator = Allocator::new();
        let mut builder = JSStrBuilder::new_in(&allocator);
        builder.push_js_char(JSChar::from_code_point(0xD83D).unwrap());
        builder.push_js_char(JSChar::from_code_point(0xDD25).unwrap());
        let value = builder.finish();

        assert_eq!(value.as_str(), Some("🔥"));
        assert!(!value.has_lone_surrogate());
    }

    #[test]
    fn builder_combines_pair_across_js_str_boundary() {
        let allocator = Allocator::new();
        let lead = JSStr::from_utf16_in(&[0xD83D], &&allocator);
        let trail = JSStr::from_utf16_in(&[0xDD25], &&allocator);
        let mut builder = JSStrBuilder::new_in(&allocator);
        builder.push_js_str(lead);
        builder.push_js_str(JSStr::from(""));
        builder.push_js_str(trail);
        let value = builder.finish();

        assert_eq!(value.as_str(), Some("🔥"));
        assert!(!value.has_lone_surrogate());
    }

    #[test]
    fn builder_preserves_other_lone_surrogates() {
        let allocator = Allocator::new();
        let left = JSStr::from_utf16_in(&[0xD800, 0xD83D], &&allocator);
        let right = JSStr::from_utf16_in(&[0xDD25, 0xDC00], &&allocator);
        let mut builder = JSStrBuilder::new_in(&allocator);
        builder.push_js_str(left);
        builder.push_js_str(right);
        let value = builder.finish();

        assert!(value.has_lone_surrogate());
        assert_eq!(code_points(value), [0xD800, 0x1F525, 0xDC00]);
    }

    #[test]
    fn chars_round_trip_through_builder() {
        let allocator = Allocator::new();
        let value =
            JSStr::from_utf16_in(&[0x61, 0xD800, 0x62, 0xD83D, 0xDD25, 0xDC00], &&allocator);
        let mut builder = JSStrBuilder::new_in(&allocator);
        for js_char in value.chars() {
            builder.push_js_char(js_char);
        }

        assert_eq!(builder.finish(), value);
    }

    #[test]
    fn builder_flushes_lead_before_utf8() {
        let allocator = Allocator::new();
        let mut builder = JSStrBuilder::new_in(&allocator);
        builder.push_code_unit(0xD800);
        builder.push_str("a");
        builder.push_char('🔥');
        let value = builder.finish();

        assert!(value.has_lone_surrogate());
        assert_eq!(code_points(value), [0xD800, 0x61, 0x1F525]);
    }

    #[test]
    fn builder_matches_utf16_concatenation() {
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
                let mut builder = JSStrBuilder::new_in(&allocator);
                builder.push_js_str(left);
                builder.push_js_str(right);
                let actual = builder.finish();

                assert_eq!(actual, expected, "failed at split {split} for {units:04X?}");
                assert_eq!(actual.has_lone_surrogate(), expected.has_lone_surrogate());
            }
        }
    }

    #[test]
    fn concatenate_array_repairs_boundaries() {
        let allocator = Allocator::new();
        let lead = JSStr::from_utf16_in(&[0xD83D], &&allocator);
        let trail = JSStr::from_utf16_in(&[0xDD25], &&allocator);
        let value = JSStr::from_js_strs_array_in([JSStr::from("a"), lead, trail], &&allocator);

        assert_eq!(value.as_str(), Some("a🔥"));
    }

    #[test]
    fn equality_uses_bytes() {
        let left = JSStr::from("string-a");
        let same = JSStr::from("string-a");
        let different = JSStr::from("string-b");

        assert_eq!(left, same);
        assert_ne!(left, different);
    }

    #[test]
    fn hash_matches_str_for_utf8() {
        let value = JSStr::from("string-a");
        let mut js_str_hasher = DefaultHasher::new();
        value.hash(&mut js_str_hasher);

        let mut str_hasher = DefaultHasher::new();
        "string-a".hash(&mut str_hasher);

        assert_eq!(js_str_hasher.finish(), str_hasher.finish());
    }

    #[test]
    fn clone_in_preserves_value_and_flag() {
        let source_allocator = Allocator::new();
        let destination_allocator = Allocator::new();
        let source = JSStr::from_utf16_in(&[0x61, 0xD800], &&source_allocator);
        let cloned = source.clone_in(&destination_allocator);

        assert_eq!(source, cloned);
        assert_eq!(source.has_lone_surrogate, cloned.has_lone_surrogate);
        assert_ne!(source.as_bytes().as_ptr(), cloned.as_bytes().as_ptr());
    }

    #[test]
    fn every_utf16_code_unit_round_trips() {
        let allocator = Allocator::new();
        for unit in 0..=u16::MAX {
            let value = JSStr::from_utf16_in(&[unit], &&allocator);
            assert_eq!(value.encode_utf16().collect::<Vec<_>>(), [unit]);
        }
    }

    #[test]
    fn representative_utf16_sequences_round_trip() {
        let allocator = Allocator::new();
        for units in [
            vec![],
            vec![0x61, 0x62],
            vec![0xD800],
            vec![0xDC00],
            vec![0xD83D, 0xDD25],
            vec![0xD800, 0xD83D, 0xDD25, 0xDC00],
            vec![0xD800, 0xD800, 0xDC00, 0xDC00],
        ] {
            let value = JSStr::from_utf16_in(&units, &&allocator);
            assert_eq!(value.encode_utf16().collect::<Vec<_>>(), units);
            assert_eq!(value.utf16_len(), units.len());
        }
    }

    #[cfg(feature = "serialize")]
    #[test]
    fn estree_serialization() {
        use oxc_estree::{CompactSerializer, ESTree};

        let allocator = Allocator::new();
        let value =
            JSStr::from_utf16_in(&[0x22, 0x5C, 0x0A, 0xD800, 0x61, 0xDC00, 0xFFFD], &&allocator);
        let mut serializer = CompactSerializer::default();
        value.serialize(&mut serializer);

        assert_eq!(serializer.into_string(), "\"\\\"\\\\\\n\\ud800a\\udc00�\"");
    }
}
