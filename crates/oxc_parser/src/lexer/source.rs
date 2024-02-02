#![allow(clippy::unnecessary_safety_comment)]

use crate::MAX_LEN;

use std::{marker::PhantomData, slice, str};

// TODO: Try to speed up reverting to a checkpoint
// TODO: Is `*self.ptr` better than `self.ptr.read()`?
// TODO: Use `NonNull` for all the pointers?
// TODO: Investigate why semantic benchmarks dropped on "Reduce size of Lookahead struct" commit.
// Is there a bug?

/// `Source` holds the source text for the lexer, and provides APIs to read it.
///
/// It provides a cursor which allows consuming source text either as `char`s, or as bytes.
/// It replaces `std::str::Chars` iterator which performed the same function previously,
/// but was less flexible as only allowed consuming source char by char.
///
/// Consuming source text byte-by-byte is often more performant than char-by-char.
///
/// Implementation of `Source::next_char` is copied directly from `std::str::Chars::next`.
///
/// `Source` provides:
///
/// * Safe API for consuming source char-by-char (`Source::next_char`, `Source::peek_char`).
/// * Safe API for peeking next source byte (`Source::peek_byte`).
/// * Unsafe API for consuming source byte-by-byte (`Source::next_byte`).
/// * Mostly-safe API for rewinding to a previous position in source
///   (`Source::position`, `Source::set_position`).
///
/// # Composition of `Source`
///
/// * `start` is pointer to start of source text.
/// * `end` is pointer to end of source text.
/// * `ptr` is cursor for current position in source text.
///
/// # Invariants of `Source`
///
/// 1. `start` <= `end`
/// 2. The region of memory bounded between `start` and `end` must be initialized,
///    a single allocation, and contain the bytes of a valid UTF-8 string.
/// 3. `ptr` must always be >= `start` and <= `end`.
///    i.e. cursor always within bounds of source text `&str`, or 1 byte after last byte
///    of source text (positioned on EOF).
/// 4. `ptr` must always point to a UTF-8 character boundary, or EOF.
///    i.e. pointing to *1st* byte of a UTF-8 character.
///
/// These invariants are the same as `std::str::Chars`, except `Source` allows temporarily
/// breaking invariant (4) to step through source text byte-by-byte.
///
/// Invariants (1), (2) and (3) must be upheld at all times.
/// Invariant (4) can be temporarily broken, as long as caller ensures it's satisfied again.
///
/// Invariants (1) and (2) are enforced by initializing `start` and `end` from a valid `&str`,
/// and they are never modified after initialization.
///
/// Safe methods of `Source` enforce invariant (3) i.e. they do not allow reading past EOF.
/// Unsafe methods e.g. `Source::next_byte_unchecked` and `Source::peek_byte_unchecked`
/// require caller to uphold this invariant.
///
/// Invariant (4) is the most difficult to satisfy.
/// `Source::next_char` relies on source text being valid UTF-8 to provide a safe API which
/// upholds this invariant.
/// `Source::next_byte` requires very careful use as it may violate invariant (4).
/// That is fine temporarily, but caller *must* ensure the safety conditions of `Source::next_byte`
/// are satisfied, to restore this invariant before passing control back to other code.
/// It will often be preferable to instead use `Source::peek_byte`, followed by `Source::next_char`,
/// which are safe methods, and compiler will often reduce to equally efficient code.
#[derive(Clone)]
pub(super) struct Source<'a> {
    /// Pointer to start of source string. Never altered after initialization.
    start: *const u8,
    /// Pointer to end of source string. Never altered after initialization.
    end: *const u8,
    /// Pointer to current position in source string
    ptr: *const u8,
    /// Marker for immutable borrow of source string
    _marker: PhantomData<&'a str>,
}

impl<'a> Source<'a> {
    /// Create `Source` from `&str`.
    pub(super) fn new(mut source: &'a str) -> Self {
        // If source exceeds size limit, substitute a short source which will fail to parse.
        // `Parser::parse` will convert error to `diagnostics::OverlongSource`.
        if source.len() > MAX_LEN {
            source = "\0";
        }

        let start = source.as_ptr();
        // SAFETY: Adding `source.len()` to the starting pointer gives a pointer
        // at the end of `source`. `end` will never be dereferenced, only checked
        // for direct pointer equality with `current` to check if at end of file.
        let end = unsafe { start.add(source.len()) };

        Self { start, end, ptr: start, _marker: PhantomData }
    }

