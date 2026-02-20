use std::sync::{Arc, RwLock};

use napi::{
    Status,
    bindgen_prelude::{FnArgs, Promise, block_on},
    threadsafe_function::ThreadsafeFunction,
};
use serde_json::Value;
use tracing::debug_span;

use oxc_formatter::{
    EmbeddedFormatterCallback, ExternalCallbacks, FormatOptions, TailwindCallback,
};

/// Type alias for the init external formatter callback function signature.
/// Takes num_threads as argument and returns plugin languages.
pub type JsInitExternalFormatterCb = ThreadsafeFunction<
    // Input arguments
    FnArgs<(u32,)>, // (num_threads,)
    // Return type (what JS function returns)
    Promise<Vec<String>>,
    // Arguments (repeated)
    FnArgs<(u32,)>,
    // Error status
    Status,
    // CalleeHandled
    false,
>;

/// Type alias for the callback function signature.
/// Takes (options, code) as arguments and returns formatted code.
/// The `options` object includes `parser` field set by Rust side.
pub type JsFormatEmbeddedCb = ThreadsafeFunction<
    // Input arguments
    FnArgs<(Value, String)>, // (options, code)
    // Return type (what JS function returns)
    Promise<String>,
    // Arguments (repeated)
    FnArgs<(Value, String)>,
    // Error status
    Status,
    // CalleeHandled
    false,
>;

/// Type alias for the callback function signature.
/// Takes (options, code) as arguments and returns formatted code.
/// The `options` object includes `parser` and `filepath` fields set by Rust side.
pub type JsFormatFileCb = ThreadsafeFunction<
    // Input arguments
    FnArgs<(Value, String)>, // (options, code)
    // Return type (what JS function returns)
    Promise<String>,
    // Arguments (repeated)
    FnArgs<(Value, String)>,
    // Error status
    Status,
    // CalleeHandled
    false,
>;

/// Type alias for Tailwind class processing callback.
/// Takes (options, classes) and returns sorted array.
/// The `filepath` is included in `options`.
pub type JsSortTailwindClassesCb = ThreadsafeFunction<
    FnArgs<(Value, Vec<String>)>, // Input: (options, classes)
    Promise<Vec<String>>,         // Return: promise of sorted array
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
    init: Arc<RwLock<Option<JsInitExternalFormatterCb>>>,
    format_embedded: Arc<RwLock<Option<JsFormatEmbeddedCb>>>,
    format_file: Arc<RwLock<Option<JsFormatFileCb>>>,
    sort_tailwind: Arc<RwLock<Option<JsSortTailwindClassesCb>>>,
}

impl TsfnHandles {
    /// Drop all ThreadsafeFunctions to prevent use-after-free during V8 cleanup.
    fn cleanup(&self) {
        let _ = self.init.write().unwrap().take();
        let _ = self.format_embedded.write().unwrap().take();
        let _ = self.format_file.write().unwrap().take();
        let _ = self.sort_tailwind.write().unwrap().take();
    }
}

/// Callback function type for formatting embedded code with config.
/// Takes (options, code) and returns formatted code or an error.
/// The `options` Value is owned and includes `parser` set by the caller.
type FormatEmbeddedWithConfigCallback =
    Arc<dyn Fn(Value, &str) -> Result<String, String> + Send + Sync>;

/// Callback function type for formatting files with config.
/// Takes (options, code) and returns formatted code or an error.
/// The `options` Value is owned and includes `parser` and `filepath` set by the caller.
type FormatFileWithConfigCallback =
    Arc<dyn Fn(Value, &str) -> Result<String, String> + Send + Sync>;

/// Callback function type for init external formatter.
/// Takes num_threads and returns plugin languages.
type InitExternalFormatterCallback =
    Arc<dyn Fn(usize) -> Result<Vec<String>, String> + Send + Sync>;

/// Internal callback type for Tailwind processing with config.
/// Takes (options, classes) and returns sorted classes.
/// The `filepath` is included in `options`.
type TailwindWithConfigCallback = Arc<dyn Fn(&Value, Vec<String>) -> Vec<String> + Send + Sync>;

