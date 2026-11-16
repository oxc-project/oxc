//! [`Source`] - holds source to be parsed, stored in an arena with padding after it.

use std::{
    alloc::Layout,
    ptr::NonNull,
    slice,
    str::{self, Utf8Error},
};

use simdutf8::basic;

use oxc_allocator::Allocator;
use oxc_data_structures::assert_unchecked;
use oxc_span::SourceType;

mod reader;
mod valid_source;
pub use reader::SourceFileError;
pub use valid_source::ValidSource;

#[cfg(test)]
mod tests;

/// Source to be parsed.
///
/// A `Source` holds:
///
/// - The source bytes.
/// - The [`SourceType`] of the code.
/// - The [`Allocator`] which the source bytes are stored in.
///
/// The source bytes are always stored in the [`Allocator`]'s arena,
/// followed by [`PADDING`] zero bytes.
///
/// # Creating a `Source`
///
/// - [`from_str`] copies a `&str` into the arena.
/// - [`from_strs`] copies several `&str`s into the arena, concatenated together.
/// - [`from_bytes`] copies a `&[u8]` into the arena.
/// - [`read_from_file`] reads a file directly into the arena, without copying it again.
///   Source type is inferred from the file extension.
/// - [`read_from_file_with_source_type`] is the same as [`read_from_file`] but with
///   the specified source type. Must be used instead of [`read_from_file`] for files
///   with non-standard file extensions (e.g. `.vue`, `.svelte`, `.astro`).
/// - [`read_from_reader`] reads from a reader (e.g. stdin) directly into the arena.
///
/// `Source` does not check that the source bytes are valid UTF-8.
///
/// # Example
///
/// ```
/// use oxc_allocator::Allocator;
/// use oxc_parser::Source;
/// use oxc_span::SourceType;
///
/// let allocator = Allocator::default();
/// let source = Source::from_str("let x = 1;", SourceType::mjs(), &allocator);
/// assert_eq!(source.as_bytes(), b"let x = 1;");
/// ```
///
/// [`PADDING`]: Self::PADDING
/// [`from_str`]: Self::from_str
/// [`from_strs`]: Self::from_strs
/// [`from_bytes`]: Self::from_bytes
/// [`read_from_file`]: Self::read_from_file
/// [`read_from_file_with_source_type`]: Self::read_from_file_with_source_type
/// [`read_from_reader`]: Self::read_from_reader
//
// # Invariants
//
// 1. `ptr` points to a region of memory `len + PADDING` bytes long, within `allocator`'s arena.
// 2. `ptr` is valid for reading and writing all of that region, for the whole of `'a`.
// 3. The first `len` bytes of the region are initialized.
// 4. The `PADDING` bytes after them are zero.
// 5. No reference to any part of the region exists, except ones which borrow this `Source`.
// 6. If `is_valid_utf8` is `true`, the first `len` bytes are valid UTF-8.
//
// `Source` is not `Clone` or `Copy`. A 2nd copy would be a way to break invariant (5).
pub struct Source<'a> {
    /// Pointer to start of source bytes.
    ptr: NonNull<u8>,
    /// Length of source in bytes, not including padding.
    len: usize,
    /// `true` if source bytes are known to be valid UTF-8.
    is_valid_utf8: bool,
    /// Source type to parse source as.
    #[expect(clippy::struct_field_names)]
    source_type: SourceType,
    /// Allocator which source bytes are stored in.
    allocator: &'a Allocator,
}

impl<'a> Source<'a> {
    /// Number of zero bytes which follow the source bytes.
    pub const PADDING: usize = 64;

    /// Create a [`Source`] from a `&str`.
    ///
    /// `source_text` is copied into `allocator`.
    ///
    /// # Panics
    ///
    /// Panics if `source_text.len()` + [`PADDING`] exceeds `isize::MAX`.
    ///
    /// [`PADDING`]: Self::PADDING
    pub fn from_str(source_text: &str, source_type: SourceType, allocator: &'a Allocator) -> Self {
        // SAFETY: A `&str` is valid UTF-8
        let source =
            unsafe { Self::copy_from(source_text.as_bytes(), true, source_type, allocator) };
        source.expect("Source text is too long")
    }

