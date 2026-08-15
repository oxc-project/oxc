use std::{
    sync::{
        Arc, Condvar, LazyLock, Mutex, OnceLock,
        atomic::{AtomicU32, Ordering},
        mpsc::{Receiver, RecvTimeoutError, channel},
    },
    time::Duration,
};

use napi::{
    Status,
    bindgen_prelude::{FnArgs, Uint8Array},
    threadsafe_function::ThreadsafeFunctionCallMode,
};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use oxc_allocator::{Allocator, free_fixed_size_allocator};
use oxc_linter::{
    ExternalLinter, ExternalLinterCreateWorkspaceCb, ExternalLinterDestroyWorkspaceCb,
    ExternalLinterInitializeWorkersCb, ExternalLinterLintFileCb, ExternalLinterLoadPluginCb,
    ExternalLinterSetupRuleConfigsCb, LintFileResult, LoadPluginResult,
};

use crate::{
    generated::raw_transfer_constants::{BLOCK_ALIGN, BUFFER_SIZE},
    run::{
        JsCreateWorkspaceCb, JsDestroyWorkspaceCb, JsInitializePluginWorkersCb, JsLintFileCb,
        JsLoadPluginCb, JsSetupRuleConfigsCb,
    },
};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct PluginLoadRecord {
    plugin_url: String,
    plugin_name: Option<String>,
    plugin_name_is_alias: bool,
    workspace_uri: Option<String>,
    expected: LoadPluginResult,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct WorkerBootstrap {
    plugins: Vec<PluginLoadRecord>,
    options_json: String,
}

#[derive(Default)]
struct BootstrapState {
    plugins: Mutex<Vec<PluginLoadRecord>>,
    options_json: Mutex<Option<String>>,
}

#[derive(Default)]
struct WorkerHealth {
    failure: Mutex<Option<String>>,
    failure_reported: Condvar,
}

impl WorkerHealth {
    fn fail(&self, error: String) {
        let mut failure = self.failure.lock().unwrap();
        if failure.is_none() {
            *failure = Some(error);
            self.failure_reported.notify_all();
        }
    }

    fn get_failure(&self) -> Option<String> {
        self.failure.lock().unwrap().clone()
    }

    fn wait_for_failure(&self) -> Option<String> {
        let failure = self.failure.lock().unwrap();
        let (failure, _) = self
            .failure_reported
            .wait_timeout_while(failure, Duration::from_millis(500), |failure| failure.is_none())
            .unwrap();
        failure.clone()
    }
}

struct RegisteredWorker {
    lint_file: JsLintFileCb,
    health: Arc<WorkerHealth>,
}

struct PendingWorkerPool {
    workers: Vec<Option<RegisteredWorker>>,
    failures: Vec<Option<String>>,
}

impl PendingWorkerPool {
    fn new(worker_count: usize) -> Self {
        Self {
            workers: std::iter::repeat_with(|| None).take(worker_count).collect(),
            failures: vec![None; worker_count],
        }
    }
}

type PendingWorkerPools = FxHashMap<u32, PendingWorkerPool>;
type WorkerHealths = FxHashMap<(u32, u32), Arc<WorkerHealth>>;

static NEXT_WORKER_POOL_ID: AtomicU32 = AtomicU32::new(1);
static PENDING_WORKER_POOLS: LazyLock<Mutex<PendingWorkerPools>> =
    LazyLock::new(|| Mutex::new(FxHashMap::default()));
static WORKER_HEALTHS: LazyLock<Mutex<WorkerHealths>> =
    LazyLock::new(|| Mutex::new(FxHashMap::default()));

struct WorkerPoolGuard {
    pool_id: u32,
    worker_count: u32,
}

impl Drop for WorkerPoolGuard {
    fn drop(&mut self) {
        cleanup_worker_healths(self.pool_id, self.worker_count);
    }
}

struct InitializedLintHosts {
    callbacks: Box<[ExternalLinterLintFileCb]>,
    _worker_pool: WorkerPoolGuard,
}

type LintHosts = OnceLock<InitializedLintHosts>;

pub fn register_worker(
    pool_id: u32,
    worker_index: u32,
    lint_file: JsLintFileCb,
) -> Result<(), String> {
    let mut pools = PENDING_WORKER_POOLS.lock().unwrap();
    let pool = pools
        .get_mut(&pool_id)
        .ok_or_else(|| format!("JS plugin worker pool {pool_id} is not being initialized"))?;
    let worker_index_usize = usize::try_from(worker_index)
        .map_err(|_| format!("JS plugin worker index {worker_index} is out of range"))?;
    let worker = pool
        .workers
        .get_mut(worker_index_usize)
        .ok_or_else(|| format!("JS plugin worker index {worker_index_usize} is out of range"))?;
    if worker.is_some() {
        return Err(format!("JS plugin worker {worker_index_usize} registered more than once"));
    }

    let health = Arc::new(WorkerHealth::default());
    if let Some(error) = pool.failures[worker_index_usize].take() {
        health.fail(error);
    }
    WORKER_HEALTHS.lock().unwrap().insert((pool_id, worker_index), Arc::clone(&health));
    *worker = Some(RegisteredWorker { lint_file, health });
    Ok(())
}

pub fn report_worker_failure(pool_id: u32, worker_index: u32, error: String) {
    let mut pools = PENDING_WORKER_POOLS.lock().unwrap();
    if let Some(pool) = pools.get_mut(&pool_id)
        && let Some(failure) = pool.failures.get_mut(worker_index as usize)
    {
        if failure.is_none() {
            *failure = Some(error.clone());
        }
        if let Some(Some(worker)) = pool.workers.get(worker_index as usize) {
            worker.health.fail(error);
        }
        return;
    }
    drop(pools);

    if let Some(health) = WORKER_HEALTHS.lock().unwrap().get(&(pool_id, worker_index)) {
        health.fail(error);
    }
}

/// Wrap JS callbacks as normal Rust functions, and create [`ExternalLinter`].
pub fn create_external_linter(
    load_plugin: JsLoadPluginCb,
    setup_rule_configs: JsSetupRuleConfigsCb,
    lint_file: JsLintFileCb,
    initialize_workers: JsInitializePluginWorkersCb,
    create_workspace: JsCreateWorkspaceCb,
    destroy_workspace: JsDestroyWorkspaceCb,
) -> ExternalLinter {
    let bootstrap_state = Arc::new(BootstrapState::default());
    let lint_hosts = Arc::new(OnceLock::new());

    let rust_load_plugin = wrap_load_plugin(load_plugin, Arc::clone(&bootstrap_state));
    let rust_setup_rule_configs =
        wrap_setup_rule_configs(setup_rule_configs, Arc::clone(&bootstrap_state));
    let main_lint_file = wrap_lint_file(lint_file, None);
    let rust_lint_file = dispatch_lint_file(Arc::clone(&main_lint_file), Arc::clone(&lint_hosts));
    let rust_initialize_workers =
        wrap_initialize_workers(initialize_workers, bootstrap_state, main_lint_file, lint_hosts);
    let rust_create_workspace = wrap_create_workspace(create_workspace);
    let rust_destroy_workspace = wrap_destroy_workspace(destroy_workspace);

    ExternalLinter::new(
        rust_load_plugin,
        rust_setup_rule_configs,
        rust_lint_file,
        rust_initialize_workers,
        rust_create_workspace,
        rust_destroy_workspace,
    )
}

/// Result returned by `loadPlugin` JS callback.
#[derive(Clone, Debug, Deserialize)]
pub enum LoadPluginReturnValue {
    Success(LoadPluginResult),
    Failure(String),
}

/// Wrap `loadPlugin` JS callback as a normal Rust function.
///
/// The JS-side function is async. The returned Rust function blocks the current thread
/// until the `Promise` returned by the JS function resolves.
///
/// The returned function will panic if called outside of a Tokio runtime.
fn wrap_load_plugin(
    cb: JsLoadPluginCb,
    bootstrap_state: Arc<BootstrapState>,
) -> ExternalLinterLoadPluginCb {
    Arc::new(Box::new(move |plugin_url, plugin_name, plugin_name_is_alias, workspace_uri| {
        let record = PluginLoadRecord {
            plugin_url: plugin_url.clone(),
            plugin_name: plugin_name.clone(),
            plugin_name_is_alias,
            workspace_uri: workspace_uri.clone(),
            expected: LoadPluginResult { name: String::new(), offset: 0, rule_names: Vec::new() },
        };
        let cb = &cb;
        let res = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                cb.call_async(FnArgs::from((
                    plugin_url,
                    plugin_name,
                    plugin_name_is_alias,
                    workspace_uri,
                )))
                .await?
                .into_future()
                .await
            })
        });

        match res {
            // `loadPlugin` returns JSON string if plugin loaded successfully, or an error occurred
            Ok(json) => match serde_json::from_str(&json) {
                // Plugin loaded successfully
                Ok(LoadPluginReturnValue::Success(result)) => {
                    let mut record = record;
                    record.expected = result.clone();
                    bootstrap_state.plugins.lock().unwrap().push(record);
                    Ok(result)
                }
                // Error occurred on JS side
                Ok(LoadPluginReturnValue::Failure(err)) => Err(err),
                // Invalid JSON - should be impossible, because we control serialization on JS side
                Err(err) => {
                    Err(format!("Failed to deserialize JSON returned by `loadPlugin`: {err}"))
                }
            },
            // `loadPlugin` threw an error - should be impossible because `loadPlugin` is wrapped in try-catch
            Err(err) => Err(format!("`loadPlugin` threw an error: {err}")),
        }
    }))
}