/// External formatter that wraps a JS callback.
#[derive(Clone)]
pub struct ExternalFormatter {
    /// Handles to raw ThreadsafeFunctions for explicit cleanup
    handles: TsfnHandles,
    pub init: InitExternalFormatterCallback,
    pub format_embedded: FormatEmbeddedWithConfigCallback,
    pub format_file: FormatFileWithConfigCallback,
    pub sort_tailwindcss_classes: TailwindWithConfigCallback,
}

impl std::fmt::Debug for ExternalFormatter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExternalFormatter")
            .field("init", &"<callback>")
            .field("format_embedded", &"<callback>")
            .field("format_file", &"<callback>")
            .field("sort_tailwindcss_classes", &"<callback>")
            .finish()
    }
}

impl ExternalFormatter {
    /// Create an [`ExternalFormatter`] from JS callbacks.
    ///
    /// The ThreadsafeFunctions are wrapped in `Arc<Mutex<Option<...>>>` to allow
    /// explicit cleanup via the `cleanup()` method. This prevents use-after-free
    /// crashes on Node.js exit when V8 cleans up global handles.
    pub fn new(
        init_cb: JsInitExternalFormatterCb,
        format_embedded_cb: JsFormatEmbeddedCb,
        format_file_cb: JsFormatFileCb,
        sort_tailwindcss_classes_cb: JsSortTailwindClassesCb,
    ) -> Self {
        // Wrap TSFNs in Arc<RwLock<Option<...>>> so they can be explicitly dropped
        let init_handle = Arc::new(RwLock::new(Some(init_cb)));
        let format_embedded_handle = Arc::new(RwLock::new(Some(format_embedded_cb)));
        let format_file_handle = Arc::new(RwLock::new(Some(format_file_cb)));
        let sort_tailwind_handle = Arc::new(RwLock::new(Some(sort_tailwindcss_classes_cb)));

        // Create handles struct for cleanup
        let handles = TsfnHandles {
            init: Arc::clone(&init_handle),
            format_embedded: Arc::clone(&format_embedded_handle),
            format_file: Arc::clone(&format_file_handle),
            sort_tailwind: Arc::clone(&sort_tailwind_handle),
        };

        let rust_init = wrap_init_external_formatter(init_handle);
        let rust_format_embedded = wrap_format_embedded(format_embedded_handle);
        let rust_format_file = wrap_format_file(format_file_handle);
        let rust_tailwind = wrap_sort_tailwind_classes(sort_tailwind_handle);
        Self {
            handles,
            init: rust_init,
            format_embedded: rust_format_embedded,
            format_file: rust_format_file,
            sort_tailwindcss_classes: rust_tailwind,
        }
    }

    /// Explicitly drop all ThreadsafeFunctions to prevent use-after-free
    /// during V8 cleanup on Node.js exit.
    ///
    /// This should be called before the function returns to ensure the TSFNs
    /// are released while V8 handles are still valid.
    pub fn cleanup(&self) {
        self.handles.cleanup();
    }

    /// Initialize external formatter using the JS callback.
    pub fn init(&self, num_threads: usize) -> Result<Vec<String>, String> {
        debug_span!("oxfmt::external::init", num_threads = num_threads)
            .in_scope(|| (self.init)(num_threads))
    }

