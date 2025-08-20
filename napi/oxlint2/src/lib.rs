use std::{
    mem,
    process::{ExitCode, Termination},
    sync::{Arc, Mutex, atomic::Ordering, mpsc::channel},
};

use napi::{
    Status,
    bindgen_prelude::{FnArgs, Promise, Uint8Array},
    threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode},
};
use napi_derive::napi;

use oxc_allocator::{Allocator, free_fixed_size_allocator};
use oxlint::{
    ExternalLinter, ExternalLinterInitWorkerThreadsCb, ExternalLinterLintFileCb,
    ExternalLinterLoadPluginCb, ExternalLinterLoadPluginsCb, ExternalLinterWorkerCallbacks,
    LintFileResult, PluginLoadResult, lint as oxlint_lint,
};

mod generated {
    pub mod raw_transfer_constants;
}
use generated::raw_transfer_constants::{BLOCK_ALIGN, BUFFER_SIZE};

/// Initialize JS worker threads.
#[napi]
pub type JsInitWorkerThreadsCb = ThreadsafeFunction<
    // Arguments
    u32, // Number of threads
    // Return value
    Promise<()>,
    // Arguments (repeated)
    u32,
    // Error status
    Status,
    // CalleeHandled
    false,
>;

/// Load a JS plugin on main thread.
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

/// Load multiple JS plugins on a worker thread.
#[napi]
pub type JsLoadPluginsCb = ThreadsafeFunction<
    // Arguments
    Vec<String>, // Absolute paths of plugin files
    // Return value
    Promise<()>,
    // Arguments (repeated)
    Vec<String>,
    // Error status
    Status,
    // CalleeHandled
    false,
>;

/// Lint a file on a worker thread.
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

/// Callback functions for worker threads.
static REGISTERED_WORKERS: Mutex<Vec<ExternalLinterWorkerCallbacks>> = Mutex::new(Vec::new());

/// Register a JS worker thread.
#[napi]
pub fn register_worker(load_plugins: JsLoadPluginsCb, lint_file: JsLintFileCb) {
    let callbacks = ExternalLinterWorkerCallbacks {
        load_plugins: wrap_load_plugins(load_plugins),
        lint_file: wrap_lint_file(lint_file),
    };
    #[expect(clippy::missing_panics_doc)]
    REGISTERED_WORKERS.lock().unwrap().push(callbacks);
}

fn wrap_init_worker_threads(cb: JsInitWorkerThreadsCb) -> ExternalLinterInitWorkerThreadsCb {
    let cb = Arc::new(cb);
    Arc::new(move |thread_count| {
        Box::pin({
            let cb = Arc::clone(&cb);
            async move {
                REGISTERED_WORKERS.lock().unwrap().reserve(thread_count as usize);

                cb.call_async(thread_count)
                    .await
                    .map_err(|e| e.to_string())?
                    .into_future()
                    .await
                    .map_err(|e| e.to_string())?;

                let callbacks_vec = {
                    let mut guard = REGISTERED_WORKERS.lock().unwrap();
                    mem::take(&mut *guard)
                };

                if callbacks_vec.len() != thread_count as usize {
                    return Err(format!(
                        "Expected {} JS worker threads to be initialized, but only {} were",
                        thread_count,
                        callbacks_vec.len()
                    ));
                }

                Ok(callbacks_vec.into_boxed_slice())
            }
        })
    })
}

fn wrap_load_plugin(cb: JsLoadPluginCb) -> ExternalLinterLoadPluginCb {
    let cb = Arc::new(cb);
    Arc::new(move |plugin_name| {
        Box::pin({
            let cb = Arc::clone(&cb);
            async move {
                let result = cb
                    .call_async(plugin_name)
                    .await
                    .map_err(|e| e.to_string())?
                    .into_future()
                    .await
                    .map_err(|e| e.to_string())?;
                let plugin_load_result: PluginLoadResult =
                    serde_json::from_str(&result).map_err(|e| e.to_string())?;
                Ok(plugin_load_result)
            }
        })
    })
}

