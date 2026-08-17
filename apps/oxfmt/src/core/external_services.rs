//! Transport between napi-rs `ThreadsafeFunction`s and plain Rust callback shapes;
//! no assembly logic (the napi `SessionServices` profiles live in `core::embed::services`).
//!
//! [`ExternalServices`] wraps the JS callbacks as `Arc<dyn Fn>`s (private `wrap_*` helpers), serving:
//! - [`ExternalServices::init`]: JS-side worker-pool setup
//! - [`ExternalServices::format_file`]: Tier 3/4 whole-file delegation (`core::format`)
//! - the embed callback `Arc`s (`format_embedded` / `format_embedded_doc` / `sort_tailwindcss_classes`),
//!   consumed by the `core::embed::services` profiles

use std::sync::{Arc, RwLock};

use napi::{
    Status,
    bindgen_prelude::{FnArgs, Promise, block_on},
    threadsafe_function::ThreadsafeFunction,
};
use serde_json::Value;
use tracing::debug_span;

use crate::core::embed::{
    FormatEmbeddedDocWithConfigCallback, FormatEmbeddedWithConfigCallback,
    TailwindWithConfigCallback,
};

/// Callback function type for formatting whole files (Tier 3/4 delegation).
/// Takes (options, code) and returns formatted code or an error.
/// The `options` Value is owned and includes `parser` and `filepath` set by the caller.
pub type FormatFileWithConfigCallback =
    Arc<dyn Fn(Value, &str) -> Result<String, String> + Send + Sync>;

/// Type alias for the init external services callback function signature.
/// Takes num_threads as argument; signals JS to perform any one-time setup before formatting.
pub type JsInitExternalServicesCb = ThreadsafeFunction<
    // Input arguments
    FnArgs<(u32,)>, // (num_threads,)
    // Return type (what JS function returns)
    Promise<()>,
    // Arguments (repeated)
    FnArgs<(u32,)>,
    // Error status
    Status,
    // CalleeHandled
    false,
>;

/// Type alias for the callback function signature.
/// Takes (options, code) as arguments and returns a wrapped formatter result.
/// The `options` object includes `parser` and `filepath` fields set by Rust side.
pub type JsFormatFileCb = ThreadsafeFunction<
    // Input arguments
    FnArgs<(Value, String)>, // (options, code)
    // Return type (what JS function returns)
    Promise<Value>,
    // Arguments (repeated)
    FnArgs<(Value, String)>,
    // Error status
    Status,
    // CalleeHandled
    false,
>;

/// Type alias for the callback function signature.
/// Takes (options, code) as arguments and returns formatted code or null on error.
/// The `options` object includes `parser` field set by Rust side.
pub type JsFormatEmbeddedCb = ThreadsafeFunction<
    // Input arguments
    FnArgs<(Value, String)>, // (options, code)
    // Return type (what JS function returns)
    Promise<Option<String>>,
    // Arguments (repeated)
    FnArgs<(Value, String)>,
    // Error status
    Status,
    // CalleeHandled
    false,
>;

/// Type alias for the Doc-path callback function signature.
/// Takes (options, code) as arguments and returns a Doc JSON string or null on error.
/// The `options` object includes `parser` field set by Rust side.
pub type JsFormatEmbeddedDocCb = ThreadsafeFunction<
    // Input arguments
    FnArgs<(Value, String)>, // (options, code)
    // Return type (what JS function returns)
    Promise<Option<String>>,
    // Arguments (repeated)
    FnArgs<(Value, String)>,
    // Error status
    Status,
    // CalleeHandled
    false,
>;

/// Type alias for Tailwind class processing callback.
/// Takes (options, classes) and returns sorted array or null on error.
/// The `filepath` is included in `options`.
pub type JsSortTailwindClassesCb = ThreadsafeFunction<
    FnArgs<(Value, Vec<String>)>, // Input: (options, classes)
    Promise<Option<Vec<String>>>, // Return: promise of sorted array or null
    FnArgs<(Value, Vec<String>)>,
    Status,
    false,
>;

/// Holds raw ThreadsafeFunctions wrapped in Option for cleanup.
/// The TSFNs can be explicitly dropped via `cleanup()` to prevent
/// use-after-free during V8 cleanup on Node.js exit.
///
/// Uses `RwLock` instead of `Mutex` for better performance: the wrapper functions
/// only read from the Option (common path), while only `cleanup()` needs write access.
#[derive(Clone)]
struct TsfnHandles {
    init: Arc<RwLock<Option<JsInitExternalServicesCb>>>,
    format_file: Arc<RwLock<Option<JsFormatFileCb>>>,
    format_embedded: Arc<RwLock<Option<JsFormatEmbeddedCb>>>,
    format_embedded_doc: Arc<RwLock<Option<JsFormatEmbeddedDocCb>>>,
    sort_tailwind: Arc<RwLock<Option<JsSortTailwindClassesCb>>>,
}

