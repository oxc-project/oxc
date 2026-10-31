use std::{
    sync::{Arc, atomic::Ordering, mpsc::channel},
    time::Duration,
};

use futures_util::future::join_all;
use napi::{
    Status,
    bindgen_prelude::{FnArgs, Uint8Array},
    threadsafe_function::ThreadsafeFunctionCallMode,
};
use serde::Deserialize;

use oxc_allocator::{Allocator, free_fixed_size_allocator};
use oxc_linter::{
    ExternalLinter, ExternalLinterCreateWorkspaceCb, ExternalLinterDestroyWorkspaceCb,
    ExternalLinterLintFileCb, ExternalLinterLoadPluginCb, ExternalLinterSetupRuleConfigsCb,
    LintFileFailure, LintFileOutput, LoadPluginResult,
};

use super::js_plugin_workers::JsThreadCallbacks;
use crate::{
    generated::raw_transfer_constants::{BLOCK_ALIGN, BUFFER_SIZE},
    run::{
        JsCreateWorkspaceCb, JsDestroyWorkspaceCb, JsLintFileCb, JsLoadPluginCb,
        JsSetupRuleConfigsCb,
    },
};

/// Wrap JS callbacks as normal Rust functions, and create [`ExternalLinter`].
///
/// `js_threads` contains callbacks for each JS thread. JS thread 0 is the main JS thread.
/// Any further JS threads are JS plugin workers (`--js-threads` option).
///
/// * `lintFile` calls are routed to a single JS thread, chosen by the ID of the allocator buffer.
///   Each buffer is therefore only ever sent to one JS thread, which preserves the invariant that
///   a buffer has at most 1 owner on JS side.
/// * All other calls (`loadPlugin`, `setupRuleConfigs`, `createWorkspace`, `destroyWorkspace`)
///   are broadcast to all JS threads, so every JS thread has identical plugins, rules and options.
///
/// # Panics
/// Panics if `js_threads` is empty.
pub fn create_external_linter(js_threads: Vec<JsThreadCallbacks>) -> ExternalLinter {
    assert!(!js_threads.is_empty(), "At least 1 JS thread is required");

    let mut load_plugin = Vec::with_capacity(js_threads.len());
    let mut setup_rule_configs = Vec::with_capacity(js_threads.len());
    let mut lint_file = Vec::with_capacity(js_threads.len());
    let mut create_workspace = Vec::with_capacity(js_threads.len());
    let mut destroy_workspace = Vec::with_capacity(js_threads.len());
    for js_thread in js_threads {
        load_plugin.push(js_thread.load_plugin);
        setup_rule_configs.push(js_thread.setup_rule_configs);
        lint_file.push(js_thread.lint_file);
        create_workspace.push(js_thread.create_workspace);
        destroy_workspace.push(js_thread.destroy_workspace);
    }

    let rust_load_plugin = wrap_load_plugin(load_plugin);
    let rust_setup_rule_configs = wrap_setup_rule_configs(setup_rule_configs);
    let rust_lint_file = wrap_lint_file(lint_file);
    let rust_create_workspace = wrap_create_workspace(create_workspace);
    let rust_destroy_workspace = wrap_destroy_workspace(destroy_workspace);

    ExternalLinter::new(
        rust_load_plugin,
        rust_setup_rule_configs,
        rust_lint_file,
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

/// Wrap `loadPlugin` JS callbacks as a normal Rust function.
///
/// The JS-side function is async. The returned Rust function blocks the current thread
/// until the `Promise`s returned by the JS functions resolve.
///
/// The plugin is loaded in all JS threads concurrently. All JS threads must return identical results,
/// because rule IDs are offsets into the list of registered rules, which must match across JS threads.
///
/// The returned function will panic if called outside of a Tokio runtime.
fn wrap_load_plugin(cbs: Vec<JsLoadPluginCb>) -> ExternalLinterLoadPluginCb {
    Arc::new(Box::new(move |plugin_url, plugin_name, plugin_name_is_alias, workspace_uri| {
        let results = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(join_all(cbs.iter().map(|cb| {
                let args = FnArgs::from((
                    plugin_url.clone(),
                    plugin_name.clone(),
                    plugin_name_is_alias,
                    workspace_uri.clone(),
                ));
                async move { cb.call_async(args).await?.into_future().await }
            })))
        });

        let mut results = results.into_iter().map(parse_load_plugin_result);
        // `cbs` is never empty, so there is always a first result
        let first = results.next().unwrap()?;
        for (index, result) in results.enumerate() {
            let result = result?;
            if result.name != first.name
                || result.offset != first.offset
                || result.rule_names != first.rule_names
            {
                return Err(format!(
                    "Plugin `{}` loaded differently in JS thread {} than in JS thread 0",
                    first.name,
                    index + 1
                ));
            }
        }
        Ok(first)
    }))
}