/// Wrap `setupRuleConfigs` JS callback as a normal Rust function.
///
/// The JS-side `setupRuleConfigs` function is synchronous, but it's wrapped in a `ThreadsafeFunction`,
/// so cannot be called synchronously. Use an `mpsc::channel` to wait for the result from JS side,
/// and block current thread until `setupRuleConfigs` completes execution.
fn wrap_setup_rule_configs(
    cb: JsSetupRuleConfigsCb,
    bootstrap_state: Arc<BootstrapState>,
) -> ExternalLinterSetupRuleConfigsCb {
    Arc::new(Box::new(move |options_json: String| {
        let (tx, rx) = channel();
        let bootstrap_options_json = options_json.clone();

        // Send data to JS
        let status = cb.call_with_return_value(
            options_json,
            ThreadsafeFunctionCallMode::NonBlocking,
            move |result, _env| {
                // This call cannot fail, because `rx.recv()` below blocks until it receives a message.
                // This closure is a `FnOnce`, so it can't be called more than once, so only 1 message can be sent.
                // Therefore, `rx` cannot be dropped before this call.
                let res = tx.send(result);
                debug_assert!(res.is_ok(), "Failed to send result of `setupRuleConfigs`");
                Ok(())
            },
        );

        if status == Status::Ok {
            match rx.recv() {
                // Setup succeeded
                Ok(Ok(None)) => {
                    *bootstrap_state.options_json.lock().unwrap() = Some(bootstrap_options_json);
                    Ok(())
                }
                // Setup failed
                Ok(Ok(Some(err))) => Err(err),
                // `setupRuleConfigs` threw an error - should be impossible because it should be infallible
                Ok(Err(err)) => Err(format!("`setupRuleConfigs` threw an error: {err}")),
                // Sender "hung up" - should be impossible because closure passed to `call_with_return_value`
                // takes ownership of the sender `tx`. Unless NAPI-RS drops the closure without calling it,
                // `tx.send()` always happens before `tx` is dropped.
                Err(err) => Err(format!("`setupRuleConfigs` did not respond: {err}")),
            }
        } else {
            Err(format!("Failed to schedule `setupRuleConfigs` callback: {status:?}"))
        }
    }))
}