impl TsfnHandles {
    /// Drop all ThreadsafeFunctions to prevent use-after-free during V8 cleanup.
    fn cleanup(&self) {
        let _ = self.init.write().unwrap().take();
        let _ = self.format_file.write().unwrap().take();
        let _ = self.format_embedded.write().unwrap().take();
        let _ = self.format_embedded_doc.write().unwrap().take();
        let _ = self.sort_tailwind.write().unwrap().take();
    }
}

/// Callback function type for init external services.
/// Takes num_threads.
type InitExternalServicesCallback = Arc<dyn Fn(usize) -> Result<(), String> + Send + Sync>;

/// Transport handle to the JS-side services (Prettier formatting, Tailwind sorting),
/// each wrapped as a plain Rust callback.
#[derive(Clone)]
pub struct ExternalServices {
    /// Handles to raw ThreadsafeFunctions for explicit cleanup
    handles: TsfnHandles,
    pub init: InitExternalServicesCallback,
    pub format_file: FormatFileWithConfigCallback,
    pub format_embedded: FormatEmbeddedWithConfigCallback,
    pub format_embedded_doc: FormatEmbeddedDocWithConfigCallback,
    pub sort_tailwindcss_classes: TailwindWithConfigCallback,
}

impl std::fmt::Debug for ExternalServices {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExternalServices")
            .field("init", &"<callback>")
            .field("format_file", &"<callback>")
            .field("format_embedded", &"<callback>")
            .field("format_embedded_doc", &"<callback>")
            .field("sort_tailwindcss_classes", &"<callback>")
            .finish()
    }
}

impl ExternalServices {
    /// Create an [`ExternalServices`] from JS callbacks.
    ///
    /// The ThreadsafeFunctions are wrapped in `Arc<Mutex<Option<...>>>` to allow
    /// explicit cleanup via the `cleanup()` method. This prevents use-after-free
    /// crashes on Node.js exit when V8 cleans up global handles.
    pub fn new(
        format_file_cb: JsFormatFileCb,
        format_embedded_cb: JsFormatEmbeddedCb,
        format_embedded_doc_cb: JsFormatEmbeddedDocCb,
        sort_tailwindcss_classes_cb: JsSortTailwindClassesCb,
    ) -> Self {
        // Wrap TSFNs in Arc<RwLock<Option<...>>> so they can be explicitly dropped
        let init_handle = Arc::new(RwLock::new(None));
        let format_file_handle = Arc::new(RwLock::new(Some(format_file_cb)));
        let format_embedded_handle = Arc::new(RwLock::new(Some(format_embedded_cb)));
        let format_embedded_doc_handle = Arc::new(RwLock::new(Some(format_embedded_doc_cb)));
        let sort_tailwind_handle = Arc::new(RwLock::new(Some(sort_tailwindcss_classes_cb)));

        // Create handles struct for cleanup
        let handles = TsfnHandles {
            init: Arc::clone(&init_handle),
            format_file: Arc::clone(&format_file_handle),
            format_embedded: Arc::clone(&format_embedded_handle),
            format_embedded_doc: Arc::clone(&format_embedded_doc_handle),
            sort_tailwind: Arc::clone(&sort_tailwind_handle),
        };

        let rust_init = wrap_init_external_services(init_handle);
        let rust_format_file = wrap_format_file(format_file_handle);
        let rust_format_embedded = wrap_format_embedded(Arc::clone(&format_embedded_handle));
        let rust_format_embedded_doc = wrap_format_embedded_doc(format_embedded_doc_handle);
        let rust_tailwind = wrap_sort_tailwind_classes(sort_tailwind_handle);
        Self {
            handles,
            init: rust_init,
            format_file: rust_format_file,
            format_embedded: rust_format_embedded,
            format_embedded_doc: rust_format_embedded_doc,
            sort_tailwindcss_classes: rust_tailwind,
        }
    }

    /// Attach the init callback. Only paths that own a JS-side worker pool (CLI/LSP/Stdin) need this;
    /// the Node.js API path constructs without it and never calls [`Self::init`].
    pub fn with_init_cb(self, init_cb: JsInitExternalServicesCb) -> Self {
        *self.handles.init.write().unwrap() = Some(init_cb);
        self
    }

