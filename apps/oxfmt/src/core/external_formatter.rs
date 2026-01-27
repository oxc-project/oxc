use std::{path::Path, sync::Arc};

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
/// Takes (options, parser_name, code) as separate arguments and returns formatted code.
pub type JsFormatEmbeddedCb = ThreadsafeFunction<
    // Input arguments
    FnArgs<(Value, String, String)>, // (options, parser_name, code)
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
/// Takes (filepath, options, classes) and returns sorted array.
pub type JsSortTailwindClassesCb = ThreadsafeFunction<
    FnArgs<(String, Value, Vec<String>)>, // Input: (filepath, options, classes)
    Promise<Vec<String>>,                 // Return: promise of sorted array
    FnArgs<(String, Value, Vec<String>)>,
    Status,
    false,
>;

/// Callback function type for formatting embedded code with config.
/// Takes (options, parser_name, code) and returns formatted code or an error.
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

/// Internal callback type for Tailwind processing with config.
/// Takes (filepath, options, classes) and returns sorted classes.
type TailwindWithConfigCallback =
    Arc<dyn Fn(&str, &Value, Vec<String>) -> Vec<String> + Send + Sync>;

/// External formatter that wraps a JS callback.
#[derive(Clone)]
pub struct ExternalFormatter {
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
    pub fn new(
        init_cb: JsInitExternalFormatterCb,
        format_embedded_cb: JsFormatEmbeddedCb,
        format_file_cb: JsFormatFileCb,
        sort_tailwindcss_classes_cb: JsSortTailwindClassesCb,
    ) -> Self {
        let rust_init = wrap_init_external_formatter(init_cb);
        let rust_format_embedded = wrap_format_embedded(format_embedded_cb);
        let rust_format_file = wrap_format_file(format_file_cb);
        let rust_tailwind = wrap_sort_tailwind_classes(sort_tailwindcss_classes_cb);
        Self {
            init: rust_init,
            format_embedded: rust_format_embedded,
            format_file: rust_format_file,
            sort_tailwindcss_classes: rust_tailwind,
        }
    }

    /// Initialize external formatter using the JS callback.
    pub fn init(&self, num_threads: usize) -> Result<Vec<String>, String> {
        (self.init)(num_threads)
    }

