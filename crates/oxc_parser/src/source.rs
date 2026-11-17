//! [`Source`] holds source to be parsed, stored in an arena with padding after it.

use std::{
    alloc::Layout,
    error::Error,
    fmt::{self, Display, Formatter},
    fs::File,
    io::{self, Read},
    mem::ManuallyDrop,
    path::{Path, PathBuf},
    ptr::NonNull,
    slice,
    str::{self, Utf8Error},
};

use simdutf8::basic;

use oxc_allocator::Allocator;
use oxc_span::SourceType;

use crate::{
    MAX_LEN,
    diagnostics::{self, ParserDiagnostic},
};

/// Source to be parsed.
///
/// A `Source` holds:
///
/// - The source bytes.
/// - The [`SourceType`] of the code.
/// - The [`Allocator`] which the source bytes are stored in.
///
/// The source bytes are always stored in the [`Allocator`]'s arena,
/// followed by [`Source::PADDING`] zero bytes.
///
/// # Creating a `Source`
///
/// - [`Source::from_str`] copies a `&str` into the arena.
/// - [`Source::from_bytes`] copies a `&[u8]` into the arena.
/// - [`Source::from_file`] reads a file directly into the arena, without copying it again.
///   Source type is inferred from the file extension.
/// - [`Source::from_file_with_source_type`] is the same as [`from_file`] but with the specified
///   source type. Must be used instead of [`from_file`] for files with non-standard file extensions
///   (e.g. `.vue`, `.svelte`, `.astro`).
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
/// [`from_file`]: Self::from_file
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
    /// Panics if `source_text.len() + Source::PADDING` exceeds `isize::MAX`.
    pub fn from_str(source_text: &str, source_type: SourceType, allocator: &'a Allocator) -> Self {
        // SAFETY: A `&str` is valid UTF-8
        let source =
            unsafe { Self::copy_from(source_text.as_bytes(), true, source_type, allocator) };
        source.expect("Source text is too long")
    }

    /// Create a [`Source`] from a `&[u8]`.
    ///
    /// `source_bytes` is copied into `allocator`. It does not need to be valid UTF-8.
    ///
    /// # Panics
    ///
    /// Panics if `source_bytes.len() + Source::PADDING` exceeds `isize::MAX`.
    pub fn from_bytes(
        source_bytes: &[u8],
        source_type: SourceType,
        allocator: &'a Allocator,
    ) -> Self {
        // SAFETY: `is_valid_utf8` is `false`
        let source = unsafe { Self::copy_from(source_bytes, false, source_type, allocator) };
        source.expect("Source is too long")
    }

    /// Create a [`Source`] from a file.
    ///
    /// [`SourceType`] is inferred from the file's extension.
    /// Use [`Source::from_file_with_source_type`] to specify it instead.
    ///
    /// The file is read directly into `allocator`. Its contents do not need to be valid UTF-8.
    ///
    /// # Errors
    ///
    /// Returns [`SourceFileError`] if any of:
    ///
    /// - [`SourceType`] cannot be inferred from the file's extension.
    /// - The file cannot be read.
    /// - The file plus padding is larger than `isize::MAX` bytes.
    pub fn from_file(
        path: impl AsRef<Path>,
        allocator: &'a Allocator,
    ) -> Result<Self, SourceFileError> {
        let path = path.as_ref();
        let Ok(source_type) = SourceType::from_path(path) else {
            return Err(SourceFileError::UnknownSourceType(path.to_path_buf()));
        };
        let source = Self::read_file(path, source_type, allocator)?;
        Ok(source)
    }

    /// Create a [`Source`] from a file, with the [`SourceType`] specified.
    ///
    /// The file is read directly into `allocator`. Its contents do not need to be valid UTF-8.
    ///
    /// # Errors
    ///
    /// Returns [`io::Error`] if either:
    ///
    /// - The file cannot be read.
    /// - The file plus padding is larger than `isize::MAX` bytes.
    pub fn from_file_with_source_type(
        path: impl AsRef<Path>,
        source_type: SourceType,
        allocator: &'a Allocator,
    ) -> io::Result<Self> {
        Self::read_file(path.as_ref(), source_type, allocator)
    }

    /// Create a [`Source`] from source bytes which are already in `allocator`'s arena.
    ///
    /// The source bytes are the `len` bytes starting at `ptr`.
    /// This function writes [`Source::PADDING`] zero bytes after them.
    ///
    /// # SAFETY
    ///
    /// - `ptr` must point to a region of memory `len + Source::PADDING` bytes long,
    ///   within `allocator`'s arena.
    /// - `ptr` must be valid for reading and writing all of that region, for the whole of `'a`.
    /// - The first `len` bytes of the region must be initialized.
    /// - No reference to any part of the region may exist.
    /// - The region must not be read or written during `'a`, except via the returned `Source`,
    ///   or references obtained from it.
    /// - If `is_valid_utf8` is `true`, the first `len` bytes must be valid UTF-8.
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

    /// Get [`SourceType`] which source will be parsed as.
    #[inline]
    pub fn source_type(&self) -> SourceType {
        self.source_type
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
    /// Returns `None` if `bytes.len() + Source::PADDING` exceeds `isize::MAX`.
    ///
    /// # SAFETY
    ///
    /// If `is_valid_utf8` is `true`, `bytes` must be valid UTF-8.
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

    /// Read file at `path` into `allocator`, and create a [`Source`] from it.
    fn read_file(
        path: &Path,
        source_type: SourceType,
        allocator: &'a Allocator,
    ) -> io::Result<Self> {
        let mut file = File::open(path)?;

        match file.metadata() {
            Ok(metadata) if metadata.len() > 0 => {
                Self::read_file_with_known_size(file, metadata.len(), source_type, allocator)
            }
            _ => {
                // File system reports no size for the file (e.g. it's a pipe), or its empty.
                // Read it into a `Vec`, and then copy that into the arena.
                let mut bytes = Vec::new();
                file.read_to_end(&mut bytes)?;
                // SAFETY: `is_valid_utf8` is `false`
                let source = unsafe { Self::copy_from(&bytes, false, source_type, allocator) };
                source.ok_or_else(file_too_large)
            }
        }
    }

    /// Read contents of `file` directly into `allocator`, and create a [`Source`] from it.
    ///
    /// `file_size` is the size of the file in bytes, as reported by the file system.
    /// At most `file_size` bytes are read from the file.
    fn read_file_with_known_size(
        file: File,
        file_size: u64,
        source_type: SourceType,
        allocator: &'a Allocator,
    ) -> io::Result<Self> {
        // Allocate space for file's contents and padding in arena
        let layout = usize::try_from(file_size).ok().and_then(padded_layout);
        let Some(layout) = layout else {
            return Err(file_too_large());
        };
        let capacity = layout.size();
        let ptr = allocator.alloc_layout(layout);

        // Read contents of file into allocated space.
        //
        // * Create a `Vec` which pretends to own the allocation we just created in arena.
        // * Wrap the `Vec` in `ManuallyDrop`, so it doesn't free the memory at end of the block,
        //   or if there's a panic during reading.
        // * Use `File::take` to obtain a reader which yields no more than `file_size` bytes.
        //   This ensures it can't produce more data than we allocated space for - in case file
        //   increased in size since the call to `file.metadata()`, or `file.metadata()` returned
        //   inaccurate size.
        // * Use `Read::read_to_end` to fill the `Vec` from this reader.
        //
        // This is a hack. It's completely bananas that Rust doesn't provide a method to write into
        // a slice of uninitialized bytes, but this seems to be the only safe way to do it
        // on stable Rust.
        // https://users.rust-lang.org/t/reading-c-style-structures-from-disk/70529/7
        //
        // I (@overlookmotel) have reviewed the code for `Read::read_to_end` and it will never grow
        // the `Vec`, as long as it has sufficient capacity for the reader's contents to start with.
        // If it did, that would be UB as it would free the chunk of memory backing the `Vec`,
        // which it doesn't actually own.
        //
        // Unfortunately `Read::read_to_end`'s docs don't guarantee this behavior. But the code
        // is written specifically to avoid growing the `Vec`, and there was a PR to make sure
        // it doesn't: https://github.com/rust-lang/rust/pull/89165
        // So I think in practice we can rely on this behavior.
        //
        // Once Rust's Allocator API is stable on our MSRV, we can remove this hack,
        // and use a `Vec` with a custom allocator.
        //
        // Here the `Vec` also has `PADDING` bytes more capacity than the reader can yield,
        // so it can never be full.
        let len = {
            // SAFETY: We've allocated `capacity` bytes starting at `ptr`.
            // This `Vec` doesn't actually own that memory, but we immediately wrap it
            // in `ManuallyDrop`, so it won't free the memory on drop.
            // As long as the `Vec` doesn't grow, no UB (see above).
            let vec = unsafe { Vec::from_raw_parts(ptr.as_ptr(), 0, capacity) };
            let mut vec = ManuallyDrop::new(vec);
            let bytes_read = file.take(file_size).read_to_end(&mut vec)?;

            debug_assert_eq!(vec.capacity(), capacity);
            debug_assert_eq!(vec.len(), bytes_read);
            debug_assert!(bytes_read <= layout.size() - Source::PADDING);

            // File may have got smaller since the call to `file.metadata()`,
            // or `file.metadata()` may have reported inaccurate size.
            // So use the number of bytes which were read, not `file_size`.
            vec.len()
        };

        // SAFETY:
        // - `ptr` points to a new allocation of `file_size + PADDING` bytes in `allocator`'s arena.
        //   Nothing else has a pointer or reference to it. The `Vec` which did is gone.
        // - `len <= file_size`, because the reader yields at most `file_size` bytes.
        // - The first `len` bytes were initialized by reading the file.
        // - `is_valid_utf8` is `false`.
        Ok(unsafe { Self::from_raw_parts(ptr, len, false, source_type, allocator) })
    }
}