    /// Explicitly drop all ThreadsafeFunctions to prevent use-after-free
    /// during V8 cleanup on Node.js exit.
    ///
    /// This should be called before the function returns to ensure the TSFNs
    /// are released while V8 handles are still valid.
    pub fn cleanup(&self) {
        self.handles.cleanup();
    }

    /// Initialize the JS-side services (worker pool) using the JS callback.
    pub fn init(&self, num_threads: usize) -> Result<(), String> {
        debug_span!("oxfmt::external::init", num_threads = num_threads)
            .in_scope(|| (self.init)(num_threads))
    }

    /// Format non-js file using the JS callback.
    /// The `options` Value should already have `parser` and `filepath` set by the caller.
    pub fn format_file(&self, options: Value, code: &str) -> Result<String, String> {
        (self.format_file)(options, code)
    }

    #[cfg(test)]
    pub fn dummy() -> Self {
        // Currently, LSP tests are implemented in Rust, while the external services rely on JS.
        // Therefore, just provides a dummy that consistently returns errors.
        Self {
            handles: TsfnHandles {
                init: Arc::new(RwLock::new(None)),
                format_file: Arc::new(RwLock::new(None)),
                format_embedded: Arc::new(RwLock::new(None)),
                format_embedded_doc: Arc::new(RwLock::new(None)),
                sort_tailwind: Arc::new(RwLock::new(None)),
            },
            init: Arc::new(|_| Err("Dummy init called".to_string())),
            format_file: Arc::new(|_, _| Err("Dummy format_file called".to_string())),
            format_embedded: Arc::new(|_, _| Err("Dummy format_embedded called".to_string())),
            format_embedded_doc: Arc::new(|_, _: &str| {
                Err("Dummy format_embedded_doc called".to_string())
            }),
            sort_tailwindcss_classes: Arc::new(|_, _| vec![]),
        }
    }
}

// ---

// NOTE: These methods are all wrapped by `block_on` to run the async JS calls in a blocking manner.
//
// When called from `rayon` worker threads (Mode::Cli), this works fine.
// Because `rayon` threads are separate from the `tokio` runtime.
//
// However, in cases like `--stdin-filepath` or Node.js API calls,
// where already inside an async context (the `napi`'s `async` function),
// calling `block_on` directly would cause issues with nested async runtime access.
//
// Therefore, `block_in_place()` is used at the call site
// to temporarily convert the current async task into a blocking context.

/// Wrap JS `initExternalServices` callback as a normal Rust function.
fn wrap_init_external_services(
    cb_handle: Arc<RwLock<Option<JsInitExternalServicesCb>>>,
) -> InitExternalServicesCallback {
    Arc::new(move |num_threads: usize| {
        let guard = cb_handle.read().unwrap();
        let Some(cb) = guard.as_ref() else {
            return Err("JS callback unavailable (environment shutting down)".to_string());
        };
        #[expect(clippy::cast_possible_truncation)]
        let result = block_on(async {
            let status = cb.call_async(FnArgs::from((num_threads as u32,))).await;
            match status {
                Ok(promise) => match promise.await {
                    Ok(()) => Ok(()),
                    Err(err) => Err(err.reason),
                },
                Err(err) => Err(err.reason),
            }
        });
        drop(guard);
        result
    })
}

/// Wrap JS `formatFile` callback as a normal Rust function.
/// The `options` Value is received with `parser` and `filepath` already set by the caller.
fn wrap_format_file(
    cb_handle: Arc<RwLock<Option<JsFormatFileCb>>>,
) -> FormatFileWithConfigCallback {
    Arc::new(move |options: Value, code: &str| {
        let guard = cb_handle.read().unwrap();
        let Some(cb) = guard.as_ref() else {
            return Err("JS callback unavailable (environment shutting down)".to_string());
        };
        let result = block_on(async {
            let status = cb.call_async(FnArgs::from((options, code.to_string()))).await;
            match status {
                Ok(promise) => match promise.await {
                    Ok(result) => parse_format_file_result(result),
                    Err(err) => Err(err.reason),
                },
                Err(err) => Err(err.reason),
            }
        });
        drop(guard);
        result
    })
}

/// The `{ ok, code?, error? }` payload returned by the JS `formatFile` callback.
/// Recoverable formatter errors are passed as data instead of a rejected Promise,
/// so dropping a `napi::Error` never reaches `napi_reference_unref` during TSFN teardown.
#[derive(serde::Deserialize)]
struct FormatFileResult {
    ok: bool,
    code: Option<String>,
    error: Option<String>,
}

