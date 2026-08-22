use std::{
    borrow::{Borrow, Cow},
    fmt, hash,
    ops::Deref,
};

use cow_utils::CowUtils;
use oxc_allocator::{
    Allocator, ArenaStringBuilder, CloneIn, CloneInSemanticIds, Dummy, FromIn, GetAllocator,
};
#[cfg(feature = "serialize")]
use oxc_estree::{ESTree, Serializer as ESTreeSerializer};
use oxc_wtf8::wtf8::Wtf8CodePoints;
use oxc_wtf8::{Wtf8, Wtf8Buf};
#[cfg(feature = "serialize")]
use serde::{Serialize, Serializer as SerdeSerializer};

use crate::CompactStr;

/// An arena-allocated WTF-8 string for `oxc_allocator`.
///
/// Similar to [`crate::Str`] but can contain lone surrogates.
/// Backed by `&'a Wtf8` allocated in the arena via `Allocator::alloc_slice_copy`.
///
/// Use [`Wtf8Str::as_wtf8`] to get the underlying `&Wtf8`,
/// [`Wtf8Str::as_str`] for optional valid UTF-8 view,
/// [`Wtf8Str::as_bytes`] for raw bytes.
#[repr(transparent)]
#[derive(Clone, Copy, Eq)]
pub struct Wtf8Str<'a>(&'a Wtf8);

impl Wtf8Str<'static> {
    /// Get a [`Wtf8Str`] containing a static string.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub const fn new_const(s: &'static str) -> Self {
        // SAFETY: `&'static str` is valid WTF-8 (UTF-8 subset). `Wtf8` is transparent over `[u8]`.
        unsafe { Self(core::mem::transmute::<&'static str, &'static Wtf8>(s)) }
    }

    /// Get a [`Wtf8Str`] containing a static WTF-8 string (may contain lone surrogates).
    ///
    /// # Safety
    /// Caller must ensure `s` is valid WTF-8 bytes.
    #[inline]
    pub const unsafe fn new_const_wtf8(s: &'static Wtf8) -> Self {
        Self(s)
    }

    /// Get a [`Wtf8Str`] containing the empty string.
    #[inline]
    pub const fn empty() -> Self {
        Self::new_const("")
    }
}

impl<'a> Wtf8Str<'a> {
    /// Allocate provided `&str` into arena, and return a [`Wtf8Str<'a>`].
    #[inline]
    pub fn from_str_in(s: &str, allocator: &impl GetAllocator<'a>) -> Self {
        Self::from_bytes_in(s.as_bytes(), allocator)
    }

    /// Allocate provided `&Wtf8` into arena.
    #[inline]
    pub fn from_wtf8_in(s: &Wtf8, allocator: &impl GetAllocator<'a>) -> Self {
        Self::from_bytes_in(s.as_bytes(), allocator)
    }

    /// Allocate provided `Wtf8Buf` into arena.
    #[inline]
    pub fn from_wtf8_buf_in(buf: &Wtf8Buf, allocator: &impl GetAllocator<'a>) -> Self {
        Self::from_bytes_in(buf.as_bytes(), allocator)
    }

    /// Allocate raw WTF-8 bytes into arena.
    ///
    /// # Panics
    /// Panics if `bytes` is not valid WTF-8.
    #[inline]
    pub fn from_bytes_in(bytes: &[u8], allocator: &impl GetAllocator<'a>) -> Self {
        assert!(Wtf8::from_bytes(bytes).is_ok(), "invalid WTF-8 bytes");
        // SAFETY: validity asserted above.
        unsafe { Self::from_bytes_unchecked_in(bytes, allocator) }
    }

    /// Allocate raw WTF-8 bytes into arena without validation.
    ///
    /// # Safety
    /// Caller must ensure `bytes` is valid WTF-8.
    #[inline]
    pub unsafe fn from_bytes_unchecked_in(bytes: &[u8], allocator: &impl GetAllocator<'a>) -> Self {
        let allocated: &'a [u8] = allocator.allocator().alloc_slice_copy(bytes);
        // SAFETY: Wtf8 is transparent over [u8]; caller guarantees validity.
        let wtf8: &'a Wtf8 =
            unsafe { &*(std::ptr::from_ref::<[u8]>(allocated) as *const oxc_wtf8::Wtf8) };
        Self(wtf8)
    }

