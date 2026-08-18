//! Test infrastructure shared across formatter crates.
//!
//! - `harness`: runtime that walks fixtures, drives format passes, and assembles snapshots.
//! - `codegen`: build-script helper that emits `#[test]` functions for each fixture.
//!
//! Consumer crates use `codegen` from `build.rs` (via `[build-dependencies]`) and `harness`
//! from `tests/fixtures/mod.rs` (via `[dev-dependencies]`). This crate depends only on
//! `oxc_formatter_core`, never on language crates, so both directions stay cycle-free.

mod codegen;
#[cfg(feature = "conformance")]
pub mod conformance;
mod harness;
mod suite;

pub use codegen::{GenerateConfig, generate_tests};
pub use harness::{
    FixtureFormatter, FixtureSnapshot, OptionSet, apply_core_options, build_fixture_snapshot,
    format_options_display, resolve_options,
};
pub use suite::{ensure_prettier_suite, prettier_suite_root};
