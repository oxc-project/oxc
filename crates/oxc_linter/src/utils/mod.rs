mod config;
mod express;
mod jest;
mod jsdoc;
mod nextjs;
mod promise;
mod react;
mod react_perf;
mod tree_shaking;
mod unicorn;
mod vitest;

use std::{io, path::Path};

pub use self::{
    config::*, express::*, jest::*, jsdoc::*, nextjs::*, promise::*, react::*, react_perf::*,
    tree_shaking::*, unicorn::*, vitest::*,
};

/// List of Jest rules that have Vitest equivalents.
const VITEST_COMPATIBLE_JEST_RULES: phf::Set<&'static str> = phf::phf_set! {
    "consistent-test-it",
    "expect-expect",
    "no-alias-methods",
    "no-conditional-expect",
    "no-conditional-in-test",
    "no-commented-out-tests",
    "no-disabled-tests",
    "no-focused-tests",
    "no-identical-title",
    "no-restricted-jest-methods",
    "no-test-prefixes",
    "prefer-hooks-in-order",
    "valid-describe-callback",
    "valid-expect",
};

/// Check if the Jest rule is adapted to Vitest.
/// Many Vitest rule are essentially ports of Jest plugin rules with minor modifications.
/// For these rules, we use the corresponding jest rules with some adjustments for compatibility.
pub fn is_jest_rule_adapted_to_vitest(rule_name: &str) -> bool {
    VITEST_COMPATIBLE_JEST_RULES.contains(rule_name)
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