fn wrap_load_plugins(cb: JsLoadPluginsCb) -> ExternalLinterLoadPluginsCb {
    let cb = Arc::new(cb);
    Arc::new(move |plugin_names| {
        Box::pin({
            let cb = Arc::clone(&cb);
            async move {
                cb.call_async(plugin_names)
                    .await
                    .map_err(|e| e.to_string())?
                    .into_future()
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(())
            }
        })
    })
}

fn wrap_lint_file(cb: JsLintFileCb) -> ExternalLinterLintFileCb {
    let cb = Arc::new(cb);
    Arc::new(
        // TODO: There's no way to mark a closure as unsafe. Need to find another way.
        // Like a `ThreadId` wrapper type?
        // SAFETY: `thread_id` must be less than the number of threads in the Rayon global thread pool.
        move |file_path: String, rule_ids: Vec<u32>, allocator: &Allocator, thread_id: usize| {
            let cb = Arc::clone(&cb);

            let (tx, rx) = channel();

            // SAFETY: This crate enables the `fixed_size` feature on `oxc_allocator`,
            // so all AST `Allocator`s are created via `FixedSizeAllocator`.
            // Caller guarantees that `thread_id` is less than number of threads in Rayon global thread pool.
            let (buffer_id, buffer) = unsafe { get_buffer(allocator, thread_id) };

            // Send data to JS
            let status = cb.call_with_return_value(
                FnArgs::from((file_path, buffer_id, buffer, rule_ids)),
                ThreadsafeFunctionCallMode::NonBlocking,
                move |result, _env| {
                    let _ = match &result {
                        Ok(r) => match serde_json::from_str::<Vec<LintFileResult>>(r) {
                            Ok(v) => tx.send(Ok(v)),
                            Err(_e) => {
                                tx.send(Err("Failed to deserialize lint result".to_string()))
                            }
                        },
                        Err(e) => tx.send(Err(e.to_string())),
                    };

                    result.map(|_| ())
                },
            );

            if status != Status::Ok {
                return Err(format!("Failed to schedule callback: {status:?}"));
            }

            match rx.recv() {
                Ok(Ok(x)) => Ok(x),
                Ok(Err(e)) => Err(format!("Callback reported error: {e}")),
                Err(e) => Err(format!("Callback did not respond: {e}")),
            }
        },
    )
}