/// Convert the result of a single `loadPlugin` call into a `LoadPluginResult`.
fn parse_load_plugin_result(res: napi::Result<String>) -> Result<LoadPluginResult, String> {
    match res {
        // `loadPlugin` returns JSON string if plugin loaded successfully, or an error occurred
        Ok(json) => match serde_json::from_str(&json) {
            // Plugin loaded successfully
            Ok(LoadPluginReturnValue::Success(result)) => Ok(result),
            // Error occurred on JS side
            Ok(LoadPluginReturnValue::Failure(err)) => Err(err),
            // Invalid JSON - should be impossible, because we control serialization on JS side
            Err(err) => Err(format!("Failed to deserialize JSON returned by `loadPlugin`: {err}")),
        },
        // `loadPlugin` threw an error - should be impossible because `loadPlugin` is wrapped in try-catch
        Err(err) => Err(format!("`loadPlugin` threw an error: {err}")),
    }
}

/// Wrap `setupRuleConfigs` JS callbacks as a normal Rust function.
///
/// The JS-side `setupRuleConfigs` function is synchronous, but it's wrapped in a `ThreadsafeFunction`,
/// so cannot be called synchronously. Use an `mpsc::channel` to wait for the result from JS side,
/// and block current thread until `setupRuleConfigs` completes execution.
///
/// The call is sent to all JS threads first, and then the results are awaited, so JS threads run it concurrently.
fn wrap_setup_rule_configs(cbs: Vec<JsSetupRuleConfigsCb>) -> ExternalLinterSetupRuleConfigsCb {
    Arc::new(Box::new(move |options_json: String| {
        // Send data to all JS threads
        let receivers = cbs
            .iter()
            .map(|cb| {
                let (tx, rx) = channel();
                let status = cb.call_with_return_value(
                    options_json.clone(),
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
                (status, rx)
            })
            .collect::<Vec<_>>();

        // Wait for all JS threads to respond. Return first error, if any.
        let mut first_err = None;
        for (status, rx) in receivers {
            let res = if status == Status::Ok {
                match rx.recv() {
                    // Setup succeeded
                    Ok(Ok(None)) => Ok(()),
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
            };
            if let Err(err) = res {
                first_err.get_or_insert(err);
            }
        }

        first_err.map_or(Ok(()), Err)
    }))
}

/// Result returned by `lintFile` JS callback.
#[derive(Clone, Debug, Deserialize)]
pub enum LintFileReturnValue {
    Success(LintFileOutput),
    Failure(LintFileFailure),
}

/// Wrap `lintFile` JS callback as a normal Rust function.
///
/// The returned function creates a `Uint8Array` referencing the memory of the given `Allocator`,
/// and passes it to JS side, unless the `Allocator`'s buffer has already been sent to JS.
///
/// Unlike `loadPlugin`, `lintFile` JS callback is not async. But `ThreadsafeFunction` executes the callback
/// on its own JS thread, and therefore it may have to wait for a previous `lintFile` call to complete.
/// Use an `mpsc::channel` to wait for the result from JS side, and block current thread until `lintFile`
/// completes execution.
///
/// If there are multiple JS threads, the JS thread is chosen by the buffer ID (`buffer_id % js_thread_count`).
/// Each buffer is therefore only ever sent to a single JS thread, so it has at most 1 owner on JS side.
/// Buffer IDs are assigned sequentially, so work is spread evenly across JS threads.
fn wrap_lint_file(cbs: Vec<JsLintFileCb>) -> ExternalLinterLintFileCb {
    Arc::new(Box::new(
        move |file_path: String,
              rule_ids: Vec<u32>,
              options_ids: Vec<u32>,
              settings_json: String,
              globals_json: String,
              collect_timings: bool,
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

            // Select JS thread which owns this buffer
            let cb = &cbs[buffer_id as usize % cbs.len()];

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
                    collect_timings,
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
                match rx.recv() {
                    // `lintFile` returns `null` if no diagnostics reported, and no error occurred
                    Ok(Ok(None)) => Ok(LintFileOutput::default()),
                    // `lintFile` returns JSON string if diagnostics reported, or an error occurred
                    Ok(Ok(Some(json))) => {
                        match serde_json::from_str(&json) {
                            // Linting succeeded
                            Ok(LintFileReturnValue::Success(output)) => Ok(output),
                            // Error occurred on JS side
                            Ok(LintFileReturnValue::Failure(failure)) => Err(failure),
                            // JSON deserialization failure.
                            // Possible if rule produces fixes/suggestions with out of range offsets.
                            Err(err) => Err(format!(
                                "Failed to deserialize JSON returned by `lintFile`: {err}"
                            )
                            .into()),
                        }
                    }
                    // `lintFile` threw an error - should be impossible because `lintFile` is wrapped in try-catch
                    Ok(Err(err)) => Err(format!("`lintFile` threw an error: {err}").into()),
                    // Sender "hung up" - should be impossible because closure passed to `call_with_return_value`
                    // takes ownership of the sender `tx`. Unless NAPI-RS drops the closure without calling it,
                    // `tx.send()` always happens before `tx` is dropped.
                    Err(err) => Err(format!("`lintFile` did not respond: {err}").into()),
                }
            } else {
                Err(format!("Failed to schedule `lintFile` callback: {status:?}").into())
            }
        },
    ))
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

/// Wrap `createWorkspace` JS callbacks as a normal Rust function.
///
/// The JS-side function is async. The returned Rust function blocks the current thread
/// until the `Promise`s returned by the JS functions in all JS threads resolve.
///
/// The returned function will panic if called outside of a Tokio runtime.
fn wrap_create_workspace(cbs: Vec<JsCreateWorkspaceCb>) -> ExternalLinterCreateWorkspaceCb {
    Arc::new(Box::new(move |workspace_uri| {
        let results = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(join_all(cbs.iter().map(|cb| {
                let workspace_uri = workspace_uri.clone();
                async move { cb.call_async(workspace_uri).await?.into_future().await }
            })))
        });

        for res in results {
            // `createWorkspace` threw an error
            if let Err(err) = res {
                return Err(format!("`createWorkspace` threw an error: {err}"));
            }
        }
        Ok(())
    }))
}

