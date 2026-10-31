use std::{
    io::BufWriter,
    num::NonZeroUsize,
    process::{ExitCode, Termination},
};

use napi::{
    Status,
    bindgen_prelude::{FnArgs, Promise, Uint8Array},
    threadsafe_function::ThreadsafeFunction,
};
use napi_derive::napi;

use crate::{init::init_tracing, lint::CliRunner, result::CliRunResult};

/// JS callback to load a JS plugin.
#[napi]
pub type JsLoadPluginCb = ThreadsafeFunction<
    // Arguments
    FnArgs<(
        // File URL to load plugin from
        String,
        // Plugin name (either alias or package name).
        // If is package name, it is pre-normalized.
        Option<String>,
        // `true` if plugin name is an alias (takes priority over name that plugin defines itself)
        bool,
        // Workspace URI (e.g. `file:///path/to/workspace`).
        // `None` in CLI mode (single workspace), `Some` in LSP mode.
        Option<String>,
    )>,
    // Return value
    Promise<String>, // `PluginLoadResult`, serialized to JSON
    // Arguments (repeated)
    FnArgs<(String, Option<String>, bool, Option<String>)>,
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
        bool,               // Whether to collect per-rule timing information
        Option<String>,     // Workspace URI (`None` in CLI mode, `Some` in LSP mode)
    )>,
    // Return value
    Option<String>, // Success or failure payload serialized to JSON, or `None` if no output
    // Arguments (repeated)
    FnArgs<(
        String,
        u32,
        Option<Uint8Array>,
        Vec<u32>,
        Vec<u32>,
        String,
        String,
        bool,
        Option<String>,
    )>,
    // Error status
    Status,
    // CalleeHandled
    false,
>;

/// JS callback to setup configs.
#[napi]
pub type JsSetupRuleConfigsCb = ThreadsafeFunction<
    // Arguments
    String, // Options array, as JSON string
    // Return value
    Option<String>, // `None` for success, or `Some` containing error message
    // Arguments (repeated)
    String,
    // Error status
    Status,
    // CalleeHandled
    false,
>;

/// JS callback to create a workspace.
#[napi]
pub type JsCreateWorkspaceCb = ThreadsafeFunction<
    // Arguments
    String, // Workspace URI
    // Return value
    Promise<()>,
    // Arguments (repeated)
    String,
    // Error status
    Status,
    // CalleeHandled
    false,
>;

/// JS callback to destroy a workspace.
#[napi]
pub type JsDestroyWorkspaceCb = ThreadsafeFunction<
    // Arguments
    String, // Workspace URI
    // Return value
    (),
    // Arguments (repeated)
    String,
    // Error status
    Status,
    // CalleeHandled
    false,
>;

/// JS callback to load JavaScript config files.
#[napi]
pub type JsLoadJsConfigsCb = ThreadsafeFunction<
    // Arguments: Vec of absolute paths to JavaScript/TypeScript config files
    Vec<String>,
    // Return value: JSON string containing success/failure result
    Promise<String>,
    // Arguments (repeated)
    Vec<String>,
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
/// 3. `setup_rule_configs`: Setup configuration options.
/// 4. `lint_file`: Lint a file.
/// 5. `create_workspace`: Create a workspace.
/// 6. `destroy_workspace`: Destroy a workspace.
/// 7. `load_js_configs`: Load JavaScript config files.
///
/// Returns `true` if linting succeeded without errors, `false` otherwise.
#[expect(clippy::allow_attributes)]
#[allow(clippy::trailing_empty_array, clippy::unused_async)] // https://github.com/napi-rs/napi-rs/issues/2758
#[napi]
pub async fn lint(
    args: Vec<String>,
    load_plugin: JsLoadPluginCb,
    setup_rule_configs: JsSetupRuleConfigsCb,
    lint_file: JsLintFileCb,
    create_workspace: JsCreateWorkspaceCb,
    destroy_workspace: JsDestroyWorkspaceCb,
    load_js_configs: JsLoadJsConfigsCb,
) -> bool {
    lint_impl(
        args,
        load_plugin,
        setup_rule_configs,
        lint_file,
        create_workspace,
        destroy_workspace,
        load_js_configs,
    )
    .await
    .report()
        == ExitCode::SUCCESS
}

