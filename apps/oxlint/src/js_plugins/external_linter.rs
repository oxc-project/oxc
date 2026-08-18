use std::{
    future::Future,
    sync::{
        Arc, Mutex, PoisonError,
        atomic::{AtomicBool, Ordering},
        mpsc::{Receiver, RecvTimeoutError, channel},
    },
    time::Duration,
};

use napi::{
    Status,
    bindgen_prelude::{FnArgs, Uint8Array},
    threadsafe_function::ThreadsafeFunctionCallMode,
};
use serde::Deserialize;

use oxc_allocator::{Allocator, AllocatorPool, free_fixed_size_allocator};
use oxc_linter::{
    ExternalLinter, ExternalLinterCreateWorkspaceCb, ExternalLinterDestroyWorkspaceCb,
    ExternalLinterForgetBufferCb, ExternalLinterLintFileCb, ExternalLinterLoadPluginCb,
    ExternalLinterSetupRuleConfigsCb, LintFileResult, LoadPluginResult,
};

use crate::{
    generated::raw_transfer_constants::{BLOCK_ALIGN, BUFFER_SIZE},
    run::{
        JsCreateWorkspaceCb, JsDestroyWorkspaceCb, JsForgetBufferCb, JsLintFileCb, JsLoadPluginCb,
        JsSetupRuleConfigsCb, JsStartWorkersCb, REGISTERED_WORKERS, set_js_plugin_worker_count,
        worker_liveness,
    },
};

