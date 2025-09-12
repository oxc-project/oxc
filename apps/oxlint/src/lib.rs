// Ignore dead code warnings when building `tasks/website`, which disables `napi` Cargo feature
#![cfg_attr(not(feature = "napi"), allow(dead_code))]

mod command;
mod lint;
mod output_formatter;
mod result;
mod walk;

#[cfg(test)]
mod tester;

/// Re-exported CLI-related items for use in `tasks/website`.
pub mod cli {
    pub use super::{command::*, lint::LintRunner, result::CliRunResult};
}

// Only include code to run linter when the `napi` feature is enabled.
// Without this, `tasks/website` will not compile on Linux or Windows.
// `tasks/website` depends on `oxlint` as a normal library, which causes linker errors if NAPI is enabled.
#[cfg(feature = "napi")]
mod run;
#[cfg(feature = "napi")]
pub use run::*;

// JS plugins are only supported on 64-bit little-endian platforms at present.
// Note: `raw_transfer_constants` module will not compile on 32-bit systems.
#[cfg(all(feature = "napi", target_pointer_width = "64", target_endian = "little"))]
mod generated {
    pub mod raw_transfer_constants;
}

#[cfg(all(feature = "napi", target_pointer_width = "64", target_endian = "little"))]
mod js_plugins;

#[cfg(all(feature = "allocator", not(miri), not(target_family = "wasm")))]
#[global_allocator]
static GLOBAL: mimalloc_safe::MiMalloc = mimalloc_safe::MiMalloc;