    /// Convert this external formatter to the oxc_formatter::ExternalCallbacks type.
    /// The options (including `filepath`) are captured in the closures and passed to JS on each call.
    pub fn to_external_callbacks(
        &self,
        format_options: &FormatOptions,
        options: Value,
    ) -> ExternalCallbacks {
        let needs_embedded = !format_options.embedded_language_formatting.is_off();
        let embedded_callback: Option<EmbeddedFormatterCallback> = if needs_embedded {
            let format_embedded = Arc::clone(&self.format_embedded);
            let options_for_embedded = options.clone();
            Some(Arc::new(move |language: &str, code: &str| {
                let Some(parser_name) = language_to_prettier_parser(language) else {
                    // NOTE: Do not return `Ok(original)` here.
                    // We need to keep unsupported content as-is.
                    return Err(format!("Unsupported language: {language}"));
                };
                debug_span!("oxfmt::external::format_embedded", parser = parser_name).in_scope(
                    || {
                        // `clone()` is unavoidable here,
                        // because there may be multiple embedded sections in one JS/TS file.
                        let mut options = options_for_embedded.clone();
                        if let Value::Object(ref mut map) = options {
                            map.insert(
                                "parser".to_string(),
                                Value::String(parser_name.to_string()),
                            );
                        }
                        (format_embedded)(options, code)
                            .map(|mut code| {
                                // Remove trailing newline added by Prettier without allocation
                                // For embedded code, we never want trailing newlines, regardless of options.
                                let trimmed_len = code.trim_end().len();
                                code.truncate(trimmed_len);
                                code
                            })
                            .map_err(|err| {
                                format!(
                                    "Failed to format embedded code for parser '{parser_name}': {err}"
                                )
                            })
                    },
                )
            }))
        } else {
            None
        };

        let needs_tailwind = format_options.sort_tailwindcss.is_some();
        let tailwind_callback: Option<TailwindCallback> = if needs_tailwind {
            let sort_tailwindcss_classes = Arc::clone(&self.sort_tailwindcss_classes);
            Some(Arc::new(move |classes: Vec<String>| {
                debug_span!("oxfmt::external::sort_tailwind", classes_count = classes.len())
                    .in_scope(|| (sort_tailwindcss_classes)(&options, classes))
            }))
        } else {
            None
        };

        ExternalCallbacks::new()
            .with_embedded_formatter(embedded_callback)
            .with_tailwind(tailwind_callback)
    }

    /// Format non-js file using the JS callback.
    /// The `options` Value should already have `parser` and `filepath` set by the caller.
    pub fn format_file(&self, options: Value, code: &str) -> Result<String, String> {
        (self.format_file)(options, code)
    }

    #[cfg(test)]
    pub fn dummy() -> Self {
        // Currently, LSP tests are implemented in Rust, while our external formatter relies on JS.
        // Therefore, just provides a dummy external formatter that consistently returns errors.
        Self {
            handles: TsfnHandles {
                init: Arc::new(RwLock::new(None)),
                format_embedded: Arc::new(RwLock::new(None)),
                format_file: Arc::new(RwLock::new(None)),
                sort_tailwind: Arc::new(RwLock::new(None)),
            },
            init: Arc::new(|_| Err("Dummy init called".to_string())),
            format_embedded: Arc::new(|_, _| Err("Dummy format_embedded called".to_string())),
            format_file: Arc::new(|_, _| Err("Dummy format_file called".to_string())),
            sort_tailwindcss_classes: Arc::new(|_, _| vec![]),
        }
    }
}

// ---

/// Mapping from `oxc_formatter` language identifiers to Prettier `parser` names.
/// This is the single source of truth for supported embedded languages.
fn language_to_prettier_parser(language: &str) -> Option<&'static str> {
    match language {
        // TODO: "tagged-css" should use `scss` parser to support quasis
        "tagged-css" | "styled-jsx" => Some("css"),
        "tagged-graphql" => Some("graphql"),
        "tagged-html" => Some("html"),
        "tagged-markdown" => Some("markdown"),
        "angular-template" => Some("angular"),
        "angular-styles" => Some("scss"),
        _ => None,
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

/// Wrap JS `initExternalFormatter` callback as a normal Rust function.
fn wrap_init_external_formatter(
    cb_handle: Arc<RwLock<Option<JsInitExternalFormatterCb>>>,
) -> InitExternalFormatterCallback {
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
                    Ok(languages) => Ok(languages),
                    Err(err) => Err(err.reason.clone()),
                },
                Err(err) => Err(err.reason.clone()),
            }
        });
        drop(guard);
        result
    })
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
                    Ok(formatted_code) => Ok(formatted_code),
                    Err(err) => Err(err.reason.clone()),
                },
                Err(err) => Err(err.reason.clone()),
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
                    Ok(formatted_code) => Ok(formatted_code),
                    Err(err) => Err(err.reason.clone()),
                },
                Err(err) => Err(err.reason.clone()),
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
                    Ok(sorted) => sorted,
                    // Return original classes on error
                    Err(_) => classes,
                },
                // Return original classes on error
                Err(_) => classes,
            }
        });
        drop(guard);
        result
    })
}