    /// Borrow the underlying `&Wtf8`.
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub fn as_wtf8(&self) -> &'a Wtf8 {
        self.0
    }

    /// Return raw bytes.
    #[inline]
    pub fn as_bytes(&self) -> &'a [u8] {
        self.0.as_bytes()
    }

    /// If this WTF-8 string is valid UTF-8, return `Some(&str)`, else `None`.
    ///
    /// Lone surrogates cause `None`.
    #[inline]
    pub fn as_str(&self) -> Option<&'a str> {
        self.0.as_str()
    }

    /// Convert to string lossily: lone surrogates become U+FFFD.
    #[inline]
    pub fn to_string_lossy(self) -> Cow<'a, str> {
        self.0.to_string_lossy()
    }

    /// Return length in bytes.
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Convert into `CompactStr` lossily (for interop where UTF-8 required).
    #[inline]
    pub fn to_compact_str_lossy(self) -> CompactStr {
        CompactStr::new(&self.to_string_lossy())
    }

    /// Whether string is valid UTF-8 (i.e. contains no lone surrogates).
    #[inline]
    pub fn is_valid_utf8(&self) -> bool {
        self.as_str().is_some()
    }

    /// Try to get `&str` if valid UTF-8, otherwise `None` (alias for `as_str`).
    #[inline]
    pub fn try_as_str(&self) -> Option<&'a str> {
        self.as_str()
    }

    /// Iterate over code points (including lone surrogates as `CodePoint`).
    #[inline]
    pub fn code_points(&self) -> Wtf8CodePoints<'_> {
        self.as_wtf8().code_points()
    }

    /// Create from `Cow<'a, Wtf8>`? For API symmetry we provide from_cow
    #[inline]
    pub fn from_cow_in(cow: Cow<'a, Wtf8>, allocator: &impl GetAllocator<'a>) -> Self {
        match cow {
            Cow::Borrowed(s) => Self(s),
            Cow::Owned(buf) => Self::from_bytes_in(buf.as_bytes(), allocator),
        }
    }

    /// Create from `Wtf8Buf` Cow?
    #[inline]
    pub fn from_wtf8_cow_in(cow: Cow<'a, Wtf8>, allocator: &impl GetAllocator<'a>) -> Self {
        Self::from_cow_in(cow, allocator)
    }

    /// Returns `&str` if valid UTF-8, otherwise empty string.
    ///
    /// Convenience for linter/codegen where lone surrogates should be treated as empty.
    #[inline]
    pub fn as_str_or_default(&'a self) -> &'a str {
        self.as_str().unwrap_or("")
    }

    /// Parse the string as `F`, using `as_str().unwrap_or_default()` as source.
    /// # Errors
    ///
    /// Returns `F::Err` if parsing fails.
    #[inline]
    pub fn parse<F: std::str::FromStr>(&self) -> Result<F, F::Err> {
        self.as_str_or_default().parse()
    }

    /// Returns an iterator over `char`s, using lossy `""` fallback for lone surrogates.
    #[inline]
    pub fn chars(&'a self) -> std::str::Chars<'a> {
        self.as_str_or_default().chars()
    }

    /// Split by `pat` (char), lossy fallback.
    #[inline]
    pub fn split(&'a self, pat: char) -> std::str::Split<'a, char> {
        self.as_str_or_default().split(pat)
    }

    /// Split by `pat` (&str), lossy fallback.
    #[inline]
    pub fn split_str(&'a self, pat: &'a str) -> std::str::Split<'a, &'a str> {
        self.as_str_or_default().split(pat)
    }

    /// Split whitespace, lossy fallback.
    #[inline]
    pub fn split_whitespace(&'a self) -> std::str::SplitWhitespace<'a> {
        self.as_str_or_default().split_whitespace()
    }

    /// Whether string equals `other` ignoring ASCII case, lossy fallback.
    #[inline]
    pub fn eq_ignore_ascii_case(&self, other: &str) -> bool {
        self.as_str_or_default().eq_ignore_ascii_case(other)
    }

    /// Convert to `CompactStr` lossily (alias for `to_compact_str_lossy`).
    #[inline]
    pub fn to_compact_str(self) -> CompactStr {
        self.to_compact_str_lossy()
    }

    /// Alias for `to_compact_str_lossy` for compatibility with `Atom::into_compact_str`.
    #[inline]
    pub fn into_compact_str(self) -> CompactStr {
        self.to_compact_str_lossy()
    }

    /// Returns `true` if string contains `pat` (char), lossy fallback.
    #[inline]
    pub fn contains_char(&self, pat: char) -> bool {
        self.as_str_or_default().contains(pat)
    }

    /// Returns `true` if string contains `pat` (&str), lossy fallback.
    #[inline]
    pub fn contains_str(&self, pat: &str) -> bool {
        self.as_str_or_default().contains(pat)
    }

    /// Cow-based ASCII lowercase, lossy fallback.
    ///
    /// Delegates to [`CowUtils`], which returns a borrowed slice when unchanged.
    #[inline]
    pub fn cow_to_ascii_lowercase(&'a self) -> Cow<'a, str> {
        self.as_str_or_default().cow_to_ascii_lowercase()
    }

    /// Cow-based ASCII uppercase, lossy fallback.
    ///
    /// Delegates to [`CowUtils`], which returns a borrowed slice when unchanged.
    #[inline]
    pub fn cow_to_ascii_uppercase(&'a self) -> Cow<'a, str> {
        self.as_str_or_default().cow_to_ascii_uppercase()
    }

    /// Cow-based lowercase, lossy fallback.
    ///
    /// Delegates to [`CowUtils`], which returns a borrowed slice when unchanged.
    #[inline]
    pub fn cow_to_lowercase(&'a self) -> Cow<'a, str> {
        self.as_str_or_default().cow_to_lowercase()
    }

    /// Cow-based uppercase, lossy fallback.
    ///
    /// Delegates to [`CowUtils`], which returns a borrowed slice when unchanged.
    #[inline]
    pub fn cow_to_uppercase(&'a self) -> Cow<'a, str> {
        self.as_str_or_default().cow_to_uppercase()
    }

    /// Trim whitespace, lossy fallback.
    #[inline]
    pub fn trim(&'a self) -> &'a str {
        self.as_str_or_default().trim()
    }

    /// Whether string starts with `pat` (&str), lossy fallback.
    #[inline]
    pub fn starts_with_str(&'a self, pat: &str) -> bool {
        self.as_str_or_default().starts_with(pat)
    }

    /// Whether string ends with `pat` (&str), lossy fallback.
    #[inline]
    pub fn ends_with_str(&'a self, pat: &str) -> bool {
        self.as_str_or_default().ends_with(pat)
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
        // Reallocate bytes into new allocator directly without GetAllocator indirection
        let allocated: &'new_alloc [u8] = allocator.alloc_slice_copy(self.as_bytes());
        // SAFETY: bytes copied from a valid `Wtf8Str` are valid WTF-8; transparent over `[u8]`.
        unsafe { Wtf8Str(&*(std::ptr::from_ref::<[u8]>(allocated) as *const oxc_wtf8::Wtf8)) }
    }
}

impl<'a> Dummy<'a> for Wtf8Str<'a> {
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn dummy(_allocator: &'a Allocator) -> Self {
        // SAFETY: empty static WTF-8 is valid for any lifetime
        unsafe { core::mem::transmute::<Wtf8Str<'static>, Wtf8Str<'a>>(Wtf8Str::empty()) }
    }
}

impl<'alloc> FromIn<'alloc, &Wtf8Str<'alloc>> for Wtf8Str<'alloc> {
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn from_in(s: &Wtf8Str<'alloc>, _: &'alloc Allocator) -> Self {
        *s
    }
}

impl<'alloc> FromIn<'alloc, &'alloc Wtf8> for Wtf8Str<'alloc> {
    #[inline]
    fn from_in(s: &'alloc Wtf8, allocator: &'alloc Allocator) -> Self {
        let allocated: &'alloc [u8] = allocator.alloc_slice_copy(s.as_bytes());
        // SAFETY: source is valid WTF-8/UTF-8, so the copy is too; transparent over `[u8]`.
        unsafe { Self(&*(std::ptr::from_ref::<[u8]>(allocated) as *const oxc_wtf8::Wtf8)) }
    }
}

impl<'alloc> FromIn<'alloc, Wtf8Buf> for Wtf8Str<'alloc> {
    #[inline]
    fn from_in(s: Wtf8Buf, allocator: &'alloc Allocator) -> Self {
        let allocated: &'alloc [u8] = allocator.alloc_slice_copy(s.as_bytes());
        // SAFETY: source is valid WTF-8/UTF-8, so the copy is too; transparent over `[u8]`.
        unsafe { Self(&*(std::ptr::from_ref::<[u8]>(allocated) as *const oxc_wtf8::Wtf8)) }
    }
}

impl<'alloc> FromIn<'alloc, &str> for Wtf8Str<'alloc> {
    #[inline]
    fn from_in(s: &str, allocator: &'alloc Allocator) -> Self {
        let allocated: &'alloc [u8] = allocator.alloc_slice_copy(s.as_bytes());
        // SAFETY: source is valid WTF-8/UTF-8, so the copy is too; transparent over `[u8]`.
        unsafe { Self(&*(std::ptr::from_ref::<[u8]>(allocated) as *const oxc_wtf8::Wtf8)) }
    }
}

impl<'alloc> FromIn<'alloc, String> for Wtf8Str<'alloc> {
    #[inline]
    fn from_in(s: String, allocator: &'alloc Allocator) -> Self {
        Self::from_in(s.as_str(), allocator)
    }
}

impl<'alloc> FromIn<'alloc, &String> for Wtf8Str<'alloc> {
    #[inline]
    fn from_in(s: &String, allocator: &'alloc Allocator) -> Self {
        Self::from_in(s.as_str(), allocator)
    }
}

impl<'a> From<&'a Wtf8> for Wtf8Str<'a> {
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn from(s: &'a Wtf8) -> Self {
        Self(s)
    }
}

impl<'a> From<&'a str> for Wtf8Str<'a> {
    #[inline]
    fn from(s: &'a str) -> Self {
        // SAFETY: &str bytes are valid WTF-8.
        unsafe { Self(&*(std::ptr::from_ref::<str>(s) as *const oxc_wtf8::Wtf8)) }
    }
}

impl<'a, 'b> From<crate::Str<'b>> for Wtf8Str<'a>
where
    'b: 'a,
{
    #[inline]
    fn from(s: crate::Str<'b>) -> Self {
        // SAFETY: Str bytes are valid UTF-8, hence valid WTF-8.
        unsafe { Self(&*(std::ptr::from_ref::<str>(s.as_str()) as *const oxc_wtf8::Wtf8)) }
    }
}

impl<'a, 'b> From<&'b crate::Str<'b>> for Wtf8Str<'a>
where
    'b: 'a,
{
    #[inline]
    fn from(s: &'b crate::Str<'b>) -> Self {
        // SAFETY: `Str` bytes are valid UTF-8, hence valid WTF-8.
        unsafe { Self(&*(std::ptr::from_ref::<str>(s.as_str()) as *const oxc_wtf8::Wtf8)) }
    }
}

impl<'a, 'b> From<crate::Ident<'b>> for Wtf8Str<'a>
where
    'b: 'a,
{
    #[inline]
    fn from(s: crate::Ident<'b>) -> Self {
        // SAFETY: `Ident` bytes are valid UTF-8, hence valid WTF-8.
        unsafe { Self(&*(std::ptr::from_ref::<str>(s.as_str()) as *const oxc_wtf8::Wtf8)) }
    }
}

impl<'a, 'b> From<&'b crate::Ident<'b>> for Wtf8Str<'a>
where
    'b: 'a,
{
    #[inline]
    fn from(s: &'b crate::Ident<'b>) -> Self {
        // SAFETY: `Ident` bytes are valid UTF-8, hence valid WTF-8.
        unsafe { Self(&*(std::ptr::from_ref::<str>(s.as_str()) as *const oxc_wtf8::Wtf8)) }
    }
}

impl<'alloc> From<ArenaStringBuilder<'alloc>> for Wtf8Str<'alloc> {
    #[inline]
    fn from(s: ArenaStringBuilder<'alloc>) -> Self {
        // ArenaStringBuilder produces UTF-8 string; reinterpret as WTF-8
        let str_ref: &str = s.into_str();
        // SAFETY: `ArenaStringBuilder` produces UTF-8, hence valid WTF-8.
        unsafe { Self(&*(std::ptr::from_ref::<str>(str_ref) as *const oxc_wtf8::Wtf8)) }
    }
}

