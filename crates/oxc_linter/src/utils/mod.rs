use std::{io, path::Path};

use oxc_allocator::Allocator;
use oxc_span::Span;
use oxc_syntax::identifier::{is_identifier_part, is_identifier_start};

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
// When adding a new rule to this list, please ensure that
// the crates/oxc_linter/data/vitest_compatible_jest_rules.json
// file is also updated. The JSON file is used by the oxlint-migrate
// and eslint-plugin-oxlint repos to keep everything synced.
const VITEST_COMPATIBLE_JEST_RULES: [&str; 43] = [
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
    "no-large-snapshots",
    "no-mocks-import",
    "no-restricted-jest-methods",
    "no-restricted-matchers",
    "no-standalone-expect",
    "no-test-prefixes",
    "no-test-return-statement",
    "no-unneeded-async-expect-function",
    "prefer-called-with",
    "prefer-comparison-matcher",
    "prefer-each",
    "prefer-equality-matcher",
    "prefer-expect-resolves",
    "prefer-hooks-in-order",
    "prefer-hooks-on-top",
    "prefer-lowercase-title",
    "prefer-mock-promise-shorthand",
    "prefer-mock-return-shorthand",
    "prefer-spy-on",
    "prefer-strict-equal",
    "prefer-to-be",
    "prefer-to-contain",
    "prefer-to-have-length",
    "prefer-todo",
    "require-hook",
    "require-to-throw-message",
    "require-top-level-describe",
    "valid-describe-callback",
    "valid-expect",
];

/// List of Eslint rules that have TypeScript equivalents.
// When adding a new rule to this list, please ensure oxlint-migrate is also updated.
// See https://github.com/oxc-project/oxlint-migrate/blob/659b461eaf5b2f8a7283822ae84a5e619c86fca3/src/constants.ts#L24
// NOTE: Ensure this list is always alphabetized, otherwise the binary_search won't work.
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

/// Pads replacement text with spaces when needed to preserve token boundaries
/// with neighboring source characters.
pub fn pad_fix_with_token_boundary(source_text: &str, span: Span, replacement: &mut String) {
    if replacement.is_empty() {
        return;
    }

    let source_bytes = source_text.as_bytes();
    let replacement_bytes = replacement.as_bytes();
    let needs_pad_start = span.start > 0
        && is_identifier_part(source_bytes[span.start as usize - 1] as char)
        && is_identifier_part(replacement.chars().next().unwrap());
    let needs_pad_end = (span.end as usize) < source_bytes.len()
        && is_identifier_start(source_bytes[span.end as usize] as char)
        && !replacement_bytes.last().unwrap().is_ascii_whitespace();

    if needs_pad_start {
        replacement.insert(0, ' ');
    }
    if needs_pad_end {
        replacement.push(' ');
    }
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
    let bytes = oxc_allocator::read_file_to_arena(path, allocator)?;

    // Convert to `&str`, checking contents is valid UTF-8
    simdutf8::basic::from_utf8(bytes).map_err(|_| {
        io::Error::new(io::ErrorKind::InvalidData, "stream did not contain valid UTF-8")
    })
}

#[cfg(test)]
mod test {
    use crate::utils::{
        TYPESCRIPT_COMPATIBLE_ESLINT_RULES, VITEST_COMPATIBLE_JEST_RULES, read_to_string,
    };
    use serde_json::from_str;
    use std::path::Path;

    #[test]
    fn test_typescript_rules_list_is_alphabetized() {
        assert!(TYPESCRIPT_COMPATIBLE_ESLINT_RULES.is_sorted());
    }

    #[test]
    fn test_vitest_rules_list_is_alphabetized() {
        assert!(VITEST_COMPATIBLE_JEST_RULES.is_sorted());
    }

    #[test]
    fn test_vitest_rules_list_matches_json() {
        let json_path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("data/vitest_compatible_jest_rules.json");
        let json = read_to_string(&json_path).expect("Failed to read vitest rules JSON file");
        let json_rules: Vec<String> =
            from_str(&json).expect("Failed to parse vitest rules JSON file");
        assert!(json_rules.is_sorted(), "vitest JSON list must be alphabetized");
        let rust_rules: Vec<&str> = VITEST_COMPATIBLE_JEST_RULES.to_vec();
        assert_eq!(
            json_rules.len(),
            rust_rules.len(),
            "Rule counts differ between Rust constant and JSON, please ensure both are updated"
        );
        for (json_rule, rust_rule) in json_rules.iter().zip(rust_rules.iter()) {
            assert_eq!(json_rule, rust_rule, "Mismatch for rule: {json_rule}");
        }
    }
}
