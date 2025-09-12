use std::process::{ExitCode, Termination};

use napi::{
    Status,
    bindgen_prelude::{FnArgs, Promise, Uint8Array},
    threadsafe_function::ThreadsafeFunction,
};
use napi_derive::napi;

use oxlint::lint as oxlint_lint;

mod generated {
    pub mod raw_transfer_constants;
}

mod external_linter;
use external_linter::create_external_linter;

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

#[expect(clippy::allow_attributes)]
#[allow(clippy::trailing_empty_array, clippy::unused_async)] // https://github.com/napi-rs/napi-rs/issues/2758
#[napi]
pub async fn lint(load_plugin: JsLoadPluginCb, lint_file: JsLintFileCb) -> bool {
    let external_linter = create_external_linter(load_plugin, lint_file);
    oxlint_lint(Some(external_linter)).report() == ExitCode::SUCCESS
}