fn parse_format_file_result(result: Value) -> Result<String, String> {
    let result: FormatFileResult = serde_json::from_value(result)
        .map_err(|_| "File formatting returned an unexpected result".to_string())?;

    if result.ok {
        result.code.ok_or_else(|| "File formatting result missing `code`".to_string())
    } else {
        Err(result
            .error
            .filter(|error| !error.is_empty())
            .unwrap_or_else(|| "File formatting failed".to_string()))
    }
}

/// Wrap JS `formatEmbeddedCode` callback as a normal Rust function.
/// The `options` Value is received with `parser` already set by the caller.
fn wrap_format_embedded(
    cb_handle: Arc<RwLock<Option<JsFormatEmbeddedCb>>>,
) -> FormatEmbeddedWithConfigCallback {
    Arc::new(move |options: Value, code: &str| {
        let guard = cb_handle.read().unwrap();
        let Some(cb) = guard.as_ref() else {
            return Err("JS callback unavailable (environment shutting down)".to_string());
        };
        let result = block_on(async {
            let status = cb.call_async(FnArgs::from((options, code.to_string()))).await;
            match status {
                Ok(promise) => match promise.await {
                    Ok(Some(formatted_code)) => Ok(formatted_code),
                    Ok(None) => Err("Embedded formatting failed".to_string()),
                    // JS side never rejects; it returns `null` on error instead.
                    // `Err` here would only come from a napi-rs internal failure.
                    Err(err) => Err(err.reason),
                },
                Err(err) => Err(err.reason),
            }
        });
        drop(guard);
        result
    })
}

/// Wrap JS `formatEmbeddedDoc` callback as a normal Rust function.
/// The `options` Value is received with `parser` already set by the caller.
fn wrap_format_embedded_doc(
    cb_handle: Arc<RwLock<Option<JsFormatEmbeddedDocCb>>>,
) -> FormatEmbeddedDocWithConfigCallback {
    Arc::new(move |options: Value, code: &str| {
        let guard = cb_handle.read().unwrap();
        let Some(cb) = guard.as_ref() else {
            return Err("JS callback unavailable (environment shutting down)".to_string());
        };
        let result = block_on(async {
            let status = cb.call_async(FnArgs::from((options, code.to_string()))).await;
            match status {
                Ok(promise) => match promise.await {
                    Ok(Some(doc_json)) => Ok(doc_json),
                    Ok(None) => Err("Embedded doc formatting failed".to_string()),
                    // JS side never rejects; it returns `null` on error instead.
                    // `Err` here would only come from a napi-rs internal failure.
                    Err(err) => Err(err.reason),
                },
                Err(err) => Err(err.reason),
            }
        });
        drop(guard);
        result
    })
}

/// Wrap JS `sortTailwindClasses` callback as a normal Rust function.
fn wrap_sort_tailwind_classes(
    cb_handle: Arc<RwLock<Option<JsSortTailwindClassesCb>>>,
) -> TailwindWithConfigCallback {
    Arc::new(move |options: &Value, classes: Vec<String>| {
        let guard = cb_handle.read().unwrap();
        let Some(cb) = guard.as_ref() else {
            // Return original classes if callback unavailable
            return classes;
        };
        let result = block_on(async {
            let args = FnArgs::from((options.clone(), classes.clone()));
            match cb.call_async(args).await {
                Ok(promise) => match promise.await {
                    Ok(Some(sorted)) => sorted,
                    // JS side never rejects; it returns `null` on error instead.
                    // `Err` here would only come from a napi-rs internal failure.
                    Ok(None) | Err(_) => classes,
                },
                Err(_) => classes,
            }
        });
        drop(guard);
        result
    })
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::parse_format_file_result;

    #[test]
    fn parse_format_file_result_accepts_success_object() {
        let result = parse_format_file_result(json!({ "ok": true, "code": "formatted" }));
        assert_eq!(result, Ok("formatted".to_string()));
    }

    #[test]
    fn parse_format_file_result_preserves_error_message() {
        let result = parse_format_file_result(json!({ "ok": false, "error": "Unexpected token" }));
        assert_eq!(result, Err("Unexpected token".to_string()));
    }

    #[test]
    fn parse_format_file_result_falls_back_for_empty_error() {
        let result = parse_format_file_result(json!({ "ok": false, "error": "" }));
        assert_eq!(result, Err("File formatting failed".to_string()));
    }

    #[test]
    fn parse_format_file_result_rejects_unexpected_shape() {
        let result = parse_format_file_result(json!("formatted"));
        assert_eq!(result, Err("File formatting returned an unexpected result".to_string()));
    }
}