/// Wrap JS callbacks as normal Rust functions, and create [`ExternalLinter`].
///
/// Starts on the main JS thread. The first [`ExternalLinter::load_plugin`] call probes `K` and,
/// when `K > 1`, starts worker isolates and switches over so `createOnce` runs only on those
/// isolates. Runs that never load a JS plugin never probe a fixed-size pool and never construct a
/// `Worker`.
pub fn create_external_linter(
    load_plugin: JsLoadPluginCb,
    setup_rule_configs: JsSetupRuleConfigsCb,
    lint_file: JsLintFileCb,
    forget_buffer: JsForgetBufferCb,
    create_workspace: JsCreateWorkspaceCb,
    destroy_workspace: JsDestroyWorkspaceCb,
    start_js_workers: JsStartWorkersCb,
) -> ExternalLinter {
    let main = ExternalLinter::new(
        Box::new([wrap_load_plugin(load_plugin, None)]),
        Box::new([wrap_setup_rule_configs(setup_rule_configs, None)]),
        // No worker to die: JS plugins run on the main JS thread here.
        Box::new([wrap_lint_file(lint_file, None)]),
        Box::new([wrap_forget_buffer(forget_buffer, None)]),
        Box::new([wrap_create_workspace(create_workspace, None)]),
        Box::new([wrap_destroy_workspace(destroy_workspace, None)]),
    );

    let runtime = Arc::new(DeferredJsPlugins {
        main,
        start_js_workers,
        state: Mutex::new(DeferredJsPluginState {
            host: JsPluginHost::Main,
            started: false,
            workspaces: Vec::new(),
        }),
    });

    let load_plugin: ExternalLinterLoadPluginCb = {
        let runtime = Arc::clone(&runtime);
        Arc::new(Box::new(move |plugin_url, plugin_name, plugin_name_is_alias, workspace_uri| {
            runtime.ensure_workers()?;
            runtime.host().load_plugin(plugin_url, plugin_name, plugin_name_is_alias, workspace_uri)
        }))
    };
    let setup_rule_configs: ExternalLinterSetupRuleConfigsCb = {
        let runtime = Arc::clone(&runtime);
        Arc::new(Box::new(move |options_json| runtime.host().setup_rule_configs(options_json)))
    };
    let lint_file: ExternalLinterLintFileCb = {
        let runtime = Arc::clone(&runtime);
        Arc::new(Box::new(
            move |file_path,
                  rule_ids,
                  options_ids,
                  settings_json,
                  globals_json,
                  workspace_uri,
                  allocator| {
                runtime.host().lint_file(
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
    };
    let forget_buffer: ExternalLinterForgetBufferCb = {
        let runtime = Arc::clone(&runtime);
        Arc::new(Box::new(move |buffer_id| runtime.host().forget_buffer(buffer_id)))
    };
    let create_workspace: ExternalLinterCreateWorkspaceCb = {
        let runtime = Arc::clone(&runtime);
        Arc::new(Box::new(move |workspace_uri| runtime.create_workspace(workspace_uri)))
    };
    let destroy_workspace: ExternalLinterDestroyWorkspaceCb = {
        let runtime = Arc::clone(&runtime);
        Arc::new(Box::new(move |workspace_uri| runtime.destroy_workspace(workspace_uri)))
    };

    ExternalLinter::new(
        Box::new([load_plugin]),
        Box::new([setup_rule_configs]),
        Box::new([lint_file]),
        Box::new([forget_buffer]),
        Box::new([create_workspace]),
        Box::new([destroy_workspace]),
    )
}

enum JsPluginHost {
    Main,
    Workers(ExternalLinter),
}

struct DeferredJsPluginState {
    host: JsPluginHost,
    started: bool,
    workspaces: Vec<String>,
}

struct DeferredJsPlugins {
    main: ExternalLinter,
    start_js_workers: JsStartWorkersCb,
    state: Mutex<DeferredJsPluginState>,
}

impl DeferredJsPlugins {
    fn host(&self) -> ExternalLinter {
        let state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        match &state.host {
            JsPluginHost::Main => self.main.clone(),
            JsPluginHost::Workers(workers) => workers.clone(),
        }
    }

    /// Probe `K` and start workers on the first JS plugin load.
    ///
    /// `create_workspace` can run before this (CLI/LSP set up a workspace, then parse config).
    /// Any URIs recorded there are replayed on the workers so they match the main isolate.
    ///
    /// A boot failure falls back to the main JS thread instead of aborting the run.
    fn ensure_workers(&self) -> Result<(), String> {
        let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        if state.started {
            return Ok(());
        }

        // Probe how many fixed-size arenas this machine will actually give us, then drop the probe
        // pool. On Windows that count can be lower than the thread count, and `K` must match the
        // real pools, otherwise a buffer would route to a worker that never sees it.
        let k = AllocatorPool::new_fixed_size(rayon::current_num_threads()).len();
        set_js_plugin_worker_count(u32::try_from(k).unwrap_or(u32::MAX));

        if k <= 1 {
            state.started = true;
            return Ok(());
        }

        let start_result = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(start_workers(&self.start_js_workers, k))
        });

        match start_result {
            Ok(()) => match create_external_linter_from_workers(k) {
                Ok(workers) => {
                    let replay = state
                        .workspaces
                        .iter()
                        .try_for_each(|uri| workers.create_workspace(uri.clone()));
                    state.host = JsPluginHost::Workers(workers);
                    state.started = true;
                    replay
                }
                Err(err) => {
                    fallback_to_main_thread(&mut state, &err);
                    Ok(())
                }
            },
            Err(err) => {
                fallback_to_main_thread(&mut state, &err);
                Ok(())
            }
        }
    }

    fn create_workspace(&self, workspace_uri: String) -> Result<(), String> {
        let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        if !state.workspaces.iter().any(|uri| uri == &workspace_uri) {
            state.workspaces.push(workspace_uri.clone());
        }
        match &state.host {
            JsPluginHost::Main => self.main.create_workspace(workspace_uri),
            JsPluginHost::Workers(workers) => workers.create_workspace(workspace_uri),
        }
    }

    fn destroy_workspace(&self, workspace_uri: String) -> Result<(), String> {
        let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        state.workspaces.retain(|uri| uri != &workspace_uri);
        match &state.host {
            JsPluginHost::Main => self.main.destroy_workspace(workspace_uri),
            JsPluginHost::Workers(workers) => workers.destroy_workspace(workspace_uri),
        }
    }
}

fn fallback_to_main_thread(state: &mut DeferredJsPluginState, err: &str) {
    print_startup_fallback(err);
    set_js_plugin_worker_count(1);
    state.started = true;
}

/// Ask JS to start `k` worker isolates, and wait until all of them have registered.
async fn start_workers(cb: &JsStartWorkersCb, k: usize) -> Result<(), String> {
    let k = u32::try_from(k).map_err(|_| format!("worker count {k} does not fit in u32"))?;
    let promise =
        cb.call_async(k).await.map_err(|err| format!("`startJsWorkers` threw an error: {err}"))?;
    promise.into_future().await.map_err(|err| format!("`startJsWorkers` failed: {err}"))
}

/// Report that workers failed to start and the run is continuing on the main JS thread.
///
/// Goes to stderr rather than stdout so this cannot land in the middle of diagnostics or the
/// language server protocol.
#[expect(clippy::print_stderr)]
fn print_startup_fallback(err: &str) {
    eprintln!(
        "Failed to start JS plugin workers; running JS plugins on the main thread instead:\n{err}"
    );
}

/// Create an [`ExternalLinter`] backed by `k` JS plugin worker isolates.
///
/// Workers register their callbacks in [`REGISTERED_WORKERS`] as they boot. This takes those
/// callbacks out of the map and wraps them, one slot per worker id, so `ExternalLinter` can route
/// `lint_file` / `forget_buffer` to the worker that owns a given buffer id.
///
/// # Errors
///
/// Returns an error if any worker id in `0..k` did not register.
fn create_external_linter_from_workers(k: usize) -> Result<ExternalLinter, String> {
    let mut workers = REGISTERED_WORKERS.lock().unwrap_or_else(std::sync::PoisonError::into_inner);

    let mut load_plugin = Vec::with_capacity(k);
    let mut setup_rule_configs = Vec::with_capacity(k);
    let mut lint_file = Vec::with_capacity(k);
    let mut forget_buffer = Vec::with_capacity(k);
    let mut create_workspace = Vec::with_capacity(k);
    let mut destroy_workspace = Vec::with_capacity(k);

    for id in 0..k {
        // `id` fits in `u32` because `k` is derived from a thread count.
        #[expect(clippy::cast_possible_truncation)]
        let id = id as u32;
        let worker =
            workers.remove(&id).ok_or_else(|| format!("JS plugin worker {id} did not register"))?;

        load_plugin.push(wrap_load_plugin(worker.load_plugin, Some(worker_liveness(id))));
        setup_rule_configs
            .push(wrap_setup_rule_configs(worker.setup_rule_configs, Some(worker_liveness(id))));
        lint_file.push(wrap_lint_file(worker.lint_file, Some(worker_liveness(id))));
        forget_buffer.push(wrap_forget_buffer(worker.forget_buffer, Some(worker_liveness(id))));
        create_workspace
            .push(wrap_create_workspace(worker.create_workspace, Some(worker_liveness(id))));
        destroy_workspace
            .push(wrap_destroy_workspace(worker.destroy_workspace, Some(worker_liveness(id))));
    }

    Ok(ExternalLinter::new(
        load_plugin.into_boxed_slice(),
        setup_rule_configs.into_boxed_slice(),
        lint_file.into_boxed_slice(),
        forget_buffer.into_boxed_slice(),
        create_workspace.into_boxed_slice(),
        destroy_workspace.into_boxed_slice(),
    ))
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
///
/// `liveness` is the flag for the worker isolate this callback runs on, or `None` when JS plugins run
/// on the main JS thread and so cannot die independently of the process.
fn wrap_load_plugin(
    cb: JsLoadPluginCb,
    liveness: Option<Arc<AtomicBool>>,
) -> ExternalLinterLoadPluginCb {
    Arc::new(Box::new(move |plugin_url, plugin_name, plugin_name_is_alias, workspace_uri| {
        // Fail fast if this worker already died, rather than dispatching a call that nothing
        // will ever run and then waiting for the poll below to notice.
        if let Some(liveness) = &liveness
            && !liveness.load(Ordering::SeqCst)
        {
            return Err(WORKER_DIED_ERROR.to_string());
        }

        let cb = &cb;
        let res = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let fut = async {
                    cb.call_async(FnArgs::from((
                        plugin_url,
                        plugin_name,
                        plugin_name_is_alias,
                        workspace_uri,
                    )))
                    .await?
                    .into_future()
                    .await
                };
                wait_for_async_result(fut, liveness.as_deref()).await
            })
        });

        match res {
            // `loadPlugin` returns JSON string if plugin loaded successfully, or an error occurred
            Ok(Ok(json)) => match serde_json::from_str(&json) {
                // Plugin loaded successfully
                Ok(LoadPluginReturnValue::Success(result)) => Ok(result),
                // Error occurred on JS side
                Ok(LoadPluginReturnValue::Failure(err)) => Err(err),
                // Invalid JSON - should be impossible, because we control serialization on JS side
                Err(err) => {
                    Err(format!("Failed to deserialize JSON returned by `loadPlugin`: {err}"))
                }
            },
            // `loadPlugin` threw an error - should be impossible because `loadPlugin` is wrapped in try-catch
            Ok(Err(err)) => Err(format!("`loadPlugin` threw an error: {err}")),
            Err(err) => Err(err),
        }
    }))
}

