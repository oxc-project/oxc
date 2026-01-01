//! JavaScript config file loading support (experimental).
//!
//! This module provides support for loading `oxlint.config.ts` files using
//! Node.js native TypeScript support. This is an experimental feature.

use serde::Deserialize;
use std::path::PathBuf;

use oxc_linter::Oxlintrc;

use crate::run::JsLoadJsConfigsCb;

/// Callback type for loading JavaScript/TypeScript config files.
pub type JsConfigLoaderCb =
    Box<dyn Fn(Vec<String>) -> Result<Vec<JsConfigResult>, String> + Send + Sync>;

/// Result of loading a single JavaScript/TypeScript config file.
#[derive(Debug, Clone)]
pub struct JsConfigResult {
    pub path: PathBuf,
    pub config: Oxlintrc,
}

/// Response from JS side when loading JS configs.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum LoadJsConfigsResponse {
    Success {
        #[serde(rename = "Success")]
        success: Vec<JsConfigResultJson>,
    },
    Failure {
        #[serde(rename = "Failure")]
        failure: String,
    },
}

#[derive(Debug, Deserialize)]
struct JsConfigResultJson {
    path: String,
    config: serde_json::Value,
}

/// Create a JS config loader callback from the JS callback.
///
/// The returned function blocks the current thread until the JS callback resolves.
/// It will panic if called outside of a Tokio runtime.
pub fn create_js_config_loader(cb: JsLoadJsConfigsCb) -> JsConfigLoaderCb {
    Box::new(move |paths: Vec<String>| {
        let cb = &cb;
        let res = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current()
                .block_on(async move { cb.call_async(paths).await?.into_future().await })
        });

        match res {
            Ok(json) => parse_js_config_response(&json),
            Err(err) => Err(format!("`loadJsConfigs` threw an error: {err}")),
        }
    })
}

/// Parse the JSON response from JS side into `JsConfigResult` structs.
fn parse_js_config_response(json: &str) -> Result<Vec<JsConfigResult>, String> {
    let response: LoadJsConfigsResponse = serde_json::from_str(json)
        .map_err(|e| format!("Failed to parse JS config response: {e}"))?;

    match response {
        LoadJsConfigsResponse::Failure { failure } => Err(failure),
        LoadJsConfigsResponse::Success { success } => {
            success
                .into_iter()
                .map(|r| {
                    let path = PathBuf::from(&r.path);
                    let mut oxlintrc: Oxlintrc = serde_json::from_value(r.config)
                        .map_err(|e| format!("Failed to parse config from {}: {e}", r.path))?;

                    // Check if extends is used - not yet supported
                    if !oxlintrc.extends.is_empty() {
                        return Err(format!(
                            "`extends` in JavaScript configs is not yet supported (found in {})",
                            r.path
                        ));
                    }

                    oxlintrc.path.clone_from(&path);
                    Ok(JsConfigResult { path, config: oxlintrc })
                })
                .collect()
        }
    }
}
