use std::{
    alloc::Layout,
    fmt,
    fs::File,
    io::{self, Read},
    mem::ManuallyDrop,
    ops::Deref,
    path::Path,
    slice,
};

use oxc_allocator::{Allocator, StringBuilder};

/// Source text which is followed in memory by [`SourceCode::PADDING`] NUL bytes.
///
/// The padding lets a lexer read at, and a short distance past, the end of the source text
/// without checking bounds first: a read at the end always lands on a NUL byte, so the end
/// of the source only has to be tested for when a NUL byte is found.
///
/// The padding is not part of the source text. [`SourceCode::as_str`] returns the text alone.
///
/// # Creating a `SourceCode`
///
/// * [`SourceCode::read_in`] reads a file straight into an arena, with the padding after it.
///   The file's contents are not copied again.
/// * [`SourceCodeBuf`] owns source text and its padding. [`SourceCodeBuf::read`] reads a file
///   straight into one. Use this when the source text has to outlive the arena, or the arena
///   is reset between uses.
/// * [`SourceCode::from_padded_str`] borrows a string which already ends with the padding,
///   without copying. Use this when you own the buffer which the source text is stored in.
/// * [`SourceCode::from_str_in`] copies source text into an arena, and adds the padding.
///   This works for any `&str`. [`SourceCode::from_strs_array_in`] does the same for text
///   which is assembled from several parts. These are the only ones which copy the text.
///
/// # Example
///
/// ```
/// use oxc_allocator::Allocator;
/// use oxc_span::SourceCode;
///
/// // Copy into an arena
/// let allocator = Allocator::default();
/// let source = SourceCode::from_str_in("let x = 1;", &allocator);
/// assert_eq!(source.as_str(), "let x = 1;");
///
/// // Borrow an owned buffer which has padding added to it
/// let mut buffer = String::from("let x = 1;");
/// SourceCode::pad_string(&mut buffer);
/// let source = SourceCode::from_padded_str(&buffer).unwrap();
/// assert_eq!(source.as_str(), "let x = 1;");
/// ```
#[derive(Clone, Copy)]
pub struct SourceCode<'a> {
    /// Source text followed by `PADDING` NUL bytes.
    ///
    /// NUL is a valid single-byte UTF-8 character, so text and padding together are a valid `&str`.
    /// Holding them as a single `&str` means reading the padding is in bounds of this reference.
    padded: &'a str,
}

/// Padding which is appended to source text.
const PADDING_STR: &str = match str::from_utf8(&[0; SourceCode::PADDING]) {
    Ok(padding) => padding,
    Err(_) => unreachable!(),
};

impl<'a> SourceCode<'a> {
    /// Number of NUL bytes which follow the source text.
    pub const PADDING: usize = 64;

    /// Create a [`SourceCode`] by copying `source_text` into `allocator`, followed by padding.
    ///
    /// This performs 1 allocation of `source_text.len() + SourceCode::PADDING` bytes.
    ///
    /// # Panics
    ///
    /// Panics if `source_text.len() + SourceCode::PADDING` exceeds `isize::MAX`.
    pub fn from_str_in(source_text: &str, allocator: &'a Allocator) -> Self {
        let padded = allocator.alloc_concat_strs_array([source_text, PADDING_STR]);
        Self { padded }
    }

    /// Create a [`SourceCode`] by concatenating `strings` into `allocator`, followed by padding.
    ///
    /// Use this for source text which is assembled from parts, e.g. a snippet with a wrapper
    /// around it. The parts are copied once, into their final place.
    ///
    /// This performs 1 allocation of the total length of `strings` plus [`SourceCode::PADDING`]
    /// bytes.
    ///
    /// # Panics
    ///
    /// Panics if the total length of `strings` plus [`SourceCode::PADDING`] exceeds `isize::MAX`.
    ///
    /// # Example
    ///
    /// ```
    /// use oxc_allocator::Allocator;
    /// use oxc_span::SourceCode;
    ///
    /// let allocator = Allocator::default();
    /// let source = SourceCode::from_strs_array_in(["(", "a + b", ")"], &allocator);
    /// assert_eq!(source.as_str(), "(a + b)");
    /// ```
    pub fn from_strs_array_in<const N: usize>(
        strings: [&str; N],
        allocator: &'a Allocator,
    ) -> Self {
        let capacity = strings
            .iter()
            .try_fold(Self::PADDING, |capacity, string| capacity.checked_add(string.len()))
            .expect("attempted to create a string longer than `isize::MAX` bytes");
        let mut padded = StringBuilder::with_capacity_in(capacity, allocator);
        for string in strings {
            padded.push_str(string);
        }
        padded.push_str(PADDING_STR);
        Self { padded: padded.into_str() }
    }

