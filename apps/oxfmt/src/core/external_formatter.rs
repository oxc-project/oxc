//! Bridge between napi-rs `ThreadsafeFunction`s and the Rust callback shapes
//! consumed by the embedded orchestration in `core::embed`.
//!
//! [`ExternalFormatter`] stores the JS callbacks as `Arc<dyn Fn>`s wrapped by the private `wrap_*` helpers,
//! exposes [`ExternalFormatter::format_file`] for the Tier 3/4 file delegations,
//! and assembles the napi build's `SessionServices` via [`ExternalFormatter::session_services`],
//! delegating the actual dispatcher / string-channel / Tailwind closures to `core::embed::{string_channel, dispatcher, prettier_fallback}`.

use std::sync::{Arc, RwLock};

use napi::{
    Status,
    bindgen_prelude::{FnArgs, Promise, block_on},
    threadsafe_function::ThreadsafeFunction,
};
use serde_json::Value;
use tracing::debug_span;

use oxc_formatter_core::{SessionServices, TailwindSorter};

use crate::core::embed::{
    self, FormatEmbeddedDocWithConfigCallback, FormatEmbeddedWithConfigCallback,
    FormatFileWithConfigCallback, TailwindWithConfigCallback,
};

/// Type alias for the init external formatter callback function signature.
/// Takes num_threads as argument; signals JS to perform any one-time setup before formatting.
pub type JsInitExternalFormatterCb = ThreadsafeFunction<
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

/// Type alias for the Doc-path callback function signature (batch).
/// Takes (options, texts[]) as arguments and returns Doc JSON string[] (one per text) or null on error.
/// The `options` object includes `parser` field set by Rust side.
pub type JsFormatEmbeddedDocCb = ThreadsafeFunction<
    // Input arguments
    FnArgs<(Value, Vec<String>)>, // (options, texts)
    // Return type (what JS function returns)
    Promise<Option<Vec<String>>>,
    // Arguments (repeated)
    FnArgs<(Value, Vec<String>)>,
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
    init: Arc<RwLock<Option<JsInitExternalFormatterCb>>>,
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

/// Callback function type for init external formatter.
/// Takes num_threads.
type InitExternalFormatterCallback = Arc<dyn Fn(usize) -> Result<(), String> + Send + Sync>;

/// External formatter that wraps a JS callback.
#[derive(Clone)]
pub struct ExternalFormatter {
    /// Handles to raw ThreadsafeFunctions for explicit cleanup
    handles: TsfnHandles,
    pub init: InitExternalFormatterCallback,
    pub format_file: FormatFileWithConfigCallback,
    pub format_embedded: FormatEmbeddedWithConfigCallback,
    pub format_embedded_doc: FormatEmbeddedDocWithConfigCallback,
    pub sort_tailwindcss_classes: TailwindWithConfigCallback,
}

