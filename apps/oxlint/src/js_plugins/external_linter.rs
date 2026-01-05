use std::sync::{atomic::Ordering, mpsc::channel};

use napi::{
    Status,
    bindgen_prelude::{FnArgs, Uint8Array},
    threadsafe_function::ThreadsafeFunctionCallMode,
};
use serde::Deserialize;

use oxc_allocator::{Allocator, free_fixed_size_allocator};
use oxc_linter::{
    ExternalLinter, ExternalLinterLintFileCb, ExternalLinterLintFileWithCustomAstCb,
    ExternalLinterLoadParserCb, ExternalLinterLoadPluginCb, ExternalLinterParseFileCb,
    ExternalLinterSetupConfigsCb, ExternalLinterStripFileCb, LintFileResult, LoadParserResult,
    LoadPluginResult, ParseFileResult, StripFileResult,
};

use crate::{
    generated::raw_transfer_constants::{BLOCK_ALIGN, BUFFER_SIZE},
    run::{
        JsLintFileCb, JsLintFileWithCustomAstCb, JsLoadParserCb, JsLoadPluginCb, JsParseFileCb,
        JsSetupConfigsCb, JsStripFileCb,
    },
};

/// Wrap JS callbacks as normal Rust functions, and create [`ExternalLinter`].
pub fn create_external_linter(
    load_plugin: JsLoadPluginCb,
    setup_configs: JsSetupConfigsCb,
    lint_file: JsLintFileCb,
    load_parser: Option<JsLoadParserCb>,
    parse_file: Option<JsParseFileCb>,
    lint_file_with_custom_ast: Option<JsLintFileWithCustomAstCb>,
    strip_file: Option<JsStripFileCb>,
) -> ExternalLinter {
    let rust_load_plugin = wrap_load_plugin(load_plugin);
    let rust_setup_configs = wrap_setup_configs(setup_configs);
    let rust_lint_file = wrap_lint_file(lint_file);

    let mut linter = ExternalLinter::new(rust_load_plugin, rust_setup_configs, rust_lint_file);

    if let Some(load_parser) = load_parser {
        linter = linter.with_load_parser(wrap_load_parser(load_parser));
    }

    if let Some(parse_file) = parse_file {
        linter = linter.with_parse_file(wrap_parse_file(parse_file));
    }

    if let Some(lint_file_with_custom_ast) = lint_file_with_custom_ast {
        linter = linter.with_lint_file_with_custom_ast(wrap_lint_file_with_custom_ast(
            lint_file_with_custom_ast,
        ));
    }

    if let Some(strip_file) = strip_file {
        linter = linter.with_strip_file(wrap_strip_file(strip_file));
    }

    linter
}

/// Result returned by `loadPlugin` JS callback.
#[derive(Clone, Debug, Deserialize)]
pub enum LoadPluginReturnValue {
    Success(LoadPluginResult),
    Failure(String),
}

/// Result returned by `loadParser` JS callback.
#[derive(Clone, Debug, Deserialize)]
pub enum LoadParserReturnValue {
    Success(LoadParserResult),
    Failure(String),
}

/// Result returned by `parseFile` JS callback.
#[derive(Clone, Debug, Deserialize)]
pub enum ParseFileReturnValue {
    Success(ParseFileResult),
    Failure(String),
}

/// Result returned by `stripFile` JS callback.
#[derive(Clone, Debug, Deserialize)]
pub enum StripFileReturnValue {
    /// Stripping succeeded - contains the stripped source and mappings.
    Success(StripFileResult),
    /// The parser doesn't support stripping.
    NotSupported,
    /// Stripping failed with an error.
    Failure(String),
}

/// Wrap `loadPlugin` JS callback as a normal Rust function.
///
/// The JS-side function is async. The returned Rust function blocks the current thread
/// until the `Promise` returned by the JS function resolves.
///
/// The returned function will panic if called outside of a Tokio runtime.
fn wrap_load_plugin(cb: JsLoadPluginCb) -> ExternalLinterLoadPluginCb {
    Box::new(move |plugin_url, plugin_name, plugin_name_is_alias| {
        let cb = &cb;
        let res = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                cb.call_async(FnArgs::from((plugin_url, plugin_name, plugin_name_is_alias)))
                    .await?
                    .into_future()
                    .await
            })
        });

        match res {
            // `loadPlugin` returns JSON string if plugin loaded successfully, or an error occurred
            Ok(json) => match serde_json::from_str(&json) {
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
            Err(err) => Err(format!("`loadPlugin` threw an error: {err}")),
        }
    })
}

