mod config;
mod express;
mod jest;
mod jsdoc;
mod nextjs;
mod promise;
mod react;
mod react_perf;
mod unicorn;
mod url;
mod vitest;

use std::{io, path::Path};

pub use self::{
    config::*, express::*, jest::*, jsdoc::*, nextjs::*, promise::*, react::*, react_perf::*,
    unicorn::*, url::*, vitest::*,
};

/// List of Jest rules that have Vitest equivalents.
const VITEST_COMPATIBLE_JEST_RULES: phf::Set<&'static str> = phf::phf_set! {
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
};

// List of Eslint rules that have Typescript equivalents.
const TYPESCRIPT_COMPATIBLE_ESLINT_RULES: phf::Set<&'static str> = phf::phf_set! {
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
    // "space-infix-ops",
};

/// Check if the Jest rule is adapted to Vitest.
/// Many Vitest rule are essentially ports of Jest plugin rules with minor modifications.
/// For these rules, we use the corresponding jest rules with some adjustments for compatibility.
pub fn is_jest_rule_adapted_to_vitest(rule_name: &str) -> bool {
    VITEST_COMPATIBLE_JEST_RULES.contains(rule_name)
}

/// Check if the Eslint rule is adapted to Typescript.
/// Many Typescript rule are essentially ports of Eslint plugin rules with minor modifications.
/// For these rules, we use the corresponding eslint rules with some adjustments for compatibility.
pub fn is_eslint_rule_adapted_to_typescript(rule_name: &str) -> bool {
    TYPESCRIPT_COMPATIBLE_ESLINT_RULES.contains(rule_name)
}

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
