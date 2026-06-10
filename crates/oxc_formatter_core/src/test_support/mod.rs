//! Fixture-test infrastructure shared across formatter crates.
//!
//! Gated behind the `test_harness` Cargo feature. Split into:
//!
//! - `harness`: runtime that walks fixtures, drives format passes, and assembles snapshots.
//! - `codegen`: build-script helper that emits `#[test]` functions for each fixture.

mod codegen;
mod harness;

pub use codegen::{GenerateConfig, generate_tests};
pub use harness::{
    FixtureFormatter, FixtureSnapshot, OptionSet, build_fixture_snapshot, format_options_display,
    resolve_options,
};