/// A [`Source`] which has been validated, and so can be parsed.
///
/// Its source text is:
///
/// - No longer than `MAX_LEN`.
/// - Valid UTF-8.
/// - Followed by [`Source::PADDING`] zero bytes.
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
// 3. Nothing writes to any part of the region during `'a`.
// 4. The first `len` bytes of the region are valid UTF-8.
// 5. The `Source::PADDING` bytes after them are zero.
// 6. `len <= MAX_LEN`.
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
    /// Returns the [`ParserDiagnostic`] to report, if source is longer than `MAX_LEN`,
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
        // `source` is consumed, so nothing can write to the region after this.
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
    /// The pointer is valid for reading the source text and the [`Source::PADDING`] bytes
    /// which follow it, for the whole of `'a`.
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

/// Error returned by [`Source::from_file`].
#[derive(Debug)]
pub enum SourceFileError {
    /// [`SourceType`] could not be inferred from the file's extension.
    /// Contains the path of the file.
    UnknownSourceType(PathBuf),
    /// File could not be read.
    Io(io::Error),
}

impl Display for SourceFileError {
    /// Write the error message.
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownSourceType(path) => {
                write!(f, "Cannot infer source type from file extension: {}", path.display())
            }
            Self::Io(err) => Display::fmt(err, f),
        }
    }
}