fn wrap_initialize_workers(
    cb: JsInitializePluginWorkersCb,
    bootstrap_state: Arc<BootstrapState>,
    main_lint_file: ExternalLinterLintFileCb,
    lint_hosts: Arc<LintHosts>,
) -> ExternalLinterInitializeWorkersCb {
    Arc::new(Box::new(move |host_count| {
        if host_count <= 1 {
            return Ok(());
        }
        if lint_hosts.get().is_some() {
            return Err("JS plugin workers have already been initialized".to_string());
        }

        let worker_count = host_count - 1;
        let worker_count_u32 = u32::try_from(worker_count)
            .map_err(|_| format!("Too many JS plugin workers requested: {worker_count}"))?;
        let options_json =
            bootstrap_state.options_json.lock().unwrap().clone().ok_or_else(|| {
                "JS plugin rule configuration has not been initialized".to_string()
            })?;
        let bootstrap = WorkerBootstrap {
            plugins: bootstrap_state.plugins.lock().unwrap().clone(),
            options_json,
        };
        let bootstrap_json = serde_json::to_string(&bootstrap)
            .map_err(|err| format!("Failed to serialize JS plugin worker state: {err}"))?;

        let pool_id = NEXT_WORKER_POOL_ID.fetch_add(1, Ordering::Relaxed);
        if pool_id == 0 {
            return Err("JS plugin worker pool ID space exhausted".to_string());
        }
        {
            let mut pools = PENDING_WORKER_POOLS.lock().unwrap();
            if pools.insert(pool_id, PendingWorkerPool::new(worker_count)).is_some() {
                return Err(format!("JS plugin worker pool ID {pool_id} is already in use"));
            }
        }

        let res = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                cb.call_async(FnArgs::from((pool_id, worker_count_u32, bootstrap_json)))
                    .await?
                    .into_future()
                    .await
            })
        });

        if let Err(err) = res {
            cleanup_worker_pool(pool_id, worker_count_u32);
            return Err(format!("Failed to initialize JS plugin workers: {err}"));
        }

        let pool =
            PENDING_WORKER_POOLS.lock().unwrap().remove(&pool_id).ok_or_else(|| {
                format!("JS plugin worker pool {pool_id} disappeared during startup")
            })?;

        if let Some((worker_index, error)) = pool
            .failures
            .iter()
            .enumerate()
            .find_map(|(index, error)| error.as_ref().map(|error| (index, error.clone())))
        {
            cleanup_worker_healths(pool_id, worker_count_u32);
            return Err(format!("JS plugin worker {worker_index} failed during startup: {error}"));
        }

        let mut hosts = Vec::with_capacity(host_count);
        hosts.push(Arc::clone(&main_lint_file));
        for (worker_index, worker) in pool.workers.into_iter().enumerate() {
            let Some(worker) = worker else {
                cleanup_worker_healths(pool_id, worker_count_u32);
                return Err(format!(
                    "JS plugin worker {worker_index} did not register during startup"
                ));
            };
            hosts.push(wrap_lint_file(worker.lint_file, Some(worker.health)));
        }

        lint_hosts
            .set(InitializedLintHosts {
                callbacks: hosts.into_boxed_slice(),
                _worker_pool: WorkerPoolGuard { pool_id, worker_count: worker_count_u32 },
            })
            .map_err(|_| "JS plugin workers have already been initialized".to_string())
    }))
}