    /// Create a [`Source`] from an array of `&str`s, concatenated together.
    ///
    /// The strings are copied into `allocator` one after another, to form a single source.
    /// This copies each string once. Concatenating them into a string first,
    /// and passing that to [`from_str`], would copy them twice.
    ///
    /// # Example
    ///
    /// ```
    /// use oxc_allocator::Allocator;
    /// use oxc_parser::Source;
    /// use oxc_span::SourceType;
    ///
    /// let allocator = Allocator::default();
    /// let source = Source::from_strs(["(", "{ x: 1 }", ")"], SourceType::mjs(), &allocator);
    /// assert_eq!(source.as_bytes(), b"({ x: 1 })");
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if total length of the strings + [`PADDING`] exceeds `isize::MAX`.
    ///
    /// [`from_str`]: Self::from_str
    /// [`PADDING`]: Self::PADDING
    //
    // `#[inline(always)]` because want compiler to be able to remove the overflow checks
    // where some of `strs` are statically known
    #[expect(clippy::inline_always)]
    #[inline(always)]
    pub fn from_strs<const N: usize>(
        strs: [&str; N],
        source_type: SourceType,
        allocator: &'a Allocator,
    ) -> Self {
        // Calculate total length of all the strings concatenated.
        //
        // We have to use `checked_add` here to guard against additions wrapping around
        // if some of the input `&str`s are very long, or there's many of them.
        //
        // However, `&str`s have max length of `isize::MAX`.
        // https://users.rust-lang.org/t/does-str-reliably-have-length-isize-max/126777
        // Use `assert_unchecked!` to communicate this invariant to compiler, which allows it to
        // remove the overflow checks where some of `strs` are static, so their length is known.
        //
        // e.g. `Source::from_strs(["(", text, "\n)"], ..)` requires no checks at all on the
        // additions, because the static parts have total length of 3 bytes, and `text` has
        // max length of `isize::MAX`. `isize::MAX as usize + 3` cannot overflow `usize`.
        //
        // `StringBuilder::from_strs_array_in` in `oxc_allocator` does the same.
        #[expect(clippy::checked_conversions)]
        let len = strs.iter().try_fold(0usize, |len, s| {
            let str_len = s.len();
            // SAFETY: `&str`s have maximum length of `isize::MAX`
            unsafe { assert_unchecked!(str_len <= (isize::MAX as usize)) };
            len.checked_add(str_len)
        });

        // Check the strings plus padding are not too large for an allocation
        let len = len.filter(|&len| padded_layout(len).is_some());
        let len = len.expect("Source text is too long");

        // Create the `Source` in a separate function, to ensure that `from_strs` is inlined,
        // so that compiler has knowledge to remove the overflow checks above.
        // Compiler can choose whether or not to inline `from_strs_with_len`.
        // SAFETY: `len` is the total length of `strs`, which was calculated correctly above.
        // `padded_layout(len)` returned `Some`, so `len + PADDING <= isize::MAX`.
        unsafe { Self::from_strs_with_len(strs, len, source_type, allocator) }
    }

    /// Create a [`Source`] from an array of `&str`s, concatenated together,
    /// with their total length provided.
    ///
    /// The strings are copied into `allocator`.
    ///
    /// # SAFETY
    ///
    /// - `len` must be the total length of all the strings in `strs`.
    /// - `len` + [`PADDING`] must not exceed `isize::MAX`.
    ///
    /// [`PADDING`]: Self::PADDING
    unsafe fn from_strs_with_len<const N: usize>(
        strs: [&str; N],
        len: usize,
        source_type: SourceType,
        allocator: &'a Allocator,
    ) -> Self {
        // SAFETY: Caller guarantees `len + PADDING <= isize::MAX`. Alignment is 1.
        let layout = unsafe { Layout::from_size_align_unchecked(len + Self::PADDING, 1) };
        let ptr = allocator.alloc_layout(layout);

        let mut dest_ptr = ptr;
        for s in strs {
            let src_ptr = NonNull::from(s.as_bytes()).cast::<u8>();
            // SAFETY: `ptr` points to a new allocation of `len + PADDING` bytes, and `len` is
            // the total length of the strings. `dest_ptr` is `ptr` plus the lengths of the strings
            // before this one. So `dest_ptr` is valid for writing `s.len()` bytes, and the pointer
            // after them is in bounds of the allocation. A new allocation cannot overlap `s`.
            unsafe {
                dest_ptr.copy_from_nonoverlapping(src_ptr, s.len());
                dest_ptr = dest_ptr.add(s.len());
            }
        }

        debug_assert_eq!(dest_ptr.addr().get() - ptr.addr().get(), len);

        // SAFETY:
        // - `ptr` points to a new allocation of `len + PADDING` bytes in `allocator`'s arena.
        //   Nothing else has a pointer or reference to it.
        // - The first `len` bytes were initialized just above.
        // - Each of the strings is valid UTF-8, and a concatenation of valid UTF-8 strings
        //   is valid UTF-8.
        unsafe { Self::from_raw_parts(ptr, len, true, source_type, allocator) }
    }