impl Error for SourceFileError {
    /// Get the lower-level cause of this error, if any.
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::UnknownSourceType(_) => None,
            // `Display` impl writes `err`'s own message, so skip to what's below `err`
            Self::Io(err) => err.source(),
        }
    }
}

impl From<io::Error> for SourceFileError {
    /// Wrap an [`io::Error`] in a [`SourceFileError`].
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

/// Get [`Layout`] for `len` bytes of source, followed by [`Source::PADDING`] bytes of padding.
///
/// Returns `None` if `len + Source::PADDING` exceeds `isize::MAX` (the max size of an allocation).
fn padded_layout(len: usize) -> Option<Layout> {
    let size = len.checked_add(Source::PADDING)?;
    Layout::from_size_align(size, 1).ok()
}

/// Create error for a file which is too large to store in an arena.
fn file_too_large() -> io::Error {
    io::Error::new(io::ErrorKind::FileTooLarge, "File is too large")
}

#[cfg(test)]
mod tests {
    use std::{
        alloc::Layout,
        env, fs, io,
        path::{Path, PathBuf},
        process,
        ptr::NonNull,
        slice, str,
    };

    use oxc_allocator::Allocator;
    use oxc_span::SourceType;

    use super::{Source, SourceFileError};

    /// A file in the system's temp directory, which is deleted on drop.
    struct TempFile(PathBuf);

