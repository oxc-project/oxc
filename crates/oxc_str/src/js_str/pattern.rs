use memchr::memmem;

use super::{JSChar, JSChars, JSStr};

mod private {
    pub trait Sealed {}
}

/// A pattern for [`JSStr`] searches.
///
/// The accepted forms mirror the argument forms of `str`'s search methods
/// that consumers use: `&str`, `char`, and `FnMut(char) -> bool`.
///
/// # Lone surrogates
///
/// A [`char`] cannot represent a lone surrogate, so a `char` value or a
/// predicate never matches one. Positive predicates therefore work
/// as expected, but a negated predicate does not see every code point:
/// `value.contains(|c: char| !c.is_whitespace())` is `false` for a value that
/// is only a lone surrogate. Iterate [`JSStr::chars`] and inspect each
/// [`JSChar`] to classify all code points.
///
/// # Relationship to std's `Pattern`
///
/// This trait plays the role of `std::str::pattern::Pattern` for a [`JSStr`]
/// haystack. The std trait is unstable and cannot be implemented on stable
/// Rust, so this one stands in, and its methods follow std's names where an
/// equivalent exists. It has no `Searcher`: every caller asks a one-shot
/// question, and the delegated `str` methods already run std's searchers on
/// the UTF-8 path. `find_in` and `rfind_in` answer directly what std answers
/// through a searcher. A caller that needs incremental matching, such as
/// `split` or `match_indices`, can add a searcher method to this sealed
/// trait later without breaking users.
///
/// # Performance
///
/// A value without lone surrogates is valid UTF-8, so every search delegates
/// to `str`'s methods after the O(1) surrogate-flag check, keeping std's
/// tuned paths. Prefix and suffix tests are plain byte comparisons, which is
/// also what `str` does, and are valid for both representations without a
/// flag check. Only lone-surrogate haystacks take the WTF-8 searches in this
/// module.
///
/// # Text patterns
///
/// A `&str` or `char` pattern is searched byte for byte in the canonical WTF-8
/// representation, without decoding. This finds exactly the matches a UTF-16
/// search would find: WTF-8 is self-synchronizing, the pattern's UTF-8 bytes
/// cannot start or end inside another code point's encoding, and no valid
/// UTF-8 sequence equals a lone surrogate's `ED A0..BF 80..BF` encoding. A
/// stored surrogate pair is one four-byte code point, so a pattern matches it
/// only as that supplementary character.
pub trait JSStrPattern: private::Sealed {
    #[doc(hidden)]
    #[expect(clippy::wrong_self_convention, reason = "mirrors std's `Pattern::is_contained_in`")]
    fn is_contained_in(self, haystack: JSStr<'_>) -> bool
    where
        Self: Sized,
    {
        self.find_in(haystack).is_some()
    }
    #[doc(hidden)]
    fn find_in(self, haystack: JSStr<'_>) -> Option<usize>;
    #[doc(hidden)]
    fn rfind_in(self, haystack: JSStr<'_>) -> Option<usize>;
    #[doc(hidden)]
    #[expect(clippy::wrong_self_convention, reason = "mirrors std's `Pattern::is_prefix_of`")]
    fn is_prefix_of(self, haystack: JSStr<'_>) -> bool;
    #[doc(hidden)]
    #[expect(clippy::wrong_self_convention, reason = "mirrors std's `Pattern::is_suffix_of`")]
    fn is_suffix_of(self, haystack: JSStr<'_>) -> bool;
}

impl private::Sealed for &str {}

impl JSStrPattern for &str {
    #[inline]
    fn is_contained_in(self, haystack: JSStr<'_>) -> bool {
        match haystack.as_str() {
            Some(haystack) => haystack.contains(self),
            None => memmem::find(haystack.as_bytes(), self.as_bytes()).is_some(),
        }
    }

    #[inline]
    fn find_in(self, haystack: JSStr<'_>) -> Option<usize> {
        match haystack.as_str() {
            Some(haystack) => haystack.find(self),
            None => memmem::find(haystack.as_bytes(), self.as_bytes()),
        }
    }

    #[inline]
    fn rfind_in(self, haystack: JSStr<'_>) -> Option<usize> {
        match haystack.as_str() {
            Some(haystack) => haystack.rfind(self),
            None => memmem::rfind(haystack.as_bytes(), self.as_bytes()),
        }
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

impl JSStrPattern for char {
    #[inline]
    fn is_contained_in(self, haystack: JSStr<'_>) -> bool {
        // `str`'s character containment skips the index-producing searcher
        // that `find` needs, so an early hit costs only the `memchr` probe.
        match haystack.as_str() {
            Some(haystack) => haystack.contains(self),
            None => self.encode_utf8(&mut [0; 4]).find_in(haystack).is_some(),
        }
    }

    #[inline]
    fn find_in(self, haystack: JSStr<'_>) -> Option<usize> {
        // A UTF-8 haystack uses `str`'s character search, which dispatches to
        // `memchr` without the per-call setup of a substring finder.
        match haystack.as_str() {
            Some(haystack) => haystack.find(self),
            None => self.encode_utf8(&mut [0; 4]).find_in(haystack),
        }
    }

    #[inline]
    fn rfind_in(self, haystack: JSStr<'_>) -> Option<usize> {
        match haystack.as_str() {
            Some(haystack) => haystack.rfind(self),
            None => self.encode_utf8(&mut [0; 4]).rfind_in(haystack),
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
    fn find_in(self, haystack: JSStr<'_>) -> Option<usize> {
        find_char(haystack, self)
    }

    #[inline]
    fn rfind_in(self, haystack: JSStr<'_>) -> Option<usize> {
        rfind_char(haystack, self)
    }

    #[inline]
    fn is_prefix_of(self, haystack: JSStr<'_>) -> bool {
        first_char_matches(haystack, self)
    }

    #[inline]
    fn is_suffix_of(self, haystack: JSStr<'_>) -> bool {
        last_char_matches(haystack, self)
    }
}

/// Iterate code points with their WTF-8 byte offsets.
fn char_offsets(haystack: JSStr<'_>) -> impl Iterator<Item = (usize, JSChar)> + '_ {
    let len = haystack.len();
    let mut chars = JSChars { remaining: haystack.as_bytes() };
    std::iter::from_fn(move || {
        let offset = len - chars.remaining.len();
        chars.next().map(|c| (offset, c))
    })
}

/// Decode the last code point, if any.
fn last_char(haystack: JSStr<'_>) -> Option<JSChar> {
    let bytes = haystack.as_bytes();
    // The last non-continuation byte starts the final code point, and the
    // bytes after it are exactly that code point's continuation bytes.
    let start = bytes.iter().rposition(|&byte| byte & 0xC0 != 0x80)?;
    JSChars { remaining: &bytes[start..] }.next()
}

fn find_char(haystack: JSStr<'_>, mut matches: impl FnMut(char) -> bool) -> Option<usize> {
    match haystack.as_str() {
        Some(haystack) => haystack.find(matches),
        None => char_offsets(haystack)
            .find(|(_, c)| c.to_char().is_some_and(&mut matches))
            .map(|(offset, _)| offset),
    }
}

fn rfind_char(haystack: JSStr<'_>, mut matches: impl FnMut(char) -> bool) -> Option<usize> {
    if let Some(haystack) = haystack.as_str() {
        return haystack.rfind(matches);
    }
    let mut last = None;
    for (offset, c) in char_offsets(haystack) {
        if c.to_char().is_some_and(&mut matches) {
            last = Some(offset);
        }
    }
    last
}

fn first_char_matches(haystack: JSStr<'_>, matches: impl FnMut(char) -> bool) -> bool {
    match haystack.as_str() {
        Some(haystack) => haystack.starts_with(matches),
        None => haystack.chars().next().and_then(JSChar::to_char).is_some_and(matches),
    }
}

fn last_char_matches(haystack: JSStr<'_>, matches: impl FnMut(char) -> bool) -> bool {
    match haystack.as_str() {
        Some(haystack) => haystack.ends_with(matches),
        None => last_char(haystack).and_then(JSChar::to_char).is_some_and(matches),
    }
}
