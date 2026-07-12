//! A JavaScript string value. JS strings are sequences of UTF-16 code units
//! with no validity requirement, so a value can contain unpaired surrogate
//! halves that Rust's `str` cannot represent. `JsString` keeps the common
//! valid case as UTF-8 and falls back to code units only when the value is
//! ill-formed, so the compiler computes on true program values instead of
//! replacement characters or escape hatches.
//!
//! Both representations borrow from the compilation arena, so `JsString` is
//! `Copy`: string literals borrow the AST's arena text directly, and computed
//! values (constant folding, entity decoding) are allocated once.
//!
//! Wire format: the babel bridge transports lone surrogates as
//! `__SURROGATE_XXXX__` markers (see `sanitizeJsonSurrogates` in bridge.ts),
//! because serde_json can neither parse nor emit a lone `\uXXXX` escape.
//! Serde for `JsString` decodes and re-emits that marker form, which keeps the
//! JS side of the bridge unchanged.

use std::fmt;

use oxc_allocator::Allocator;
use oxc_str::Str;

/// Invariant: `Repr::Utf8` holds every well-formed value and `Repr::Wtf16`
/// only ill-formed ones (at least one unpaired surrogate). The derived
/// `PartialEq`/`Hash` are only sound under this invariant: a well-formed
/// value smuggled into `Wtf16` would compare unequal to its `Utf8` twin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct JsString<'a>(Repr<'a>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Repr<'a> {
    /// A well-formed string (no unpaired surrogates), stored as UTF-8.
    Utf8(Str<'a>),
    /// An ill-formed string, stored as UTF-16 code units.
    Wtf16(&'a [u16]),
}

impl<'a> JsString<'a> {
    /// Allocate a computed string into the arena.
    pub fn from_str_in(s: &str, allocator: &'a Allocator) -> Self {
        JsString(Repr::Utf8(Str::from(allocator.alloc_str(s))))
    }

    /// Build from UTF-16 code units, normalizing to UTF-8 when well-formed.
    pub fn from_code_units(units: &[u16], allocator: &'a Allocator) -> Self {
        match String::from_utf16(units) {
            Ok(s) => Self::from_str_in(&s, allocator),
            Err(_) => JsString(Repr::Wtf16(allocator.alloc_slice_copy(units))),
        }
    }

    /// The UTF-8 view, when the value is well-formed.
    pub fn as_str(self) -> Option<&'a str> {
        match self.0 {
            Repr::Utf8(s) => Some(s.as_str()),
            Repr::Wtf16(_) => None,
        }
    }

    pub fn code_units(self) -> Vec<u16> {
        match self.0 {
            Repr::Utf8(s) => s.as_str().encode_utf16().collect(),
            Repr::Wtf16(units) => units.to_vec(),
        }
    }

    /// Length in UTF-16 code units (JS `String.prototype.length`).
    pub fn len_utf16(self) -> usize {
        match self.0 {
            Repr::Utf8(s) => s.as_str().encode_utf16().count(),
            Repr::Wtf16(units) => units.len(),
        }
    }

    /// The value with unpaired surrogates replaced by U+FFFD, for consumers
    /// whose string type cannot represent ill-formed values.
    pub fn to_string_lossy(self) -> String {
        match self.0 {
            Repr::Utf8(s) => s.as_str().to_string(),
            Repr::Wtf16(units) => String::from_utf16_lossy(units),
        }
    }

    /// Decode the bridge wire form: a UTF-8 string in which lone surrogates
    /// appear as `__SURROGATE_XXXX__` markers (uppercase hex, mirroring what
    /// `sanitizeJsonSurrogates` emits and `restoreJsonSurrogates` accepts).
    ///
    /// All scanning is byte-wise: a marker is 18 ASCII bytes, so byte-slice
    /// comparisons cannot land on a UTF-8 char boundary the way `str` range
    /// indexing can when multibyte text follows the prefix.
    pub fn from_marker_string(s: &str, allocator: &'a Allocator) -> Self {
        const PREFIX: &[u8] = b"__SURROGATE_";
        const MARKER_LEN: usize = 18;
        if !s.contains("__SURROGATE_") {
            return Self::from_str_in(s, allocator);
        }
        let bytes = s.as_bytes();
        let mut units: Vec<u16> = Vec::with_capacity(s.len());
        let mut pos = 0;
        let mut segment_start = 0;
        while let Some(found) = s[pos..].find("__SURROGATE_") {
            let idx = pos + found;
            let tail = &bytes[idx..];
            let well_formed = tail.len() >= MARKER_LEN
                && &tail[MARKER_LEN - 2..MARKER_LEN] == b"__"
                && tail[PREFIX.len()..PREFIX.len() + 4]
                    .iter()
                    .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_lowercase());
            if well_formed {
                let hex = std::str::from_utf8(&tail[PREFIX.len()..PREFIX.len() + 4])
                    .expect("ascii hex is valid utf8");
                let unit = u16::from_str_radix(hex, 16).expect("validated hex digits");
                units.extend(s[segment_start..idx].encode_utf16());
                units.push(unit);
                pos = idx + MARKER_LEN;
                segment_start = pos;
            } else {
                // Not a well-formed marker: keep the literal text and continue
                // scanning after the prefix.
                pos = idx + PREFIX.len();
            }
        }
        units.extend(s[segment_start..].encode_utf16());
        JsString::from_code_units(&units, allocator)
    }

    /// Encode to the bridge wire form (markers for unpaired surrogates).
    pub fn to_marker_string(self) -> String {
        match self.0 {
            Repr::Utf8(s) => s.as_str().to_string(),
            Repr::Wtf16(units) => {
                let mut out = String::with_capacity(units.len() * 2);
                let mut iter = units.iter().copied().peekable();
                while let Some(unit) = iter.next() {
                    match unit {
                        0xD800..=0xDBFF => {
                            if let Some(&next) = iter.peek() {
                                if (0xDC00..=0xDFFF).contains(&next) {
                                    iter.next();
                                    let cp = 0x10000
                                        + ((unit as u32 - 0xD800) << 10)
                                        + (next as u32 - 0xDC00);
                                    out.push(char::from_u32(cp).expect("valid supplementary"));
                                    continue;
                                }
                            }
                            out.push_str(&format!("__SURROGATE_{unit:04X}__"));
                        }
                        0xDC00..=0xDFFF => {
                            out.push_str(&format!("__SURROGATE_{unit:04X}__"));
                        }
                        _ => {
                            out.push(
                                char::from_u32(unit as u32).expect("BMP non-surrogate is a char"),
                            );
                        }
                    }
                }
                out
            }
        }
    }

    /// Render as JS-source-style escaped text, matching the form TS's debug
    /// printer produces via JSON.stringify: unpaired surrogates print as
    /// lowercase `\udXXX` escapes inside the otherwise UTF-8 text.
    pub fn to_escaped_string(self) -> String {
        match self.0 {
            Repr::Utf8(s) => s.as_str().to_string(),
            Repr::Wtf16(units) => {
                let mut out = String::with_capacity(units.len() * 2);
                let mut iter = units.iter().copied().peekable();
                while let Some(unit) = iter.next() {
                    match unit {
                        0xD800..=0xDBFF => {
                            if let Some(&next) = iter.peek() {
                                if (0xDC00..=0xDFFF).contains(&next) {
                                    iter.next();
                                    let cp = 0x10000
                                        + ((unit as u32 - 0xD800) << 10)
                                        + (next as u32 - 0xDC00);
                                    out.push(char::from_u32(cp).expect("valid supplementary"));
                                    continue;
                                }
                            }
                            out.push_str(&format!("\\u{unit:04x}"));
                        }
                        0xDC00..=0xDFFF => {
                            out.push_str(&format!("\\u{unit:04x}"));
                        }
                        _ => {
                            out.push(
                                char::from_u32(unit as u32).expect("BMP non-surrogate is a char"),
                            );
                        }
                    }
                }
                out
            }
        }
    }
}

impl<'a> From<&'a str> for JsString<'a> {
    /// Borrow an arena (or `'static`) string directly. A `&str` is valid UTF-8
    /// and so cannot contain an unpaired surrogate; constructing `Utf8`
    /// directly preserves the invariant.
    fn from(s: &'a str) -> Self {
        JsString(Repr::Utf8(Str::from(s)))
    }
}

impl<'a> From<Str<'a>> for JsString<'a> {
    fn from(s: Str<'a>) -> Self {
        JsString(Repr::Utf8(s))
    }
}

impl PartialEq<str> for JsString<'_> {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == Some(other)
    }
}

impl PartialEq<&str> for JsString<'_> {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == Some(*other)
    }
}

impl fmt::Display for JsString<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_escaped_string())
    }
}
