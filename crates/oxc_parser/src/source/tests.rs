use std::{
    alloc::Layout,
    env, fs,
    io::{self, Read},
    path::{Path, PathBuf},
    process,
    ptr::{self, NonNull},
    slice, str,
};

use oxc_allocator::Allocator;
use oxc_span::SourceType;

use super::{Source, SourceFileError, reader::READER_INITIAL_CAPACITY};

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
    let mut inputs: Vec<Vec<u8>> = [&b""[..], b"x", b"let x = 1;", "日本語".as_bytes(), b"a\0b"]
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
fn from_strs() {
    let allocator = Allocator::default();

    let source = Source::from_strs([], SourceType::tsx(), &allocator);
    assert_source(&source, b"", true);
    assert_eq!(source.source_type(), SourceType::tsx());

    let source = Source::from_strs([""], SourceType::tsx(), &allocator);
    assert_source(&source, b"", true);

    let text = "let x = 1;";
    let source = Source::from_strs([text], SourceType::tsx(), &allocator);
    assert_source(&source, b"let x = 1;", true);
    // Source is copied
    assert_ne!(source.as_bytes().as_ptr(), text.as_ptr());

    let source = Source::from_strs(["(", "{ x: 1 }", "\n)"], SourceType::tsx(), &allocator);
    assert_source(&source, b"({ x: 1 }\n)", true);

    let source = Source::from_strs(["", "日本", "", "語", ""], SourceType::tsx(), &allocator);
    assert_source(&source, "日本語".as_bytes(), true);

    // Same string more than once
    let text = "a\0b";
    let source = Source::from_strs([text, text, text], SourceType::tsx(), &allocator);
    assert_source(&source, b"a\0ba\0ba\0b", true);
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

/// `Source::into_raw_parts` returns parts which `Source::from_raw_parts`
/// can recreate the `Source` from, after the `Allocator` has been moved.
#[test]
fn into_raw_parts() {
    let allocator = Allocator::default();
    let text = "let x = 1;";

    let source = Source::from_str(text, SourceType::cjs(), &allocator);
    let parts = source.into_raw_parts();
    let (source_ptr, len, is_valid_utf8, source_type, source_allocator) = parts;
    assert_eq!(len, text.len());
    assert!(is_valid_utf8);
    assert_eq!(source_type, SourceType::cjs());
    assert!(ptr::eq(source_allocator, &raw const allocator));

    let source = Source::from_bytes(text.as_bytes(), SourceType::cjs(), &allocator);
    assert!(!source.into_raw_parts().2);

    // Move the `Allocator`. The source bytes it contains do not move.
    let allocator = Box::new(allocator);

    // SAFETY: `source_ptr` and `len` are from a `Source` which was created in `allocator`.
    // That `Source` no longer exists, and nothing else refers to its source bytes.
    // They are valid UTF-8.
    let source = unsafe { Source::from_raw_parts(source_ptr, len, true, source_type, &allocator) };
    assert_source(&source, text.as_bytes(), true);
    assert_eq!(source.as_bytes().as_ptr(), source_ptr.as_ptr().cast_const());
}

#[test]
fn read_from_file() {
    let allocator = Allocator::default();
    for (index, bytes) in inputs().iter().enumerate() {
        let file = TempFile::new(&format!("read_from_file_{index}.ts"), bytes);
        let source = Source::read_from_file(file.path(), &allocator).unwrap();
        assert_source(&source, bytes, false);
        assert_eq!(source.source_type(), SourceType::from_path(file.path()).unwrap());
    }
}

#[test]
fn read_from_file_infers_source_type() {
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
        let source_type = Source::read_from_file(file.path(), &allocator).unwrap().source_type();
        assert_eq!(source_type.is_typescript(), is_typescript);
        assert_eq!(source_type.is_jsx(), is_jsx);
    }
}

#[test]
fn read_from_file_errors() {
    let allocator = Allocator::default();

    // File does not exist
    let path = TempFile::path_for("does_not_exist.js");
    let err = Source::read_from_file(&path, &allocator).err().unwrap();
    let SourceFileError::Io(io_err) = &err else { panic!("Wrong error: {err:?}") };
    assert_eq!(io_err.kind(), io::ErrorKind::NotFound);
    assert_eq!(err.to_string(), io_err.to_string());

    // Unknown extension
    let file = TempFile::new("unknown_extension.vue", b"<script></script>");
    let err = Source::read_from_file(file.path(), &allocator).err().unwrap();
    let SourceFileError::UnknownSourceType(err_path) = &err else { panic!("Wrong error: {err:?}") };
    assert_eq!(err_path, file.path());
    assert_eq!(
        err.to_string(),
        format!("Cannot infer source type from file extension: {}", file.path().display())
    );

    // No extension
    let path = TempFile::path_for("no_extension");
    let err = Source::read_from_file(&path, &allocator).err().unwrap();
    assert!(matches!(err, SourceFileError::UnknownSourceType(_)));
}

/// Reader which yields `bytes`, at most `chunk_len` bytes per read.
struct ChunkedReader<'b> {
    bytes: &'b [u8],
    chunk_len: usize,
}

impl Read for ChunkedReader<'_> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let len = buf.len().min(self.chunk_len).min(self.bytes.len());
        let (chunk, rest) = self.bytes.split_at(len);
        buf[..len].copy_from_slice(chunk);
        self.bytes = rest;
        Ok(len)
    }
}