    /// Convert this external formatter to the oxc_formatter::ExternalCallbacks type.
    /// The filepath and options are captured in the closures and passed to JS on each call.
    pub fn to_external_callbacks(
        &self,
        filepath: &Path,
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
                (format_embedded)(&options_for_embedded, parser_name, code)
            }))
        } else {
            None
        };

        let needs_tailwind = format_options.experimental_tailwindcss.is_some();
        let tailwind_callback: Option<TailwindCallback> = if needs_tailwind {
            let file_path = filepath.to_string_lossy().to_string();
            let sort_tailwindcss_classes = Arc::clone(&self.sort_tailwindcss_classes);
            Some(Arc::new(move |classes: Vec<String>| {
                (sort_tailwindcss_classes)(&file_path, &options, classes)
            }))
        } else {
            None
        };

        ExternalCallbacks::new()
            .with_embedded_formatter(embedded_callback)
            .with_tailwind(tailwind_callback)
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

    #[cfg(test)]
    pub fn dummy() -> Self {
        // Currently, LSP tests are implemented in Rust, while our external formatter relies on JS.
        // Therefore, just provides a dummy external formatter that consistently returns errors.
        Self {
            init: Arc::new(|_| Err("Dummy init called".to_string())),
            format_embedded: Arc::new(|_, _, _| Err("Dummy format_embedded called".to_string())),
            format_file: Arc::new(|_, _, _, _| Err("Dummy format_file called".to_string())),
            sort_tailwindcss_classes: Arc::new(|_, _, _| vec![]),
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
fn wrap_init_external_formatter(cb: JsInitExternalFormatterCb) -> InitExternalFormatterCallback {
    Arc::new(move |num_threads: usize| {
        debug_span!("oxfmt::external::init", num_threads = num_threads).in_scope(|| {
            block_on(async {
                #[expect(clippy::cast_possible_truncation)]
                let status = cb.call_async(FnArgs::from((num_threads as u32,))).await;
                match status {
                    Ok(promise) => match promise.await {
                        Ok(languages) => Ok(languages),
                        Err(err) => {
                            Err(format!("JS initExternalFormatter promise rejected: {err}"))
                        }
                    },
                    Err(err) => {
                        Err(format!("Failed to call JS initExternalFormatter callback: {err}"))
                    }
                }
            })
        })
    })
}

/// Wrap JS `formatEmbeddedCode` callback as a normal Rust function.
fn wrap_format_embedded(cb: JsFormatEmbeddedCb) -> FormatEmbeddedWithConfigCallback {
    Arc::new(move |options: &Value, parser_name: &str, code: &str| {
        debug_span!("oxfmt::external::format_embedded", parser = %parser_name).in_scope(|| {
            block_on(async {
                let status = cb
                    .call_async(FnArgs::from((
                        options.clone(),
                        parser_name.to_string(),
                        code.to_string(),
                    )))
                    .await;
                match status {
                    Ok(promise) => match promise.await {
                        Ok(mut formatted_code) => {
                            // Trim trailing newline added by Prettier without allocation
                            let trimmed_len = formatted_code.trim_end().len();
                            formatted_code.truncate(trimmed_len);
                            Ok(formatted_code)
                        }
                        Err(err) => Err(format!(
                            "JS formatter promise rejected for parser '{parser_name}': {err}"
                        )),
                    },
                    Err(err) => Err(format!(
                        "Failed to call JS formatting callback for parser '{parser_name}': {err}"
                    )),
                }
            })
        })
    })
}

/// Wrap JS `formatFile` callback as a normal Rust function.
fn wrap_format_file(cb: JsFormatFileCb) -> FormatFileWithConfigCallback {
    Arc::new(move |options: &Value, parser_name: &str, file_name: &str, code: &str| {
        debug_span!("oxfmt::external::format_file", parser = %parser_name, file = %file_name).in_scope(|| {
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
    })
}

/// Wrap JS `sortTailwindClasses` callback as a normal Rust function.
fn wrap_sort_tailwind_classes(cb: JsSortTailwindClassesCb) -> TailwindWithConfigCallback {
    Arc::new(move |filepath: &str, options: &Value, classes: Vec<String>| {
        debug_span!("oxfmt::external::sort_tailwind", classes_count = classes.len()).in_scope(
            || {
                block_on(async {
                    let args =
                        FnArgs::from((filepath.to_string(), options.clone(), classes.clone()));
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
            },
        )
    })
}

/// Wrap JS `formatEmbeddedCode` callback for use in Prettier plugin context.
/// This creates an `EmbeddedFormatterCallback` that can be used by oxc_formatter.
pub fn wrap_format_embedded_only(
    cb: JsFormatEmbeddedCb,
    options: Value,
) -> EmbeddedFormatterCallback {
    let wrapped = wrap_format_embedded(cb);
    Arc::new(move |language: &str, code: &str| {
        let Some(parser_name) = language_to_prettier_parser(language) else {
            return Err(format!("Unsupported language: {language}"));
        };
        (wrapped)(&options, parser_name, code)
    })
}

/// Wrap JS `sortTailwindClasses` callback for use in Prettier plugin context.
/// This creates a `TailwindCallback` that can be used by oxc_formatter.
/// The filepath and options are captured in the closure.
pub fn wrap_sort_tailwind_for_doc(
    cb: JsSortTailwindClassesCb,
    filepath: String,
    options: Value,
) -> TailwindCallback {
    let wrapped = wrap_sort_tailwind_classes(cb);
    Arc::new(move |classes: Vec<String>| (wrapped)(&filepath, &options, classes))
}