fn cleanup_worker_pool(pool_id: u32, worker_count: u32) {
    PENDING_WORKER_POOLS.lock().unwrap().remove(&pool_id);
    cleanup_worker_healths(pool_id, worker_count);
}

fn cleanup_worker_healths(pool_id: u32, worker_count: u32) {
    let mut healths = WORKER_HEALTHS.lock().unwrap();
    for worker_index in 0..worker_count {
        healths.remove(&(pool_id, worker_index));
    }
}

fn dispatch_lint_file(
    main_lint_file: ExternalLinterLintFileCb,
    lint_hosts: Arc<LintHosts>,
) -> ExternalLinterLintFileCb {
    Arc::new(Box::new(
        move |file_path,
              rule_ids,
              options_ids,
              settings_json,
              globals_json,
              workspace_uri,
              allocator| {
            let host = if let Some(hosts) = lint_hosts.get() {
                // SAFETY: External rules are only run with allocators obtained from a fixed-size
                // allocator pool. The worker pool is initialized before any file is linted, and its
                // size never changes, so a buffer always maps to the same JavaScript isolate.
                let buffer_id = unsafe { allocator.fixed_size_metadata_ptr().as_ref().id };
                &hosts.callbacks[buffer_id as usize % hosts.callbacks.len()]
            } else {
                &main_lint_file
            };

            host(
                file_path,
                rule_ids,
                options_ids,
                settings_json,
                globals_json,
                workspace_uri,
                allocator,
            )
        },
    ))
}