    /// Read a UTF-8 encoded file straight into `allocator`, followed by padding.
    ///
    /// The file's contents are written into the arena by the read itself. They are not read
    /// into another buffer first, and are not copied afterwards.
    ///
    /// If the file reports no size (e.g. a pipe), it is read into a temporary buffer first,
    /// and then copied into the arena.
    ///
    /// # Errors
    ///
    /// Returns [`io::Error`] if any of:
    ///
    /// * The file cannot be read.
    /// * The file's contents are not valid UTF-8.
    /// * The file plus padding is larger than `isize::MAX` bytes.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use oxc_allocator::Allocator;
    /// use oxc_span::SourceCode;
    ///
    /// let allocator = Allocator::default();
    /// let source = SourceCode::read_in("src/index.js", &allocator)?;
    /// println!("{}", source.as_str());
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn read_in<P: AsRef<Path>>(path: P, allocator: &'a Allocator) -> io::Result<Self> {
        read_file_in(path.as_ref(), allocator).map(|padded| Self { padded })
    }

    /// Create a [`SourceCode`] from a string which already ends with padding, without copying.
    ///
    /// `padded` must be the source text followed by [`SourceCode::PADDING`] NUL bytes.
    /// The source text is everything before those bytes.
    /// [`SourceCode::pad_string`] adds the padding to a `String`.
    ///
    /// Returns `None` if `padded` does not end with [`SourceCode::PADDING`] NUL bytes.
    pub fn from_padded_str(padded: &'a str) -> Option<Self> {
        let source_len = padded.len().checked_sub(Self::PADDING)?;
        let padding = &padded.as_bytes()[source_len..];
        if padding.iter().all(|&byte| byte == 0) { Some(Self { padded }) } else { None }
    }

    /// Append [`SourceCode::PADDING`] NUL bytes to `source_text`,
    /// so it can be passed to [`SourceCode::from_padded_str`].
    ///
    /// This does not reallocate if `source_text` has at least [`SourceCode::PADDING`] bytes
    /// of spare capacity.
    pub fn pad_string(source_text: &mut String) {
        source_text.push_str(PADDING_STR);
    }

    /// Get the source text, without padding.
    #[inline]
    pub fn as_str(self) -> &'a str {
        let source_len = self.len();
        // The byte at `source_len` is the first NUL byte of the padding, which is a UTF-8
        // character boundary, so this slice operation cannot panic
        &self.padded[..source_len]
    }

    /// Get the source text followed by its [`SourceCode::PADDING`] NUL bytes.
    #[inline]
    pub fn as_padded_str(self) -> &'a str {
        self.padded
    }

    /// Get length of the source text in bytes, excluding padding.
    #[inline]
    pub fn len(self) -> usize {
        self.padded.len() - Self::PADDING
    }

    /// Returns `true` if the source text is empty.
    #[inline]
    pub fn is_empty(self) -> bool {
        self.len() == 0
    }
}

impl fmt::Debug for SourceCode<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("SourceCode").field(&self.as_str()).finish()
    }
}

/// Owned source text, followed in memory by [`SourceCode::PADDING`] NUL bytes.
///
/// This is the owned counterpart of [`SourceCode`], as `String` is of `&str`.
/// Use it to hold source text outside of an arena, e.g. when the same source is parsed
/// more than once, or the arena is reset while the source text stays alive.
///
/// [`SourceCodeBuf::as_source_code`] borrows it as a [`SourceCode`], without copying.
///
/// The padding is not part of the source text. `SourceCodeBuf` dereferences to the text alone.
///
/// # Example
///
/// ```
/// use oxc_span::SourceCodeBuf;
///
/// let source = SourceCodeBuf::from("let x = 1;");
/// assert_eq!(source.as_str(), "let x = 1;");
/// assert_eq!(source.as_source_code().as_str(), "let x = 1;");
/// ```
#[derive(Clone)]
pub struct SourceCodeBuf {
    /// Source text followed by `SourceCode::PADDING` NUL bytes.
    padded: String,
}

