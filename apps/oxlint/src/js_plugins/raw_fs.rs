use std::{
    fs::{self, File},
    io::{self, Read},
    mem::ManuallyDrop,
    path::Path,
    ptr, slice,
};

use oxc_allocator::Allocator;
use oxc_linter::RuntimeFileSystem;

/// File system used when JS plugins are in use.
///
/// Identical to `OsFileSystem`, except that `read_to_arena_str` reads the file's contents into
/// start of the allocator, instead of the end. This conforms to what raw transfer needs.
///
/// Must only be used in conjunction with `AllocatorPool` created with `new_fixed_size`,
/// which wraps `Allocator`s with a custom `Drop` impl, which makes `read_to_arena_str` safe.
///
/// This is a temporary solution. When we replace `bumpalo` with our own allocator, all strings
/// will be written at start of the arena, so then `OsFileSystem` will work fine, and we can
/// remove `RawTransferFileSystem`. TODO: Do that!
pub struct RawTransferFileSystem;

impl RuntimeFileSystem for RawTransferFileSystem {
    /// Read file from disk into start of `allocator`.
    ///
    /// # SAFETY
    /// `allocator` must not be dropped after calling this method.
    /// See [`Allocator::alloc_bytes_start`] for more details.
    ///
    /// This should be an unsafe method, but can't because we're implementing a safe trait method.
    fn read_to_arena_str<'a>(
        &self,
        path: &Path,
        allocator: &'a Allocator,
    ) -> Result<&'a str, std::io::Error> {
        // SAFETY: Caller promises not to allow `allocator` to be dropped
        unsafe { read_to_arena_str(path, allocator) }
    }

    fn write_file(&self, path: &Path, content: &str) -> Result<(), std::io::Error> {
        fs::write(path, content)
    }
}

/// Read the contents of a UTF-8 encoded file directly into arena allocator,
/// at the *start* of the arena, instead of the end.
///
/// Avoids intermediate allocations if file size is known in advance.
///
/// This function opens the file at `path`, reads its entire contents into memory
/// allocated from the given [`Allocator`], validates that the bytes are valid UTF-8,
/// and returns a borrowed `&str` pointing to the allocator-backed data.
///
/// This is useful for performance-critical workflows where zero-copy string handling is desired,
/// such as parsing large source files in memory-constrained or throughput-sensitive environments.
///
/// # Parameters
///
/// - `path`: The path to the file to read.
/// - `allocator`: The [`Allocator`] into which the file contents will be allocated.
///
/// # Errors
///
/// Returns [`io::Error`] if any of:
///
/// - The file cannot be read.
/// - The file's contents are not valid UTF-8.
/// - The file's size exceeds the capacity of `allocator`.
///
/// # SAFETY
/// `allocator` must not be dropped after calling this method.
/// See [`Allocator::alloc_bytes_start`] for more details.
unsafe fn read_to_arena_str<'alloc>(
    path: &Path,
    allocator: &'alloc Allocator,
) -> io::Result<&'alloc str> {
    let file = File::open(path)?;

    let bytes = if let Ok(metadata) = file.metadata() {
        // SAFETY: Caller guarantees `allocator` is not dropped after calling this method
        unsafe { read_to_arena_bytes_known_size(file, metadata.len(), allocator) }
    } else {
        // SAFETY: Caller guarantees `allocator` is not dropped after calling this method
        unsafe { read_to_arena_bytes_unknown_size(file, allocator) }
    }?;

    // Convert to `&str`, checking contents is valid UTF-8
    simdutf8::basic::from_utf8(bytes).map_err(|_| {
        io::Error::new(io::ErrorKind::InvalidData, "stream did not contain valid UTF-8")
    })
}

/// Read contents of file directly into arena.
///
/// # SAFETY
/// `allocator` must not be dropped after calling this method.
/// See [`Allocator::alloc_bytes_start`] for more details.
unsafe fn read_to_arena_bytes_known_size(
    file: File,
    size: u64,
    allocator: &Allocator,
) -> io::Result<&[u8]> {
    // Check file is not larger than `usize::MAX` bytes.
    // Note: We don't need to check `size` is not larger than `isize::MAX` bytes here,
    // because `Allocator::alloc_bytes_start` does a size check.
    let Ok(mut size) = usize::try_from(size) else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File is larger than `usize::MAX` bytes",
        ));
    };

    // Allocate space for string in allocator.
    // SAFETY: Caller guarantees `allocator` is not dropped after calling this method.
    let ptr = unsafe { allocator.alloc_bytes_start(size) };

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
    // I (@overlookmotel) have reviewed the code for `Read::read_to_end` and it will never grow the `Vec`,
    // as long as it has sufficient capacity for the reader's contents to start with.
    // If it did, that would be UB as it would free the chunk of memory backing the `Vec`,
    // which it doesn't actually own.
    //
    // Unfortunately `Read::read_to_end`'s docs don't guarantee this behavior. But the code is written
    // specifically to avoid growing the `Vec`, and there was a PR to make sure it doesn't:
    // https://github.com/rust-lang/rust/pull/89165
    // So I think in practice we can rely on this behavior.
    {
        // SAFETY: We've allocated `size` bytes starting at `ptr`.
        // This `Vec` doesn't actually own that memory, but we immediately wrap it in `ManuallyDrop`,
        // so it won't free the memory on drop. As long as the `Vec` doesn't grow, no UB (see above).
        let vec = unsafe { Vec::from_raw_parts(ptr.as_ptr(), 0, size) };
        let mut vec = ManuallyDrop::new(vec);
        let bytes_written = file.take(size as u64).read_to_end(&mut vec)?;

        debug_assert!(vec.capacity() == size);
        debug_assert!(vec.len() == bytes_written);

        // Update `size`, in case file was altered and got smaller since the call to `file.metadata()`,
        // or `file.metadata()` reported inaccurate size
        size = vec.len();
    }

    // SAFETY: `size` bytes were written starting at `ptr`.
    // Those bytes will remain untouched until the `Allocator` is reset, so returning a `&[u8]` with
    // same lifetime as the `&Allocator` borrow is valid.
    let bytes = unsafe { slice::from_raw_parts(ptr.as_ptr(), size) };
    Ok(bytes)
}

/// Fallback for when file size is unknown.
/// Read file contents into a `Vec`, and then copy into arena.
///
/// # SAFETY
/// `allocator` must not be dropped after calling this method.
/// See [`Allocator::alloc_bytes_start`] for more details.
unsafe fn read_to_arena_bytes_unknown_size(
    mut file: File,
    allocator: &Allocator,
) -> io::Result<&[u8]> {
    // Read file into a `Vec`
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;

    // Copy bytes into start of allocator chunk.
    //
    // SAFETY:
    // * `alloc_bytes_start` allocates space for `len` bytes at start of the arena chunk.
    //   That allocation cannot overlap the allocation owned by `bytes` vec.
    // * After `copy_nonoverlapping` call, `len` bytes starting from `dst` are initialized,
    //   so safe to create a byte slice referencing those bytes.
    // * Those bytes will remain untouched until the `Allocator` is reset, so returning a `&[u8]` with
    //   same lifetime as the `&Allocator` borrow is valid.
    // * Caller guarantees `allocator` is not dropped after calling this method.
    let slice = unsafe {
        let src = bytes.as_ptr();
        let len = bytes.len();
        let dst = allocator.alloc_bytes_start(len).as_ptr();
        ptr::copy_nonoverlapping(src, dst, len);
        slice::from_raw_parts(dst, len)
    };

    Ok(slice)
}
