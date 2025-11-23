use std::{
    alloc::Layout,
    fs::File,
    io::{self, Read},
    mem::ManuallyDrop,
    path::Path,
    slice,
};

use oxc_allocator::Allocator;

mod comment;
mod config;
mod express;
mod jest;
mod jsdoc;
mod nextjs;
mod promise;
mod react;
mod react_perf;
mod regex;
mod typescript;
mod unicorn;
mod url;
mod vitest;
mod vue;

pub use self::{
    comment::*, config::*, express::*, jest::*, jsdoc::*, nextjs::*, promise::*, react::*,
    react_perf::*, regex::*, typescript::*, unicorn::*, url::*, vitest::*, vue::*,
};

/// List of Jest rules that have Vitest equivalents.
// When adding a new rule to this list, please ensure oxlint-migrate is also updated.
// See https://github.com/oxc-project/oxlint-migrate/blob/2c336c67d75adb09a402ae66fb3099f1dedbe516/scripts/constants.ts
const VITEST_COMPATIBLE_JEST_RULES: [&str; 35] = [
    "consistent-test-it",
    "expect-expect",
    "max-expects",
    "max-nested-describe",
    "no-alias-methods",
    "no-commented-out-tests",
    "no-conditional-expect",
    "no-conditional-in-test",
    "no-disabled-tests",
    "no-duplicate-hooks",
    "no-focused-tests",
    "no-hooks",
    "no-identical-title",
    "no-interpolation-in-snapshots",
    "no-restricted-jest-methods",
    "no-restricted-matchers",
    "no-standalone-expect",
    "no-test-prefixes",
    "no-test-return-statement",
    "prefer-comparison-matcher",
    "prefer-each",
    "prefer-equality-matcher",
    "prefer-expect-resolves",
    "prefer-hooks-in-order",
    "prefer-hooks-on-top",
    "prefer-lowercase-title",
    "prefer-mock-promise-shorthand",
    "prefer-strict-equal",
    "prefer-to-be",
    "prefer-to-have-length",
    "prefer-todo",
    "require-to-throw-message",
    "require-top-level-describe",
    "valid-describe-callback",
    "valid-expect",
];

/// List of Eslint rules that have TypeScript equivalents.
// When adding a new rule to this list, please ensure oxlint-migrate is also updated.
// See https://github.com/oxc-project/oxlint-migrate/blob/659b461eaf5b2f8a7283822ae84a5e619c86fca3/src/constants.ts#L24
const TYPESCRIPT_COMPATIBLE_ESLINT_RULES: [&str; 18] = [
    "class-methods-use-this",
    "default-param-last",
    "init-declarations",
    "max-params",
    "no-array-constructor",
    "no-dupe-class-members",
    "no-empty-function",
    "no-invalid-this",
    "no-loop-func",
    "no-loss-of-precision",
    "no-magic-numbers",
    "no-redeclare",
    "no-restricted-imports",
    "no-shadow",
    "no-unused-expressions",
    "no-unused-vars",
    "no-use-before-define",
    "no-useless-constructor",
    // these rules are equivalents, but not supported
    // "block-spacing",
    // "brace-style",
    // "comma-dangle",
    // "comma-spacing",
    // "func-call-spacing",
    // "indent",
    // "key-spacing",
    // "keyword-spacing",
    // "lines-around-comment",
    // "lines-between-class-members",
    // "no-extra-parens",
    // "no-extra-semi",
    // "object-curly-spacing",
    // "padding-line-between-statements",
    // "quotes",
    // "semi",
    // "space-before-blocks",
    // "space-before-function-paren",
    // "space-infix-ops"
];

/// Check if the Jest rule is adapted to Vitest.
/// Many Vitest rule are essentially ports of Jest plugin rules with minor modifications.
/// For these rules, we use the corresponding jest rules with some adjustments for compatibility.
pub fn is_jest_rule_adapted_to_vitest(rule_name: &str) -> bool {
    VITEST_COMPATIBLE_JEST_RULES.binary_search(&rule_name).is_ok()
}

/// Check if the Eslint rule is adapted to Typescript.
/// Many Typescript rule are essentially ports of Eslint plugin rules with minor modifications.
/// For these rules, we use the corresponding eslint rules with some adjustments for compatibility.
pub fn is_eslint_rule_adapted_to_typescript(rule_name: &str) -> bool {
    TYPESCRIPT_COMPATIBLE_ESLINT_RULES.binary_search(&rule_name).is_ok()
}

/// Reads the content of a path and returns it.
/// This function is faster than native `fs:read_to_string`.
///
/// # Errors
/// When the content of the path is not a valid UTF-8 bytes
pub fn read_to_string(path: &Path) -> io::Result<String> {
    // `simdutf8` is faster than `std::str::from_utf8` which `fs::read_to_string` uses internally
    let bytes = std::fs::read(path)?;
    if simdutf8::basic::from_utf8(&bytes).is_err() {
        // Same error as `fs::read_to_string` produces (`io::Error::INVALID_UTF8`)
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "stream did not contain valid UTF-8",
        ));
    }
    // SAFETY: `simdutf8` has ensured it's a valid UTF-8 string
    Ok(unsafe { String::from_utf8_unchecked(bytes) })
}

/// Read the contents of a UTF-8 encoded file directly into arena allocator.
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
/// - The file is larger than `isize::MAX` bytes.
pub fn read_to_arena_str<'alloc>(
    path: &Path,
    allocator: &'alloc Allocator,
) -> io::Result<&'alloc str> {
    let file = File::open(path)?;

    let bytes = if let Ok(metadata) = file.metadata() {
        read_to_arena_bytes_known_size(file, metadata.len(), allocator)
    } else {
        read_to_arena_bytes_unknown_size(file, allocator)
    }?;

    // Convert to `&str`, checking contents is valid UTF-8
    simdutf8::basic::from_utf8(bytes).map_err(|_| {
        io::Error::new(io::ErrorKind::InvalidData, "stream did not contain valid UTF-8")
    })
}

/// Read contents of file directly into arena.
fn read_to_arena_bytes_known_size(
    file: File,
    size: u64,
    allocator: &Allocator,
) -> io::Result<&[u8]> {
    // Check file is not larger than `isize::MAX` bytes (the max size of an allocation)
    let Ok(size) = isize::try_from(size) else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "File is larger than `isize::MAX` bytes",
        ));
    };
    #[expect(clippy::cast_sign_loss)]
    let mut size = size as usize;

    // Allocate space for string in allocator.
    // SAFETY: We checked above that `size <= isize::MAX`. `&str` has no alignment requirements.
    let layout = unsafe { Layout::from_size_align_unchecked(size, 1) };
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

    // SAFETY: `size` bytes were written starting at `ptr`
    let bytes = unsafe { slice::from_raw_parts(ptr.as_ptr(), size) };
    Ok(bytes)
}

/// Fallback for when file size is unknown.
/// Read file contents into a `Vec`, and then copy into arena.
fn read_to_arena_bytes_unknown_size(mut file: File, allocator: &Allocator) -> io::Result<&[u8]> {
    // Read file into a `Vec`
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;

    // Allocate bytes into arena
    Ok(allocator.alloc_slice_copy(&bytes))
}
