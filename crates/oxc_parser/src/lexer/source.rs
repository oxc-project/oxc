#![allow(clippy::unnecessary_safety_comment)]

use crate::MAX_LEN;

use std::{marker::PhantomData, slice, str};

/// `Source` holds the source text for the lexer, and provides APIs to read it.
///
/// It provides a cursor which allows consuming source text either as `char`s, or as bytes.
/// It replaces `std::str::Chars` iterator which performed the same function previously,
/// but was less flexible as only allowed consuming source char by char.
///
/// Consuming source text byte-by-byte is often more performant than char-by-char.
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
    pub(super) fn new(mut source_text: &'a str) -> Self {
        // If source text exceeds size limit, substitute a short source text which will fail to parse.
        // `Parser::parse` will convert error to `diagnostics::OverlongSource`.
        if source_text.len() > MAX_LEN {
            source_text = "\0";
        }

        let start = source_text.as_ptr();
        // SAFETY: Adding `source_text.len()` to the starting pointer gives a pointer
        // at the end of `source_text`. `end` will never be dereferenced, only checked
        // for direct pointer equality with `ptr` to check if at end of file.
        let end = unsafe { start.add(source_text.len()) };

        Self { start, end, ptr: start, _marker: PhantomData }
    }

    /// Get entire source text as `&str`.
    #[inline]
    pub(super) fn whole(&self) -> &'a str {
        // SAFETY: `start` and `end` are created from a `&str` in `Source::new`,
        // so guaranteed to be start and end of a valid UTF-8 string
        unsafe {
            let len = self.end as usize - self.start as usize;
            let slice = slice::from_raw_parts(self.start, len);
            str::from_utf8_unchecked(slice)
        }
    }

    /// Get remaining source text as `&str`.
    #[inline]
    pub(super) fn remaining(&self) -> &'a str {
        // SAFETY:
        // `start` and `end` are created from a `&str` in `Source::new` so span a single allocation.
        // Invariant of `Source` is that `ptr` is always >= `start` and <= `end`,
        // so a slice spanning `ptr` to `end` will always be part of of a single allocation.
        // Invariant of `Source` is that `ptr` is always on a UTF-8 character boundary,
        // so slice from `ptr` to `end` will always be a valid UTF-8 string.
        unsafe {
            let len = self.end as usize - self.ptr as usize;
            let slice = slice::from_raw_parts(self.ptr, len);
            debug_assert!(slice.is_empty() || !is_utf8_cont_byte(slice[0]));
            str::from_utf8_unchecked(slice)
        }
    }

    /// Return whether at end of source.
    #[inline]
    pub(super) fn is_eof(&self) -> bool {
        self.ptr == self.end
    }

    /// Get current position.
    ///
    /// The `SourcePosition` returned is guaranteed to be within bounds of `&str` that `Source`
    /// was created from, and on a UTF-8 character boundary, so can be used by caller
    /// to later move current position of this `Source` using `Source::set_position`.
    ///
    /// `SourcePosition` lives as long as the source text `&str` that `Source` was created from.
    #[inline]
    pub(super) fn position(&self) -> SourcePosition<'a> {
        SourcePosition { ptr: self.ptr, _marker: PhantomData }
    }

    /// Move current position.
    ///
    /// # SAFETY
    /// `pos` must be created from this `Source`, not another `Source`.
    /// If this is the case, the invariants of `Source` are guaranteed to be upheld.
    #[inline]
    pub(super) unsafe fn set_position(&mut self, pos: SourcePosition) {
        // `SourcePosition` always upholds the invariants of `Source`,
        // as long as it's created from this `Source`.
        // SAFETY: `read_u8`'s contract is upheld by:
        // * The preceding checks that `pos.ptr` >= `self.start` and < `self.end`.
        // * `Source`'s invariants guarantee that `self.start` - `self.end` contains allocated memory.
        // * `Source::new` takes an immutable ref `&str`, guaranteeing that the memory `pos.ptr`
        //   addresses cannot be aliased by a `&mut` ref as long as `Source` exists.
        // * `SourcePosition` can only live as long as the `&str` underlying `Source`.
        debug_assert!(
            pos.ptr >= self.start
                && pos.ptr <= self.end
                && (pos.ptr == self.end || !is_utf8_cont_byte(read_u8(pos.ptr)))
        );
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
        // This assertion is essential to ensure safety of `read_u8()` call below.
        // Without this check, calling `back(0)` on an empty `Source` would cause reading
        // out of bounds.
        // Compiler should remove this assertion when inlining this function,
        // as long as it can deduce from calling code that `n` is non-zero.
        assert!(n > 0, "Cannot call `Source::back` with 0");

        // Ensure not attempting to go back to before start of source
        let offset = self.ptr as usize - self.start as usize;
        assert!(n <= offset, "Cannot go back {n} bytes - only {offset} bytes consumed");

        // SAFETY: We have checked that `n` is less than distance between `start` and `ptr`,
        // so `new_ptr` cannot be outside of allocation of original `&str`
        let new_ptr = unsafe { self.ptr.sub(n) };

        // Enforce invariant that `ptr` must be positioned on a UTF-8 character boundary.
        // SAFETY: `new_ptr` is in bounds of original `&str`, and `n > 0` assertion ensures
        // not at the end, so valid to read a byte.
        // `Source`'s invariants guarantee that `self.start` - `self.end` contains allocated memory.
        // `Source::new` takes an immutable ref `&str`, guaranteeing that the memory `new_ptr`
        // addresses cannot be aliased by a `&mut` ref as long as `Source` exists.
        let byte = unsafe { read_u8(new_ptr) };
        assert!(!is_utf8_cont_byte(byte), "Offset is not on a UTF-8 character boundary");

        // Move current position. The checks above satisfy `Source`'s invariants.
        self.ptr = new_ptr;
    }

    /// Get next char of source, and advance position to after it.
    #[inline]
    pub(super) fn next_char(&mut self) -> Option<char> {
        // Check not at EOF and handle ASCII bytes
        let byte = self.peek_byte()?;
        if byte.is_ascii() {
            // SAFETY: We already exited if at EOF, so `ptr < end`.
            // So incrementing `ptr` cannot result in `ptr > end`.
            // Current byte is ASCII, so incremented `ptr` must be on a UTF-8 character boundary.
            unsafe { self.ptr = self.ptr.add(1) };
            return Some(byte as char);
        }

        // Multi-byte Unicode character.
        // Check invariant that `ptr` is on a UTF-8 character boundary.
        debug_assert!(!is_utf8_cont_byte(byte));

        // Create a `Chars` iterator, get next char from it, and then update `self.ptr`
        // to match `Chars` iterator's updated pointer afterwards.
        // `Chars` iterator upholds same invariants as `Source`, so its pointer is guaranteed
        // to be valid as `self.ptr`.
        let mut chars = self.remaining().chars();
        // SAFETY: We know that there's a byte to be consumed, so `chars.next()` must return `Some(_)`
        let c = unsafe { chars.next().unwrap_unchecked() };
        self.ptr = chars.as_str().as_ptr();
        Some(c)
    }

    /// Get next byte of source, and advance position to after it.
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
    #[allow(dead_code)]
    #[inline]
    unsafe fn next_byte(&mut self) -> Option<u8> {
        if self.is_eof() {
            None
        } else {
            // SAFETY: Safe to read from `ptr` as we just checked it's not out of bounds
            Some(self.next_byte_unchecked())
        }
    }

    /// Get next bytes of source, and advance position to after it, without EOF bounds-check.
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
    /// 1. The byte returned is ASCII.
    /// 2. Further calls to `Source::next_byte` or `Source::next_byte_unchecked` are made
    ///    to consume the rest of the multi-byte UTF-8 character, before calling any other methods
    ///    of `Source` (even safe methods) which rely on `Source` being positioned on a UTF-8
    ///    character boundary, or before passing control back to other safe code which may call them.
    ///
    /// In particular, safe methods `Source::next_char`, `Source::peek_char`, and `Source::remaining`
    /// are *not* safe to call until one of above conditions is satisfied.
    #[allow(dead_code)]
    #[inline]
    unsafe fn next_byte_unchecked(&mut self) -> u8 {
        // SAFETY: Caller guarantees not at end of file i.e. `ptr != end`.
        // Methods of this type provide no way for `ptr` to be before `start` or after `end`.
        // Therefore always valid to read a byte from `ptr`, and incrementing `ptr` cannot result
        // in `ptr > end`.
        let byte = self.peek_byte_unchecked();
        self.ptr = self.ptr.add(1);
        byte
    }

    /// Peek next char of source, without consuming it.
    #[inline]
    pub(super) fn peek_char(&self) -> Option<char> {
        // Check not at EOF and handle ASCII bytes
        let byte = self.peek_byte()?;
        if byte.is_ascii() {
            return Some(byte as char);
        }

        // Multi-byte Unicode character.
        // Check invariant that `ptr` is on a UTF-8 character boundary.
        debug_assert!(!is_utf8_cont_byte(byte));

        // Create a `Chars` iterator, and get next char from it
        let mut chars = self.remaining().chars();
        // SAFETY: We know that there's a byte to be consumed, so `chars.next()` must return `Some(_)`.
        // Could just return `chars.next()` here, but making it clear to compiler that this branch
        // always returns `Some(_)` may help it optimize the caller. Compiler seems to have difficulty
        // "seeing into" `Chars` iterator and making deductions.
        let c = unsafe { chars.next().unwrap_unchecked() };
        Some(c)
    }

    /// Peek next next char of source, without consuming it.
    #[inline]
    pub(super) fn peek_char2(&self) -> Option<char> {
        // Handle EOF
        if self.is_eof() {
            return None;
        }

        // Check invariant that `ptr` is on a UTF-8 character boundary.
        debug_assert!(!is_utf8_cont_byte(self.peek_byte().unwrap()));

        let mut chars = self.remaining().chars();
        // SAFETY: We already checked not at EOF, so `chars.next()` must return `Some(_)`
        unsafe { chars.next().unwrap_unchecked() };
        chars.next()
    }

    /// Peek next byte of source without consuming it.
    #[inline]
    pub(super) fn peek_byte(&self) -> Option<u8> {
        if self.is_eof() {
            None
        } else {
            // SAFETY: Safe to read from `ptr` as we just checked it's not out of bounds
            Some(unsafe { self.peek_byte_unchecked() })
        }
    }

    /// Peek next byte of source without consuming it, without EOF bounds-check.
    ///
    /// # SAFETY
    /// Caller must ensure `Source` is not at end of file.
    #[inline]
    pub(super) unsafe fn peek_byte_unchecked(&self) -> u8 {
        // SAFETY: Caller guarantees `ptr` is before `end` (i.e. not at end of file).
        // Methods of this type provide no way to allow `ptr` to be before `start`.
        // `Source`'s invariants guarantee that `self.start` - `self.end` contains allocated memory.
        // `Source::new` takes an immutable ref `&str`, guaranteeing that the memory `self.ptr`
        // addresses cannot be aliased by a `&mut` ref as long as `Source` exists.
        debug_assert!(self.ptr >= self.start && self.ptr < self.end);
        read_u8(self.ptr)
    }
}

