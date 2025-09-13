use std::process::{ExitCode, Termination};

use napi::{
    Status,
    bindgen_prelude::{FnArgs, Promise, Uint8Array},
    threadsafe_function::ThreadsafeFunction,
};
use napi_derive::napi;

use oxlint::lint as oxlint_lint;

// JS plugins are only supported on 64-bit little-endian platforms at present.
// Note: `raw_transfer_constants` module will not compile on 32-bit systems.
#[cfg(all(target_pointer_width = "64", target_endian = "little"))]
mod generated {
    pub mod raw_transfer_constants;
}

#[cfg(all(target_pointer_width = "64", target_endian = "little"))]
mod external_linter;

/// JS callback to load a JS plugin.
#[napi]
pub type JsLoadPluginCb = ThreadsafeFunction<
    // Arguments
    String, // Absolute path of plugin file
    // Return value
    Promise<String>, // `PluginLoadResult`, serialized to JSON
    // Arguments (repeated)
    String,
    // Error status
    Status,
    // CalleeHandled
    false,
>;

/// JS callback to lint a file.
#[napi]
pub type JsLintFileCb = ThreadsafeFunction<
    // Arguments
    FnArgs<(
        String,             // Absolute path of file to lint
        u32,                // Buffer ID
        Option<Uint8Array>, // Buffer (optional)
        Vec<u32>,           // Array of rule IDs
    )>,
    // Return value
    String, // `Vec<LintFileResult>`, serialized to JSON
    // Arguments (repeated)
    FnArgs<(String, u32, Option<Uint8Array>, Vec<u32>)>,
    // Error status
    Status,
    // CalleeHandled
    false,
>;

/// NAPI entry point.
///
/// JS side passes in two callbacks:
/// 1. `load_plugin`: Load a JS plugin from a file path.
/// 2. `lint_file`: Lint a file.
///
/// Returns `true` if linting succeeded without errors, `false` otherwise.
#[expect(clippy::allow_attributes)]
#[allow(clippy::trailing_empty_array, clippy::unused_async)] // https://github.com/napi-rs/napi-rs/issues/2758
#[napi]
pub async fn lint(load_plugin: JsLoadPluginCb, lint_file: JsLintFileCb) -> bool {
    // JS plugins are only supported on 64-bit little-endian platforms at present
    #[cfg(all(target_pointer_width = "64", target_endian = "little"))]
    let external_linter = Some(external_linter::create_external_linter(load_plugin, lint_file));

    #[cfg(not(all(target_pointer_width = "64", target_endian = "little")))]
    let external_linter = None;

    oxlint_lint(external_linter).report() == ExitCode::SUCCESS
}