/// Result returned by `lintFile` JS callback.
#[derive(Clone, Debug, Deserialize)]
pub enum LintFileReturnValue {
    Success(Vec<LintFileResult>),
    Failure(String),
}

/// Wrap `lintFile` JS callback as a normal Rust function.
///
/// The returned function creates a `Uint8Array` referencing the memory of the given `Allocator`,
/// and passes it to JS side, unless the `Allocator`'s buffer has already been sent to JS.
///
/// Unlike `loadPlugin`, `lintFile` JS callback is not async. But `ThreadsafeFunction` executes the callback
/// on main JS thread, and therefore it may have to wait for a previous `lintFile` call to complete.
/// Use an `mpsc::channel` to wait for the result from JS side, and block current thread until `lintFile`
/// completes execution.
fn wrap_lint_file(cb: JsLintFileCb, health: Option<Arc<WorkerHealth>>) -> ExternalLinterLintFileCb {
    Arc::new(Box::new(
        move |file_path: String,
              rule_ids: Vec<u32>,
              options_ids: Vec<u32>,
              settings_json: String,
              globals_json: String,
              workspace_uri: Option<String>,
              allocator: &Allocator| {
            let (tx, rx) = channel();

            // SAFETY: This function is only called when an `ExternalLinter` exists.
            // When that is the case, the `AllocatorPool` used to create `Allocator`s is created with
            // `AllocatorPool::new_fixed_size`, so all `Allocator`s are created via `FixedSizeAllocator`.
            // This is somewhat sketchy, as we don't have a type-level guarantee of this invariant,
            // but it does hold at present.
            // TODO: Close this soundness hole with type-level guarantees.
            let (buffer_id, buffer) = unsafe { get_buffer(allocator) };

            // Send data to JS
            let status = cb.call_with_return_value(
                FnArgs::from((
                    file_path,
                    buffer_id,
                    buffer,
                    rule_ids,
                    options_ids,
                    settings_json,
                    globals_json,
                    workspace_uri,
                )),
                ThreadsafeFunctionCallMode::NonBlocking,
                move |result, _env| {
                    // This call cannot fail, because `rx.recv()` below blocks until it receives a message.
                    // This closure is a `FnOnce`, so it can't be called more than once, so only 1 message can be sent.
                    // Therefore, `rx` cannot be dropped before this call.
                    let res = tx.send(result);
                    debug_assert!(res.is_ok(), "Failed to send result of `lintFile`");
                    Ok(())
                },
            );

            if status == Status::Ok {
                match receive_lint_result(&rx, health.as_deref()) {
                    // `lintFile` returns `null` if no diagnostics reported, and no error occurred
                    Ok(None) => Ok(Vec::new()),
                    // `lintFile` returns JSON string if diagnostics reported, or an error occurred
                    Ok(Some(json)) => {
                        match serde_json::from_str(&json) {
                            // Diagnostics reported
                            Ok(LintFileReturnValue::Success(diagnostics)) => Ok(diagnostics),
                            // Error occurred on JS side
                            Ok(LintFileReturnValue::Failure(err)) => Err(err),
                            // JSON deserialization failure.
                            // Possible if rule produces fixes/suggestions with out of range offsets.
                            Err(err) => Err(format!(
                                "Failed to deserialize JSON returned by `lintFile`: {err}"
                            )),
                        }
                    }
                    // `lintFile` threw an error - should be impossible because `lintFile` is wrapped in try-catch
                    Err(err) => Err(err),
                }
            } else {
                // When a worker exits, N-API can report `Closing` just before Node delivers the
                // worker's `exit` event to the main isolate. Wait briefly for that more useful
                // health error instead of emitting a generic scheduling failure.
                let worker_error = health.as_ref().and_then(|health| health.wait_for_failure());
                Err(worker_error.unwrap_or_else(|| {
                    format!("Failed to schedule `lintFile` callback: {status:?}")
                }))
            }
        },
    ))
}