/// Wrap `setupRuleConfigs` JS callback as a normal Rust function.
///
/// The JS-side `setupRuleConfigs` function is synchronous, but it's wrapped in a `ThreadsafeFunction`,
/// so cannot be called synchronously. Use an `mpsc::channel` to wait for the result from JS side,
/// and block current thread until `setupRuleConfigs` completes execution.
///
/// `liveness` is the flag for the worker isolate this callback runs on, or `None` when JS plugins run
/// on the main JS thread and so cannot die independently of the process.
fn wrap_setup_rule_configs(
    cb: JsSetupRuleConfigsCb,
    liveness: Option<Arc<AtomicBool>>,
) -> ExternalLinterSetupRuleConfigsCb {
    Arc::new(Box::new(move |options_json: String| {
        // Fail fast if this worker already died, rather than dispatching a call that nothing
        // will ever run and then waiting for the poll below to notice.
        if let Some(liveness) = &liveness
            && !liveness.load(Ordering::SeqCst)
        {
            return Err(WORKER_DIED_ERROR.to_string());
        }

        let (tx, rx) = channel();

        // Send data to JS
        let status = cb.call_with_return_value(
            options_json,
            ThreadsafeFunctionCallMode::NonBlocking,
            move |result, _env| {
                // This call cannot fail, because `wait_for_result` below blocks until it receives a
                // message or the worker dies. This closure is a `FnOnce`, so it can't be called more
                // than once, so only 1 message can be sent. Therefore, `rx` cannot be dropped before
                // this call unless the worker isolate is torn down.
                let res = tx.send(result);
                debug_assert!(res.is_ok(), "Failed to send result of `setupRuleConfigs`");
                Ok(())
            },
        );

        if status == Status::Ok {
            match wait_for_result(&rx, liveness.as_deref()) {
                // Setup succeeded
                Ok(Ok(None)) => Ok(()),
                // Setup failed
                Ok(Ok(Some(err))) => Err(err),
                // `setupRuleConfigs` threw an error - should be impossible because it should be infallible
                Ok(Err(err)) => Err(format!("`setupRuleConfigs` threw an error: {err}")),
                Err(err) => Err(err),
            }
        } else {
            Err(format!("Failed to schedule `setupRuleConfigs` callback: {status:?}"))
        }
    }))
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
///
/// `liveness` is the flag for the worker isolate this callback runs on, or `None` when JS plugins run
/// on the main JS thread and so cannot die independently of the process.
fn wrap_lint_file(cb: JsLintFileCb, liveness: Option<Arc<AtomicBool>>) -> ExternalLinterLintFileCb {
    Arc::new(Box::new(
        move |file_path: String,
              rule_ids: Vec<u32>,
              options_ids: Vec<u32>,
              settings_json: String,
              globals_json: String,
              workspace_uri: Option<String>,
              allocator: &Allocator| {
            // Fail fast if this worker already died, rather than dispatching a call that nothing
            // will ever run and then waiting for the poll below to notice.
            if let Some(liveness) = &liveness
                && !liveness.load(Ordering::SeqCst)
            {
                return Err(WORKER_DIED_ERROR.to_string());
            }

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
                    // `wait_for_result` can return early if the worker dies, which drops `rx`.
                    // A late send is then a no-op, not a logic error.
                    let _ = tx.send(result);
                    Ok(())
                },
            );

            if status == Status::Ok {
                match wait_for_result(&rx, liveness.as_deref()) {
                    // `lintFile` returns `null` if no diagnostics reported, and no error occurred
                    Ok(Ok(None)) => Ok(Vec::new()),
                    // `lintFile` returns JSON string if diagnostics reported, or an error occurred
                    Ok(Ok(Some(json))) => {
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
                    Ok(Err(err)) => Err(format!("`lintFile` threw an error: {err}")),
                    Err(err) => Err(err),
                }
            } else {
                Err(format!("Failed to schedule `lintFile` callback: {status:?}"))
            }
        },
    ))
}