impl<'a> From<Wtf8Str<'a>> for &'a Wtf8 {
    #[expect(clippy::inline_always)]
    #[inline(always)]
    fn from(s: Wtf8Str<'a>) -> Self {
        s.as_wtf8()
    }
}

impl<'a> From<Wtf8Str<'a>> for &'a [u8] {
    #[inline]
    fn from(s: Wtf8Str<'a>) -> Self {
        s.as_bytes()
    }
}

impl Deref for Wtf8Str<'_> {
    type Target = str;
    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_str_or_default()
    }
}

impl AsRef<Wtf8> for Wtf8Str<'_> {
    #[inline]
    fn as_ref(&self) -> &Wtf8 {
        self.0
    }
}

impl AsRef<str> for Wtf8Str<'_> {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str_or_default()
    }
}

impl Borrow<Wtf8> for Wtf8Str<'_> {
    #[inline]
    fn borrow(&self) -> &Wtf8 {
        self.0
    }
}

impl Borrow<str> for Wtf8Str<'_> {
    #[inline]
    fn borrow(&self) -> &str {
        self.as_str_or_default()
    }
}

impl PartialEq<Wtf8Str<'_>> for Wtf8Str<'_> {
    #[inline]
    fn eq(&self, other: &Wtf8Str<'_>) -> bool {
        self.0 == other.0
    }
}

