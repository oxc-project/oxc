//! Arena-allocated WTF-8 atom: [`Wtf8Atom`].
//!
//! [`Wtf8Atom`] is the WTF-8 analogue of [`Atom`], storing a reference into the arena
//! that may represent a string with lone surrogates.
//!
//! [`Atom`]: oxc_str::Atom

use std::{borrow::Cow, fmt, hash, ops::Deref};

use oxc_allocator::{Allocator, CloneIn, Dummy, FromIn, TakeIn};
use oxc_span::ContentEq;
use oxc_str::Atom;
#[cfg(feature = "serialize")]
use serde::{Serialize, Serializer as SerdeSerializer};

use crate::Wtf8;

/// An arena-allocated WTF-8 string atom.
///
/// Analogous to [`Atom<'a>`][oxc_str::Atom], but the underlying string may contain
/// lone surrogate code units (U+D800..=U+DFFF) encoded in WTF-8.
///
/// `Wtf8Atom<'a>` is a thin reference — it is [`Copy`] and cheap to clone.
///
/// # Conversions
///
/// | Target          | Method                  | Notes                               |
/// |-----------------|-------------------------|-------------------------------------|
/// | `&Wtf8`         | [`as_wtf8`]             | Always succeeds                     |
/// | `Option<&str>`  | [`as_str`]              | `None` if string has lone surrogates|
/// | `Cow<str>`      | [`to_str_lossy`]        | Replaces surrogates with `U+FFFD`   |
/// | `Option<Atom>`  | [`try_into_atom`]       | `None` if string has lone surrogates|
///
/// [`as_wtf8`]: Wtf8Atom::as_wtf8
/// [`as_str`]: Wtf8Atom::as_str
/// [`to_str_lossy`]: Wtf8Atom::to_str_lossy
/// [`try_into_atom`]: Wtf8Atom::try_into_atom
#[repr(transparent)]
#[derive(Clone, Copy, Eq)]
pub struct Wtf8Atom<'a>(&'a Wtf8);

impl<'a> Wtf8Atom<'a> {
    // ─── Construction ─────────────────────────────────────────────────────────

    /// Create a [`Wtf8Atom`] from a `&Wtf8` reference, borrowing from the same lifetime.
    ///
    /// No allocation is performed.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub const fn from_wtf8(s: &'a Wtf8) -> Self {
        Self(s)
    }

    /// Create a [`Wtf8Atom`] from a UTF-8 `&str`, borrowing from the same lifetime.
    ///
    /// No allocation is performed. Since WTF-8 is a superset of UTF-8, this always succeeds.
    #[expect(clippy::inline_always, clippy::should_implement_trait)]
    #[inline(always)]
    pub fn from_str(s: &'a str) -> Self {
        Self(Wtf8::from_str(s))
    }

    /// Create an empty [`Wtf8Atom`].
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub fn empty() -> Self {
        Self::from_str("")
    }

    // ─── Accessors ────────────────────────────────────────────────────────────

    /// Borrow the underlying [`Wtf8`] slice.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub fn as_wtf8(&self) -> &'a Wtf8 {
        self.0
    }

    /// Try to get a `&str` view of this atom.
    ///
    /// Returns `None` if the string contains lone surrogates.
    #[inline]
    pub fn as_str(&self) -> Option<&'a str> {
        self.0.as_str()
    }

    /// Convert to a lossy `Cow<str>`.
    ///
    /// Returns `Cow::Borrowed` when the string is valid UTF-8 (no lone surrogates).
    /// Returns `Cow::Owned` with lone surrogates replaced by `U+FFFD` otherwise.
    #[inline]
    pub fn to_str_lossy(self) -> Cow<'a, str> {
        self.0.to_str_lossy()
    }

    // ─── Conversion ───────────────────────────────────────────────────────────

    /// Try to convert this atom into an [`Atom<'a>`].
    ///
    /// Returns `None` if the string contains lone surrogates.
    #[inline]
    pub fn try_into_atom(self) -> Option<Atom<'a>> {
        let s = self.0.as_str()?;
        Some(Atom::from(s))
    }

    /// Return `true` if the string is well-formed UTF-8 (contains no lone surrogates).
    #[inline]
    pub fn is_well_formed(&self) -> bool {
        self.0.is_well_formed()
    }

    /// Return `true` if the string contains at least one lone surrogate code point.
    #[inline]
    pub fn contains_lone_surrogates(&self) -> bool {
        self.0.contains_lone_surrogates()
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Allocator traits
// ──────────────────────────────────────────────────────────────────────────────

impl<'new_alloc> CloneIn<'new_alloc> for Wtf8Atom<'_> {
    type Cloned = Wtf8Atom<'new_alloc>;

    #[inline]
    fn clone_in(&self, allocator: &'new_alloc Allocator) -> Self::Cloned {
        Wtf8Atom::from_in(self.0, allocator)
    }
}