    /// Get entire source as `&str`.
    #[inline]
    pub(super) fn whole(&self) -> &'a str {
        // SAFETY: `start` and `end` are created from a `&str` in `Source::new`,
        // so guaranteed to be start and end of a valid UTF-8 string.
        unsafe {
            let len = self.end as usize - self.start as usize;
            let slice = slice::from_raw_parts(self.start, len);
            str::from_utf8_unchecked(slice)
        }
    }

    /// Get remaining source as `&str`.
    #[inline]
    pub(super) fn remaining(&self) -> &'a str {
        // SAFETY:
        // `start` and `end` are created from a `&str` in `Source::new` so span a single allocation.
        // Contract of `Source` is that `ptr` is always `>= start` and `<= end`,
        // so a slice spanning `ptr` to `end` will always be part of of a single allocation.
        // Contract of `Source` is that `ptr` is always on a UTF-8 character boundary,
        // so slice from `ptr` to `end` will always be a valid UTF-8 string.
        unsafe {
            let len = self.end as usize - self.ptr as usize;
            let slice = slice::from_raw_parts(self.ptr, len);
            debug_assert!(slice.is_empty() || !is_utf8_cont_byte(slice[0]));
            str::from_utf8_unchecked(slice)
        }
    }

    // Return if at end of source.
    #[inline]
    pub(super) fn is_eof(&self) -> bool {
        self.ptr == self.end
    }

    /// Get source position.
    /// The `SourcePosition` returned is guaranteed to be within bounds of `&str` that `Source`
    /// was created from, and on a UTF-8 character boundary, so can be used by caller
    /// to later move current position of this `Source` using `Source::set_position`.
    #[inline]
    pub(super) fn position(&self) -> SourcePosition<'a> {
        SourcePosition { ptr: self.ptr, _marker: PhantomData }
    }

    /// Move current position in source.
    // TODO: Should this be unsafe? It's possible to create a `SourcePosition` from a *different*
    // `Source`, which would violate `Source`'s invariants.
    #[inline]
    pub(super) fn set_position(&mut self, pos: SourcePosition) {
        // `SourcePosition` always upholds the invariants of `Source`
        debug_assert!(pos.ptr >= self.start && pos.ptr <= self.end);
        // SAFETY: We just checked `pos.ptr` is within bounds of source `&str`,
        // so safe to read from if not at end
        debug_assert!(pos.ptr == self.end || !is_utf8_cont_byte(unsafe { pos.ptr.read() }));
        self.ptr = pos.ptr;
    }

    /// Get current position in source, relative to start of source.
    #[allow(clippy::cast_possible_truncation)]
    #[inline]
    pub(super) fn offset(&self) -> u32 {
        // Cannot overflow `u32` because of `MAX_LEN` check in `Source::new`
        (self.ptr as usize - self.start as usize) as u32
    }

    /// Move current position back by `n` bytes.
    ///
    /// # Panic
    /// Panics if:
    /// * `n` is 0.
    /// * `n` is greater than current offset in source.
    /// * Moving back `n` bytes would not place current position on a UTF-8 character boundary.
    #[inline]
    pub(super) fn back(&mut self, n: usize) {
        assert!(n > 0, "Cannot call `Source::back` with 0");

        // Ensure not attempting to go back to before start of source
        let bytes_consumed = self.ptr as usize - self.start as usize;
        assert!(
            n <= bytes_consumed,
            "Cannot go back {n} bytes - only {bytes_consumed} bytes consumed"
        );

        // SAFETY: We have checked that `n` is less than distance between `start` and `ptr`,
        // so this cannot move `ptr` outside of allocation of original `&str`
        let new_ptr = unsafe { self.ptr.sub(n) };

        // Enforce invariant that `ptr` must be positioned on a UTF-8 character boundary.
        // SAFETY: `new_ptr` is in bounds of original `&str`, and `n > 0` assertion ensures
        // not at the end, so valid to read a byte.
        let byte = unsafe { new_ptr.read() };
        assert!(!is_utf8_cont_byte(byte), "Offset is not on a UTF-8 character boundary");

        // Move current position. The checks above satisfy `Source`'s invariants.
        self.ptr = new_ptr;
    }

    /// Get next char and move `current` on to after it.
    #[inline]
    pub(super) fn next_char(&mut self) -> Option<char> {
        self.next_code_point().map(|ch| {
            debug_assert!(char::try_from(ch).is_ok());
            // SAFETY:
            // `Source` is created from a `&str`, so between `start` and `end` must be valid UTF-8.
            // Invariant of `Source` is that `ptr` must always be positioned on a UTF-8 character boundary.
            // Therefore `ch` must be a valid Unicode Scalar Value.
            unsafe { char::from_u32_unchecked(ch) }
        })
    }

    /// Get next code point.
    /// Copied from implementation of `std::str::Chars`.
    /// https://doc.rust-lang.org/src/core/str/validations.rs.html#36
    #[allow(clippy::cast_lossless)]
    #[inline]
    fn next_code_point(&mut self) -> Option<u32> {
        // Decode UTF-8.
        // SAFETY: If next byte is not ASCII, this function consumes further bytes until end of UTF-8
        // character sequence, leaving `ptr` positioned on next UTF-8 character boundary, or at EOF.
        let x = unsafe { self.next_byte() }?;
        if x < 128 {
            return Some(x as u32);
        }

        // TODO: Mark this branch `#[cold]`?

        debug_assert!(is_utf8_valid_byte(x) && !is_utf8_cont_byte(x));

        // Multibyte case follows
        // Decode from a byte combination out of: [[[x y] z] w]
        // NOTE: Performance is sensitive to the exact formulation here
        let init = utf8_first_byte(x, 2);
        // SAFETY: `Source` contains a valid UTF-8 string, and 1st byte is not ASCII,
        // so guaranteed there is a further byte to be consumed.
        let y = unsafe { self.next_byte_unchecked() };
        let mut ch = utf8_acc_cont_byte(init, y);
        if x >= 0xE0 {
            // [[x y z] w] case
            // 5th bit in 0xE0 .. 0xEF is always clear, so `init` is still valid
            // SAFETY: `Source` contains a valid UTF-8 string, and 1st byte indicates it is start
            // of a 3 or 4-byte sequence, so guaranteed there is a further byte to be consumed.
            let z = unsafe { self.next_byte_unchecked() };
            let y_z = utf8_acc_cont_byte((y & CONT_MASK) as u32, z);
            ch = init << 12 | y_z;
            if x >= 0xF0 {
                // [x y z w] case
                // use only the lower 3 bits of `init`
                // SAFETY: `Source` contains a valid UTF-8 string, and 1st byte indicates it is start
                // of a 4-byte sequence, so guaranteed there is a further byte to be consumed.
                let w = unsafe { self.next_byte_unchecked() };
                ch = (init & 7) << 18 | utf8_acc_cont_byte(y_z, w);
            }
        }

        Some(ch)
    }

    /// Get next byte of source, if not at EOF.
    ///
    /// # SAFETY
    /// This function may leave `Source` positioned in middle of a UTF-8 character sequence,
    /// which would violate one of `Source`'s invariants.
    ///
    /// This is OK temporarily, but caller *must* ensure the invariant is restored again.
    ///
    /// Caller must ensure one of:
    ///
    /// 1. No byte is returned (end of file).
    /// 2. The byte returned is ASCII.
    /// 3. Further calls to `Source::next_byte` or `Source::next_byte_unchecked` are made
    ///    to consume the rest of the multi-byte UTF-8 character, before calling any other methods
    ///    of `Source` (even safe methods) which rely on `Source` being positioned on a UTF-8
    ///    character boundary, or before passing control back to other safe code which may call them.
    ///
    /// In particular, safe methods `Source::next_char`, `Source::peek_char`, and `Source::remaining`
    /// are *not* safe to call until one of above conditions is satisfied.
    ///
    /// It will often be preferable to instead use `Source::peek_byte`, followed by `Source::next_char`,
    /// which are safe methods, and compiler will often reduce to equally efficient code, if calling
    /// code tests the byte returned. e.g.:
    ///
    /// ```
    /// // Consume a space
    /// let byte = source.peek_byte();
    /// if byte == Some(b' ') {
    ///   source.next_char().unwrap();
    /// }
    /// ```
    #[inline]
    unsafe fn next_byte(&mut self) -> Option<u8> {
        if self.ptr == self.end {
            // TODO: Mark this branch `#[cold]`?
            None
        } else {
            // SAFETY: Safe to read from `ptr` as we just checked it's not out of bounds
            Some(self.next_byte_unchecked())
        }
    }

    /// Get next byte of source, without bounds-check.
    ///
    /// # SAFETY
    /// Caller must ensure `Source` is not at end of file.
    ///
    /// This function may leave `Source` positioned in middle of a UTF-8 character sequence,
    /// which would violate one of `Source`'s invariants.
    ///
    /// This is OK temporarily, but caller *must* ensure the invariant is restored again.
    ///
    /// Caller must ensure one of:
    ///
    /// 1. No byte is returned (end of file).
    /// 2. The byte returned is ASCII.
    /// 3. Further calls to `Source::next_byte` or `Source::next_byte_unchecked` are made
    ///    to consume the rest of the multi-byte UTF-8 character, before calling any other methods
    ///    of `Source` (even safe methods) which rely on `Source` being positioned on a UTF-8
    ///    character boundary, or before passing control back to other safe code which may call them.
    ///
    /// In particular, safe methods `Source::next_char`, `Source::peek_char`, and `Source::remaining`
    /// are *not* safe to call until one of above conditions is satisfied.
    #[inline]
    unsafe fn next_byte_unchecked(&mut self) -> u8 {
        // SAFETY: Caller guarantees not at end of file i.e. `self.ptr != self.end`.
        // Methods of this type provide no way for `self.ptr` to be before `self.start`
        // or after `self.end`. Therefore always valid to read a byte from `self.ptr`,
        // and incrementing `self.ptr` cannot result in `self.ptr > self.end`.
        let byte = self.peek_byte_unchecked();
        self.ptr = self.ptr.add(1);
        byte
    }

    /// Get next char, without consuming it.
    #[inline]
    pub(super) fn peek_char(&self) -> Option<char> {
        self.clone().next_char()
    }

    /// Peek next byte of source without consuming it, if not at EOF.
    #[inline]
    pub(super) fn peek_byte(&self) -> Option<u8> {
        if self.ptr == self.end {
            // TODO: Mark this branch `#[cold]`?
            None
        } else {
            // SAFETY: Safe to read from `ptr` as we just checked it's not out of bounds
            Some(unsafe { self.peek_byte_unchecked() })
        }
    }

    /// Peek next byte of source without consuming it, without bounds-check.
    ///
    /// SAFETY: Caller must ensure `ptr < end` i.e. not at end of file.
    #[inline]
    pub(super) unsafe fn peek_byte_unchecked(&self) -> u8 {
        // SAFETY: Caller guarantees `ptr` is before `end` (i.e. not at end of file).
        // Methods of this type provide no way to allow `ptr` to be before `start`.
        debug_assert!(self.ptr >= self.start && self.ptr < self.end);
        self.ptr.read()
    }
}

