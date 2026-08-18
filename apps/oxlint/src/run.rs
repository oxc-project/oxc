use std::{
    io::BufWriter,
    process::{ExitCode, Termination},
    sync::{
        Arc, LazyLock, Mutex,
        atomic::{AtomicBool, AtomicU32, Ordering},
    },
};

use napi::{
    Status,
    bindgen_prelude::{FnArgs, FromNapiValue, Object, Promise, Uint8Array},
    threadsafe_function::ThreadsafeFunction,
};
use napi_derive::napi;
use rustc_hash::FxHashMap;

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

/// JS callback to drop a cached raw-transfer buffer.
#[napi]
pub type JsForgetBufferCb = ThreadsafeFunction<
    // Arguments
    u32, // Buffer ID
    // Return value
    (),
    // Arguments (repeated)
    u32,
    // Error status
    Status,
    // CalleeHandled
    false,
>;

/// JS callback to start JS plugin worker isolates.
///
/// Called at most once per `lint()`, with `K`, when the first JS plugin is loaded.
/// Resolves when every worker has registered its callbacks.
#[napi]
pub type JsStartWorkersCb = ThreadsafeFunction<
    // Arguments
    u32, // Number of workers to start (`K`)
    // Return value
    Promise<()>,
    // Arguments (repeated)
    u32,
    // Error status
    Status,
    // CalleeHandled
    false,
>;

/// Callbacks registered by a JS plugin worker isolate.
///
/// Populated by [`register_worker`]. Later used to route `ExternalLinter` through workers.
pub struct RegisteredWorker {
    pub load_plugin: JsLoadPluginCb,
    pub lint_file: JsLintFileCb,
    pub forget_buffer: JsForgetBufferCb,
    pub setup_rule_configs: JsSetupRuleConfigsCb,
    pub create_workspace: JsCreateWorkspaceCb,
    pub destroy_workspace: JsDestroyWorkspaceCb,
}

/// Process-wide JS worker callbacks, keyed by worker `id`.
pub static REGISTERED_WORKERS: LazyLock<Mutex<FxHashMap<u32, RegisteredWorker>>> =
    LazyLock::new(|| Mutex::new(FxHashMap::default()));

/// Liveness flag per JS plugin worker isolate, keyed by worker `id`.
///
/// A flag is cleared by [`notify_worker_died`] and read by the `lintFile` wrapper, which must not
/// block forever waiting on a worker that will never run its queued callback.
static WORKER_IS_ALIVE: LazyLock<Mutex<FxHashMap<u32, Arc<AtomicBool>>>> =
    LazyLock::new(|| Mutex::new(FxHashMap::default()));

/// Liveness flag for worker `id`, creating it (alive) if this is the first mention of that worker.
///
/// Both this and [`notify_worker_died`] go through the same map, so it doesn't matter whether the
/// worker dies before or after the `ExternalLinter` picked up its flag.
pub fn worker_liveness(id: u32) -> Arc<AtomicBool> {
    let mut flags = WORKER_IS_ALIVE.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    Arc::clone(flags.entry(id).or_insert_with(|| Arc::new(AtomicBool::new(true))))
}

/// Record that a JS plugin worker isolate has died.
///
/// Called from `cli.ts` when a worker emits `error` or `exit` after it became ready. Any queued
/// `lintFile` call on that worker will never run, so waiting threads have to be released with an
/// error rather than blocking forever.
///
/// Not called for an intentional `terminate()` during shutdown.
#[napi]
pub fn notify_worker_died(id: u32) {
    worker_liveness(id).store(false, Ordering::SeqCst);
}

/// Process-wide JS plugin worker count `K`. `0` means unset (no `id >= K` check).
static JS_PLUGIN_WORKER_COUNT: AtomicU32 = AtomicU32::new(0);

/// Set the number of JS plugin worker isolates (`K`).
///
/// Call before workers register. `register_worker` then rejects `id >= k`.
/// Pass `0` to clear the bound.
#[napi]
pub fn set_js_plugin_worker_count(k: u32) {
    // `lint()` can be invoked more than once in the same Node process. Drop the previous
    // generation's registrations and liveness flags so a dead worker from run 1 cannot mark
    // run 2's worker with the same id dead-on-arrival, and so leftover TSFNs do not point at
    // terminated isolates.
    REGISTERED_WORKERS.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clear();
    WORKER_IS_ALIVE.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clear();
    JS_PLUGIN_WORKER_COUNT.store(k, Ordering::SeqCst);
}