impl<'a> Dummy<'a> for Wtf8Atom<'a> {
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn dummy(_allocator: &'a Allocator) -> Self {
        Wtf8Atom::empty()
    }
}

impl<'alloc> FromIn<'alloc, &Wtf8> for Wtf8Atom<'alloc> {
    /// Allocate a [`Wtf8`] slice into the arena and return a [`Wtf8Atom`].
    #[inline]
    fn from_in(s: &Wtf8, allocator: &'alloc Allocator) -> Self {
        let bytes = allocator.alloc_slice_copy(s.as_bytes());
        // SAFETY: bytes is a copy of valid WTF-8 bytes;
        // `Wtf8` is `#[repr(transparent)]` over `[u8]` so the fat-pointer cast is valid.
        #[expect(clippy::ref_as_ptr)]
        let wtf8 = unsafe { &*(bytes as *const [u8] as *const Wtf8) };
        Self(wtf8)
    }
}

impl<'alloc> FromIn<'alloc, &str> for Wtf8Atom<'alloc> {
    #[inline]
    fn from_in(s: &str, allocator: &'alloc Allocator) -> Self {
        let arena_str = allocator.alloc_str(s);
        Self(Wtf8::from_str(arena_str))
    }
}

impl<'alloc> FromIn<'alloc, String> for Wtf8Atom<'alloc> {
    #[inline]
    fn from_in(s: String, allocator: &'alloc Allocator) -> Self {
        Self::from_in(s.as_str(), allocator)
    }
}

impl ContentEq for Wtf8Atom<'_> {
    fn content_eq(&self, other: &Self) -> bool {
        self == other
    }
}

impl<'a> TakeIn<'a> for Wtf8Atom<'a> {}

// ──────────────────────────────────────────────────────────────────────────────
// Standard trait implementations
// ──────────────────────────────────────────────────────────────────────────────

impl<'a> From<&'a Wtf8> for Wtf8Atom<'a> {
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn from(s: &'a Wtf8) -> Self {
        Self::from_wtf8(s)
    }
}

impl<'a> From<&'a str> for Wtf8Atom<'a> {
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn from(s: &'a str) -> Self {
        Self::from_str(s)
    }
}

impl<'a> From<Atom<'a>> for Wtf8Atom<'a> {
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn from(atom: Atom<'a>) -> Self {
        // `Atom` is always valid UTF-8, so this is always valid WTF-8.
        Self::from_str(atom.as_str())
    }
}

impl Deref for Wtf8Atom<'_> {
    type Target = Wtf8;

    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn deref(&self) -> &Wtf8 {
        self.0
    }
}

impl PartialEq for Wtf8Atom<'_> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl PartialEq<str> for Wtf8Atom<'_> {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl PartialEq<&str> for Wtf8Atom<'_> {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl PartialEq<Wtf8> for Wtf8Atom<'_> {
    #[inline]
    fn eq(&self, other: &Wtf8) -> bool {
        self.0 == other
    }
}

impl PartialEq<Wtf8Atom<'_>> for str {
    #[inline]
    fn eq(&self, other: &Wtf8Atom<'_>) -> bool {
        self == other.0
    }
}

impl hash::Hash for Wtf8Atom<'_> {
    #[inline]
    fn hash<H: hash::Hasher>(&self, hasher: &mut H) {
        self.0.hash(hasher);
    }
}

impl fmt::Debug for Wtf8Atom<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.0, f)
    }
}

impl fmt::Display for Wtf8Atom<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.0, f)
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Serialization
// ──────────────────────────────────────────────────────────────────────────────

#[cfg(feature = "serialize")]
impl Serialize for Wtf8Atom<'_> {
    fn serialize<S: SerdeSerializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        // serde's JSON serializer does not support lone surrogates.
        // Replace them with U+FFFD for compatibility.
        match self.0.to_str_lossy() {
            Cow::Borrowed(s) => Serialize::serialize(s, serializer),
            Cow::Owned(ref s) => Serialize::serialize(s.as_str(), serializer),
        }
    }
}

#[cfg(feature = "serialize")]
impl oxc_estree::ESTree for Wtf8Atom<'_> {
    fn serialize<S: oxc_estree::Serializer>(&self, mut serializer: S) {
        let buf = serializer.buffer_mut();
        buf.print_ascii_byte(b'"');
        write_wtf8_as_json_str(self.0.as_bytes(), buf);
        buf.print_ascii_byte(b'"');
    }
}