/// Wrapper around a pointer to a position in `Source`.
#[derive(Debug, Clone, Copy)]
pub struct SourcePosition<'a> {
    ptr: *const u8,
    _marker: PhantomData<&'a u8>,
}

/// Return if byte is a UTF-8 continuation byte.
#[inline]
const fn is_utf8_cont_byte(byte: u8) -> bool {
    // 0x80 - 0xBF are continuation bytes i.e. not 1st byte of a UTF-8 character sequence
    byte >= 0x80 && byte < 0xC0
}

/// Read `u8` from `*const u8` pointer.
///
/// Using `as_ref()` for reading is copied from `core::slice::iter::next`.
/// https://doc.rust-lang.org/src/core/slice/iter.rs.html#132
/// https://doc.rust-lang.org/src/core/slice/iter/macros.rs.html#156-168
///
/// This is about 7% faster than `*ptr` or `ptr.read()`, presumably because it tells the compiler
/// it can rely on the memory being immutable, because if a `&mut` reference existed, that would
/// violate Rust's aliasing rules.
///
/// # SAFETY
/// Caller must ensure pointer is non-null, and points to allocated, initialized memory.
/// Pointer must point to within an object for which no `&mut` references are currently held.
#[inline]
unsafe fn read_u8(ptr: *const u8) -> u8 {
    // SAFETY: Caller guarantees pointer is non-null, and points to allocated, initialized memory.
    // Caller guarantees no mutable references to same memory exist, thus upholding Rust's aliasing rules.
    // Pointer is "dereferenceable" by definition as a `u8` is 1 byte and cannot span multiple objects.
    // Alignment is not relevant as `u8` is aligned on 1 (i.e. no alignment requirements).
    debug_assert!(!ptr.is_null());
    *ptr.as_ref().unwrap_unchecked()
}