impl SourceCodeBuf {
    /// Read a UTF-8 encoded file into a [`SourceCodeBuf`].
    ///
    /// The buffer is allocated with room for the file's contents and the padding,
    /// so the contents are not copied after they have been read.
    ///
    /// # Errors
    ///
    /// Returns [`io::Error`] if any of:
    ///
    /// * The file cannot be read.
    /// * The file's contents are not valid UTF-8.
    /// * The buffer cannot be allocated.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use oxc_span::SourceCodeBuf;
    ///
    /// let source = SourceCodeBuf::read("src/index.js")?;
    /// println!("{}", source.as_str());
    /// # Ok::<(), std::io::Error>(())
    /// ```
    pub fn read<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        read_file(path.as_ref()).map(|padded| Self { padded })
    }

    /// Create a [`SourceCodeBuf`] from a `String`, by appending padding to it.
    ///
    /// This does not copy `source_text` if it has at least [`SourceCode::PADDING`] bytes
    /// of spare capacity. Otherwise the `String` is reallocated.
    pub fn from_string(mut source_text: String) -> Self {
        source_text.reserve_exact(SourceCode::PADDING);
        SourceCode::pad_string(&mut source_text);
        Self { padded: source_text }
    }

    /// Borrow as a [`SourceCode`]. This does not copy the source text.
    #[inline]
    pub fn as_source_code(&self) -> SourceCode<'_> {
        SourceCode { padded: &self.padded }
    }

    /// Get the source text, without padding.
    #[inline]
    pub fn as_str(&self) -> &str {
        self.as_source_code().as_str()
    }

    /// Get length of the source text in bytes, excluding padding.
    #[inline]
    pub fn len(&self) -> usize {
        self.padded.len() - SourceCode::PADDING
    }

    /// Returns `true` if the source text is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Convert into a `String` containing the source text, without padding.
    ///
    /// This does not copy the source text.
    pub fn into_string(self) -> String {
        let source_len = self.len();
        let mut source_text = self.padded;
        source_text.truncate(source_len);
        source_text
    }
}

impl Default for SourceCodeBuf {
    /// Create an empty [`SourceCodeBuf`]. It contains only the padding.
    fn default() -> Self {
        Self::from_string(String::new())
    }
}

impl From<String> for SourceCodeBuf {
    /// See [`SourceCodeBuf::from_string`].
    fn from(source_text: String) -> Self {
        Self::from_string(source_text)
    }
}

impl From<&str> for SourceCodeBuf {
    /// Copy `source_text` into a new buffer, followed by padding.
    fn from(source_text: &str) -> Self {
        let mut padded = String::with_capacity(source_text.len() + SourceCode::PADDING);
        padded.push_str(source_text);
        SourceCode::pad_string(&mut padded);
        Self { padded }
    }
}

impl Deref for SourceCodeBuf {
    type Target = str;

    #[inline]
    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<str> for SourceCodeBuf {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Debug for SourceCodeBuf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("SourceCodeBuf").field(&self.as_str()).finish()
    }
}

/// Read file at `path` into a `String`, followed by [`SourceCode::PADDING`] NUL bytes.
fn read_file(path: &Path) -> io::Result<String> {
    let mut file = File::open(path)?;

    // Allocate space for the file's contents and the padding in one go,
    // so neither reading the file nor adding the padding reallocates.
    // The size is only a hint. `read_to_end` grows the buffer if the file is larger than reported.
    let size = file.metadata().map_or(0, |metadata| metadata.len());
    let capacity = usize::try_from(size).unwrap_or(usize::MAX).saturating_add(SourceCode::PADDING);
    let mut bytes = Vec::new();
    bytes.try_reserve_exact(capacity)?;
    file.read_to_end(&mut bytes)?;

    validate_utf8(&bytes)?;
    // SAFETY: `validate_utf8` has checked `bytes` is valid UTF-8
    let mut padded = unsafe { String::from_utf8_unchecked(bytes) };
    padded.reserve_exact(SourceCode::PADDING);
    SourceCode::pad_string(&mut padded);
    Ok(padded)
}

