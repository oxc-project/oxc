use std::sync::Arc;

use napi::{
    Status,
    bindgen_prelude::{FnArgs, Promise, block_on},
    threadsafe_function::ThreadsafeFunction,
};
use oxc_formatter::EmbeddedFormatterCallback;

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
/// Takes (tag_name, code) as separate arguments and returns formatted code.
pub type JsFormatFileCb = ThreadsafeFunction<
    // Input arguments
    FnArgs<(String, String)>, // (file_name, code) as separate arguments
    // Return type (what JS function returns)
    Promise<String>,
    // Arguments (repeated)
    FnArgs<(String, String)>,
    // Error status
    Status,
    // CalleeHandled
    false,
>;

/// Callback function type for formatting files.
/// Takes (file_name, code) and returns formatted code or an error.
type FileFormatterCallback = Arc<dyn Fn(&str, &str) -> Result<String, String> + Send + Sync>;

/// External formatter that wraps a JS callback.
#[derive(Clone)]
pub struct ExternalFormatter {
    pub format_embedded: EmbeddedFormatterCallback,
    pub format_file: FileFormatterCallback,
}

impl std::fmt::Debug for ExternalFormatter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExternalFormatter")
            .field("format_embedded", &"<callback>")
            .field("format_file", &"<callback>")
            .finish()
    }
}

impl ExternalFormatter {
    /// Create an [`ExternalFormatter`] from JS callbacks.
    pub fn new(format_embedded_cb: JsFormatEmbeddedCb, format_file_cb: JsFormatFileCb) -> Self {
        let rust_format_embedded = wrap_format_embedded(format_embedded_cb);
        let rust_format_file = wrap_format_file(format_file_cb);
        Self { format_embedded: rust_format_embedded, format_file: rust_format_file }
    }

    /// Convert this external formatter to the oxc_formatter::EmbeddedFormatter type
    pub fn to_embedded_formatter(&self) -> oxc_formatter::EmbeddedFormatter {
        let callback = Arc::clone(&self.format_embedded);
        // The callback already expects &str, so just use it directly
        oxc_formatter::EmbeddedFormatter::new(callback)
    }

    /// Format non-js file using the JS callback.
    pub fn format_file(&self, file_name: &str, code: &str) -> Result<String, String> {
        (self.format_file)(file_name, code)
    }
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
fn wrap_format_file(cb: JsFormatFileCb) -> EmbeddedFormatterCallback {
    Arc::new(move |file_name: &str, code: &str| {
        block_on(async {
            let status =
                cb.call_async(FnArgs::from((file_name.to_string(), code.to_string()))).await;
            match status {
                Ok(promise) => match promise.await {
                    Ok(formatted_code) => Ok(formatted_code),
                    Err(err) => {
                        Err(format!("JS formatter promise rejected for file '{file_name}': {err}"))
                    }
                },
                Err(err) => Err(format!(
                    "Failed to call JS formatting callback for file '{file_name}': {err}"
                )),
            }
        })
    })
}