/// Error reported when a JS plugin callback cannot complete because its worker isolate died.
const WORKER_DIED_ERROR: &str = "JS plugin worker died before the callback returned";

/// How often to re-check whether a worker died while waiting for a JS plugin callback.
///
/// Only reached when the result hasn't arrived yet, so this costs one wakeup per interval per
/// in-flight call, and bounds how long a worker's death can stall the threads waiting on it.
const WORKER_DEATH_POLL_INTERVAL: Duration = Duration::from_millis(250);

/// Wait for the value that a JS plugin callback sends back from JS.
///
/// With `liveness`, the wait is interrupted if that worker isolate dies. A dead worker will never run
/// the callback that completes this channel, and every call routed to it would otherwise
/// block forever. Polling rather than a flat timeout, because a slow rule on a large file is normal
/// and must not be cut short.
///
/// Without `liveness` (JS plugins on the main JS thread), block until the callback runs, exactly as
/// before: there is no worker that can die on its own.
fn wait_for_result<T>(rx: &Receiver<T>, liveness: Option<&AtomicBool>) -> Result<T, String> {
    let Some(liveness) = liveness else {
        // Sender "hung up" - should be impossible because closure passed to `call_with_return_value`
        // takes ownership of the sender `tx`. Unless NAPI-RS drops the closure without calling it,
        // `tx.send()` always happens before `tx` is dropped.
        return rx.recv().map_err(|err| format!("JS plugin callback did not respond: {err}"));
    };

    loop {
        match rx.recv_timeout(WORKER_DEATH_POLL_INTERVAL) {
            Ok(value) => return Ok(value),
            Err(RecvTimeoutError::Timeout) => {
                if !liveness.load(Ordering::SeqCst) {
                    return Err(WORKER_DIED_ERROR.to_string());
                }
            }
            // NAPI-RS dropped the callback without calling it, which is what happens if the worker's
            // environment is torn down with the call still queued.
            Err(err @ RecvTimeoutError::Disconnected) => {
                return Err(format!("JS plugin callback did not respond: {err}"));
            }
        }
    }
}