/// Read file at `path` into `allocator`, followed by [`SourceCode::PADDING`] NUL bytes.
fn read_file_in<'a>(path: &Path, allocator: &'a Allocator) -> io::Result<&'a str> {
    let mut file = File::open(path)?;

    match file.metadata() {
        // A size of 0 is reported for files whose size is not known in advance (e.g. pipes),
        // as well as for empty files. Take the fallback path for both.
        Ok(metadata) if metadata.len() > 0 => {
            let padded = read_file_with_known_size_in(file, metadata.len(), allocator)?;
            // The padding is NUL bytes, which are valid UTF-8, so text and padding are checked together
            validate_utf8(padded)
        }
        _ => {
            // Read file into a `Vec`, and then copy it into the arena
            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes)?;
            let source_text = validate_utf8(&bytes)?;
            if isize::try_from(source_text.len().saturating_add(SourceCode::PADDING)).is_err() {
                return Err(file_too_large());
            }
            Ok(allocator.alloc_concat_strs_array([source_text, PADDING_STR]))
        }
    }
}

/// Read contents of `file` directly into `allocator`, followed by [`SourceCode::PADDING`] NUL bytes.
///
/// At most `size` bytes are read from the file.
fn read_file_with_known_size_in(file: File, size: u64, allocator: &Allocator) -> io::Result<&[u8]> {
    // Check file plus padding is not larger than `isize::MAX` bytes (the max size of an allocation)
    let capacity = usize::try_from(size)
        .ok()
        .and_then(|size| size.checked_add(SourceCode::PADDING))
        .filter(|&capacity| isize::try_from(capacity).is_ok());
    let Some(capacity) = capacity else {
        return Err(file_too_large());
    };

    // Allocate space for file's contents and padding in allocator.
    // SAFETY: We checked above that `capacity <= isize::MAX`. `u8` has no alignment requirements.
    let layout = unsafe { Layout::from_size_align_unchecked(capacity, 1) };
    let ptr = allocator.alloc_layout(layout);

    // Read contents of file into allocated space.
    //
    // * Create a `Vec` which pretends to own the allocation we just created in arena.
    // * Wrap the `Vec` in `ManuallyDrop`, so it doesn't free the memory at end of the block,
    //   or if there's a panic during reading.
    // * Use `File::take` to obtain a reader which yields no more than `size` bytes.
    //   This ensures it can't produce more data than we allocated space for - in case file increased
    //   in size since the call to `file.metadata()`, or `file.metadata()` returned inaccurate size.
    // * Use `Read::read_to_end` to fill the `Vec` from this reader.
    //
    // This is a hack. It's completely bananas that Rust doesn't provide a method to write into
    // a slice of uninitialized bytes, but this seems to be the only safe way to do it on stable Rust.
    // https://users.rust-lang.org/t/reading-c-style-structures-from-disk/70529/7
    //
    // `Read::read_to_end` only grows the `Vec` when it is full. The reader yields at most `size` bytes,
    // and the `Vec` has capacity for `size + SourceCode::PADDING` bytes, so it can never be full.
    // If it did grow, that would be UB as it would free the chunk of memory backing the `Vec`,
    // which it doesn't actually own.
    //
    // Unfortunately `Read::read_to_end`'s docs don't guarantee this behavior. But the code is written
    // specifically to avoid growing the `Vec` unnecessarily, and there was a PR to make sure it doesn't:
    // https://github.com/rust-lang/rust/pull/89165
    // So in practice we can rely on this behavior.
    let source_len = {
        // SAFETY: We've allocated `capacity` bytes starting at `ptr`.
        // This `Vec` doesn't actually own that memory, but we immediately wrap it in `ManuallyDrop`,
        // so it won't free the memory on drop. As long as the `Vec` doesn't grow, no UB (see above).
        let vec = unsafe { Vec::from_raw_parts(ptr.as_ptr(), 0, capacity) };
        let mut vec = ManuallyDrop::new(vec);
        let bytes_read = file.take(size).read_to_end(&mut vec)?;

        assert_eq!(vec.capacity(), capacity);
        debug_assert_eq!(vec.len(), bytes_read);

        // File may have got smaller since the call to `file.metadata()`,
        // or `file.metadata()` may have reported inaccurate size.
        // So use the number of bytes which were read, not `size`.
        vec.len()
    };

    // Write the padding after the file's contents.
    // SAFETY: `source_len <= size`, because the reader yields at most `size` bytes.
    // The allocation is `size + SourceCode::PADDING` bytes, so the `SourceCode::PADDING` bytes
    // starting at `ptr + source_len` are within it.
    unsafe { ptr.add(source_len).write_bytes(0, SourceCode::PADDING) };

    // SAFETY: The first `source_len` bytes starting at `ptr` were initialized by reading the file,
    // and the next `SourceCode::PADDING` bytes were initialized just above.
    // The memory is in the arena, so it lives as long as the borrow of `allocator`.
    let bytes = unsafe { slice::from_raw_parts(ptr.as_ptr(), source_len + SourceCode::PADDING) };
    Ok(bytes)
}