/// Write WTF-8 bytes as a JSON string body (without surrounding quotes).
///
/// Lone surrogates (WTF-8 sequences `0xED 0xA0..=0xBF 0x80..=0xBF`) are output as
/// `\uXXXX` JSON escape sequences. All other characters use standard JSON escaping.
#[cfg(feature = "serialize")]
fn write_wtf8_as_json_str(bytes: &[u8], buf: &mut oxc_data_structures::code_buffer::CodeBuffer) {
    let mut i = 0;
    let mut chunk_start = 0;

    while i < bytes.len() {
        let b = bytes[i];

        // Check for lone surrogate: 0xED followed by 0xA0..=0xBF and a continuation byte.
        if b == 0xED
            && i + 2 < bytes.len()
            && (0xA0..=0xBF).contains(&bytes[i + 1])
            && bytes[i + 2] & 0xC0 == 0x80
        {
            // Flush the chunk before the surrogate.
            // SAFETY: chunk is valid UTF-8 (no surrogates).
            let chunk = unsafe { std::str::from_utf8_unchecked(&bytes[chunk_start..i]) };
            buf.print_str(chunk);

            let b1 = bytes[i + 1];
            let b2 = bytes[i + 2];
            let code_unit = ((u16::from(b1) & 0x3F) << 6) | (u16::from(b2) & 0x3F);
            let surrogate = 0xD000u16 | code_unit;

            // Write \uXXXX escape.
            buf.print_str("\\u");
            write_hex4(surrogate, buf);

            i += 3;
            chunk_start = i;
            continue;
        }

        // Standard JSON escaping for control characters and special characters.
        let escape: Option<&[u8]> = match b {
            0x08 => Some(b"\\b"),
            0x09 => Some(b"\\t"),
            0x0A => Some(b"\\n"),
            0x0C => Some(b"\\f"),
            0x0D => Some(b"\\r"),
            b'"' => Some(b"\\\""),
            b'\\' => Some(b"\\\\"),
            0x00..=0x1F | 0x7F => {
                // Control character — write \u00XX.
                // Flush chunk before.
                // SAFETY: bytes before this point are valid UTF-8.
                let chunk = unsafe { std::str::from_utf8_unchecked(&bytes[chunk_start..i]) };
                buf.print_str(chunk);
                buf.print_str("\\u00");
                write_hex2(b, buf);
                i += 1;
                chunk_start = i;
                continue;
            }
            _ => None,
        };

        if let Some(esc) = escape {
            // SAFETY: bytes between escapes are valid UTF-8 (lone surrogates are handled above).
            let chunk = unsafe { std::str::from_utf8_unchecked(&bytes[chunk_start..i]) };
            buf.print_str(chunk);
            // SAFETY: escape sequences are all valid UTF-8.
            let esc_str = unsafe { std::str::from_utf8_unchecked(esc) };
            buf.print_str(esc_str);
            i += 1;
            chunk_start = i;
        } else {
            // Multi-byte sequences: advance by the full sequence length.
            if b < 0x80 {
                i += 1;
            } else if b < 0xE0 {
                i += 2;
            } else if b < 0xF0 {
                i += 3;
            } else {
                i += 4;
            }
        }
    }

    // Flush the remaining chunk.
    if chunk_start < bytes.len() {
        // SAFETY: remaining bytes are valid UTF-8 (lone surrogates were handled in the loop).
        let chunk = unsafe { std::str::from_utf8_unchecked(&bytes[chunk_start..]) };
        buf.print_str(chunk);
    }
}

/// Write a `u16` value as 4 uppercase hex digits.
#[cfg(feature = "serialize")]
#[inline]
fn write_hex4(v: u16, buf: &mut oxc_data_structures::code_buffer::CodeBuffer) {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let bytes = [
        HEX[((v >> 12) & 0xF) as usize],
        HEX[((v >> 8) & 0xF) as usize],
        HEX[((v >> 4) & 0xF) as usize],
        HEX[(v & 0xF) as usize],
    ];
    // SAFETY: `bytes` is always valid ASCII.
    buf.print_str(unsafe { std::str::from_utf8_unchecked(&bytes) });
}

/// Write a byte as 2 uppercase hex digits.
#[cfg(feature = "serialize")]
#[inline]
fn write_hex2(v: u8, buf: &mut oxc_data_structures::code_buffer::CodeBuffer) {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let bytes = [HEX[((v >> 4) & 0xF) as usize], HEX[(v & 0xF) as usize]];
    // SAFETY: `bytes` is always valid ASCII.
    buf.print_str(unsafe { std::str::from_utf8_unchecked(&bytes) });
}

