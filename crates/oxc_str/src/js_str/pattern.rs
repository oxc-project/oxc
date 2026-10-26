use memchr::{memchr, memmem, memrchr};

use super::{JSChar, JSChars, JSStr};

mod private {
    pub trait Sealed {}
}

/// A search pattern for [`JSStr`]: `&str`, `char`, or `FnMut(char) -> bool`.
///
/// Character patterns and predicates never match lone surrogates; predicates
/// are not called for them. Even a negated predicate such as
/// `|c: char| !c.is_whitespace()` does not match a lone surrogate.
/// Use [`JSStr::chars`] to inspect every code point, including surrogates.
pub trait JSStrPattern: private::Sealed {
    /// Return the WTF-8 byte offset of the first match in `haystack`, or `None`.
    fn find_in(self, haystack: JSStr<'_>) -> Option<usize>;

    /// Return the WTF-8 byte offset of the last match in `haystack`, or `None`.
    fn rfind_in(self, haystack: JSStr<'_>) -> Option<usize>;

    /// Return whether this pattern matches the start of `haystack`.
    #[expect(clippy::wrong_self_convention, reason = "mirrors std's `Pattern::is_prefix_of`")]
    fn is_prefix_of(self, haystack: JSStr<'_>) -> bool;

    /// Return whether this pattern matches the end of `haystack`.
    #[expect(clippy::wrong_self_convention, reason = "mirrors std's `Pattern::is_suffix_of`")]
    fn is_suffix_of(self, haystack: JSStr<'_>) -> bool;
}

impl private::Sealed for &str {}

// UTF-8 patterns cannot start or end inside a WTF-8 code point or match a lone
// surrogate's encoding, so byte searches preserve code point boundaries.
impl JSStrPattern for &str {
    #[inline]
    fn find_in(self, haystack: JSStr<'_>) -> Option<usize> {
        memmem::find(haystack.as_bytes(), self.as_bytes())
    }

    #[inline]
    fn rfind_in(self, haystack: JSStr<'_>) -> Option<usize> {
        memmem::rfind(haystack.as_bytes(), self.as_bytes())
    }

    #[inline]
    fn is_prefix_of(self, haystack: JSStr<'_>) -> bool {
        haystack.as_bytes().starts_with(self.as_bytes())
    }

    #[inline]
    fn is_suffix_of(self, haystack: JSStr<'_>) -> bool {
        haystack.as_bytes().ends_with(self.as_bytes())
    }
}

impl private::Sealed for char {}

// ASCII bytes cannot occur inside a multibyte WTF-8 code point, and other characters
// search as their UTF-8 encoding.
impl JSStrPattern for char {
    #[inline]
    fn find_in(self, haystack: JSStr<'_>) -> Option<usize> {
        if self.is_ascii() {
            memchr(self as u8, haystack.as_bytes())
        } else {
            self.encode_utf8(&mut [0; 4]).find_in(haystack)
        }
    }

    #[inline]
    fn rfind_in(self, haystack: JSStr<'_>) -> Option<usize> {
        if self.is_ascii() {
            memrchr(self as u8, haystack.as_bytes())
        } else {
            self.encode_utf8(&mut [0; 4]).rfind_in(haystack)
        }
    }

    #[inline]
    fn is_prefix_of(self, haystack: JSStr<'_>) -> bool {
        self.encode_utf8(&mut [0; 4]).is_prefix_of(haystack)
    }

    #[inline]
    fn is_suffix_of(self, haystack: JSStr<'_>) -> bool {
        self.encode_utf8(&mut [0; 4]).is_suffix_of(haystack)
    }
}

// `FnMut` is a fundamental trait, so this blanket impl cannot overlap with the
// concrete impls above.
impl<F: FnMut(char) -> bool> private::Sealed for F {}

impl<F: FnMut(char) -> bool> JSStrPattern for F {
    #[inline]
    fn find_in(mut self, haystack: JSStr<'_>) -> Option<usize> {
        if let Some(haystack) = haystack.as_str() {
            return haystack.find(self);
        }
        let mut chars = JSChars { remaining: haystack.as_bytes() };
        loop {
            let offset = haystack.len() - chars.remaining.len();
            if chars.next()?.to_char().is_some_and(&mut self) {
                return Some(offset);
            }
        }
    }

    #[inline]
    fn rfind_in(mut self, haystack: JSStr<'_>) -> Option<usize> {
        if let Some(haystack) = haystack.as_str() {
            return haystack.rfind(self);
        }
        // Walk code points from the end, as `str::rfind` does, so the predicate sees
        // them in the same order.
        let mut chars = JSChars { remaining: haystack.as_bytes() };
        while let Some(c) = chars.next_back() {
            if c.to_char().is_some_and(&mut self) {
                return Some(chars.remaining.len());
            }
        }
        None
    }

    #[inline]
    fn is_prefix_of(self, haystack: JSStr<'_>) -> bool {
        haystack.chars().next().and_then(JSChar::to_char).is_some_and(self)
    }

    #[inline]
    fn is_suffix_of(self, haystack: JSStr<'_>) -> bool {
        let mut chars = JSChars { remaining: haystack.as_bytes() };
        chars.next_back().and_then(JSChar::to_char).is_some_and(self)
    }
}
