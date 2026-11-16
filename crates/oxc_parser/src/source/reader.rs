//! Methods to create a [`Source`] from a file or reader.

use std::{
    cmp::min,
    error::Error,
    fmt::{self, Display, Formatter},
    fs::File,
    io::{self, Read},
    mem::ManuallyDrop,
    path::{Path, PathBuf},
    ptr::NonNull,
};

use oxc_allocator::{Allocator, ArenaVec};
use oxc_span::SourceType;

use super::{Source, padded_layout};

/// Initial capacity of the buffer which data of unknown length is read into, in bytes.
pub(super) const READER_INITIAL_CAPACITY: usize = 8 * 1024;

/// Maximum capacity of the buffer which data of unknown length is read into, in bytes.
///
/// This is the maximum capacity of an [`ArenaVec<u8>`].
/// This capacity includes [`Source::PADDING`] bytes which must be written after the data,
/// so maximum size of source which can be read is [`Source::PADDING`] less than this.
const READER_MAX_CAPACITY: usize = cfg_select! {
    target_pointer_width = "64" => u32::MAX as usize,
    _ => isize::MAX as usize,
};

impl<'a> Source<'a> {
    /// Create a [`Source`] from a file.
    ///
    /// [`SourceType`] is inferred from the file's extension.
    /// Use [`read_from_file_with_source_type`] to specify it instead.
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
    /// - The file is not a regular file (e.g. it's a pipe), and its contents are longer than
    ///   the longest source which can be buffered (see [`read_from_reader`]).
    ///
    /// [`read_from_file_with_source_type`]: Self::read_from_file_with_source_type
    /// [`read_from_reader`]: Self::read_from_reader
    pub fn read_from_file(
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
    /// Returns [`io::Error`] if any of:
    ///
    /// - The file cannot be read.
    /// - The file plus padding is larger than `isize::MAX` bytes.
    /// - The file is not a regular file (e.g. it's a pipe), and its contents are longer than
    ///   the longest source which can be buffered (see [`read_from_reader`]).
    ///
    /// [`read_from_reader`]: Self::read_from_reader
    pub fn read_from_file_with_source_type(
        path: impl AsRef<Path>,
        source_type: SourceType,
        allocator: &'a Allocator,
    ) -> io::Result<Self> {
        Self::read_file(path.as_ref(), source_type, allocator)
    }

    /// Create a [`Source`] from a reader (e.g. stdin).
    ///
    /// All of `reader`'s data is read directly into `allocator`, up to the end of the stream.
    /// The data does not need to be valid UTF-8.
    ///
    /// Use this when the length of the data is not known in advance. To read a file,
    /// use [`read_from_file`] or [`read_from_file_with_source_type`].
    ///
    /// # Example
    ///
    /// ```
    /// use std::io::Cursor;
    ///
    /// use oxc_allocator::Allocator;
    /// use oxc_parser::Source;
    /// use oxc_span::SourceType;
    ///
    /// let allocator = Allocator::default();
    /// // Could be `std::io::stdin().lock()`, or any other type which implements `Read`
    /// let reader = Cursor::new("let x = 1;");
    /// let source = Source::read_from_reader(reader, SourceType::mjs(), &allocator).unwrap();
    /// assert_eq!(source.as_bytes(), b"let x = 1;");
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`io::Error`] if either:
    ///
    /// - Reading from `reader` fails.
    /// - The data is longer than the longest source which can be buffered.
    ///   That's just under 4 GiB on 64-bit systems, and just under 2 GiB on 32-bit systems.
    ///
    /// [`read_from_file`]: Self::read_from_file
    /// [`read_from_file_with_source_type`]: Self::read_from_file_with_source_type
    pub fn read_from_reader<R: Read>(
        reader: R,
        source_type: SourceType,
        allocator: &'a Allocator,
    ) -> io::Result<Self> {
        Self::read_with_unknown_size(reader, source_type, allocator)
    }

    /// Read file at `path` into `allocator`, and create a [`Source`] from it.
    ///
    /// # Errors
    ///
    /// Returns [`io::Error`] if any of:
    ///
    /// - Reading the file fails.
    /// - The file is larger than `isize::MAX` - [`PADDING`] bytes.
    /// - The file is not a regular file (e.g. it's a pipe), and its contents are longer than
    ///   [`READER_MAX_CAPACITY`] - [`PADDING`] bytes.
    ///
    /// [`PADDING`]: Self::PADDING
    fn read_file(
        path: &Path,
        source_type: SourceType,
        allocator: &'a Allocator,
    ) -> io::Result<Self> {
        let file = File::open(path)?;

        match file.metadata() {
            // File system knows the size of a regular file, so its contents can be read into
            // an allocation of exactly the right size. That includes an empty file.
            //
            // Files in `procfs` (`/proc` on Linux) are an exception. They are regular files,
            // but report a size of 0, even though reading them yields data. They are read
            // as empty here. That's not a problem, as source code does not live in `procfs`.
            Ok(metadata) if metadata.is_file() => {
                Self::read_file_with_known_size(file, metadata.len(), source_type, allocator)
            }
            // Anything else which can be opened as a file (e.g. a pipe) can yield data,
            // but its size is not known in advance. So read until the end of the data.
            _ => Self::read_with_unknown_size(file, source_type, allocator),
        }
    }

