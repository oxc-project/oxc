use std::sync::Arc;

use napi::{
    Status,
    bindgen_prelude::{FnArgs, Promise, block_on},
    threadsafe_function::ThreadsafeFunction,
};
use serde_json::Value;

use oxc_formatter::TailwindCallback;

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
/// Takes (options, tag_name, code) as separate arguments and returns formatted code.
pub type JsFormatEmbeddedCb = ThreadsafeFunction<
    // Input arguments
    FnArgs<(Value, String, String)>, // (options, tag_name, code)
    // Return type (what JS function returns)
    Promise<String>,
    // Arguments (repeated)
    FnArgs<(Value, String, String)>,
    // Error status
    Status,
    // CalleeHandled
    false,
>;

/// Type alias for the callback function signature.
/// Takes (options, parser_name, file_name, code) as separate arguments and returns formatted code.
pub type JsFormatFileCb = ThreadsafeFunction<
    // Input arguments
    FnArgs<(Value, String, String, String)>, // (options, parser_name, file_name, code)
    // Return type (what JS function returns)
    Promise<String>,
    // Arguments (repeated)
    FnArgs<(Value, String, String, String)>,
    // Error status
    Status,
    // CalleeHandled
    false,
>;

/// Type alias for Tailwind class processing callback.
/// Takes (filepath, classes) and returns sorted array.
pub type JsTailwindCb = ThreadsafeFunction<
    FnArgs<(String, Vec<String>)>, // Input: (filepath, classes)
    Promise<Vec<String>>,          // Return: promise of sorted array
    FnArgs<(String, Vec<String>)>,
    Status,
    false,
>;

/// Callback function type for formatting embedded code with config.
/// Takes (options, tag_name, code) and returns formatted code or an error.
type FormatEmbeddedWithConfigCallback =
    Arc<dyn Fn(&Value, &str, &str) -> Result<String, String> + Send + Sync>;

/// Callback function type for formatting files with config.
/// Takes (options, parser_name, file_name, code) and returns formatted code or an error.
type FormatFileWithConfigCallback =
    Arc<dyn Fn(&Value, &str, &str, &str) -> Result<String, String> + Send + Sync>;

/// Callback function type for init external formatter.
/// Takes num_threads and returns plugin languages.
type InitExternalFormatterCallback =
    Arc<dyn Fn(usize) -> Result<Vec<String>, String> + Send + Sync>;

/// External formatter that wraps a JS callback.
#[derive(Clone)]
pub struct ExternalFormatter {
    pub init: InitExternalFormatterCallback,
    pub format_embedded: FormatEmbeddedWithConfigCallback,
    pub format_file: FormatFileWithConfigCallback,
    pub process_tailwind: TailwindCallback,
}

impl std::fmt::Debug for ExternalFormatter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExternalFormatter")
            .field("init", &"<callback>")
            .field("format_embedded", &"<callback>")
            .field("format_file", &"<callback>")
            .field("process_tailwind", &"<callback>")
            .finish()
    }
}

impl ExternalFormatter {
    /// Create an [`ExternalFormatter`] from JS callbacks.
    pub fn new(
        init_cb: JsInitExternalFormatterCb,
        format_embedded_cb: JsFormatEmbeddedCb,
        format_file_cb: JsFormatFileCb,
        tailwind_cb: JsTailwindCb,
    ) -> Self {
        let rust_init = wrap_init_external_formatter(init_cb);
        let rust_format_embedded = wrap_format_embedded(format_embedded_cb);
        let rust_format_file = wrap_format_file(format_file_cb);
        let rust_tailwind = wrap_tailwind(tailwind_cb);
        Self {
            init: rust_init,
            format_embedded: rust_format_embedded,
            format_file: rust_format_file,
            process_tailwind: rust_tailwind,
        }
    }