/// Wrap `destroyWorkspace` JS callbacks as a normal Rust function.
///
/// The JS-side `destroyWorkspace` function is synchronous, but it's wrapped in a `ThreadsafeFunction`,
/// so cannot be called synchronously. Use an `mpsc::channel` to wait for the result from JS side.
///
/// Uses a timeout to prevent indefinite blocking during shutdown, which can cause issues
/// in multi-root workspace scenarios where multiple workspaces are being destroyed concurrently.
fn wrap_destroy_workspace(cbs: Vec<JsDestroyWorkspaceCb>) -> ExternalLinterDestroyWorkspaceCb {
    Arc::new(Box::new(move |workspace_uri| {
        // Send data to all JS threads
        let receivers = cbs
            .iter()
            .map(|cb| {
                let (tx, rx) = channel();
                let status = cb.call_with_return_value(
                    workspace_uri.clone(),
                    ThreadsafeFunctionCallMode::NonBlocking,
                    move |result, _env| {
                        // Ignore send errors - the receiver may have timed out
                        let _ = tx.send(result);
                        Ok(())
                    },
                );
                (status, rx)
            })
            .collect::<Vec<_>>();

        let mut first_err = None;
        for (status, rx) in receivers {
            let res = if status == Status::Ok {
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
            };
            if let Err(err) = res {
                first_err.get_or_insert(err);
            }
        }

        first_err.map_or(Ok(()), Err)
    }))
}