/// Convert `bytes` to a `&str`, checking it is valid UTF-8.
fn validate_utf8(bytes: &[u8]) -> io::Result<&str> {
    // `simdutf8` is faster than `std::str::from_utf8`.
    // Return same error as `fs::read_to_string` produces.
    simdutf8::basic::from_utf8(bytes).map_err(|_| {
        io::Error::new(io::ErrorKind::InvalidData, "stream did not contain valid UTF-8")
    })
}

fn file_too_large() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, "File is larger than `isize::MAX` bytes")
}

#[cfg(test)]
mod test {
    use std::{
        fs, io,
        path::{Path, PathBuf},
    };

    use oxc_allocator::Allocator;

    use super::{SourceCode, SourceCodeBuf};

    /// A file in the system's temp directory, which is deleted on drop.
    struct TempFile(PathBuf);

    impl TempFile {
        fn new(name: &str, contents: &[u8]) -> Self {
            let file_name = format!("oxc_span_source_code_{}_{name}", std::process::id());
            let path = std::env::temp_dir().join(file_name);
            fs::write(&path, contents).unwrap();
            Self(path)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempFile {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.0);
        }
    }

    /// Source texts of sizes around the sizes which `Read::read_to_end` treats specially.
    fn texts() -> Vec<String> {
        let mut texts: Vec<String> =
            ["", "x", "let x = 1;", "日本語", "a\0b"].into_iter().map(String::from).collect();
        for len in [31, 32, 33, 63, 64, 65, 8191, 8192, 8193, 100_000] {
            texts.push("abcdefg\n".repeat(len).chars().take(len).collect());
        }
        texts
    }

    fn assert_padded(source: SourceCode<'_>, text: &str) {
        assert_eq!(source.as_str(), text);
        assert_eq!(source.len(), text.len());
        let padded = source.as_padded_str();
        assert_eq!(padded.len(), text.len() + SourceCode::PADDING);
        assert!(padded.as_bytes()[text.len()..].iter().all(|&byte| byte == 0));
    }

    #[test]
    fn from_strs_array_in() {
        let allocator = Allocator::default();
        assert_padded(SourceCode::from_strs_array_in([], &allocator), "");
        assert_padded(SourceCode::from_strs_array_in(["", ""], &allocator), "");
        assert_padded(SourceCode::from_strs_array_in(["(", "a + b", ")"], &allocator), "(a + b)");
        assert_padded(SourceCode::from_strs_array_in(["日本", "", "語"], &allocator), "日本語");
    }

    #[test]
    fn read_in() {
        let allocator = Allocator::default();
        for (index, text) in texts().iter().enumerate() {
            let file = TempFile::new(&format!("read_in_{index}"), text.as_bytes());
            let source = SourceCode::read_in(file.path(), &allocator).unwrap();
            assert_padded(source, text);
        }
    }

    #[test]
    fn read_in_errors() {
        let allocator = Allocator::default();

        let missing = std::env::temp_dir().join("oxc_span_source_code_does_not_exist");
        let err = SourceCode::read_in(&missing, &allocator).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::NotFound);