    /// Create a [`Source`] from a `&[u8]`.
    ///
    /// `source_bytes` is copied into `allocator`. It does not need to be valid UTF-8.
    ///
    /// # Panics
    ///
    /// Panics if `source_bytes.len()` + [`PADDING`] exceeds `isize::MAX`.
    ///
    /// [`PADDING`]: Self::PADDING
    pub fn from_bytes(
        source_bytes: &[u8],
        source_type: SourceType,
        allocator: &'a Allocator,
    ) -> Self {
        // SAFETY: `is_valid_utf8` is `false`
        let source = unsafe { Self::copy_from(source_bytes, false, source_type, allocator) };
        source.expect("Source is too long")
    }

    /// Create a [`Source`] from source bytes which are already in `allocator`'s arena.
    ///
    /// The source bytes are the `len` bytes starting at `ptr`.
    /// This function writes [`PADDING`] zero bytes after them.
    ///
    /// # SAFETY
    ///
    /// - `ptr` must point to a region of memory `len` + [`PADDING`] bytes long,
    ///   within `allocator`'s arena.
    /// - `ptr` must be valid for reading and writing all of that region, for the whole of `'a`.
    /// - The first `len` bytes of the region must be initialized.
    /// - No reference to any part of the region may exist.
    /// - The region must not be read or written during `'a`, except via the returned `Source`,
    ///   or references obtained from it.
    /// - If `is_valid_utf8` is `true`, the first `len` bytes must be valid UTF-8.
    ///
    /// [`PADDING`]: Self::PADDING
    pub unsafe fn from_raw_parts(
        ptr: NonNull<u8>,
        len: usize,
        is_valid_utf8: bool,
        source_type: SourceType,
        allocator: &'a Allocator,
    ) -> Self {
        // SAFETY: Caller guarantees `ptr` is valid for writing `len + PADDING` bytes,
        // and that no reference to the last `PADDING` bytes of them exists
        unsafe { ptr.add(len).write_bytes(0, Self::PADDING) };

        Self { ptr, len, is_valid_utf8, source_type, allocator }
    }

    /// Convert this [`Source`] into its raw parts.
    ///
    /// Returns the same values as [`from_raw_parts`] takes, in the same order:
    ///
    /// - Pointer to the start of the source bytes.
    /// - Length of the source in bytes, not including padding.
    /// - `true` if the source bytes are known to be valid UTF-8.
    /// - [`SourceType`] of the source.
    /// - [`Allocator`] which the source bytes are stored in.
    ///
    /// The source bytes remain in the [`Allocator`]. A `Source` borrows its [`Allocator`],
    /// so the two cannot be moved together (e.g. to another thread). The pointer, length
    /// and other values can be moved along with the [`Allocator`], and the [`Source`] recreated
    /// later with [`from_raw_parts`].
    ///
    /// # Read-only
    ///
    /// The pointer is valid for reading the source bytes for the whole of `'a`.
    ///
    /// It must be treated as read-only. Nothing may be written through it, or through any pointer
    /// derived from it. The only exception is passing it to [`from_raw_parts`] to recreate
    /// the [`Source`].
    ///
    /// Code which created this [`Source`] may rely on the source bytes being unchanged
    /// (e.g. to recreate it later as known-valid UTF-8).
    ///
    /// [`from_raw_parts`]: Self::from_raw_parts
    #[inline]
    pub fn into_raw_parts(self) -> (NonNull<u8>, usize, bool, SourceType, &'a Allocator) {
        (self.ptr, self.len, self.is_valid_utf8, self.source_type, self.allocator)
    }

    /// Get [`SourceType`] which source will be parsed as.
    #[inline]
    pub fn source_type(&self) -> SourceType {
        self.source_type
    }

    /// Set [`SourceType`] which source will be parsed as.
    ///
    /// # Example
    ///
    /// ```
    /// use oxc_allocator::Allocator;
    /// use oxc_parser::Source;
    /// use oxc_span::SourceType;
    ///
    /// let allocator = Allocator::default();
    /// let mut source = Source::from_str("<div />", SourceType::mjs(), &allocator);
    ///
    /// // Enable JSX
    /// source.set_source_type(source.source_type().with_jsx(true));
    /// assert!(source.source_type().is_jsx());
    /// ```
    #[inline]
    pub fn set_source_type(&mut self, source_type: SourceType) {
        self.source_type = source_type;
    }