/// Run the linter.
async fn lint_impl(
    args: Vec<String>,
    load_plugin: JsLoadPluginCb,
    setup_rule_configs: JsSetupRuleConfigsCb,
    lint_file: JsLintFileCb,
    create_workspace: JsCreateWorkspaceCb,
    destroy_workspace: JsDestroyWorkspaceCb,
    load_js_configs: JsLoadJsConfigsCb,
) -> CliRunResult {
    // Convert String args to OsString for compatibility with bpaf
    let args: Vec<std::ffi::OsString> = args.into_iter().map(std::ffi::OsString::from).collect();

    // Take JS plugin workers registered by the JS side (`--js-threads` option) before anything else,
    // so the registry is drained on every path, including early returns below
    #[cfg(all(target_pointer_width = "64", target_endian = "little"))]
    let js_plugin_workers = crate::js_plugins::take_registered_js_plugin_workers();

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

    // Both LSP and CLI use `tracing` for logging
    init_tracing();

    // Initialize thread pool before setting up JS threads, as JS thread count is capped at thread count.
    // LSP mode doesn't use this thread pool.
    if !command.lsp {
        command.handle_threads();
    }

    // JS plugins are only supported on 64-bit little-endian platforms at present
    #[cfg(all(target_pointer_width = "64", target_endian = "little"))]
    let (external_linter, js_config_loader) = {
        use crate::js_plugins::{JsThreadCallbacks, create_external_linter};

        let js_config_loader = Some(crate::js_config::create_js_config_loader(load_js_configs));

        // JS thread 0 is the main JS thread.
        // Additional JS threads are JS plugin workers spawned by JS side when `--js-threads` option is used.
        // LSP mode only uses the main JS thread.
        let mut js_threads = vec![JsThreadCallbacks {
            load_plugin,
            setup_rule_configs,
            lint_file,
            create_workspace,
            destroy_workspace,
        }];
        if !command.lsp {
            js_threads.extend(js_plugin_workers);

            // Each JS thread owns the fixed-size allocator buffers with `buffer_id % js_thread_count == index`,
            // and there is 1 buffer per thread. So any JS threads beyond the thread count would never receive
            // any files to lint. Cap JS thread count at thread count, and drop callbacks for excess JS threads.
            let requested_js_threads = command.misc_options.js_threads.map_or(1, NonZeroUsize::get);
            let thread_count = rayon::current_num_threads();
            let max_js_threads = requested_js_threads.min(thread_count);
            if requested_js_threads > thread_count {
                #[expect(clippy::print_stderr)]
                {
                    eprintln!(
                        "Warning: `--js-threads {requested_js_threads}` exceeds thread count ({thread_count}). Using {thread_count} JS threads."
                    );
                }
            }
            js_threads.truncate(max_js_threads);

            // JS side spawns workers based on its own parsing of `--js-threads`.
            // This should always agree with Rust-side parsing, but warn if it doesn't.
            if js_threads.len() != max_js_threads {
                #[expect(clippy::print_stderr)]
                {
                    eprintln!(
                        "Warning: Expected {max_js_threads} JS threads, but only {} were started.",
                        js_threads.len()
                    );
                }
            }
        }

        let external_linter = Some(create_external_linter(js_threads));
        (external_linter, js_config_loader)
    };
    #[cfg(not(all(target_pointer_width = "64", target_endian = "little")))]
    let (external_linter, js_config_loader) = {
        let (_, _, _, _, _, _) = (
            load_plugin,
            setup_rule_configs,
            lint_file,
            create_workspace,
            destroy_workspace,
            load_js_configs,
        );
        (None, None)
    };

    // If --lsp flag is set, run the language server
    if command.lsp {
        crate::lsp::run_lsp(external_linter, js_config_loader).await;
        return CliRunResult::LintSucceeded;
    }

    // stdio is blocked by LineWriter, use a BufWriter to reduce syscalls.
    // See `https://github.com/rust-lang/rust/issues/60673`.
    let mut stdout = BufWriter::new(std::io::stdout());

    let mut cli_runner = CliRunner::new(command, external_linter);
    #[cfg(feature = "napi")]
    {
        cli_runner = cli_runner.with_config_loader(js_config_loader);
    }

    cli_runner.run(&mut stdout)
}

/// Register a JS plugin worker (a Node.js worker thread which runs JS plugins).
///
/// Called from JS plugin workers spawned by JS side when `--js-threads` option is used,
/// before main thread calls `lint`. Each worker passes in its own callbacks.
/// `lint` broadcasts plugin loading and config setup to all JS threads, and distributes files between them.
#[napi]
pub fn register_js_plugin_worker(
    load_plugin: JsLoadPluginCb,
    setup_rule_configs: JsSetupRuleConfigsCb,
    lint_file: JsLintFileCb,
    create_workspace: JsCreateWorkspaceCb,
    destroy_workspace: JsDestroyWorkspaceCb,
) {
    #[cfg(all(target_pointer_width = "64", target_endian = "little"))]
    crate::js_plugins::register_js_plugin_worker(crate::js_plugins::JsThreadCallbacks {
        load_plugin,
        setup_rule_configs,
        lint_file,
        create_workspace,
        destroy_workspace,
    });
    #[cfg(not(all(target_pointer_width = "64", target_endian = "little")))]
    let (_, _, _, _, _) =
        (load_plugin, setup_rule_configs, lint_file, create_workspace, destroy_workspace);
}

#[cfg(all(target_pointer_width = "64", target_endian = "little"))]
pub use crate::js_plugins::parse::{get_buffer_offset, parse_raw_sync};

/// Returns `true` if raw transfer is supported on this platform.
#[napi]
pub fn raw_transfer_supported() -> bool {
    cfg!(all(target_pointer_width = "64", target_endian = "little"))
}
