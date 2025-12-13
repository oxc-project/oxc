use std::{
    io::BufWriter,
    process::{ExitCode, Termination},
};

use napi::{
    Status,
    bindgen_prelude::{FnArgs, Promise, Uint8Array},
    threadsafe_function::ThreadsafeFunction,
};
use napi_derive::napi;

use crate::{
    init::{init_miette, init_tracing},
    lint::CliRunner,
    result::CliRunResult,
};

/// JS callback to load a JS plugin.
#[napi]
pub type JsLoadPluginCb = ThreadsafeFunction<
    // Arguments
    FnArgs<(String, Option<String>)>, // Absolute path of plugin file, optional package name
    // Return value
    Promise<String>, // `PluginLoadResult`, serialized to JSON
    // Arguments (repeated)
    FnArgs<(String, Option<String>)>,
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
        Vec<u32>,           // Array of options IDs
        String,             // Settings for the file, as JSON string
        String,             // Globals for the file, as JSON string
    )>,
    // Return value
    Option<String>, // `Vec<LintFileResult>`, serialized to JSON, or `None` if no diagnostics
    // Arguments (repeated)
    FnArgs<(String, u32, Option<Uint8Array>, Vec<u32>, Vec<u32>, String, String)>,
    // Error status
    Status,
    // CalleeHandled
    false,
>;

/// JS callback to setup configs.
#[napi]
pub type JsSetupConfigsCb = ThreadsafeFunction<
    // Arguments
    String, // Options array, as JSON string
    // Return value
    (), // `void`
    // Arguments (repeated)
    String,
    // Error status
    Status,
    // CalleeHandled
    false,
>;

/// NAPI entry point.
///
/// JS side passes in:
/// 1. `args`: Command line arguments (process.argv.slice(2))
/// 2. `load_plugin`: Load a JS plugin from a file path.
/// 3. `setup_configs`: Setup configuration options.
/// 4. `lint_file`: Lint a file.
///
/// Returns `true` if linting succeeded without errors, `false` otherwise.
#[expect(clippy::allow_attributes)]
#[allow(clippy::trailing_empty_array, clippy::unused_async)] // https://github.com/napi-rs/napi-rs/issues/2758
#[napi]
pub async fn lint(
    args: Vec<String>,
    load_plugin: JsLoadPluginCb,
    setup_configs: JsSetupConfigsCb,
    lint_file: JsLintFileCb,
) -> bool {
    lint_impl(args, load_plugin, setup_configs, lint_file).await.report() == ExitCode::SUCCESS
}

/// Run the linter.
async fn lint_impl(
    args: Vec<String>,
    load_plugin: JsLoadPluginCb,
    setup_configs: JsSetupConfigsCb,
    lint_file: JsLintFileCb,
) -> CliRunResult {
    // Convert String args to OsString for compatibility with bpaf
    let args: Vec<std::ffi::OsString> = args.into_iter().map(std::ffi::OsString::from).collect();

    let command = {
        let cmd = crate::cli::lint_command();
        match cmd.run_inner(&*args) {
            Ok(cmd) => cmd,
            Err(e) => {
                e.print_message(100);
                return if e.exit_code() == 0 {
                    CliRunResult::LintSucceeded
                } else {
                    CliRunResult::InvalidOptionConfig
                };
            }
        }
    };

    // If --lsp flag is set, run the language server
    if command.lsp {
        crate::lsp::run_lsp().await;
        return CliRunResult::LintSucceeded;
    }

    init_tracing();
    init_miette();

    command.handle_threads();

    // JS plugins are only supported on 64-bit little-endian platforms at present
    #[cfg(all(target_pointer_width = "64", target_endian = "little"))]
    let external_linter =
        Some(crate::js_plugins::create_external_linter(load_plugin, setup_configs, lint_file));
    #[cfg(not(all(target_pointer_width = "64", target_endian = "little")))]
    let external_linter = {
        let (_, _, _) = (load_plugin, setup_configs, lint_file);
        None
    };

    // stdio is blocked by LineWriter, use a BufWriter to reduce syscalls.
    // See `https://github.com/rust-lang/rust/issues/60673`.
    let mut stdout = BufWriter::new(std::io::stdout());

    CliRunner::new(command, external_linter).run(&mut stdout)
}

#[cfg(all(target_pointer_width = "64", target_endian = "little"))]
pub use crate::js_plugins::parse::{get_buffer_offset, parse_raw_sync};

/// Returns `true` if raw transfer is supported on this platform.
#[napi]
pub fn raw_transfer_supported() -> bool {
    cfg!(all(target_pointer_width = "64", target_endian = "little"))
}