/// Reader which yields `bytes`, and then fails.
struct FailingReader<'b> {
    bytes: &'b [u8],
}

impl Read for FailingReader<'_> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.bytes.is_empty() {
            return Err(io::Error::other("Reader failed"));
        }
        self.bytes.read(buf)
    }
}

/// Byte strings to read with `Source::read_from_reader`.
///
/// [`inputs`], plus lengths around the sizes where:
/// - Buffer fills up, and has to grow to read more data.
/// - Buffer has no room left for the padding after the data, and has to grow to make room.
fn reader_inputs() -> Vec<Vec<u8>> {
    let mut inputs = inputs();
    for multiple in [1, 2, 4] {
        let capacity = READER_INITIAL_CAPACITY * multiple;
        for boundary in [capacity - Source::PADDING, capacity] {
            for len in [boundary - 1, boundary, boundary + 1] {
                inputs.push(b"abcdefg\n".iter().copied().cycle().take(len).collect());
            }
        }
    }
    inputs
}

#[test]
fn read_from_reader() {
    let allocator = Allocator::default();
    for bytes in reader_inputs() {
        for chunk_len in [1, 7, 100, 8192, usize::MAX] {
            let reader = ChunkedReader { bytes: &bytes, chunk_len };
            let source = Source::read_from_reader(reader, SourceType::mjs(), &allocator).unwrap();
            assert_source(&source, &bytes, false);
            assert_eq!(source.source_type(), SourceType::mjs());
        }
    }
}

/// The buffer grows in place, if the arena's current chunk has room.
/// No smaller buffers are left behind in the arena.
#[test]
fn read_from_reader_grows_in_place() {
    const LEN: usize = 100_000;
    let bytes: Vec<u8> = b"abcdefg\n".iter().copied().cycle().take(LEN).collect();

    let allocator = Allocator::with_capacity(1024 * 1024);
    let reader = ChunkedReader { bytes: &bytes, chunk_len: usize::MAX };
    let source = Source::read_from_reader(reader, SourceType::mjs(), &allocator).unwrap();
    assert_source(&source, &bytes, false);

    // Arena contains only the buffer, which doubled in size until it was large enough
    let mut capacity = READER_INITIAL_CAPACITY;
    while capacity < LEN + Source::PADDING {
        capacity *= 2;
    }
    assert_eq!(allocator.used_bytes(), capacity);
}

#[test]
fn read_from_reader_errors() {
    let allocator = Allocator::default();
    for bytes in reader_inputs() {
        let reader = FailingReader { bytes: &bytes };
        let err = Source::read_from_reader(reader, SourceType::mjs(), &allocator).err().unwrap();
        assert_eq!(err.kind(), io::ErrorKind::Other);
    }
}

#[test]
fn set_source_type() {
    let allocator = Allocator::default();
    let text = "let x = 1;";

    let mut source = Source::from_str(text, SourceType::mjs(), &allocator);
    let ptr = source.as_bytes().as_ptr();
    source.set_source_type(SourceType::tsx());

    assert_eq!(source.source_type(), SourceType::tsx());
    // Source bytes are not altered, or moved
    assert_source(&source, text.as_bytes(), true);
    assert_eq!(source.as_bytes().as_ptr(), ptr);
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

/// `Source::into_str` returns the source text, or an error if it's not valid UTF-8.
#[test]
fn into_str() {
    let allocator = Allocator::default();

    // Created from a `&str`
    let source = Source::from_str("日本語", SourceType::mjs(), &allocator);
    assert_eq!(source.into_str(), Ok("日本語"));

    // Created from bytes which are valid UTF-8
    let source = Source::from_bytes("日本語".as_bytes(), SourceType::mjs(), &allocator);
    assert_eq!(source.into_str(), Ok("日本語"));

    // Created from bytes which are not valid UTF-8
    let source = Source::from_bytes(b"let x = '\xFF';", SourceType::mjs(), &allocator);
    assert_eq!(source.into_str().unwrap_err().valid_up_to(), 9);

    // Returned `&str` is the `Source`'s own copy of the source, and outlives the `Source`
    let source = Source::from_str("let x = 1;", SourceType::mjs(), &allocator);
    let ptr = source.as_bytes().as_ptr();
    let source_text = source.into_str().unwrap();
    assert_eq!(source_text.as_ptr(), ptr);
    assert_eq!(source_text, "let x = 1;");
}

#[test]
fn read_from_file_with_source_type() {
    let allocator = Allocator::default();

    // Extension is ignored
    for (index, bytes) in inputs().iter().enumerate() {
        let file = TempFile::new(&format!("with_source_type_{index}.vue"), bytes);
        let source =
            Source::read_from_file_with_source_type(file.path(), SourceType::tsx(), &allocator)
                .unwrap();
        assert_source(&source, bytes, false);
        assert_eq!(source.source_type(), SourceType::tsx());
    }

    // File does not exist
    let path = TempFile::path_for("does_not_exist.vue");
    let err = Source::read_from_file_with_source_type(&path, SourceType::tsx(), &allocator)
        .err()
        .unwrap();
    assert_eq!(err.kind(), io::ErrorKind::NotFound);
}
