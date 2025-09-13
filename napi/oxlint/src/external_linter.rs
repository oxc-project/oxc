use std::sync::{Arc, atomic::Ordering, mpsc::channel};

use napi::{
    Status,
    bindgen_prelude::{FnArgs, Uint8Array},
    threadsafe_function::ThreadsafeFunctionCallMode,
};

use oxc_allocator::{Allocator, free_fixed_size_allocator};
use oxlint::{
    ExternalLinter, ExternalLinterLintFileCb, ExternalLinterLoadPluginCb, LintFileResult,
    PluginLoadResult,
};

use crate::{
    JsLintFileCb, JsLoadPluginCb,
    generated::raw_transfer_constants::{BLOCK_ALIGN, BUFFER_SIZE},
};

/// Wrap JS callbacks as normal Rust functions, and create [`ExternalLinter`].
pub fn create_external_linter(
    load_plugin: JsLoadPluginCb,
    lint_file: JsLintFileCb,
) -> ExternalLinter {
    let rust_load_plugin = wrap_load_plugin(load_plugin);
    let rust_lint_file = wrap_lint_file(lint_file);

    ExternalLinter::new(rust_load_plugin, rust_lint_file)
}

/// Wrap `loadPlugin` JS callback as a normal Rust function.
///
/// The JS-side function is async. The returned Rust function blocks the current thread
/// until the `Promise` returned by the JS function resolves.
///
/// The returned function will panic if called outside of a Tokio runtime.
fn wrap_load_plugin(cb: JsLoadPluginCb) -> ExternalLinterLoadPluginCb {
    let cb = Arc::new(cb);
    Arc::new(move |plugin_path| {
        let cb = Arc::clone(&cb);
        tokio::task::block_in_place(move || {
            tokio::runtime::Handle::current().block_on(async move {
                let result = cb.call_async(plugin_path).await?.into_future().await?;
                let plugin_load_result: PluginLoadResult = serde_json::from_str(&result)?;
                Ok(plugin_load_result)
            })
        })
    })
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
fn wrap_lint_file(cb: JsLintFileCb) -> ExternalLinterLintFileCb {
    let cb = Arc::new(cb);
    Arc::new(move |file_path: String, rule_ids: Vec<u32>, allocator: &Allocator| {
        let cb = Arc::clone(&cb);

        let (tx, rx) = channel();

        // SAFETY: This function is only called when an `ExternalLinter` exists.
        // When that is the case, the `AllocatorPool` used to create `Allocator`s is created with
        // `AllocatorPool::new_fixed_size`, so all `Allocator`s are created via `FixedSizeAllocator`.
        // This is somewhat sketchy, as we don't have a type-level guarantee of this invariant,
        // but it does hold at present.
        // When we replace `bumpalo` with a custom allocator, we can close this soundness hole.
        // TODO: Do that.
        let (buffer_id, buffer) = unsafe { get_buffer(allocator) };

        // Send data to JS
        let status = cb.call_with_return_value(
            FnArgs::from((file_path, buffer_id, buffer, rule_ids)),
            ThreadsafeFunctionCallMode::NonBlocking,
            move |result, _env| {
                let _ = match &result {
                    Ok(r) => match serde_json::from_str::<Vec<LintFileResult>>(r) {
                        Ok(v) => tx.send(Ok(v)),
                        Err(_e) => tx.send(Err("Failed to deserialize lint result".to_string())),
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
    })
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
    // stored at the pointer returned by `Allocator::fixed_size_metadata_ptr`.
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
