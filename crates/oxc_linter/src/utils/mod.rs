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
mod unicorn;
mod url;
mod vitest;

use std::{
    fs::File,
    io::{self, Read},
    path::Path,
};

use oxc_allocator::Allocator;

pub use self::{
    comment::*, config::*, express::*, jest::*, jsdoc::*, nextjs::*, promise::*, react::*,
    react_perf::*, regex::*, unicorn::*, url::*, vitest::*,
};

/// List of Jest rules that have Vitest equivalents.
const VITEST_COMPATIBLE_JEST_RULES: [&str; 34] = [
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
    "prefer-to-have-length",
    "prefer-todo",
    "require-to-throw-message",
    "require-top-level-describe",
    "valid-describe-callback",
    "valid-expect",
];

// List of Eslint rules that have Typescript equivalents.
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

/// Read the contents of a UTF-8 encoded file directly into a bump allocator, avoiding intermediate allocations.
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
/// # Returns
///
/// On success, returns a `&str` reference into the allocator containing the file's contents.
/// On failure, returns an `io::Error` if the file cannot be read or if the contents are not valid UTF-8.
///
/// # Safety
///
/// - The underlying buffer returned by [`Allocator::alloc_raw_bytes`] is uninitialized.
///   It is fully written to by `read_exact` before being interpreted as a string.
/// - UTF-8 validity is explicitly checked using `simdutf8`, ensuring that
///   `from_utf8_unchecked` is used safely.
///
/// # Panics
///
/// - Panics if the file's reported size is larger than `usize::MAX`.
///
/// [`Allocator`]: oxc_allocator::Allocator
/// [`Allocator::alloc_raw_bytes`]: oxc_allocator::Allocator::alloc_raw_bytes
#[expect(unused)]
pub fn read_into_allocator<'alloc>(
    path: &Path,
    allocator: &'alloc Allocator,
) -> io::Result<&'alloc str> {
    let mut file = File::open(path)?;
    let size = file.metadata().ok().map(|m| usize::try_from(m.len()).unwrap()).filter(|&s| s > 0);

    let buf: &mut [u8] = if let Some(size) = size {
        let buf = allocator.alloc_raw_bytes(size);
        file.read_exact(buf)?;
        buf
    } else {
        // fallback path: read to Vec, then copy into allocator
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;

        if simdutf8::basic::from_utf8(&bytes).is_err() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "stream did not contain valid UTF-8",
            ));
        }

        // SAFETY: already validated as UTF-8
        return Ok(allocator.alloc_str(unsafe { std::str::from_utf8_unchecked(&bytes) }));
    };

    if simdutf8::basic::from_utf8(buf).is_err() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "stream did not contain valid UTF-8",
        ));
    }

    // SAFETY: simdutf8 validated it
    Ok(unsafe { std::str::from_utf8_unchecked(buf) })
}