/// Get buffer ID of the `Allocator` and, if it hasn't already been sent to this JS thread,
/// create a `Uint8Array` referencing the `Allocator`'s memory.
///
/// Each buffer is sent over to each JS thread only once.
/// JS side stores them in an array (indexed by buffer ID), and holds them until process ends.
/// This means there's only ever 1 instance of a buffer on Rust side, and 1 on each JS thread,
/// which makes it simpler to avoid use-after-free or double-free problems.
///
/// So only create a `Uint8Array` if it's not already sent to this JS thread.
///
/// Whether the buffer has already been send to this JS thread is tracked by a series of `bool` flags
/// stored in the `Allocator`'s memory, just before the `ChunkFooter`.
/// There's a `bool` for each thread in the Rayon global thread.
///
/// # SAFETY
/// * `allocator` must have been created via `FixedSizeAllocator`.
/// * `thread_id` must be less than number of threads in Rayon global thread pool.
unsafe fn get_buffer(
    allocator: &Allocator,
    thread_id: usize,
) -> (
    u32,                // Buffer ID
    Option<Uint8Array>, // Buffer, if not already sent to this JS thread
) {
    // SAFETY: Caller guarantees `Allocator` was created by a `FixedSizeAllocator`.
    // We only create an immutable ref from this pointer.
    let metadata_ptr = unsafe { allocator.fixed_size_metadata_ptr() };
    // SAFETY: Fixed-size allocators always have a valid `FixedSizeAllocatorMetadata`
    // stored at the pointer returned by `Allocator::fixed_size_metadata_ptr`.
    let metadata = unsafe { metadata_ptr.as_ref() };

    let buffer_id = metadata.id;

    // Get whether this buffer has already been sent to this JS thread.
    //
    // This is tracked by a series of `bool` flags stored in the `Allocator`'s memory,
    // just before the `ChunkFooter`.
    // `FixedSizeAllocator` initialized N x `bool` flags, where N is the number of threads in Rayon's
    // global thread pool.
    // These flags reside in the slice of memory ranging from `data_end_ptr() - N` to `data_end_ptr() - 1`.
    //
    // We don't know how many threads there are here, so work backwards from the end.
    // * Flag for thread 0 is at `data_end_ptr() - 1`
    // * Flag for thread 1 is at `data_end_ptr() - 2`, etc.
    //
    // SAFETY: Caller guarantees `thread_id` is less than number of threads in Rayon global thread pool.
    // Therefore `data_end_ptr() - (thread_id + 1)` points to the flag for this thread,
    // and it must be a valid initialized `bool`.
    let sent_to_js_thread =
        unsafe { allocator.data_end_ptr().cast::<bool>().sub(thread_id + 1).as_mut() };

    // If buffer has already been sent to this JS thread, don't send it again
    if *sent_to_js_thread {
        return (buffer_id, None);
    }

    // Buffer has not already been sent to JS. Send it.

    // Record that this buffer has now been sent to this JS thread
    *sent_to_js_thread = true;

    // Increment reference count for this allocator
    // TODO: Is `SeqCst` excessive here?
    metadata.ref_count.fetch_add(1, Ordering::SeqCst);

    // Get pointer to start of allocator chunk.
    // Note: `Allocator::data_ptr` would not provide the right pointer, because source text
    // gets written to start of the allocator chunk, and `data_ptr` gets moved to after it.
    // SAFETY: Fixed-size allocators have their chunk aligned on `BLOCK_ALIGN`,
    // and size less than `BLOCK_ALIGN`. So we can get pointer to start of `Allocator` chunk
    // by rounding down to next multiple of `BLOCK_ALIGN`. That can't go out of bounds of
    // the backing allocation.
    let chunk_ptr = unsafe {
        let ptr = metadata_ptr.cast::<u8>();
        let offset = ptr.as_ptr() as usize % BLOCK_ALIGN;
        ptr.sub(offset)
    };

    // SAFETY:
    // Range of memory starting at `chunk_ptr` and encompassing `BUFFER_SIZE` is all within
    // the allocation backing the `Allocator`.
    //
    // We can't prove that no mutable references to data in the buffer exist,
    // but there shouldn't be any, because linter doesn't mutate the AST.
    // Anyway, I (@overlookmotel) am not sure if the aliasing rules apply to code in another
    // language. Probably not, as JS code is outside the domain of the "Rust abstract machine".
    // As long as we don't mutate data in the buffer on JS side, it should be fine.
    //
    // On the other side, while many immutable references to data in the buffer exist
    // (`AstKind`s for every AST node), JS side does not mutate the data in the buffer,
    // so that shouldn't break the guarantees of `&` references.
    //
    // This is all a bit wavy, but such is the way with sharing memory outside of Rust.
    let buffer = unsafe {
        Uint8Array::with_external_data(chunk_ptr.as_ptr(), BUFFER_SIZE, move |_ptr, _len| {
            free_fixed_size_allocator(metadata_ptr);
        })
    };

    (buffer_id, Some(buffer))
}

#[expect(clippy::allow_attributes)]
#[allow(clippy::trailing_empty_array, clippy::unused_async)] // https://github.com/napi-rs/napi-rs/issues/2758
#[napi]
pub async fn lint(init_worker_threads: JsInitWorkerThreadsCb, load_plugin: JsLoadPluginCb) -> bool {
    let rust_init_worker_threads = wrap_init_worker_threads(init_worker_threads);
    let rust_load_plugin = wrap_load_plugin(load_plugin);

    oxlint_lint(Some(ExternalLinter::new(rust_init_worker_threads, rust_load_plugin))).report()
        == ExitCode::SUCCESS
}