impl PartialEq<Wtf8> for Wtf8Str<'_> {
    #[inline]
    fn eq(&self, other: &Wtf8) -> bool {
        self.0 == other
    }
}

impl PartialEq<Wtf8Str<'_>> for Wtf8 {
    #[inline]
    fn eq(&self, other: &Wtf8Str<'_>) -> bool {
        self == other.0
    }
}

impl PartialEq<&Wtf8> for Wtf8Str<'_> {
    #[inline]
    fn eq(&self, other: &&Wtf8) -> bool {
        self.0 == *other
    }
}

impl PartialEq<Wtf8Str<'_>> for &Wtf8 {
    #[inline]
    fn eq(&self, other: &Wtf8Str<'_>) -> bool {
        *self == other.0
    }
}

impl PartialEq<str> for Wtf8Str<'_> {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.as_str().is_some_and(|s| s == other)
    }
}

impl PartialEq<Wtf8Str<'_>> for str {
    #[inline]
    fn eq(&self, other: &Wtf8Str<'_>) -> bool {
        other == self
    }
}

impl PartialEq<&str> for Wtf8Str<'_> {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        self == *other
    }
}

impl PartialEq<Wtf8Str<'_>> for &str {
    #[inline]
    fn eq(&self, other: &Wtf8Str<'_>) -> bool {
        other == *self
    }
}