fn receive_lint_result(
    rx: &Receiver<napi::Result<Option<String>>>,
    health: Option<&WorkerHealth>,
) -> Result<Option<String>, String> {
    if let Some(health) = health {
        loop {
            match rx.recv_timeout(Duration::from_millis(100)) {
                Ok(Ok(result)) => return Ok(result),
                Ok(Err(err)) => {
                    return Err(health
                        .wait_for_failure()
                        .unwrap_or_else(|| format!("`lintFile` threw an error: {err}")));
                }
                Err(RecvTimeoutError::Disconnected) => {
                    return Err(health.wait_for_failure().unwrap_or_else(|| {
                        "`lintFile` callback disconnected without responding".to_string()
                    }));
                }
                Err(RecvTimeoutError::Timeout) => {
                    if let Some(error) = health.get_failure() {
                        return Err(error);
                    }
                }
            }
        }
    }

    match rx.recv() {
        Ok(Ok(result)) => Ok(result),
        Ok(Err(err)) => Err(format!("`lintFile` threw an error: {err}")),
        Err(err) => Err(format!("`lintFile` did not respond: {err}")),
    }
}

/// Get buffer ID of the `Allocator` and, if it hasn't already been sent to JS,
/// create a `Uint8Array` referencing the `Allocator`'s memory.
///
/// Each buffer is sent over to JS only once.
/// JS side stores them in an array (indexed by buffer ID), and holds them until process ends.
/// This means there's only ever 1 instance of a buffer on Rust side, and 1 on JS side,
/// which makes it simpler to avoid use-after-free or double-free problems.
///
/// So only create a `Uint8Array` if it's not already sent to JS.
///
/// Whether the buffer has already been sent to JS is tracked by a flag in `FixedSizeAllocatorMetadata`,
/// which is stored in memory backing the `Allocator`.
///
/// # SAFETY
/// `allocator` must have been created via `FixedSizeAllocator`
unsafe fn get_buffer(
    allocator: &Allocator,
) -> (
    u32,                // Buffer ID
    Option<Uint8Array>, // Buffer, if not already sent to JS
) {
    // SAFETY: Caller guarantees `Allocator` was created by a `FixedSizeAllocator`.
    // We only create an immutable ref from this pointer.
    let metadata_ptr = unsafe { allocator.fixed_size_metadata_ptr() };
    // SAFETY: Fixed-size allocators always have a valid `FixedSizeAllocatorMetadata`
    // stored at the pointer returned by `Allocator::fixed_size_metadata_ptr`
    let metadata = unsafe { metadata_ptr.as_ref() };

    let buffer_id = metadata.id;

    // Get whether this buffer has already been sent to JS
    // TODO: Is `SeqCst` excessive here?
    let already_sent_to_js = metadata.is_double_owned.swap(true, Ordering::SeqCst);

    // If buffer has already been sent to JS, don't send it again
    if already_sent_to_js {
        return (buffer_id, None);
    }

    // Buffer has not already been sent to JS. Send it.

    // Get pointer to start of allocator chunk.
    // SAFETY: Fixed-size allocators have their chunk aligned on `BLOCK_ALIGN`, and size less than `BLOCK_ALIGN`.
    // So we can get pointer to start of `Allocator` chunk by rounding down to next multiple of `BLOCK_ALIGN`.
    // That can't go out of bounds of the backing allocation.
    let chunk_ptr = unsafe {
        let ptr = metadata_ptr.cast::<u8>();
        let offset = ptr.addr().get() % BLOCK_ALIGN;
        ptr.sub(offset)
    };

    // SAFETY:
    // Range of memory starting at `chunk_ptr` and encompassing `BUFFER_SIZE` is all within
    // the allocation backing the `Allocator`.
    //
    // We can't prove that no mutable references to data in the buffer exist,
    // but there shouldn't be any, because linter doesn't mutate the AST.
    // Anyway, I (@overlookmotel) am not sure if the aliasing rules apply to code in another language.
    // Probably not, as JS code is outside the domain of the "Rust abstract machine".
    // As long as we don't mutate data in the buffer on JS side, it should be fine.
    //
    // On the other side, while many immutable references to data in the buffer exist (`AstKind`s for every AST node),
    // JS side does not mutate the data in the buffer, so that shouldn't break the guarantees of `&` references.
    //
    // This is all a bit wavy, but such is the way with sharing memory outside of Rust.
    //
    // The `Uint8Array` shared with JS covers the allocatable region plus `RawTransferMetadata`.
    // It does not include `FixedSizeAllocatorMetadata` or `ChunkFooter`, which sit at the end of the block.
    let buffer = unsafe {
        Uint8Array::with_external_data(chunk_ptr.as_ptr(), BUFFER_SIZE, move |_ptr, _len| {
            free_fixed_size_allocator(metadata_ptr);
        })
    };

    (buffer_id, Some(buffer))
}

