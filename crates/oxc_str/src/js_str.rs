use std::{
    fmt::{self, Write},
    hash::{Hash, Hasher},
    iter::FusedIterator,
    marker::PhantomData,
    ptr::NonNull,
    slice, str,
};

use oxc_allocator::{Allocator, CloneIn, CloneInSemanticIds, Dummy, GetAllocator};

use crate::{Ident, JSChar, Str};

/// An immutable JavaScript string borrowed from source text or arena memory.
///
/// JavaScript strings can contain lone surrogates, which Rust's [`prim@str`] cannot
/// represent. `JSStr` stores [canonical WTF-8](https://wtf-8.codeberg.page/):
/// Unicode scalar values use UTF-8, lone surrogates use three bytes, and a
/// surrogate pair uses one four-byte supplementary character.
///
/// # Invariants
///
/// * The bytes are canonical WTF-8. In particular, a lead surrogate encoding
///   cannot be immediately followed by a trail surrogate encoding.
/// * `has_lone_surrogate` is true if and only if the bytes encode a lone surrogate.
///   When false, the bytes are valid UTF-8.
/// * The pointer references `len` initialized bytes, valid and immutable for `'a`.
/// * The byte length fits in `u32` and does not exceed `isize::MAX`.
///
/// All constructors maintain these invariants. The bytes remain private;
/// consumers use [`as_str`](Self::as_str), [`chars`](Self::chars), or
/// [`encode_utf16`](Self::encode_utf16) to read the value.
///
/// Equality and hashing use the canonical bytes. Ordering by these bytes would
/// differ from JavaScript's UTF-16 ordering, so `JSStr` does not implement `Ord`.
///
/// ```
/// use oxc_allocator::Allocator;
/// use oxc_str::JSStrBuilder;
///
/// let allocator = Allocator::new();
/// let mut builder = JSStrBuilder::new_in(&allocator);
/// builder.push_code_unit(0xD800);
/// builder.push('a');
/// let value = builder.into_js_str();
/// assert_eq!(value.as_str(), None);
/// assert_eq!(value.encode_utf16().collect::<Vec<_>>(), [0xD800, 0x61]);
/// ```
///
/// Borrowing source text does not extend its lifetime:
/// ```compile_fail
/// use oxc_str::JSStr;
/// let value;
/// {
///     let source = String::from("hello");
///     value = JSStr::from(source.as_str());
/// }
/// println!("{value:?}");
/// ```
///
/// Arena-backed strings prevent resetting the arena while still in use:
/// ```compile_fail
/// use oxc_allocator::Allocator;
/// use oxc_str::JSStrBuilder;
/// let mut allocator = Allocator::new();
/// let mut builder = JSStrBuilder::new_in(&allocator);
/// builder.push_code_unit(0xD800);
/// let value = builder.into_js_str();
/// allocator.reset();
/// println!("{value:?}");
/// ```
#[derive(Clone, Copy)]
#[repr(C)]
pub struct JSStr<'a> {
    ptr: NonNull<u8>,
    len: u32,
    has_lone_surrogate: bool,
    _marker: PhantomData<&'a [u8]>,
}

// Raw AST transfer reads the bool niche for `Option<JSStr>::None`.
// Verify it at compile time so a compiler layout change cannot silently corrupt
// cooked template values. Reading an uninitialized byte here fails const evaluation.
const _: () = {
    assert!(size_of::<Option<JSStr<'_>>>() == size_of::<JSStr<'_>>());
    let none: Option<JSStr<'_>> = None;
    let offset = std::mem::offset_of!(JSStr<'_>, has_lone_surrogate);
    // SAFETY: The offset is within `none`, which has the same size as `JSStr`.
    // Const evaluation also checks that the niche byte is initialized.
    let niche = unsafe { (&raw const none).cast::<u8>().add(offset).read() };
    assert!(niche == 2);
};

impl JSStr<'static> {
    /// Return the empty string without allocating.
    #[inline]
    pub const fn empty() -> Self {
        Self::from_str("")
    }
}