/// Wait for an async JS callback, polling `liveness` so a dead worker cannot stall forever.
///
/// Does not impose a deadline on a live worker: the sleep is only a poll interval. Without
/// `liveness` (JS plugins on the main JS thread), wait until the future completes.
async fn wait_for_async_result<T>(
    fut: impl Future<Output = T>,
    liveness: Option<&AtomicBool>,
) -> Result<T, String> {
    let Some(liveness) = liveness else {
        return Ok(fut.await);
    };

    tokio::pin!(fut);
    loop {
        tokio::select! {
            biased;
            result = &mut fut => return Ok(result),
            () = tokio::time::sleep(WORKER_DEATH_POLL_INTERVAL) => {
                if !liveness.load(Ordering::SeqCst) {
                    return Err(WORKER_DIED_ERROR.to_string());
                }
            }
        }
    }
}

/// Wrap `forgetBuffer` JS callback as a normal Rust function.
///
/// The JS side just nulls a slot in its buffer cache, so there's no return value to wait for.
/// Dispatch it non-blocking and don't block the calling thread: the buffer's memory is freed by the
/// `Uint8Array` finalizer once JS garbage collects the view, which cannot happen synchronously.
fn wrap_forget_buffer(
    cb: JsForgetBufferCb,
    liveness: Option<Arc<AtomicBool>>,
) -> ExternalLinterForgetBufferCb {
    Arc::new(Box::new(move |buffer_id: u32| {
        if let Some(liveness) = &liveness
            && !liveness.load(Ordering::SeqCst)
        {
            return Err(WORKER_DIED_ERROR.to_string());
        }
        let status = cb.call(buffer_id, ThreadsafeFunctionCallMode::NonBlocking);
        if status == Status::Ok {
            Ok(())
        } else {
            Err(format!("Failed to schedule `forgetBuffer` callback: {status:?}"))
        }
    }))
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
///
/// `liveness` is the flag for the worker isolate this callback runs on, or `None` when JS plugins run
/// on the main JS thread and so cannot die independently of the process.
fn wrap_create_workspace(
    cb: JsCreateWorkspaceCb,
    liveness: Option<Arc<AtomicBool>>,
) -> ExternalLinterCreateWorkspaceCb {
    Arc::new(Box::new(move |workspace_uri| {
        // Fail fast if this worker already died, rather than dispatching a call that nothing
        // will ever run and then waiting for the poll below to notice.
        if let Some(liveness) = &liveness
            && !liveness.load(Ordering::SeqCst)
        {
            return Err(WORKER_DIED_ERROR.to_string());
        }

        let cb = &cb;
        let res = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let fut = async { cb.call_async(workspace_uri).await?.into_future().await };
                wait_for_async_result(fut, liveness.as_deref()).await
            })
        });

        match res {
            // `createWorkspace` completed successfully
            Ok(Ok(())) => Ok(()),
            // `createWorkspace` threw an error
            Ok(Err(err)) => Err(format!("`createWorkspace` threw an error: {err}")),
            Err(err) => Err(err),
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
fn wrap_destroy_workspace(
    cb: JsDestroyWorkspaceCb,
    liveness: Option<Arc<AtomicBool>>,
) -> ExternalLinterDestroyWorkspaceCb {
    Arc::new(Box::new(move |workspace_uri| {
        // A dead isolate is already gone; do not sit on the 5s shutdown timeout for it.
        if let Some(liveness) = &liveness
            && !liveness.load(Ordering::SeqCst)
        {
            return Ok(());
        }

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

#[cfg(test)]
mod tests {
    use std::{sync::mpsc::channel, thread, time::Instant};

    use super::*;

    fn block_on_async<T>(fut: impl Future<Output = T>) -> T {
        tokio::runtime::Builder::new_current_thread().enable_time().build().unwrap().block_on(fut)
    }

    /// A worker that dies with a JS plugin callback queued never runs the callback that completes the
    /// channel, so the wait has to end by itself. Before this was fixed, every file routed to that
    /// worker blocked forever.
    #[test]
    fn wait_ends_when_the_worker_dies_without_responding() {
        let (tx, rx) = channel::<()>();
        let alive = AtomicBool::new(true);

        thread::scope(|scope| {
            scope.spawn(|| {
                thread::sleep(Duration::from_millis(50));
                alive.store(false, Ordering::SeqCst);
            });

            let start = Instant::now();
            let err = wait_for_result(&rx, Some(&alive)).unwrap_err();
            assert_eq!(err, WORKER_DIED_ERROR);
            // Bounded by one poll interval after the death, not by the 5s-scale timeouts elsewhere
            assert!(start.elapsed() < Duration::from_secs(2), "took {:?}", start.elapsed());
        });

        // `tx` outlives the wait, so the wait ended because of the death, not a disconnect
        drop(tx);
    }

    /// A slow rule on a large file legitimately takes longer than the poll interval. Polling must not
    /// turn into a deadline.
    #[test]
    fn wait_keeps_waiting_while_the_worker_is_alive() {
        let (tx, rx) = channel();
        let alive = AtomicBool::new(true);

        thread::scope(|scope| {
            scope.spawn(move || {
                thread::sleep(WORKER_DEATH_POLL_INTERVAL * 3);
                tx.send("diagnostics").unwrap();
            });

            assert_eq!(wait_for_result(&rx, Some(&alive)).unwrap(), "diagnostics");
        });
    }

    /// The result still gets through if it arrives at about the same time as the death.
    #[test]
    fn wait_returns_a_result_that_arrived_before_the_death_was_seen() {
        let (tx, rx) = channel();
        let alive = AtomicBool::new(true);

        tx.send("diagnostics").unwrap();
        alive.store(false, Ordering::SeqCst);

        assert_eq!(wait_for_result(&rx, Some(&alive)).unwrap(), "diagnostics");
    }

    /// NAPI-RS dropping the callback without calling it is reported, not treated as a worker death.
    #[test]
    fn wait_reports_a_dropped_callback() {
        let (tx, rx) = channel::<()>();
        let alive = AtomicBool::new(true);

        drop(tx);

        let err = wait_for_result(&rx, Some(&alive)).unwrap_err();
        assert!(err.starts_with("JS plugin callback did not respond"), "{err}");
    }

    /// With JS plugins on the main JS thread there is no worker that can die independently, so the
    /// wait stays unbounded, exactly as it was before workers existed.
    #[test]
    fn wait_without_a_worker_blocks_until_the_callback_runs() {
        let (tx, rx) = channel();

        thread::scope(|scope| {
            scope.spawn(move || {
                thread::sleep(WORKER_DEATH_POLL_INTERVAL * 2);
                tx.send("diagnostics").unwrap();
            });

            assert_eq!(wait_for_result(&rx, None).unwrap(), "diagnostics");
        });
    }

    /// Same contract as `wait_for_result`, for the async TSFN path used by `loadPlugin` and
    /// `createWorkspace`. A pending future plus a death must not wait forever.
    #[test]
    fn async_wait_ends_when_the_worker_dies_without_completing() {
        let alive = AtomicBool::new(true);

        thread::scope(|scope| {
            scope.spawn(|| {
                thread::sleep(Duration::from_millis(50));
                alive.store(false, Ordering::SeqCst);
            });

            let start = Instant::now();
            let err =
                block_on_async(wait_for_async_result(std::future::pending::<()>(), Some(&alive)))
                    .unwrap_err();
            assert_eq!(err, WORKER_DIED_ERROR);
            assert!(start.elapsed() < Duration::from_secs(2), "took {:?}", start.elapsed());
        });
    }

    /// A slow async callback that outlasts the poll interval must not be treated as a deadline.
    #[test]
    fn async_wait_keeps_waiting_while_the_worker_is_alive() {
        let alive = AtomicBool::new(true);
        let fut = async {
            tokio::time::sleep(WORKER_DEATH_POLL_INTERVAL * 3).await;
            "diagnostics"
        };
        assert_eq!(
            block_on_async(wait_for_async_result(fut, Some(&alive))).unwrap(),
            "diagnostics"
        );
    }

    /// The result still gets through if it is ready at about the same time as the death.
    #[test]
    fn async_wait_returns_a_result_that_arrived_before_the_death_was_seen() {
        let alive = AtomicBool::new(true);
        alive.store(false, Ordering::SeqCst);
        assert_eq!(
            block_on_async(wait_for_async_result(async { "diagnostics" }, Some(&alive))).unwrap(),
            "diagnostics"
        );
    }

    /// With JS plugins on the main JS thread there is no worker that can die independently, so the
    /// wait stays unbounded.
    #[test]
    fn async_wait_without_a_worker_blocks_until_the_future_completes() {
        let fut = async {
            tokio::time::sleep(WORKER_DEATH_POLL_INTERVAL * 2).await;
            "diagnostics"
        };
        assert_eq!(block_on_async(wait_for_async_result(fut, None)).unwrap(), "diagnostics");
    }
}
