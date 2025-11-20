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

/// External formatter that wraps a JS callback.
#[derive(Clone)]
pub struct ExternalFormatter {
    pub format_embedded: EmbeddedFormatterCallback,
}

impl std::fmt::Debug for ExternalFormatter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExternalFormatter").field("format_embedded", &"<callback>").finish()
    }
}

impl ExternalFormatter {
    pub fn new(format_embedded: EmbeddedFormatterCallback) -> Self {
        Self { format_embedded }
    }

    /// Convert this external formatter to the oxc_formatter::EmbeddedFormatter type
    pub fn to_embedded_formatter(&self) -> oxc_formatter::EmbeddedFormatter {
        let callback = Arc::clone(&self.format_embedded);
        // The callback already expects &str, so just use it directly
        oxc_formatter::EmbeddedFormatter::new(callback)
    }
}

/// Wrap JS `formatEmbeddedCode` callback as a normal Rust function.
///
/// Uses a channel to capture the result from the JS callback.
pub fn wrap_format_embedded(cb: JsFormatEmbeddedCb) -> EmbeddedFormatterCallback {
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

/// Create an [`ExternalFormatter`] from JS callbacks.
pub fn create_external_formatter(format_embedded_cb: JsFormatEmbeddedCb) -> ExternalFormatter {
    let rust_format_embedded = wrap_format_embedded(format_embedded_cb);
    ExternalFormatter::new(rust_format_embedded)
}