impl<'a> JSStr<'a> {
    /// Borrow UTF-8 bytes. Their validity already proves the WTF-8 invariant.
    #[inline]
    const fn from_str(value: &'a str) -> Self {
        assert!(value.len() <= u32::MAX as usize, "JavaScript string exceeds u32::MAX bytes");
        // SAFETY: `str` is UTF-8 (thus canonical WTF-8 without surrogates), and
        // its bytes are immutable and valid for `'a`. The length was checked.
        unsafe { Self::from_bytes_unchecked(value.as_bytes(), false) }
    }

    /// Copy UTF-8 text into an arena.
    ///
    /// # Panics
    /// Panics if the byte length exceeds `u32::MAX`.
    #[inline]
    pub fn from_str_in(value: &str, allocator: &impl GetAllocator<'a>) -> Self {
        // Check the length before allocating or copying.
        JSStr::from(value).clone_in(allocator.allocator())
    }

    /// Borrow the value as UTF-8, or return `None` if it contains a lone surrogate.
    ///
    /// This checks the cached flag in O(1); it does not scan the bytes.
    #[inline]
    pub fn as_str(self) -> Option<&'a str> {
        if self.has_lone_surrogate() {
            None
        } else {
            // SAFETY: By `JSStr`'s invariant, canonical WTF-8 with a false
            // surrogate flag is valid UTF-8.
            Some(unsafe { str::from_utf8_unchecked(self.as_bytes()) })
        }
    }

    /// Return the byte length of the WTF-8 representation.
    ///
    /// Use [`len_utf16`](Self::len_utf16) for JavaScript's string length.
    #[inline]
    pub const fn len(self) -> usize {
        self.len as usize
    }

    /// Return whether the string is empty.
    #[inline]
    pub const fn is_empty(self) -> bool {
        self.len == 0
    }

    /// Return whether the string contains a lone surrogate, in O(1).
    #[inline]
    pub const fn has_lone_surrogate(self) -> bool {
        self.has_lone_surrogate
    }

    /// Count UTF-16 code units, including lone surrogates.
    ///
    /// This scans the bytes in O(n) time.
    pub fn len_utf16(self) -> usize {
        // Every non-continuation byte starts one code point. Four-byte code
        // points contribute a second code unit. Valid WTF-8 has no other bytes
        // >= 0xF0, so this counts code units without decoding individual points.
        self.as_bytes()
            .iter()
            .map(|&byte| usize::from(byte & 0xC0 != 0x80) + usize::from(byte >= 0xF0))
            .sum()
    }

    /// Iterate code points. Paired surrogates yield one supplementary `JSChar`.
    #[inline]
    pub fn chars(self) -> impl FusedIterator<Item = JSChar> + Clone + 'a {
        JSChars { remaining: self.as_bytes() }
    }

    /// Iterate UTF-16 code units, preserving lone surrogates.
    #[inline]
    pub fn encode_utf16(self) -> impl FusedIterator<Item = u16> + Clone + 'a {
        EncodeUtf16 { chars: JSChars { remaining: self.as_bytes() }, pending: 0 }
    }

    #[inline]
    pub(super) fn as_bytes(self) -> &'a [u8] {
        // SAFETY: `JSStr`'s pointer references `len` initialized bytes, valid
        // and immutable for `'a`, including the zero-length case.
        unsafe { slice::from_raw_parts(self.ptr.as_ptr(), self.len()) }
    }

    /// Borrow bytes whose encoding and metadata have already been established.
    ///
    /// # Safety
    /// * `bytes` must be canonical WTF-8.
    /// * `has_lone_surrogate` must exactly describe whether they encode a lone surrogate.
    /// * `bytes.len()` must fit in `u32`.
    ///
    /// The reference itself guarantees initialization, immutability, lifetime,
    /// and the `isize::MAX` bound.
    #[inline]
    #[expect(clippy::cast_possible_truncation, reason = "the caller guarantees the length fits")]
    pub(super) const unsafe fn from_bytes_unchecked(
        bytes: &'a [u8],
        has_lone_surrogate: bool,
    ) -> Self {
        Self {
            ptr: NonNull::from_ref(bytes).cast(),
            len: bytes.len() as u32,
            has_lone_surrogate,
            _marker: PhantomData,
        }
    }
}

// SAFETY: The only referenced storage is an immutable byte slice valid for
// `'a`. `JSStr` exposes neither mutation nor an allocator reference.
unsafe impl Send for JSStr<'_> {}
// SAFETY: Sharing `JSStr` only shares immutable bytes, as with `&[u8]`.
unsafe impl Sync for JSStr<'_> {}

