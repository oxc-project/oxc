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

use crate::{lint::LintRunner, result::CliRunResult};

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
    lint_impl(load_plugin, lint_file).report() == ExitCode::SUCCESS
}

/// Run the linter.
fn lint_impl(load_plugin: JsLoadPluginCb, lint_file: JsLintFileCb) -> CliRunResult {
    init_tracing();
    init_miette();

    let mut args = std::env::args_os();
    // If first arg is `node`, also skip script path (`node script.js ...`).
    // Otherwise, just skip first arg (`oxlint ...`).
    if args.next().is_some_and(|arg| arg == "node") {
        args.next();
    }
    let args = args.collect::<Vec<_>>();

    let cmd = crate::cli::lint_command();
    let command = match cmd.run_inner(&*args) {
        Ok(cmd) => cmd,
        Err(e) => {
            e.print_message(100);
            return if e.exit_code() == 0 {
                CliRunResult::LintSucceeded
            } else {
                CliRunResult::InvalidOptionConfig
            };
        }
    };

    command.handle_threads();

    let external_linter = if command.js_plugins {
        // JS plugins are only supported on 64-bit little-endian platforms at present
        #[cfg(all(target_pointer_width = "64", target_endian = "little"))]
        {
            Some(super::js_plugins::create_external_linter(load_plugin, lint_file))
        }

        #[cfg(not(all(target_pointer_width = "64", target_endian = "little")))]
        #[expect(clippy::print_stderr)]
        {
            eprintln!(
                "ERROR: JS plugins are only supported on 64-bit little-endian platforms at present"
            );
            return CliRunResult::InvalidOptionConfig;
        }
    } else {
        None
    };

    // stdio is blocked by LineWriter, use a BufWriter to reduce syscalls.
    // See `https://github.com/rust-lang/rust/issues/60673`.
    let mut stdout = BufWriter::new(std::io::stdout());

    LintRunner::new(command, external_linter).run(&mut stdout)
}

/// Initialize the data which relies on `is_atty` system calls so they don't block subsequent threads.
fn init_miette() {
    miette::set_hook(Box::new(|_| Box::new(miette::MietteHandlerOpts::new().build()))).unwrap();
}

/// To debug `oxc_resolver`:
/// `OXC_LOG=oxc_resolver oxlint --import-plugin`
fn init_tracing() {
    use tracing_subscriber::{filter::Targets, prelude::*};

    // Usage without the `regex` feature.
    // <https://github.com/tokio-rs/tracing/issues/1436#issuecomment-918528013>
    tracing_subscriber::registry()
        .with(std::env::var("OXC_LOG").map_or_else(
            |_| Targets::new(),
            |env_var| {
                use std::str::FromStr;
                Targets::from_str(&env_var).unwrap()
            },
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();
}