/// Wrap `loadParser` JS callback as a normal Rust function.
///
/// The JS-side function is async. The returned Rust function blocks the current thread
/// until the `Promise` returned by the JS function resolves.
///
/// The returned function will panic if called outside of a Tokio runtime.
fn wrap_load_parser(cb: JsLoadParserCb) -> ExternalLinterLoadParserCb {
    Box::new(move |parser_url, parser_options_json| {
        let cb = &cb;
        let res = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                cb.call_async(FnArgs::from((parser_url, parser_options_json)))
                    .await?
                    .into_future()
                    .await
            })
        });

        match res {
            // `loadParser` returns JSON string if parser loaded successfully, or an error occurred
            Ok(json) => match serde_json::from_str(&json) {
                // Parser loaded successfully
                Ok(LoadParserReturnValue::Success(result)) => Ok(result),
                // Error occurred on JS side
                Ok(LoadParserReturnValue::Failure(err)) => Err(err),
                // Invalid JSON - should be impossible, because we control serialization on JS side
                Err(err) => {
                    Err(format!("Failed to deserialize JSON returned by `loadParser`: {err}"))
                }
            },
            // `loadParser` threw an error - should be impossible because `loadParser` is wrapped in try-catch
            Err(err) => Err(format!("`loadParser` threw an error: {err}")),
        }
    })
}

/// Wrap `parseFile` JS callback as a normal Rust function.
///
/// The JS-side function is synchronous. Use an `mpsc::channel` to wait for the result
/// from JS side, and block current thread until `parseFile` completes execution.
fn wrap_parse_file(cb: JsParseFileCb) -> ExternalLinterParseFileCb {
    Box::new(move |parser_id, file_path, source_text, parser_options_json| {
        let (tx, rx) = channel();

        // Send data to JS
        let status = cb.call_with_return_value(
            FnArgs::from((parser_id, file_path, source_text, parser_options_json)),
            ThreadsafeFunctionCallMode::NonBlocking,
            move |result, _env| {
                // This call cannot fail, because `rx.recv()` below blocks until it receives a message.
                // This closure is a `FnOnce`, so it can't be called more than once, so only 1 message can be sent.
                // Therefore, `rx` cannot be dropped before this call.
                let res = tx.send(result);
                debug_assert!(res.is_ok(), "Failed to send result of `parseFile`");
                Ok(())
            },
        );

        if status == Status::Ok {
            match rx.recv() {
                // `parseFile` returns JSON string
                Ok(Ok(json)) => match serde_json::from_str(&json) {
                    // Parsing succeeded
                    Ok(ParseFileReturnValue::Success(result)) => Ok(result),
                    // Error occurred on JS side (e.g., parse error)
                    Ok(ParseFileReturnValue::Failure(err)) => Err(err),
                    // Invalid JSON - should be impossible, because we control serialization on JS side
                    Err(err) => {
                        Err(format!("Failed to deserialize JSON returned by `parseFile`: {err}"))
                    }
                },
                // `parseFile` threw an error - should be impossible because `parseFile` is wrapped in try-catch
                Ok(Err(err)) => Err(format!("`parseFile` threw an error: {err}")),
                // Sender "hung up" - should be impossible because closure passed to `call_with_return_value`
                // takes ownership of the sender `tx`. Unless NAPI-RS drops the closure without calling it,
                // `tx.send()` always happens before `tx` is dropped.
                Err(err) => Err(format!("`parseFile` did not respond: {err}")),
            }
        } else {
            Err(format!("Failed to schedule `parseFile` callback: {status:?}"))
        }
    })
}