/// Wrap `createWorkspace` JS callback as a normal Rust function.
///
/// The JS-side function is async. The returned Rust function blocks the current thread
/// until the `Promise` returned by the JS function resolves.
///
/// The returned function will panic if called outside of a Tokio runtime.
fn wrap_create_workspace(cb: JsCreateWorkspaceCb) -> ExternalLinterCreateWorkspaceCb {
    Arc::new(Box::new(move |workspace_uri| {
        let cb = &cb;
        let res = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current()
                .block_on(async move { cb.call_async(workspace_uri).await?.into_future().await })
        });

        match res {
            // `createWorkspace` completed successfully
            Ok(()) => Ok(()),
            // `createWorkspace` threw an error
            Err(err) => Err(format!("`createWorkspace` threw an error: {err}")),
        }
    }))
}

/// Wrap `destroyWorkspace` JS callback as a normal Rust function.
///
/// The JS-side `destroyWorkspace` function is synchronous, but it's wrapped in a `ThreadsafeFunction`,
/// so cannot be called synchronously. Use an `mpsc::channel` to wait for the result from JS side.
///
/// Uses a timeout to prevent indefinite blocking during shutdown, which can cause issues
/// in multi-root workspace scenarios where multiple workspaces are being destroyed concurrently.
fn wrap_destroy_workspace(cb: JsDestroyWorkspaceCb) -> ExternalLinterDestroyWorkspaceCb {
    Arc::new(Box::new(move |workspace_uri| {
        let (tx, rx) = channel();

        // Send data to JS
        let status = cb.call_with_return_value(
            workspace_uri,
            ThreadsafeFunctionCallMode::NonBlocking,
            move |result, _env| {
                // Ignore send errors - the receiver may have timed out
                let _ = tx.send(result);
                Ok(())
            },
        );

        if status == Status::Ok {
            // Use a timeout to prevent blocking indefinitely during shutdown.
            // If JS side doesn't respond within the timeout, we proceed with shutdown anyway.
            match rx.recv_timeout(Duration::from_secs(5)) {
                // Destroying workspace succeeded
                Ok(Ok(()))
                // Timeout or sender dropped - proceed with shutdown
                | Err(_) => Ok(()),
                // `destroyWorkspace` threw an error
                Ok(Err(err)) => Err(format!("`destroyWorkspace` threw an error: {err}")),
            }
        } else {
            Err(format!("Failed to schedule `destroyWorkspace` callback: {status:?}"))
        }
    }))
}