impl PartialEq<String> for Wtf8Str<'_> {
    #[inline]
    fn eq(&self, other: &String) -> bool {
        self == other.as_str()
    }
}

impl PartialEq<Wtf8Str<'_>> for String {
    #[inline]
    fn eq(&self, other: &Wtf8Str<'_>) -> bool {
        other == self.as_str()
    }
}

impl hash::Hash for Wtf8Str<'_> {
    #[inline]
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl fmt::Debug for Wtf8Str<'_> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.0, f)
    }
}

impl fmt::Display for Wtf8Str<'_> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Display lossily? Use Wtf8's Display which shows raw bytes? Use lossy.
        // Wtf8's Display not defined, but Debug escapes surrogates. For Display we want to show replacement.
        // Use to_string_lossy.
        fmt::Display::fmt(&self.to_string_lossy(), f)
    }
}

#[cfg(feature = "serialize")]
impl Serialize for Wtf8Str<'_> {
    fn serialize<S: SerdeSerializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        // Serialize as escaped string: lone surrogates as \uXXXX, literal \u escapes escaped.
        // Mirrors SWC's Wtf8Atom serde: convert to escaped String then serialize_str.
        let escaped = wtf8_serialize::wtf8_to_escaped_string(self.0);
        Serialize::serialize(&escaped, serializer)
    }
}

