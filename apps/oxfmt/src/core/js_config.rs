use std::sync::Arc;

use napi::{
    Status,
    bindgen_prelude::Promise,
    threadsafe_function::ThreadsafeFunction,
};
use serde_json::Value;

/// JS callback to load a single JavaScript/TypeScript config file.
/// Takes an absolute path string and returns the config object directly.
pub type JsLoadJsConfigCb = ThreadsafeFunction<
    // Arguments: absolute path to config file
    String,
    // Return value: config object as serde_json::Value
    Promise<Value>,
    // Arguments (repeated)
    String,
    // Error status
    Status,
    // CalleeHandled
    false,
>;

/// Callback type for loading a JavaScript/TypeScript config file.
///
/// Wraps the NAPI `ThreadsafeFunction` and blocks the current thread
/// until the JS callback resolves.
/// Uses `Arc` so it can be shared across CLI, Stdin, and LSP code paths.
pub type JsConfigLoaderCb = Arc<dyn Fn(String) -> Result<Value, String> + Send + Sync>;

/// Create a JS config loader callback from the NAPI JS callback.
///
/// The returned function blocks the current thread until the JS callback resolves.
/// It will panic if called outside of a Tokio runtime.
pub fn create_js_config_loader(cb: JsLoadJsConfigCb) -> JsConfigLoaderCb {
    Arc::new(move |path: String| {
        let cb = &cb;
        let res = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current()
                .block_on(async move { cb.call_async(path).await?.into_future().await })
        });

        match res {
            Ok(value) => {
                if !value.is_object() {
                    return Err(
                        "Configuration file must have a default export that is an object."
                            .to_string(),
                    );
                }
                Ok(value)
            }
            Err(err) => Err(format!("`loadJsConfig` threw an error: {err}")),
        }
    })
}
