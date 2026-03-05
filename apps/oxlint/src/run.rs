use std::{
    ffi::OsString,
    fs::OpenOptions,
    io::BufWriter,
    io::Write,
    path::PathBuf,
    process::{ExitCode, Termination},
    time::{SystemTime, UNIX_EPOCH},
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

const OXLINT_RUNTIME_DEBUG_ENV: &str = "OXLINT_RUNTIME_DEBUG";

fn runtime_debug_path() -> Option<PathBuf> {
    let path_or_flag = std::env::var_os(OXLINT_RUNTIME_DEBUG_ENV)?;
    let value = path_or_flag.to_string_lossy();

    if value.is_empty() || value == "1" || value.eq_ignore_ascii_case("true") {
        return Some(std::env::temp_dir().join("oxlint-runtime.log"));
    }

    Some(PathBuf::from(path_or_flag))
}

fn sanitize_for_log(value: &str) -> String {
    value.replace('\n', "\\n").replace('\r', "\\r")
}

fn runtime_debug_log(
    event: &str,
    args: &[OsString],
    lsp_mode: Option<bool>,
    details: Option<&str>,
) {
    let Some(path) = runtime_debug_path() else {
        return;
    };

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs_f64())
        .unwrap_or_default();
    let cwd = std::env::current_dir()
        .map(|dir| dir.display().to_string())
        .unwrap_or_else(|_| "<unknown>".to_string());
    let args_rendered = args
        .iter()
        .map(|arg| sanitize_for_log(&arg.to_string_lossy()))
        .collect::<Vec<_>>()
        .join(", ");
    let env_keys = [
        "VSCODE_PID",
        "VSCODE_IPC_HOOK_CLI",
        "VSCODE_IPC_HOOK",
        "ELECTRON_RUN_AS_NODE",
        "TERM_PROGRAM",
        "npm_lifecycle_event",
        "npm_execpath",
    ];
    let env_rendered = env_keys
        .iter()
        .filter_map(|key| {
            std::env::var_os(key)
                .map(|value| format!("{key}={}", sanitize_for_log(&value.to_string_lossy())))
        })
        .collect::<Vec<_>>()
        .join(", ");
    let mode = lsp_mode
        .map_or("unknown".to_string(), |is_lsp| if is_lsp { "lsp" } else { "cli" }.to_string());
    let details = details.map(sanitize_for_log).unwrap_or_default();

    let report = format!(
        "\n=== oxlint runtime ===\n\
         pid: {}\n\
         timestamp_unix_s: {:.3}\n\
         event: {}\n\
         mode: {}\n\
         cwd: {}\n\
         args: [{}]\n\
         env: {}\n\
         details: {}\n\
         === end oxlint runtime ===\n",
        std::process::id(),
        timestamp,
        event,
        mode,
        sanitize_for_log(&cwd),
        args_rendered,
        env_rendered,
        details
    );

    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = file.write_all(report.as_bytes());
        let _ = file.flush();
    }
}

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
        Option<String>,     // Workspace URI (`None` in CLI mode, `Some` in LSP mode)
    )>,
    // Return value
    Option<String>, // `Vec<LintFileResult>`, serialized to JSON, or `None` if no diagnostics
    // Arguments (repeated)
    FnArgs<(String, u32, Option<Uint8Array>, Vec<u32>, Vec<u32>, String, String, Option<String>)>,
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
    runtime_debug_log("lint_impl_enter", &args, None, None);

    let command = {
        let cmd = crate::cli::lint_command();
        match cmd.run_inner(&*args) {
            Ok(cmd) => cmd,
            Err(e) => {
                runtime_debug_log(
                    "lint_impl_arg_parse_error",
                    &args,
                    None,
                    Some("failed to parse args"),
                );
                e.print_message(100);
                return if e.exit_code() == 0 {
                    CliRunResult::LintSucceeded
                } else {
                    CliRunResult::InvalidOptionConfig
                };
            }
        }
    };
    runtime_debug_log("lint_impl_parsed_command", &args, Some(command.lsp), None);

    // Both LSP and CLI use `tracing` for logging
    init_tracing();

    // JS plugins are only supported on 64-bit little-endian platforms at present
    #[cfg(all(target_pointer_width = "64", target_endian = "little"))]
    let (external_linter, js_config_loader) = {
        let js_config_loader = Some(crate::js_config::create_js_config_loader(load_js_configs));
        let external_linter = Some(crate::js_plugins::create_external_linter(
            load_plugin,
            setup_rule_configs,
            lint_file,
            create_workspace,
            destroy_workspace,
        ));
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
        runtime_debug_log("lint_impl_enter_lsp", &args, Some(true), None);
        crate::lsp::run_lsp(external_linter, js_config_loader).await;
        runtime_debug_log("lint_impl_exit_lsp", &args, Some(true), None);
        return CliRunResult::LintSucceeded;
    }

    init_miette();

    command.handle_threads();

    // stdio is blocked by LineWriter, use a BufWriter to reduce syscalls.
    // See `https://github.com/rust-lang/rust/issues/60673`.
    let mut stdout = BufWriter::new(std::io::stdout());

    let mut cli_runner = CliRunner::new(command, external_linter);
    #[cfg(feature = "napi")]
    {
        cli_runner = cli_runner.with_config_loader(js_config_loader);
    }

    runtime_debug_log("lint_impl_enter_cli_runner", &args, Some(false), None);
    let result = cli_runner.run(&mut stdout);
    let details = format!("result={result:?}");
    runtime_debug_log("lint_impl_exit_cli_runner", &args, Some(false), Some(&details));
    result
}

#[cfg(all(target_pointer_width = "64", target_endian = "little"))]
pub use crate::js_plugins::parse::{get_buffer_offset, parse_raw_sync};

/// Returns `true` if raw transfer is supported on this platform.
#[napi]
pub fn raw_transfer_supported() -> bool {
    cfg!(all(target_pointer_width = "64", target_endian = "little"))
}
