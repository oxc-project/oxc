use std::sync::Arc;

use napi::{
    Status,
    bindgen_prelude::FnArgs,
    threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode},
};
use oxc_formatter::EmbeddedFormatterCallback;

/// Type alias for the callback function signature.
/// Takes (tag_name, code) as separate arguments and returns formatted code.
pub type JsFormatEmbeddedCb = ThreadsafeFunction<
    // Input arguments
    FnArgs<(String, String)>, // (tag_name, code) as separate arguments
    // Return type (what JS function returns)
    String,
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
    let cb = Arc::new(cb);
    Arc::new(move |tag_name: &str, code: &str| {
        let cb = Arc::clone(&cb);

        // Use a channel to capture the result from the JS callback
        let (tx, rx) = std::sync::mpsc::channel();

        let tag_name_str = tag_name.to_string();
        let code_str = code.to_string();

        // Call the JS function with separate arguments
        let status = cb.call_with_return_value(
            FnArgs::from((tag_name_str.clone(), code_str)),
            ThreadsafeFunctionCallMode::Blocking,
            move |result: Result<String, napi::Error>, _env| {
                // Send the result through the channel
                let _ = tx.send(result);
                Ok(())
            },
        );

        if status != napi::Status::Ok {
            return Err(format!(
                "Failed to call JS formatter for tag '{tag_name_str}': {status:?}"
            ));
        }

        // Wait for the result from the channel
        match rx.recv() {
            Ok(Ok(formatted)) => Ok(formatted),
            Ok(Err(e)) => Err(format!("JS formatter failed for tag '{tag_name_str}': {e}")),
            Err(_) => {
                Err(format!("Failed to receive result from JS formatter for tag '{tag_name_str}'"))
            }
        }
    })
}

/// Create an [`ExternalFormatter`] from JS callbacks.
pub fn create_external_formatter(format_embedded_cb: JsFormatEmbeddedCb) -> ExternalFormatter {
    let rust_format_embedded = wrap_format_embedded(format_embedded_cb);
    ExternalFormatter::new(rust_format_embedded)
}