/// Wrapper around a pointer to a position in `Source`.
#[derive(Debug, Clone, Copy)]
pub struct SourcePosition<'a> {
    ptr: *const u8,
    _marker: PhantomData<&'a str>,
}

/// Mask of the value bits of a continuation byte.
/// Copied from implementation of `std::str::Chars`.
/// https://doc.rust-lang.org/src/core/str/validations.rs.html#274
const CONT_MASK: u8 = 0b0011_1111;

/// Returns the initial codepoint accumulator for the first byte.
/// The first byte is special, only want bottom 5 bits for width 2, 4 bits
/// for width 3, and 3 bits for width 4.
/// Copied from implementation of `std::str::Chars`.
/// https://doc.rust-lang.org/src/core/str/validations.rs.html#11
#[inline]
const fn utf8_first_byte(byte: u8, width: u32) -> u32 {
    (byte & (0x7F >> width)) as u32
}

/// Returns the value of `ch` updated with continuation byte `byte`.
/// Copied from implementation of `std::str::Chars`.
/// https://doc.rust-lang.org/src/core/str/validations.rs.html#17
#[inline]
const fn utf8_acc_cont_byte(ch: u32, byte: u8) -> u32 {
    (ch << 6) | (byte & CONT_MASK) as u32
}

/// Return if byte is a UTF-8 continuation byte.
#[inline]
const fn is_utf8_cont_byte(byte: u8) -> bool {
    // 0x80 - 0xBF are continuation bytes i.e. not 1st byte of a UTF-8 character sequence
    byte >= 0x80 && byte < 0xC0
}

/// Return if byte is a valid UTF-8 byte.
#[inline]
const fn is_utf8_valid_byte(byte: u8) -> bool {
    // All values are valid except 0xF8 - 0xFF
    byte < 0xF8
}