    /// Read contents of `file` directly into `allocator`, and create a [`Source`] from it.
    ///
    /// `file_size` is the size of the file in bytes, as reported by the file system.
    /// At most `file_size` bytes are read from the file.
    ///
    /// # Errors
    ///
    /// Returns [`io::Error`] if either:
    ///
    /// - Reading the file fails.
    /// - The file is larger than `isize::MAX` - [`PADDING`] bytes.
    ///
    /// [`PADDING`]: Self::PADDING
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

    /// Read all of `reader`'s data directly into `allocator`, and create a [`Source`] from it.
    ///
    /// This is for data whose length is not known in advance. The buffer which the data is read
    /// into starts small, and doubles in size each time it fills up.
    ///
    /// Returns `Err` if `reader` yields more than [`READER_MAX_CAPACITY`] - [`PADDING`] bytes.
    ///
    /// [`PADDING`]: Self::PADDING
    fn read_with_unknown_size<R: Read>(
        mut reader: R,
        source_type: SourceType,
        allocator: &'a Allocator,
    ) -> io::Result<Self> {
        // Buffer which the data is read into.
        //
        // Growing this `ArenaVec` usually does not leave its old allocation behind as dead space.
        // If the `ArenaVec` is the arena's most recent allocation, and the arena's current chunk
        // has room, the arena extends the allocation downwards, and moves the data to the new
        // start of it. Nothing else allocates in the arena during this function, so the `ArenaVec`
        // is always the most recent allocation.
        let mut buffer = ArenaVec::<u8>::with_capacity_in(READER_INITIAL_CAPACITY, &allocator);

        loop {
            // Read into all of the buffer's spare capacity.
            // At this point, buffer always has some spare capacity. It starts with some,
            // and it's grown below whenever it gets full.
            //
            // This uses the same hack as `read_file_with_known_size` does - a `Vec` which pretends
            // to own the spare capacity. `Read::take` limits the reader to yielding no more bytes
            // than the `Vec` has capacity for, so `Read::read_to_end` does not grow the `Vec`
            // (see the comments in `read_file_with_known_size`).
            let len = buffer.len();
            let spare_capacity = buffer.capacity() - len;
            let bytes_read = {
                // SAFETY: `len <= capacity`, so `len` bytes after start of the buffer
                // is in bounds of the buffer's allocation, or the end of it
                let spare_ptr = unsafe { buffer.as_mut_ptr().add(len) };
                // SAFETY: Buffer's allocation has `spare_capacity` bytes starting at `spare_ptr`.
                // This `Vec` doesn't actually own that memory, but we immediately wrap it
                // in `ManuallyDrop`, so it won't free the memory on drop.
                // As long as the `Vec` doesn't grow, no UB (see above).
                let vec = unsafe { Vec::from_raw_parts(spare_ptr, 0, spare_capacity) };
                let mut vec = ManuallyDrop::new(vec);
                let limit = spare_capacity as u64;
                let bytes_read = reader.by_ref().take(limit).read_to_end(&mut vec)?;

                debug_assert_eq!(vec.capacity(), spare_capacity);
                debug_assert_eq!(vec.len(), bytes_read);

                bytes_read
            };

            let new_len = len + bytes_read;

            // SAFETY: `read_to_end` initialized the `bytes_read` bytes after the first `len` bytes.
            // `bytes_read <= spare_capacity`, so new length is within the buffer's capacity.
            unsafe { buffer.set_len(new_len) };

            if bytes_read < spare_capacity {
                // Buffer is not full, so `reader` is exhausted
                break;
            }

            // Buffer is full. It has to grow, either to hold more data, or (if `reader` happens
            // to be exhausted) to make room for the padding. So double its capacity and read again.
            // `new_len == capacity` here because the buffer is full.
            debug_assert_eq!(new_len, buffer.capacity());

            if new_len == READER_MAX_CAPACITY {
                // Buffer is full at max capacity, so no room for more data, or the padding
                return Err(file_too_large());
            }

            let extra_capacity = min(new_len, READER_MAX_CAPACITY - new_len);
            buffer.reserve_exact(extra_capacity);
        }

        // Make room for the padding after the data, if there isn't room already
        let len = buffer.len();
        if buffer.capacity() - len < Self::PADDING {
            if len > (READER_MAX_CAPACITY - Self::PADDING) {
                // Padding would take buffer over its max capacity
                return Err(file_too_large());
            }
            buffer.reserve_exact(Self::PADDING);
        }

        // SAFETY: An `ArenaVec`'s pointer is never null
        let ptr = unsafe { NonNull::new_unchecked(buffer.as_mut_ptr()) };

        // SAFETY:
        // - `ptr` points to the buffer's allocation, which is in `allocator`'s arena.
        //   `capacity >= len + PADDING` (ensured just above), so the allocation is at least
        //   `len + PADDING` bytes. `buffer` is not used again, and has no `Drop` impl,
        //   so nothing else has a pointer or reference to the allocation.
        // - The first `len` bytes were initialized by reading from `reader`.
        // - `is_valid_utf8` is `false`.
        Ok(unsafe { Self::from_raw_parts(ptr, len, false, source_type, allocator) })
    }
}

/// Create error for a file which is too large to store in an arena.
fn file_too_large() -> io::Error {
    io::Error::new(io::ErrorKind::FileTooLarge, "File is too large")
}

/// Error returned by [`Source::read_from_file`].
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