    impl TempFile {
        /// Create a temp file called `name`, containing `contents`.
        fn new(name: &str, contents: &[u8]) -> Self {
            let path = Self::path_for(name);
            fs::write(&path, contents).unwrap();
            Self(path)
        }

        /// Get path for a temp file called `name`. Does not create the file.
        fn path_for(name: &str) -> PathBuf {
            env::temp_dir().join(format!("oxc_parser_source_{}_{name}", process::id()))
        }

        /// Get path of this temp file.
        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempFile {
        /// Delete the temp file.
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.0);
        }
    }

    /// Byte strings to create `Source`s from.
    /// Lengths are around the sizes which `Read::read_to_end` treats specially.
    /// Some are not valid UTF-8.
    fn inputs() -> Vec<Vec<u8>> {
        let mut inputs: Vec<Vec<u8>> =
            [&b""[..], b"x", b"let x = 1;", "日本語".as_bytes(), b"a\0b"]
                .into_iter()
                .map(<[u8]>::to_vec)
                .collect();
        for len in [31, 32, 33, 63, 64, 65, 8191, 8192, 8193, 100_000] {
            inputs.push(b"abcdefg\n".iter().copied().cycle().take(len).collect());
        }
        inputs.push(b"let x = '\xFF';".to_vec());
        inputs
    }

    /// Assert that `source` contains `bytes`, followed by `Source::PADDING` zero bytes.
    fn assert_source(source: &Source<'_>, bytes: &[u8], is_valid_utf8: bool) {
        assert_eq!(source.as_bytes(), bytes);
        assert_eq!(source.len(), bytes.len());
        assert_eq!(source.is_empty(), bytes.is_empty());
        assert_eq!(source.is_valid_utf8, is_valid_utf8);

        // SAFETY: Source bytes are followed by `PADDING` initialized bytes
        let padding = unsafe {
            let padding_ptr = source.ptr.add(source.len);
            slice::from_raw_parts(padding_ptr.as_ptr(), Source::PADDING)
        };
        assert!(padding.iter().all(|&byte| byte == 0));
    }

    #[test]
    fn from_str() {
        let allocator = Allocator::default();
        for text in ["", "x", "let x = 1;", "日本語", "a\0b"] {
            let source = Source::from_str(text, SourceType::tsx(), &allocator);
            assert_source(&source, text.as_bytes(), true);
            assert_eq!(source.source_type(), SourceType::tsx());
            // Source is copied
            assert_ne!(source.as_bytes().as_ptr(), text.as_ptr());
        }
    }

    #[test]
    fn from_bytes() {
        let allocator = Allocator::default();
        for bytes in inputs() {
            let source = Source::from_bytes(&bytes, SourceType::mjs(), &allocator);
            assert_source(&source, &bytes, false);
            assert_eq!(source.source_type(), SourceType::mjs());
            // Source is copied
            assert_ne!(source.as_bytes().as_ptr(), bytes.as_ptr());
        }
    }

    #[test]
    fn from_raw_parts() {
        let allocator = Allocator::default();
        let text = "let x = 1;";

        // Allocate space for source and padding, and fill all of it with non-zero bytes
        let layout = Layout::from_size_align(text.len() + Source::PADDING, 1).unwrap();
        let ptr = allocator.alloc_layout(layout);
        // SAFETY: `ptr` points to an allocation of `layout.size()` bytes
        unsafe {
            ptr.write_bytes(0xFF, layout.size());
            ptr.copy_from_nonoverlapping(NonNull::from(text.as_bytes()).cast(), text.len());
        }

        // SAFETY: `ptr` points to a new allocation of `text.len() + PADDING` bytes in `allocator`,
        // which nothing else refers to. The first `text.len()` bytes are a copy of `text`.
        let source =
            unsafe { Source::from_raw_parts(ptr, text.len(), true, SourceType::cjs(), &allocator) };
        assert_source(&source, text.as_bytes(), true);
        assert_eq!(source.source_type(), SourceType::cjs());
        // Source is not copied
        assert_eq!(source.as_bytes().as_ptr(), ptr.as_ptr().cast_const());
    }