#[cfg(feature = "serialize")]
impl ESTree for Wtf8Str<'_> {
    fn serialize<S: ESTreeSerializer>(&self, mut serializer: S) {
        if let Some(s) = self.as_str() {
            // Valid UTF-8 – delegate to &str's ESTree (handles JSON escaping)
            ESTree::serialize(s, serializer);
            return;
        }
        // Contains lone surrogates – need WTF-8 aware JSON escaping
        let buffer = serializer.buffer_mut();
        buffer.print_ascii_byte(b'"');
        for cp in self.as_wtf8().code_points() {
            if let Some(c) = cp.to_char() {
                match c {
                    '"' => buffer.print_str("\\\""),
                    '\\' => buffer.print_str("\\\\"),
                    '\n' => buffer.print_str("\\n"),
                    '\r' => buffer.print_str("\\r"),
                    '\t' => buffer.print_str("\\t"),
                    '\x08' => buffer.print_str("\\b"),
                    '\x0C' => buffer.print_str("\\f"),
                    '\x0B' => buffer.print_str("\\v"),
                    '\0' => buffer.print_str("\\0"),
                    c if c.is_control() => {
                        buffer.print_str("\\u");
                        let hex = format!("{:04x}", c as u32);
                        buffer.print_str(&hex);
                    }
                    _ => {
                        let mut buf = [0u8; 4];
                        let s = c.encode_utf8(&mut buf);
                        buffer.print_str(s);
                    }
                }
            } else {
                buffer.print_str("\\u");
                let hex = format!("{:04x}", cp.to_u32());
                buffer.print_str(&hex);
            }
        }
        buffer.print_ascii_byte(b'"');
    }
}

/// Helpers for WTF-8 serialization.
#[cfg(feature = "serialize")]
pub mod wtf8_serialize {
    use oxc_wtf8::Wtf8;
    use std::fmt::Write;

    /// Convert WTF-8 to escaped string for JSON/ESTree.
    /// Lone surrogates become `\uXXXX`, literal `\u` followed by 4 hex digits becomes `\\u`.
    /// This mirrors SWC's `convert_wtf8_to_raw` in `wtf8_atom.rs`.
    pub fn wtf8_to_escaped_string(s: &Wtf8) -> String {
        let mut result = String::with_capacity(s.len());
        let mut iter = s.code_points();
        // One-item pushback so lookahead never needs a second iterator.
        let mut pending: Option<oxc_wtf8::CodePoint> = None;

        while let Some(cp) = pending.take().or_else(|| iter.next()) {
            if let Some(c) = cp.to_char() {
                if c == '\\' {
                    // Check for `\uXXXX` (literal escape) — must be re-escaped as `\\u`.
                    match pending.take().or_else(|| iter.next()) {
                        Some(next) if next.to_u32() == 'u' as u32 => {
                            // Look at the following 4 code points; put them back if not all hex.
                            let mut hex: [Option<char>; 4] = [None; 4];
                            let mut all_hex = true;
                            for slot in &mut hex {
                                if let Some(n) = pending.take().or_else(|| iter.next()) {
                                    match n.to_char() {
                                        Some(ch) if ch.is_ascii_hexdigit() => *slot = Some(ch),
                                        _ => {
                                            all_hex = false;
                                            break;
                                        }
                                    }
                                } else {
                                    all_hex = false;
                                    break;
                                }
                            }
                            if all_hex {
                                // Consume the 'u' and emit an escaped backslash before it.
                                result.push_str("\\\\u");
                                for ch in hex.iter().flatten() {
                                    result.push(*ch);
                                }
                            } else {
                                result.push(c);
                                result.push('u');
                                for ch in hex.iter().flatten() {
                                    result.push(*ch);
                                }
                            }
                        }
                        Some(next) => {
                            result.push(c);
                            pending = Some(next);
                        }
                        None => result.push(c),
                    }
                } else {
                    result.push(c);
                }
            } else {
                // Lone surrogate
                let _ = write!(result, "\\u{{{:04X}}}", cp.to_u32());
            }
        }
        result
    }
}

/// Create a [`Wtf8Str<'static>`] for a static string literal (valid UTF-8).
#[macro_export]
macro_rules! static_wtf8_str {
    ($s:literal) => {
        $crate::Wtf8Str::new_const($s)
    };
}