/// Wrap `setupConfigs` JS callback as a normal Rust function.
///
/// The JS-side `setupConfigs` function is synchronous, but it's wrapped in a `ThreadsafeFunction`,
/// so cannot be called synchronously. Use an `mpsc::channel` to wait for the result from JS side,
/// and block current thread until `setupConfigs` completes execution.
fn wrap_setup_configs(cb: JsSetupConfigsCb) -> ExternalLinterSetupConfigsCb {
    Box::new(move |options_json: String| {
        let (tx, rx) = channel();

        // Send data to JS
        let status = cb.call_with_return_value(
            options_json,
            ThreadsafeFunctionCallMode::NonBlocking,
            move |result, _env| {
                // This call cannot fail, because `rx.recv()` below blocks until it receives a message.
                // This closure is a `FnOnce`, so it can't be called more than once, so only 1 message can be sent.
                // Therefore, `rx` cannot be dropped before this call.
                let res = tx.send(result);
                debug_assert!(res.is_ok(), "Failed to send result of `setupConfigs`");
                Ok(())
            },
        );

        if status == Status::Ok {
            match rx.recv() {
                // Setup succeeded
                Ok(Ok(None)) => Ok(()),
                // Setup failed
                Ok(Ok(Some(err))) => Err(err),
                // `setupConfigs` threw an error - should be impossible because it should be infallible
                Ok(Err(err)) => Err(format!("`setupConfigs` threw an error: {err}")),
                // Sender "hung up" - should be impossible because closure passed to `call_with_return_value`
                // takes ownership of the sender `tx`. Unless NAPI-RS drops the closure without calling it,
                // `tx.send()` always happens before `tx` is dropped.
                Err(err) => Err(format!("`setupConfigs` did not respond: {err}")),
            }
        } else {
            Err(format!("Failed to schedule `setupConfigs` callback: {status:?}"))
        }
    })
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
fn wrap_lint_file(cb: JsLintFileCb) -> ExternalLinterLintFileCb {
    Box::new(
        move |file_path: String,
              rule_ids: Vec<u32>,
              options_ids: Vec<u32>,
              settings_json: String,
              globals_json: String,
              allocator: &Allocator| {
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
                FnArgs::from((
                    file_path,
                    buffer_id,
                    buffer,
                    rule_ids,
                    options_ids,
                    settings_json,
                    globals_json,
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
                    Ok(Ok(None)) => Ok(Vec::new()),
                    // `lintFile` returns JSON string if diagnostics reported, or an error occurred
                    Ok(Ok(Some(json))) => {
                        match serde_json::from_str(&json) {
                            // Diagnostics reported
                            Ok(LintFileReturnValue::Success(diagnostics)) => Ok(diagnostics),
                            // Error occurred on JS side
                            Ok(LintFileReturnValue::Failure(err)) => Err(err),
                            // Invalid JSON - should be impossible, because we control serialization on JS side
                            Err(err) => Err(format!(
                                "Failed to deserialize JSON returned by `lintFile`: {err}"
                            )),
                        }
                    }
                    // `lintFile` threw an error - should be impossible because `lintFile` is wrapped in try-catch
                    Ok(Err(err)) => Err(format!("`lintFile` threw an error: {err}")),
                    // Sender "hung up" - should be impossible because closure passed to `call_with_return_value`
                    // takes ownership of the sender `tx`. Unless NAPI-RS drops the closure without calling it,
                    // `tx.send()` always happens before `tx` is dropped.
                    Err(err) => Err(format!("`lintFile` did not respond: {err}")),
                }
            } else {
                Err(format!("Failed to schedule `lintFile` callback: {status:?}"))
            }
        },
    )
}

/// Wrap `lintFileWithCustomAst` JS callback as a normal Rust function.
///
/// This is similar to `wrap_lint_file`, but accepts pre-parsed AST as JSON
/// instead of a binary buffer. Used for files parsed by custom parsers.
fn wrap_lint_file_with_custom_ast(
    cb: JsLintFileWithCustomAstCb,
) -> ExternalLinterLintFileWithCustomAstCb {
    Box::new(
        move |file_path: String,
              source_text: String,
              ast_json: String,
              rule_ids: Vec<u32>,
              options_ids: Vec<u32>,
              settings_json: String,
              globals_json: String,
              parser_services_json: String| {
            let (tx, rx) = channel();

            // Send data to JS
            let status = cb.call_with_return_value(
                FnArgs::from((
                    file_path,
                    source_text,
                    ast_json,
                    rule_ids,
                    options_ids,
                    settings_json,
                    globals_json,
                    parser_services_json,
                )),
                ThreadsafeFunctionCallMode::NonBlocking,
                move |result, _env| {
                    let res = tx.send(result);
                    debug_assert!(res.is_ok(), "Failed to send result of `lintFileWithCustomAst`");
                    Ok(())
                },
            );

            if status == Status::Ok {
                match rx.recv() {
                    // `lintFileWithCustomAst` returns `null` if no diagnostics reported
                    Ok(Ok(None)) => Ok(Vec::new()),
                    // `lintFileWithCustomAst` returns JSON string if diagnostics reported, or an error occurred
                    Ok(Ok(Some(json))) => {
                        match serde_json::from_str(&json) {
                            // Diagnostics reported
                            Ok(LintFileReturnValue::Success(diagnostics)) => Ok(diagnostics),
                            // Error occurred on JS side
                            Ok(LintFileReturnValue::Failure(err)) => Err(err),
                            // Invalid JSON - should be impossible, because we control serialization on JS side
                            Err(err) => Err(format!(
                                "Failed to deserialize JSON returned by `lintFileWithCustomAst`: {err}"
                            )),
                        }
                    }
                    // `lintFileWithCustomAst` threw an error
                    Ok(Err(err)) => Err(format!("`lintFileWithCustomAst` threw an error: {err}")),
                    // Sender "hung up"
                    Err(err) => Err(format!("`lintFileWithCustomAst` did not respond: {err}")),
                }
            } else {
                Err(format!("Failed to schedule `lintFileWithCustomAst` callback: {status:?}"))
            }
        },
    )
}

/// Wrap `stripFile` JS callback as a normal Rust function.
///
/// This strips custom syntax from files before linting with Rust rules.
/// Returns `None` if the parser doesn't support stripping.
fn wrap_strip_file(cb: JsStripFileCb) -> ExternalLinterStripFileCb {
    Box::new(move |parser_id, file_path, source_text, parser_options_json| {
        let (tx, rx) = channel();

        // Send data to JS
        let status = cb.call_with_return_value(
            FnArgs::from((parser_id, file_path, source_text, parser_options_json)),
            ThreadsafeFunctionCallMode::NonBlocking,
            move |result, _env| {
                let res = tx.send(result);
                debug_assert!(res.is_ok(), "Failed to send result of `stripFile`");
                Ok(())
            },
        );

        if status == Status::Ok {
            match rx.recv() {
                // `stripFile` returns `null` if the parser doesn't support stripping
                Ok(Ok(None)) => Ok(None),
                // `stripFile` returns JSON string with stripped source or error
                Ok(Ok(Some(json))) => match serde_json::from_str(&json) {
                    // Stripping succeeded
                    Ok(StripFileReturnValue::Success(result)) => Ok(Some(result)),
                    // Parser doesn't support stripping
                    Ok(StripFileReturnValue::NotSupported) => Ok(None),
                    // Error occurred on JS side
                    Ok(StripFileReturnValue::Failure(err)) => Err(err),
                    // Invalid JSON - should be impossible, because we control serialization on JS side
                    Err(err) => {
                        Err(format!("Failed to deserialize JSON returned by `stripFile`: {err}"))
                    }
                },
                // `stripFile` threw an error
                Ok(Err(err)) => Err(format!("`stripFile` threw an error: {err}")),
                // Sender "hung up"
                Err(err) => Err(format!("`stripFile` did not respond: {err}")),
            }
        } else {
            Err(format!("Failed to schedule `stripFile` callback: {status:?}"))
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_strip_file_return_value_deserialize_success() {
        let json = r#"{"Success": {"source": "const x = 1;", "mappings": []}}"#;
        let result: StripFileReturnValue = serde_json::from_str(json).unwrap();
        match result {
            StripFileReturnValue::Success(r) => {
                assert_eq!(r.source, "const x = 1;");
                assert!(r.mappings.is_empty());
            }
            _ => panic!("Expected Success variant"),
        }
    }

    #[test]
    fn test_strip_file_return_value_deserialize_not_supported() {
        // Rust serde unit variants can be deserialized from `{"Variant": null}`
        let json = r#"{"NotSupported": null}"#;
        let result: StripFileReturnValue = serde_json::from_str(json).unwrap();
        assert!(matches!(result, StripFileReturnValue::NotSupported));
    }

    #[test]
    fn test_strip_file_return_value_deserialize_failure() {
        let json = r#"{"Failure": "Parser error: invalid syntax"}"#;
        let result: StripFileReturnValue = serde_json::from_str(json).unwrap();
        match result {
            StripFileReturnValue::Failure(err) => {
                assert_eq!(err, "Parser error: invalid syntax");
            }
            _ => panic!("Expected Failure variant"),
        }
    }

    #[test]
    fn test_strip_file_return_value_deserialize_not_supported_wrong_format() {
        // This is the wrong format - using `true` instead of `null` for unit variant
        // This test documents what NOT to do, and ensures we catch it
        let json = r#"{"NotSupported": true}"#;
        let result: Result<StripFileReturnValue, _> = serde_json::from_str(json);
        assert!(result.is_err(), "Unit variants must use null, not true");
    }
}