    #[test]
    fn from_file() {
        let allocator = Allocator::default();
        for (index, bytes) in inputs().iter().enumerate() {
            let file = TempFile::new(&format!("from_file_{index}.ts"), bytes);
            let source = Source::from_file(file.path(), &allocator).unwrap();
            assert_source(&source, bytes, false);
            assert_eq!(source.source_type(), SourceType::from_path(file.path()).unwrap());
        }
    }

    #[test]
    fn from_file_infers_source_type() {
        let allocator = Allocator::default();
        // (file name, is TypeScript, is JSX)
        let cases = [
            ("infer.js", false, false),
            ("infer.jsx", false, true),
            ("infer.ts", true, false),
            ("infer.tsx", true, true),
        ];
        for (name, is_typescript, is_jsx) in cases {
            let file = TempFile::new(name, b"x");
            let source_type = Source::from_file(file.path(), &allocator).unwrap().source_type();
            assert_eq!(source_type.is_typescript(), is_typescript);
            assert_eq!(source_type.is_jsx(), is_jsx);
        }
    }

    #[test]
    fn from_file_errors() {
        let allocator = Allocator::default();

        // File does not exist
        let path = TempFile::path_for("does_not_exist.js");
        let err = Source::from_file(&path, &allocator).err().unwrap();
        let SourceFileError::Io(io_err) = &err else { panic!("Wrong error: {err:?}") };
        assert_eq!(io_err.kind(), io::ErrorKind::NotFound);
        assert_eq!(err.to_string(), io_err.to_string());

        // Unknown extension
        let file = TempFile::new("unknown_extension.vue", b"<script></script>");
        let err = Source::from_file(file.path(), &allocator).err().unwrap();
        let SourceFileError::UnknownSourceType(err_path) = &err else {
            panic!("Wrong error: {err:?}")
        };
        assert_eq!(err_path, file.path());
        assert_eq!(
            err.to_string(),
            format!("Cannot infer source type from file extension: {}", file.path().display())
        );

        // No extension
        let path = TempFile::path_for("no_extension");
        let err = Source::from_file(&path, &allocator).err().unwrap();
        assert!(matches!(err, SourceFileError::UnknownSourceType(_)));
    }

    #[test]
    fn as_str() {
        let allocator = Allocator::default();

        // Created from a `&str`. No validation required.
        let mut source = Source::from_str("日本語", SourceType::mjs(), &allocator);
        assert!(source.is_valid_utf8);
        assert_eq!(source.as_str(), Ok("日本語"));

        // Created from bytes which are valid UTF-8. Validated on 1st call only.
        let mut source = Source::from_bytes("日本語".as_bytes(), SourceType::mjs(), &allocator);
        assert!(!source.is_valid_utf8);
        assert_eq!(source.as_str(), Ok("日本語"));
        assert!(source.is_valid_utf8);
        assert_eq!(source.as_str(), Ok("日本語"));

        // Created from bytes which are not valid UTF-8
        let mut source = Source::from_bytes(b"let x = '\xFF';", SourceType::mjs(), &allocator);
        assert!(!source.is_valid_utf8);
        let err = source.as_str().unwrap_err();
        assert_eq!(err.valid_up_to(), 9);
        assert!(!source.is_valid_utf8);
        assert_eq!(source.as_str(), Err(err));

        // Same result as `str::from_utf8` for inputs of all lengths
        for bytes in inputs() {
            let mut source = Source::from_bytes(&bytes, SourceType::mjs(), &allocator);
            let expected = str::from_utf8(&bytes);
            assert_eq!(source.as_str(), expected);
            assert_eq!(source.is_valid_utf8, expected.is_ok());
        }
    }

    #[test]
    fn from_file_with_source_type() {
        let allocator = Allocator::default();

        // Extension is ignored
        for (index, bytes) in inputs().iter().enumerate() {
            let file = TempFile::new(&format!("with_source_type_{index}.vue"), bytes);
            let source =
                Source::from_file_with_source_type(file.path(), SourceType::tsx(), &allocator)
                    .unwrap();
            assert_source(&source, bytes, false);
            assert_eq!(source.source_type(), SourceType::tsx());
        }

        // File does not exist
        let path = TempFile::path_for("does_not_exist.vue");
        let err =
            Source::from_file_with_source_type(&path, SourceType::tsx(), &allocator).err().unwrap();
        assert_eq!(err.kind(), io::ErrorKind::NotFound);
    }
}