// ──────────────────────────────────────────────────────────────────────────────
// Tests
// ──────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use oxc_allocator::Allocator;

    use super::*;

    #[test]
    fn from_str() {
        let s = Wtf8Atom::from_str("hello");
        assert_eq!(s.as_str(), Some("hello"));
    }

    #[test]
    fn empty() {
        let s = Wtf8Atom::empty();
        assert_eq!(s.as_str(), Some(""));
        assert!(s.is_empty());
    }

    #[test]
    fn from_wtf8_with_surrogate() {
        let bytes = [0xED, 0xA0, 0x80]; // U+D800
        // SAFETY: bytes are a valid lone-surrogate WTF-8 sequence (U+D800).
        let wtf8 = unsafe { Wtf8::from_bytes_unchecked(&bytes) };
        let atom = Wtf8Atom::from_wtf8(wtf8);
        assert!(atom.as_str().is_none());
        assert!(atom.contains_lone_surrogates());
    }

    #[test]
    fn try_into_atom_success() {
        let atom: Wtf8Atom = Wtf8Atom::from_str("hello");
        let result = atom.try_into_atom();
        assert_eq!(result.as_deref(), Some("hello"));
    }

    #[test]
    fn try_into_atom_fail() {
        let bytes = [0xED, 0xA0, 0x80];
        // SAFETY: bytes are a valid lone-surrogate WTF-8 sequence (U+D800).
        let wtf8 = unsafe { Wtf8::from_bytes_unchecked(&bytes) };
        let atom = Wtf8Atom::from_wtf8(wtf8);
        assert!(atom.try_into_atom().is_none());
    }

    #[test]
    fn to_str_lossy_no_surrogates() {
        let atom = Wtf8Atom::from_str("hello");
        assert!(matches!(atom.to_str_lossy(), Cow::Borrowed("hello")));
    }

    #[test]
    fn to_str_lossy_with_surrogate() {
        let bytes = [b'a', 0xED, 0xA0, 0x80, b'b'];
        // SAFETY: bytes are a valid WTF-8 sequence (ASCII 'a', lone surrogate U+D800, ASCII 'b').
        let wtf8 = unsafe { Wtf8::from_bytes_unchecked(&bytes) };
        let atom = Wtf8Atom::from_wtf8(wtf8);
        let lossy = atom.to_str_lossy();
        assert_eq!(&*lossy, "a\u{FFFD}b");
    }

    #[test]
    fn from_in_str() {
        let allocator = Allocator::new();
        let atom = Wtf8Atom::from_in("hello", &allocator);
        assert_eq!(atom.as_str(), Some("hello"));
    }

    #[test]
    fn from_in_wtf8() {
        let allocator = Allocator::new();
        let bytes = [b'a', 0xED, 0xB0, 0x80, b'b'];
        // SAFETY: bytes are a valid WTF-8 sequence (ASCII 'a', lone trail surrogate U+DC00, ASCII 'b').
        let wtf8 = unsafe { Wtf8::from_bytes_unchecked(&bytes) };
        let atom = Wtf8Atom::from_in(wtf8, &allocator);
        assert!(atom.contains_lone_surrogates());
    }

    #[test]
    fn clone_in() {
        let allocator = Allocator::new();
        let bytes = [0xED, 0xA0, 0x80];
        // SAFETY: bytes are a valid lone-surrogate WTF-8 sequence (U+D800).
        let wtf8 = unsafe { Wtf8::from_bytes_unchecked(&bytes) };
        let atom = Wtf8Atom::from_wtf8(wtf8);
        let cloned = atom.clone_in(&allocator);
        assert_eq!(atom, cloned);
    }

    #[test]
    fn equality() {
        let a = Wtf8Atom::from_str("hello");
        let b = Wtf8Atom::from_str("hello");
        let c = Wtf8Atom::from_str("world");
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn debug_display_no_surrogate() {
        let a = Wtf8Atom::from_str("hello");
        assert_eq!(format!("{a}"), "hello");
        assert_eq!(format!("{a:?}"), "\"hello\"");
    }

    #[test]
    fn debug_display_with_surrogate() {
        let bytes = [0xED, 0xA0, 0x80]; // U+D800
        // SAFETY: bytes are a valid lone-surrogate WTF-8 sequence (U+D800).
        let wtf8 = unsafe { Wtf8::from_bytes_unchecked(&bytes) };
        let a = Wtf8Atom::from_wtf8(wtf8);
        // Display replaces with replacement char
        assert_eq!(format!("{a}"), "\u{FFFD}");
        // Debug shows \u{D800}
        assert_eq!(format!("{a:?}"), "\"\\u{D800}\"");
    }

    #[test]
    fn from_atom() {
        let allocator = Allocator::new();
        let atom: Atom = Atom::from_in("hello", &allocator);
        let wtf8_atom = Wtf8Atom::from(atom);
        assert_eq!(wtf8_atom.as_str(), Some("hello"));
    }
}