impl std::fmt::Debug for ExternalFormatter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExternalFormatter")
            .field("init", &"<callback>")
            .field("format_file", &"<callback>")
            .field("format_embedded", &"<callback>")
            .field("format_embedded_doc", &"<callback>")
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

        let rust_init = wrap_init_external_formatter(init_handle);
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
    pub fn with_init_cb(self, init_cb: JsInitExternalFormatterCb) -> Self {
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

    /// Initialize external formatter using the JS callback.
    pub fn init(&self, num_threads: usize) -> Result<(), String> {
        debug_span!("oxfmt::external::init", num_threads = num_threads)
            .in_scope(|| (self.init)(num_threads))
    }

    /// Format non-js file using the JS callback.
    /// The `options` Value should already have `parser` and `filepath` set by the caller.
    pub fn format_file(&self, options: Value, code: &str) -> Result<String, String> {
        (self.format_file)(options, code)
    }

    /// Assemble the napi build's [`SessionServices`] for a root formatter run:
    /// the registry dispatcher with this formatter's Prettier Doc→IR fallback,
    /// the string channel, and the Tailwind sorter.
    /// The options (including `filepath`) are captured in the closures and passed to JS on each call.
    ///
    /// `dispatch_config` carries the resolved config; per-language options are mapped lazily at dispatch time
    /// (see `embed::dispatcher::ResolvedDispatchConfig`).
    /// The pure build's twin is `embed::fence::session_services`.
    ///
    /// Actual closure assembly lives in `core::embed::{string_channel, dispatcher, prettier_fallback}`;
    /// this method just bridges the napi-held callback `Arc`s into those factories.
    ///
    /// NOTE: Tailwind data paths
    /// `sort_tailwindcss_classes` is wired into THREE distinct callback slots,
    /// because Tailwind class sorting has four legitimate paths depending on
    /// where the classes live and how they reach printing:
    ///
    /// 1. Standalone JS/TS (`className` / functions / custom attributes):
    ///    `oxc_formatter` collects classes into `FormatElement::TailwindClass`.
    ///    When the entry document is finalized,
    ///    the printer sorts them in one host batch via the session's `tailwind_sorter`.
    /// 2. Standalone CSS / SCSS / LESS (`@apply` at top level):
    ///    the CSS root's session carries the same sorter (`CssFormatOptions::sort_tailwindcss`
    ///    only switches collection); classes sort once at finalize, no embedded boundary involved.
    /// 3. Embedded CSS (css-in-js + Angular `@Component({ styles })`):
    ///    `oxc_formatter_css::format_to_ir()` returns pre-sort `@apply` classes
    ///    in `DispatchResult::tailwind_classes`.
    ///    The JS parent re-indexes them into its own collector (`remap_tailwind_into`)
    ///    so they ride the SAME parent batch as path 1.
    ///    The standalone CSS sort closure is NOT invoked here.
    /// 4. JSDoc fenced CSS (Markdown code fence in JSDoc descriptions):
    ///    Goes through the string channel's fence adapter,
    ///    which returns a formatted string (not parent-integrated IR),
    ///    so there is no parent index space to remap into:
    ///    the adapter sorts the returned `DispatchResult::tailwind_classes` itself before printing.
    ///    The `string_channel::build_embedded_callback` factory receives the sorter for this case.
    ///
    /// All four paths use the SAME `sort_tailwindcss_classes` napi callback; only the wiring differs.
    /// Moving sorting to the wrong layer (e.g. sorting inside embedded CSS instead of remapping)
    /// would double-sort or drop classes.
    /// `DispatchResult::remap_tailwind_into`'s printer `debug_assert` catches dropped remaps.
    ///
    /// The dispatcher and the string channel consult the same off-predicate
    /// (`ResolvedDispatchConfig::is_embedded_formatting_enabled`),
    /// so the two channels never diverge on the off-semantics.
    pub fn session_services(
        &self,
        dispatch_config: &Arc<embed::dispatcher::ResolvedDispatchConfig>,
    ) -> SessionServices {
        let needs_embedded = dispatch_config.is_embedded_formatting_enabled();
        // Built only when the off-gate passes; `root_dispatcher` re-checks the
        // same predicate, so an off run allocates neither fallback nor dispatcher.
        let fallback = needs_embedded.then(|| {
            embed::prettier_fallback::build_prettier_fallback(
                Arc::clone(dispatch_config),
                Arc::clone(&self.format_embedded_doc),
            )
        });

        let tailwind_sorter = self.tailwind_sorter(dispatch_config);

        let string_embedder = needs_embedded.then(|| {
            embed::string_channel::build_string_embedder(
                Arc::clone(&self.format_embedded),
                tailwind_sorter.clone(),
                Arc::clone(dispatch_config),
            )
        });

        SessionServices {
            dispatcher: dispatch_config.root_dispatcher(fallback),
            string_embedder,
            tailwind_sorter,
        }
    }

    /// The ONE pre-bound Tailwind sorter (options JSON applied here, once),
    /// shared by every consumer: the JS root's session service, the fence
    /// adapter, and the standalone CSS root.
    pub fn tailwind_sorter(
        &self,
        dispatch_config: &Arc<embed::dispatcher::ResolvedDispatchConfig>,
    ) -> Option<TailwindSorter> {
        dispatch_config.is_tailwind_enabled().then(|| {
            let sort = Arc::clone(&self.sort_tailwindcss_classes);
            let dispatch_config = Arc::clone(dispatch_config);
            Arc::new(move |classes: Vec<String>| {
                debug_span!("oxfmt::external::sort_tailwind", classes_count = classes.len())
                    .in_scope(|| (sort)(dispatch_config.external_options(), classes))
            }) as TailwindSorter
        })
    }

    #[cfg(test)]
    pub fn dummy() -> Self {
        // Currently, LSP tests are implemented in Rust, while our external formatter relies on JS.
        // Therefore, just provides a dummy external formatter that consistently returns errors.
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
            format_embedded_doc: Arc::new(|_, _: &[&str]| {
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

/// Wrap JS `formatEmbeddedDoc` callback as a normal Rust function (batch).
/// The `options` Value is received with `parser` already set by the caller.
fn wrap_format_embedded_doc(
    cb_handle: Arc<RwLock<Option<JsFormatEmbeddedDocCb>>>,
) -> FormatEmbeddedDocWithConfigCallback {
    Arc::new(move |options: Value, texts: &[&str]| {
        let guard = cb_handle.read().unwrap();
        let Some(cb) = guard.as_ref() else {
            return Err("JS callback unavailable (environment shutting down)".to_string());
        };
        let texts_owned: Vec<String> = texts.iter().map(|t| (*t).to_string()).collect();
        let result = block_on(async {
            let status = cb.call_async(FnArgs::from((options, texts_owned))).await;
            match status {
                Ok(promise) => match promise.await {
                    Ok(Some(doc_jsons)) => Ok(doc_jsons),
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