        // Invalid UTF-8
        let file = TempFile::new("read_in_invalid_utf8", b"let x = '\xFF';");
        let err = SourceCode::read_in(file.path(), &allocator).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    }

    #[test]
    fn buf_read() {
        for (index, text) in texts().iter().enumerate() {
            let file = TempFile::new(&format!("buf_read_{index}"), text.as_bytes());
            let source = SourceCodeBuf::read(file.path()).unwrap();
            assert_padded(source.as_source_code(), text);
            assert_eq!(source.as_str(), text);
            assert_eq!(source.len(), text.len());
            assert_eq!(source.is_empty(), text.is_empty());
            assert_eq!(&source.into_string(), text);
        }
    }

    #[test]
    fn buf_read_errors() {
        let missing = std::env::temp_dir().join("oxc_span_source_code_does_not_exist");
        let err = SourceCodeBuf::read(&missing).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::NotFound);

        // Invalid UTF-8
        let file = TempFile::new("buf_read_invalid_utf8", b"let x = '\xFF';");
        let err = SourceCodeBuf::read(file.path()).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    }

    #[test]
    fn buf_from_string() {
        // Does not reallocate when the `String` has room for the padding
        let mut string = String::with_capacity(3 + SourceCode::PADDING);
        string.push_str("abc");
        let ptr = string.as_ptr();
        let source = SourceCodeBuf::from_string(string);
        assert_padded(source.as_source_code(), "abc");
        assert_eq!(source.as_str().as_ptr(), ptr);

        // Converting back does not reallocate either
        let string = source.into_string();
        assert_eq!(string, "abc");
        assert_eq!(string.as_ptr(), ptr);

        for text in texts() {
            assert_padded(SourceCodeBuf::from(text.clone()).as_source_code(), &text);
            assert_padded(SourceCodeBuf::from(text.as_str()).as_source_code(), &text);
        }
    }

    #[test]
    fn buf_default_and_clone() {
        let source = SourceCodeBuf::default();
        assert!(source.is_empty());
        assert_padded(source.as_source_code(), "");

        let source = SourceCodeBuf::from("let x = 1;");
        let clone = source.clone();
        assert_padded(clone.as_source_code(), "let x = 1;");
        assert_ne!(clone.as_str().as_ptr(), source.as_str().as_ptr());
    }

    #[test]
    fn buf_deref() {
        let source = SourceCodeBuf::from("let x = 1;");
        let text: &str = &source;
        assert_eq!(text, "let x = 1;");
        assert!(source.starts_with("let"));
        assert_eq!(format!("{source:?}"), r#"SourceCodeBuf("let x = 1;")"#);
    }

    #[test]
    fn from_str_in() {
        let allocator = Allocator::default();
        for text in ["", "x", "let x = 1;", "日本語", "a\0b"] {
            let source = SourceCode::from_str_in(text, &allocator);
            assert_eq!(source.as_str(), text);
            assert_eq!(source.len(), text.len());
            assert_eq!(source.is_empty(), text.is_empty());

            let padded = source.as_padded_str();
            assert_eq!(padded.len(), text.len() + SourceCode::PADDING);
            assert!(padded.as_bytes()[text.len()..].iter().all(|&byte| byte == 0));
            assert_eq!(padded.as_ptr(), source.as_str().as_ptr());
        }
    }

    #[test]
    fn from_padded_str() {
        let mut buffer = String::from("let x = 1;");
        SourceCode::pad_string(&mut buffer);
        let source = SourceCode::from_padded_str(&buffer).unwrap();
        assert_eq!(source.as_str(), "let x = 1;");
        // Zero-copy
        assert_eq!(source.as_str().as_ptr(), buffer.as_ptr());
        assert_eq!(source.as_padded_str().len(), buffer.len());
    }

    #[test]
    fn from_padded_str_trailing_nul_in_source() {
        // NUL bytes before the padding are part of the source text
        let mut buffer = String::from("x\0\0");
        SourceCode::pad_string(&mut buffer);
        let source = SourceCode::from_padded_str(&buffer).unwrap();
        assert_eq!(source.as_str(), "x\0\0");
    }

    #[test]
    fn from_padded_str_rejects_missing_padding() {
        assert!(SourceCode::from_padded_str("").is_none());
        assert!(SourceCode::from_padded_str("let x = 1;").is_none());

        // 1 byte too few
        let too_short = "\0".repeat(SourceCode::PADDING - 1);
        assert!(SourceCode::from_padded_str(&too_short).is_none());

        // Non-NUL byte within the padding
        let mut buffer = String::from("x");
        SourceCode::pad_string(&mut buffer);
        buffer.push('y');
        assert!(SourceCode::from_padded_str(&buffer).is_none());
    }

    #[test]
    fn pad_string_does_not_reallocate_with_spare_capacity() {
        let mut buffer = String::with_capacity(3 + SourceCode::PADDING);
        buffer.push_str("abc");
        let ptr = buffer.as_ptr();
        SourceCode::pad_string(&mut buffer);
        assert_eq!(buffer.as_ptr(), ptr);
        assert_eq!(buffer.len(), 3 + SourceCode::PADDING);
    }
}