impl<'a> From<&'a str> for JSStr<'a> {
    /// Borrow UTF-8 without allocating. Panics if its byte length exceeds `u32::MAX`.
    #[inline]
    fn from(value: &'a str) -> Self {
        Self::from_str(value)
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

impl PartialEq for JSStr<'_> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl Eq for JSStr<'_> {}

impl PartialEq<str> for JSStr<'_> {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.as_bytes() == other.as_bytes()
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
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_bytes().hash(state);
    }
}

impl fmt::Debug for JSStr<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_char('"')?;
        for c in self.chars() {
            if let Some(c) = c.to_char() {
                for escaped in c.escape_debug() {
                    f.write_char(escaped)?;
                }
            } else {
                write!(f, "\\u{:04x}", c.to_u32())?;
            }
        }
        f.write_char('"')
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
        let bytes = allocator.alloc_slice_copy(self.as_bytes());
        // SAFETY: Copying identical bytes preserves canonical WTF-8, the
        // surrogate flag, and the u32 length bound. Storage belongs to the new arena.
        unsafe { JSStr::from_bytes_unchecked(bytes, self.has_lone_surrogate()) }
    }
}

impl<'a> Dummy<'a> for JSStr<'a> {
    #[inline]
    fn dummy(_allocator: &'a Allocator) -> Self {
        JSStr::empty()
    }
}

/// `remaining` is always canonical WTF-8 starting at a code-point boundary.
#[derive(Clone)]
struct JSChars<'a> {
    remaining: &'a [u8],
}

impl Iterator for JSChars<'_> {
    type Item = JSChar;

    #[inline]
    fn next(&mut self) -> Option<JSChar> {
        let (&first, rest) = self.remaining.split_first()?;
        let (value, len) = if first < 0x80 {
            (u32::from(first), 0)
        } else {
            // SAFETY: `remaining` is complete, valid WTF-8 at a code-point
            // boundary. Its leading byte therefore determines how many
            // continuation bytes are present. Their bit patterns and the
            // encoding's range restrictions guarantee a value <= 0x10FFFF.
            unsafe {
                let second = u32::from(*rest.get_unchecked(0) & 0x3F);
                if first < 0xE0 {
                    ((u32::from(first & 0x1F) << 6) | second, 1)
                } else {
                    let third = u32::from(*rest.get_unchecked(1) & 0x3F);
                    if first < 0xF0 {
                        ((u32::from(first & 0x0F) << 12) | (second << 6) | third, 2)
                    } else {
                        let fourth = u32::from(*rest.get_unchecked(2) & 0x3F);
                        ((u32::from(first & 7) << 18) | (second << 12) | (third << 6) | fourth, 3)
                    }
                }
            }
        };
        // SAFETY: The complete code point has `len` continuation bytes, as
        // established above. The suffix begins at the next code-point boundary.
        self.remaining = unsafe { rest.get_unchecked(len..) };
        // SAFETY: Decoding valid WTF-8 produces a code point <= 0x10FFFF.
        Some(unsafe { JSChar::from_u32_unchecked(value) })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.remaining.len();
        (len.div_ceil(4), Some(len))
    }
}

impl FusedIterator for JSChars<'_> {}

#[derive(Clone)]
struct EncodeUtf16<'a> {
    chars: JSChars<'a>,
    /// Zero when empty, otherwise a trailing surrogate (which cannot be zero).
    pending: u16,
}

impl Iterator for EncodeUtf16<'_> {
    type Item = u16;

    #[inline]
    #[expect(clippy::cast_possible_truncation, reason = "code units are masked or range-checked")]
    fn next(&mut self) -> Option<u16> {
        if self.pending != 0 {
            let trail = self.pending;
            self.pending = 0;
            return Some(trail);
        }
        let value = self.chars.next()?.to_u32();
        if value <= 0xFFFF {
            Some(value as u16)
        } else {
            let offset = value - 0x10000;
            self.pending = 0xDC00 | (offset & 0x3FF) as u16;
            Some(0xD800 | (offset >> 10) as u16)
        }
    }
}

impl FusedIterator for EncodeUtf16<'_> {}

#[cfg(test)]
#[path = "js_str/tests.rs"]
mod tests;