    /// Initialize external formatter using the JS callback.
    pub fn init(&self, num_threads: usize) -> Result<Vec<String>, String> {
        (self.init)(num_threads)
    }

    /// Convert this external formatter to the oxc_formatter::EmbeddedFormatter type.
    /// The options is captured in the closure and passed to JS on each call.
    pub fn to_embedded_formatter(&self, options: Value) -> oxc_formatter::EmbeddedFormatter {
        let format_embedded = Arc::clone(&self.format_embedded);
        let callback =
            Arc::new(move |tag_name: &str, code: &str| (format_embedded)(&options, tag_name, code));
        oxc_formatter::EmbeddedFormatter::new(callback)
    }

    /// Format non-js file using the JS callback.
    pub fn format_file(
        &self,
        options: &Value,
        parser_name: &str,
        file_name: &str,
        code: &str,
    ) -> Result<String, String> {
        (self.format_file)(options, parser_name, file_name, code)
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
fn wrap_init_external_formatter(cb: JsInitExternalFormatterCb) -> InitExternalFormatterCallback {
    Arc::new(move |num_threads: usize| {
        block_on(async {
            #[expect(clippy::cast_possible_truncation)]
            let status = cb.call_async(FnArgs::from((num_threads as u32,))).await;
            match status {
                Ok(promise) => match promise.await {
                    Ok(languages) => Ok(languages),
                    Err(err) => Err(format!("JS initExternalFormatter promise rejected: {err}")),
                },
                Err(err) => Err(format!("Failed to call JS initExternalFormatter callback: {err}")),
            }
        })
    })
}

/// Wrap JS `formatEmbeddedCode` callback as a normal Rust function.
fn wrap_format_embedded(cb: JsFormatEmbeddedCb) -> FormatEmbeddedWithConfigCallback {
    Arc::new(move |options: &Value, tag_name: &str, code: &str| {
        block_on(async {
            let status = cb
                .call_async(FnArgs::from((options.clone(), tag_name.to_string(), code.to_string())))
                .await;
            match status {
                Ok(promise) => match promise.await {
                    Ok(formatted_code) => Ok(formatted_code),
                    Err(err) => {
                        Err(format!("JS formatter promise rejected for tag '{tag_name}': {err}"))
                    }
                },
                Err(err) => Err(format!(
                    "Failed to call JS formatting callback for tag '{tag_name}': {err}"
                )),
            }
        })
    })
}

/// Wrap JS `formatFile` callback as a normal Rust function.
fn wrap_format_file(cb: JsFormatFileCb) -> FormatFileWithConfigCallback {
    Arc::new(move |options: &Value, parser_name: &str, file_name: &str, code: &str| {
        block_on(async {
            let status = cb
                .call_async(FnArgs::from((
                    options.clone(),
                    parser_name.to_string(),
                    file_name.to_string(),
                    code.to_string(),
                )))
                .await;
            match status {
                Ok(promise) => match promise.await {
                    Ok(formatted_code) => Ok(formatted_code),
                    Err(err) => Err(format!(
                        "JS formatFile promise rejected for file: '{file_name}', parser: '{parser_name}': {err}"
                    )),
                },
                Err(err) => Err(format!(
                    "Failed to call JS formatFile callback for file: '{file_name}', parser: '{parser_name}': {err}"
                )),
            }
        })
    })
}

/// Wrap JS `processTailwindClasses` callback as a normal Rust function.
fn wrap_tailwind(cb: JsTailwindCb) -> TailwindCallback {
    Arc::new(move |filepath: &str, classes: Vec<String>| {
        block_on(async {
            let args = FnArgs::from((filepath.to_string(), classes.clone()));
            match cb.call_async(args).await {
                Ok(promise) => match promise.await {
                    Ok(sorted) => sorted,
                    Err(_) => {
                        // Return original classes on error
                        classes
                    }
                },
                Err(_) => {
                    // Return original classes on error
                    classes
                }
            }
        })
    })
}