    /// Get [`Allocator`] which source bytes are stored in.
    #[inline]
    pub fn allocator(&self) -> &'a Allocator {
        self.allocator
    }

    /// Get length of source in bytes, not including padding.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if source is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Get source bytes, without padding.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        // SAFETY: `ptr` points to `len` initialized bytes.
        // They cannot be written to while `self` is borrowed.
        unsafe { slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }

    /// Get source as a `&str`.
    ///
    /// If source is not already known to be valid UTF-8, this validates it,
    /// which is a pass over the whole source.
    /// The result is recorded, so later calls do not validate again.
    ///
    /// # Errors
    ///
    /// Returns [`Utf8Error`] if source is not valid UTF-8.
    #[inline]
    pub fn as_str(&mut self) -> Result<&str, Utf8Error> {
        if !self.is_valid_utf8 {
            self.validate_utf8()?;
        }

        // SAFETY: Source bytes are valid UTF-8.
        // Either that was already known, or `validate_utf8` just checked it.
        Ok(unsafe { str::from_utf8_unchecked(self.as_bytes()) })
    }

    /// Consume this [`Source`], and get its source as a `&str` which lives as long as
    /// the [`Allocator`] does.
    ///
    /// If source is not already known to be valid UTF-8, this validates it,
    /// which is a pass over the whole source.
    ///
    /// # Errors
    ///
    /// Returns [`Utf8Error`] if source is not valid UTF-8.
    #[inline]
    pub fn into_str(mut self) -> Result<&'a str, Utf8Error> {
        if !self.is_valid_utf8 {
            self.validate_utf8()?;
        }

        // SAFETY: `ptr` points to `len` initialized bytes, which are valid for reads for `'a`.
        // This `Source` is consumed, so nothing can write to them after this.
        let bytes = unsafe { slice::from_raw_parts(self.ptr.as_ptr(), self.len) };

        // SAFETY: Source bytes are valid UTF-8.
        // Either that was already known, or `validate_utf8` just checked it.
        Ok(unsafe { str::from_utf8_unchecked(bytes) })
    }

    /// Check that source bytes are valid UTF-8, and record that they are.
    ///
    /// # Errors
    ///
    /// Returns [`Utf8Error`] if source bytes are not valid UTF-8.
    fn validate_utf8(&mut self) -> Result<(), Utf8Error> {
        let bytes = self.as_bytes();

        // `simdutf8` is faster than `str::from_utf8`, but its error doesn't say where
        // the invalid bytes are. If validation fails, validate again with `str::from_utf8`,
        // to get an error which does.
        if basic::from_utf8(bytes).is_err() {
            str::from_utf8(bytes)?;
        }

        self.is_valid_utf8 = true;
        Ok(())
    }

    /// Copy `bytes` into `allocator`, and create a [`Source`] from the copy.
    ///
    /// Returns `None` if `bytes.len()` + [`PADDING`] exceeds `isize::MAX`.
    ///
    /// # SAFETY
    ///
    /// If `is_valid_utf8` is `true`, `bytes` must be valid UTF-8.
    ///
    /// [`PADDING`]: Self::PADDING
    unsafe fn copy_from(
        bytes: &[u8],
        is_valid_utf8: bool,
        source_type: SourceType,
        allocator: &'a Allocator,
    ) -> Option<Self> {
        let len = bytes.len();
        let layout = padded_layout(len)?;
        let ptr = allocator.alloc_layout(layout);

        // SAFETY: `ptr` points to a new allocation of `len + PADDING` bytes,
        // so it's valid for writing `len` bytes, and cannot overlap `bytes`
        unsafe { ptr.copy_from_nonoverlapping(NonNull::from(bytes).cast::<u8>(), len) };

        // SAFETY:
        // - `ptr` points to a new allocation of `len + PADDING` bytes in `allocator`'s arena.
        //   Nothing else has a pointer or reference to it.
        // - The first `len` bytes were initialized just above.
        // - Caller guarantees `bytes` is valid UTF-8 if `is_valid_utf8` is `true`.
        //   The first `len` bytes are a copy of `bytes`.
        Some(unsafe { Self::from_raw_parts(ptr, len, is_valid_utf8, source_type, allocator) })
    }
}

/// Get [`Layout`] for `len` bytes of source, followed by [`PADDING`] bytes of padding.
///
/// Returns `None` if `len` + [`PADDING`] exceeds `isize::MAX` (the max size of an allocation).
///
/// [`PADDING`]: Source::PADDING
fn padded_layout(len: usize) -> Option<Layout> {
    let size = len.checked_add(Source::PADDING)?;
    Layout::from_size_align(size, 1).ok()
}