/// Creates a [`Wtf8Str`] using interpolation of runtime expressions, allocated in arena.
#[macro_export]
macro_rules! format_wtf8_str {
    ($allocator:expr, $($arg:tt)*) => {{
        let s = oxc_allocator::Allocator::alloc_str(&$allocator, &format!($($arg)*));
        // Actually need to allocate via Wtf8Str::from_str_in
        $crate::Wtf8Str::from_str_in(&format!($($arg)*), $allocator)
    }};
}

#[cfg(test)]
mod test {
    use oxc_allocator::Allocator;
    use oxc_wtf8::Wtf8Buf;

    use super::Wtf8Str;

    #[test]
    fn wtf8_str_basic() {
        let allocator = Allocator::new();
        let allocator = &allocator;
        let w = Wtf8Buf::from_ill_formed_utf16(&[0xD800]);
        let s = Wtf8Str::from_wtf8_buf_in(&w, &allocator);
        assert_eq!(s.as_bytes(), &[0xED, 0xA0, 0x80]);
        assert!(s.as_str().is_none());
        assert_eq!(s.len(), 3);
        assert!(!s.is_empty());
        assert_eq!(s.to_string_lossy(), "\u{FFFD}");
    }

    #[test]
    fn wtf8_str_valid_utf8() {
        let allocator = Allocator::new();
        let allocator = &allocator;
        let s = Wtf8Str::from_str_in("hello", &allocator);
        assert_eq!(s.as_str(), Some("hello"));
        assert_eq!(s.as_bytes(), b"hello");
        assert_eq!(s.to_string_lossy(), "hello");
    }

    #[test]
    fn wtf8_str_clone_in() {
        use oxc_allocator::{Allocator, CloneIn};
        let allocator1 = Allocator::new();
        let allocator1 = &allocator1;
        let s1 = Wtf8Str::from_str_in("test", &allocator1);
        let allocator2 = Allocator::new();
        let s2 = s1.clone_in(&allocator2);
        assert_eq!(s1, s2);
        assert_eq!(s2.as_str(), Some("test"));
    }

    #[test]
    fn wtf8_str_surrogate_pair_combined() {
        let allocator = Allocator::new();
        let allocator = &allocator;
        // Pair should be stored as single codepoint, not two surrogates
        let w = Wtf8Buf::from_ill_formed_utf16(&[0xD83D, 0xDE00]); // 😀
        let s = Wtf8Str::from_wtf8_buf_in(&w, &allocator);
        assert!(s.as_str().is_some());
        assert_eq!(s.as_str().unwrap(), "😀");
    }

    #[cfg(feature = "serialize")]
    #[test]
    fn wtf8_str_estree_lone_surrogate() {
        use oxc_estree::{CompactSerializer, ESTree};

        let allocator = Allocator::new();
        let allocator = &allocator;

        // Lone surrogate should be escaped as \uXXXX
        let w = Wtf8Buf::from_ill_formed_utf16(&[0xD800]);
        let s = Wtf8Str::from_wtf8_buf_in(&w, &allocator);
        let mut serializer = CompactSerializer::default();
        s.serialize(&mut serializer);
        assert_eq!(serializer.into_string(), r#""\ud800""#);

        // Valid UTF-8 should delegate to &str handling (including " escaping)
        let s2 = Wtf8Str::from_str_in(r#"a"b"#, &allocator);
        let mut serializer2 = CompactSerializer::default();
        s2.serialize(&mut serializer2);
        assert_eq!(serializer2.into_string(), r#""a\"b""#);

        // Multiple surrogates
        let w2 = Wtf8Buf::from_ill_formed_utf16(&[0xD834, 0xD835]);
        let s3 = Wtf8Str::from_wtf8_buf_in(&w2, &allocator);
        let mut serializer3 = CompactSerializer::default();
        s3.serialize(&mut serializer3);
        assert_eq!(serializer3.into_string(), r#""\ud834\ud835""#);

        // Mixed valid and lone surrogates with quotes
        let mut w3 = Wtf8Buf::from_str("a\"b");
        // SAFETY: 0xD800 is within the code point range.
        w3.push(unsafe { oxc_wtf8::CodePoint::from_u32_unchecked(0xD800) });
        w3.push_str("c");
        let s4 = Wtf8Str::from_wtf8_buf_in(&w3, &allocator);
        let mut serializer4 = CompactSerializer::default();
        s4.serialize(&mut serializer4);
        assert_eq!(serializer4.into_string(), r#""a\"b\ud800c""#);
    }
}