/// Register JS plugin callbacks for a worker isolate.
///
/// Called from `worker.ts` after the worker starts. Callbacks are stored
/// process-wide, keyed by `id`.
///
/// # Errors
///
/// Returns an error if any required field is missing from `options`,
/// if `id` is already registered, or if `K` is set and `id >= K`.
#[napi]
pub fn register_worker(
    #[napi(
        ts_arg_type = "{ id: number, loadPlugin: JsLoadPluginCb, lintFile: JsLintFileCb, forgetBuffer: JsForgetBufferCb, setupRuleConfigs: JsSetupRuleConfigsCb, createWorkspace: JsCreateWorkspaceCb, destroyWorkspace: JsDestroyWorkspaceCb }"
    )]
    options: Object,
) -> napi::Result<()> {
    let id = require_field::<u32>(&options, "id")?;
    let k = JS_PLUGIN_WORKER_COUNT.load(Ordering::SeqCst);
    if k > 0 && id >= k {
        return Err(napi::Error::from_reason(format!(
            "registerWorker: worker id {id} is >= K ({k})"
        )));
    }
    let load_plugin = require_field::<JsLoadPluginCb>(&options, "loadPlugin")?;
    let lint_file = require_field::<JsLintFileCb>(&options, "lintFile")?;
    let forget_buffer = require_field::<JsForgetBufferCb>(&options, "forgetBuffer")?;
    let setup_rule_configs = require_field::<JsSetupRuleConfigsCb>(&options, "setupRuleConfigs")?;
    let create_workspace = require_field::<JsCreateWorkspaceCb>(&options, "createWorkspace")?;
    let destroy_workspace = require_field::<JsDestroyWorkspaceCb>(&options, "destroyWorkspace")?;

    let mut workers = REGISTERED_WORKERS.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    if workers.contains_key(&id) {
        return Err(napi::Error::from_reason(format!("registerWorker: duplicate worker id {id}")));
    }
    workers.insert(
        id,
        RegisteredWorker {
            load_plugin,
            lint_file,
            forget_buffer,
            setup_rule_configs,
            create_workspace,
            destroy_workspace,
        },
    );
    Ok(())
}

fn require_field<T: FromNapiValue>(options: &Object, field: &'static str) -> napi::Result<T> {
    options
        .get(field)?
        .ok_or_else(|| napi::Error::from_reason(format!("registerWorker: missing `{field}`")))
}

/// NAPI entry point.
///
/// JS side passes in:
/// 1. `args`: Command line arguments (process.argv.slice(2))
/// 2. `load_plugin`: Load a JS plugin from a file path.
/// 3. `setup_rule_configs`: Setup configuration options.
/// 4. `lint_file`: Lint a file.
/// 5. `forget_buffer`: Drop a cached raw-transfer buffer.
/// 6. `create_workspace`: Create a workspace.
/// 7. `destroy_workspace`: Destroy a workspace.
/// 8. `load_js_configs`: Load JavaScript config files.
/// 9. `start_js_workers`: Start `K` JS plugin worker isolates when a JS plugin is first loaded.
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
    forget_buffer: JsForgetBufferCb,
    create_workspace: JsCreateWorkspaceCb,
    destroy_workspace: JsDestroyWorkspaceCb,
    load_js_configs: JsLoadJsConfigsCb,
    start_js_workers: JsStartWorkersCb,
) -> bool {
    lint_impl(
        args,
        load_plugin,
        setup_rule_configs,
        lint_file,
        forget_buffer,
        create_workspace,
        destroy_workspace,
        load_js_configs,
        start_js_workers,
    )
    .await
    .report()
        == ExitCode::SUCCESS
}

/// Run the linter.
#[expect(clippy::too_many_arguments)]
async fn lint_impl(
    args: Vec<String>,
    load_plugin: JsLoadPluginCb,
    setup_rule_configs: JsSetupRuleConfigsCb,
    lint_file: JsLintFileCb,
    forget_buffer: JsForgetBufferCb,
    create_workspace: JsCreateWorkspaceCb,
    destroy_workspace: JsDestroyWorkspaceCb,
    load_js_configs: JsLoadJsConfigsCb,
    start_js_workers: JsStartWorkersCb,
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

    // Both LSP and CLI use `tracing` for logging
    init_tracing();

    // Lock the Rayon thread count before anything reads it. `K` is derived from it, and both the
    // JS workers and the allocator pools that feed them have to agree on that number.
    // The language server relies on this too, so it happens before the `--lsp` branch below.
    command.handle_threads();

    // JS plugins are only supported on 64-bit little-endian platforms at present.
    // Workers are not started here: `load_plugin` starts them only if a JS plugin is configured.
    #[cfg(all(target_pointer_width = "64", target_endian = "little"))]
    let (external_linter, js_config_loader) = {
        let js_config_loader = Some(crate::js_config::create_js_config_loader(load_js_configs));
        let external_linter = crate::js_plugins::create_external_linter(
            load_plugin,
            setup_rule_configs,
            lint_file,
            forget_buffer,
            create_workspace,
            destroy_workspace,
            start_js_workers,
        );
        (Some(external_linter), js_config_loader)
    };
    #[cfg(not(all(target_pointer_width = "64", target_endian = "little")))]
    let (external_linter, js_config_loader) = {
        let (_, _, _, _, _, _, _, _) = (
            load_plugin,
            setup_rule_configs,
            lint_file,
            forget_buffer,
            create_workspace,
            destroy_workspace,
            load_js_configs,
            start_js_workers,
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

#[cfg(all(target_pointer_width = "64", target_endian = "little"))]
pub use crate::js_plugins::parse::{get_buffer_offset, parse_raw_sync};

/// Returns `true` if raw transfer is supported on this platform.
#[napi]
pub fn raw_transfer_supported() -> bool {
    cfg!(all(target_pointer_width = "64", target_endian = "little"))
}
