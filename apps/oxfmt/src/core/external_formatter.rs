use std::sync::Arc;

use napi::{
    Status,
    bindgen_prelude::{FnArgs, Promise, block_on},
    threadsafe_function::ThreadsafeFunction,
};
use tokio::task::block_in_place;

use oxc_formatter::EmbeddedFormatterCallback;

/// Type alias for the setup config callback function signature.
/// Takes (config_json, num_threads) as arguments and returns plugin languages.
pub type JsSetupConfigCb = ThreadsafeFunction<
    // Input arguments
    FnArgs<(String, u32)>, // (config_json, num_threads)
    // Return type (what JS function returns)
    Promise<Vec<String>>,
    // Arguments (repeated)
    FnArgs<(String, u32)>,
    // Error status
    Status,
    // CalleeHandled
    false,
>;

/// Type alias for the callback function signature.
/// Takes (tag_name, code) as separate arguments and returns formatted code.
pub type JsFormatEmbeddedCb = ThreadsafeFunction<
    // Input arguments
    FnArgs<(String, String)>, // (tag_name, code) as separate arguments
    // Return type (what JS function returns)
    Promise<String>,
    // Arguments (repeated)
    FnArgs<(String, String)>,
    // Error status
    Status,
    // CalleeHandled
    false,
>;

/// Type alias for the callback function signature.
/// Takes (parser_name, file_name, code) as separate arguments and returns formatted code.
pub type JsFormatFileCb = ThreadsafeFunction<
    // Input arguments
    FnArgs<(String, String, String)>, // (parser_name, file_name, code) as separate arguments
    // Return type (what JS function returns)
    Promise<String>,
    // Arguments (repeated)
    FnArgs<(String, String, String)>,
    // Error status
    Status,
    // CalleeHandled
    false,
>;

/// Type alias for Tailwind class processing callback.
/// Takes array of class strings and returns nothing (for POC logging).
pub type JsTailwindCb = ThreadsafeFunction<
    Vec<String>, // Input: array of class strings
    Promise<()>, // Return: void promise
    Vec<String>,
    Status,
    false,
>;

/// Callback function type for formatting files.
/// Takes (parser_name, file_name, code) and returns formatted code or an error.
type FormatFileCallback = Arc<dyn Fn(&str, &str, &str) -> Result<String, String> + Send + Sync>;

/// Callback function type for setup config.
/// Takes (config_json, num_threads) and returns plugin languages.
type SetupConfigCallback = Arc<dyn Fn(&str, usize) -> Result<Vec<String>, String> + Send + Sync>;

/// Callback function type for processing Tailwind classes.
/// Takes Vec of class strings found in JSX attributes.
type TailwindCallback = Arc<dyn Fn(Vec<String>) + Send + Sync>;

/// External formatter that wraps a JS callback.
#[derive(Clone)]
pub struct ExternalFormatter {
    pub setup_config: SetupConfigCallback,
    pub format_embedded: EmbeddedFormatterCallback,
    pub format_file: FormatFileCallback,
    pub process_tailwind: Option<TailwindCallback>,
}

impl std::fmt::Debug for ExternalFormatter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExternalFormatter")
            .field("setup_config", &"<callback>")
            .field("format_embedded", &"<callback>")
            .field("format_file", &"<callback>")
            .field("process_tailwind", &"<callback>")
            .finish()
    }
}

impl ExternalFormatter {
    /// Create an [`ExternalFormatter`] from JS callbacks.
    pub fn new(
        setup_config_cb: JsSetupConfigCb,
        format_embedded_cb: JsFormatEmbeddedCb,
        format_file_cb: JsFormatFileCb,
        tailwind_cb: JsTailwindCb,
    ) -> Self {
        let rust_setup_config = wrap_setup_config(setup_config_cb);
        let rust_format_embedded = wrap_format_embedded(format_embedded_cb);
        let rust_format_file = wrap_format_file(format_file_cb);
        let rust_tailwind = wrap_tailwind(tailwind_cb);

        Self {
            setup_config: rust_setup_config,
            format_embedded: rust_format_embedded,
            format_file: rust_format_file,
            process_tailwind: Some(rust_tailwind),
        }
    }

    /// Setup Prettier config using the JS callback.
    pub fn setup_config(
        &self,
        config_json: &str,
        num_threads: usize,
    ) -> Result<Vec<String>, String> {
        (self.setup_config)(config_json, num_threads)
    }

    /// Convert this external formatter to the oxc_formatter::EmbeddedFormatter type
    pub fn to_embedded_formatter(&self) -> oxc_formatter::EmbeddedFormatter {
        let callback = Arc::clone(&self.format_embedded);
        // The callback already expects &str, so just use it directly
        oxc_formatter::EmbeddedFormatter::new(callback)
    }

    /// Format non-js file using the JS callback.
    pub fn format_file(
        &self,
        parser_name: &str,
        file_name: &str,
        code: &str,
    ) -> Result<String, String> {
        (self.format_file)(parser_name, file_name, code)
    }
}

// ---

/// Wrap JS `setupConfig` callback as a normal Rust function.
// NOTE: Use `block_in_place()` because this is called from a sync context, unlike the others
fn wrap_setup_config(cb: JsSetupConfigCb) -> SetupConfigCallback {
    Arc::new(move |config_json: &str, num_threads: usize| {
        block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                #[expect(clippy::cast_possible_truncation)]
                let status = cb
                    .call_async(FnArgs::from((config_json.to_string(), num_threads as u32)))
                    .await;
                match status {
                    Ok(promise) => match promise.await {
                        Ok(languages) => Ok(languages),
                        Err(err) => Err(format!("JS setupConfig promise rejected: {err}")),
                    },
                    Err(err) => Err(format!("Failed to call JS setupConfig callback: {err}")),
                }
            })
        })
    })
}

/// Wrap JS `formatEmbeddedCode` callback as a normal Rust function.
fn wrap_format_embedded(cb: JsFormatEmbeddedCb) -> EmbeddedFormatterCallback {
    Arc::new(move |tag_name: &str, code: &str| {
        block_on(async {
            let status =
                cb.call_async(FnArgs::from((tag_name.to_string(), code.to_string()))).await;
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
fn wrap_format_file(cb: JsFormatFileCb) -> FormatFileCallback {
    Arc::new(move |parser_name: &str, file_name: &str, code: &str| {
        block_on(async {
            let status = cb
                .call_async(FnArgs::from((
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
    Arc::new(move |classes: Vec<String>| {
        // Use block_on to call async function synchronously (like formatEmbedded)
        // This is acceptable for a logging callback
        let _ = block_on(async {
            match cb.call_async(classes).await {
                Ok(promise) => {
                    if let Err(err) = promise.await {
                        eprintln!("[oxfmt] Tailwind callback promise rejected: {}", err);
                    }
                }
                Err(err) => {
                    eprintln!("[oxfmt] Failed to call Tailwind callback: {}", err);
                }
            }
        });
    })
}
