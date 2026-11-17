//! [`ValidSource`] - a [`Source`] which has been validated, and so can be parsed.

use std::{ptr::NonNull, slice, str};

use oxc_allocator::Allocator;
use oxc_span::SourceType;

use crate::{
    MAX_LEN,
    diagnostics::{self, ParserDiagnostic},
};

use super::Source;

/// A [`Source`] which has been validated, and so can be parsed.
///
/// Its source text is:
///
/// - Valid UTF-8.
/// - Followed by [`Source::PADDING`] zero bytes.
/// - No longer than [`MAX_LEN`].
///
/// The only way to create a `ValidSource` is [`ValidSource::new`], which performs the validation.
/// `ParserImpl::new` takes a `ValidSource`, so a `ParserImpl` cannot be created for a source
/// which has not been validated.
///
/// The length limit is required for soundness. If source text was longer than `MAX_LEN`,
/// offsets in `Token`s could wrap around, and `ParserImpl::token_source` relies on them
/// being accurate.
//
// # Invariants
//
// 1. `ptr` points to a region of memory `len + Source::PADDING` bytes long.
// 2. `ptr` is valid for reading all of that region, for the whole of `'a`.
// 3. The first `len` bytes of the region are valid UTF-8.
// 4. Nothing writes to those `len` bytes during `'a`.
// 5. The `Source::PADDING` bytes after them are zero. Nothing writes to them while
//    this `ValidSource`, or anything created from it which can read the padding
//    (a `Lexer`, and its `SourcePosition`s), exists.
// 6. `len <= MAX_LEN`.
//
// (5) is weaker than (4) because the padding of a section of a `Source` is only borrowed
// from the rest of the source for the duration of the parse. See `Source::with_section`.
pub struct ValidSource<'a> {
    /// Pointer to start of source text.
    ptr: NonNull<u8>,
    /// Length of source text in bytes, not including padding.
    len: usize,
    /// Source type to parse source as.
    source_type: SourceType,
    /// Allocator which source text is stored in.
    allocator: &'a Allocator,
}

impl<'a> ValidSource<'a> {
    /// Validate a [`Source`], and convert it to a [`ValidSource`].
    ///
    /// # Errors
    ///
    /// Returns the [`ParserDiagnostic`] to report, if source is longer than [`MAX_LEN`],
    /// or is not valid UTF-8.
    #[inline]
    pub fn new(mut source: Source<'a>) -> Result<Self, ParserDiagnostic<'a>> {
        // Check length first, to avoid checking a very long source is valid UTF-8 for no reason
        if source.len > MAX_LEN {
            return Err(diagnostics::overlong_source());
        }

        if !source.is_valid_utf8 && source.validate_utf8().is_err() {
            return Err(diagnostics::file_appears_to_be_binary());
        }

        // `Source`'s invariants cover all of `ValidSource`'s, except for the 2 checked above.
        // `source` is consumed, so nothing can write to the source text after this.
        // If `source` is a section of another `Source`, its padding is written to again
        // after the parse, but not before this `ValidSource` and anything created from it
        // which can read the padding are gone. `Source::with_section`'s caller guarantees that.
        let Source { ptr, len, source_type, allocator, .. } = source;
        Ok(Self { ptr, len, source_type, allocator })
    }

    /// Get source text.
    #[inline]
    pub fn source_text(&self) -> &'a str {
        // SAFETY: `ptr` points to `len` bytes, which are valid for reads for `'a`,
        // and nothing writes to them during `'a`
        let bytes = unsafe { slice::from_raw_parts(self.ptr.as_ptr(), self.len) };
        // SAFETY: Those bytes are valid UTF-8
        unsafe { str::from_utf8_unchecked(bytes) }
    }

    /// Get pointer to start of source text.
    ///
    /// The pointer is valid for reading the source text for the whole of `'a`.
    /// It's valid for reading the [`Source::PADDING`] bytes which follow the source text
    /// only until the parse this `ValidSource` was created for ends.
    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        self.ptr.as_ptr().cast_const()
    }

    /// Get length of source text in bytes, not including padding.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Get [`SourceType`] which source will be parsed as.
    #[inline]
    pub fn source_type(&self) -> SourceType {
        self.source_type
    }

    /// Get [`Allocator`] which source text is stored in.
    #[inline]
    pub fn allocator(&self) -> &'a Allocator {
        self.allocator
    }
}
