use serde::Deserialize;
use std::path::PathBuf;

use oxc_diagnostics::OxcDiagnostic;
use oxc_linter::Oxlintrc;

use crate::run::JsLoadJsConfigsCb;

/// Callback type for loading JavaScript/TypeScript config files.
///
/// Returns raw `serde_json::Value` per config file. The caller is responsible
/// for any field extraction (`--config-field`) and deserialization into `Oxlintrc`.
pub type JsConfigLoaderCb =
    Box<dyn Fn(Vec<String>) -> Result<Vec<JsRawConfigResult>, Vec<OxcDiagnostic>> + Send + Sync>;

/// Result of loading a single JavaScript/TypeScript config file.
///
/// Contains the raw JSON value before deserialization into `Oxlintrc`.
#[derive(Debug, Clone)]
pub struct JsRawConfigResult {
    pub path: PathBuf,
    pub value: serde_json::Value,
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
        #[serde(rename = "Failures")]
        failures: Vec<LoadJsConfigsResponseFailure>,
    },
    Error {
        #[serde(rename = "Error")]
        error: String,
    },
}

#[derive(Debug, Deserialize)]
struct LoadJsConfigsResponseFailure {
    path: String,
    error: String,
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
            Err(err) => {
                Err(vec![OxcDiagnostic::error(format!("`loadJsConfigs` threw an error: {err}"))])
            }
        }
    })
}

/// Deserialize a raw JS config `serde_json::Value` into `Oxlintrc`.
///
/// Handles the JS-config-specific `extends` field (which contains inline config
/// objects rather than file paths).
pub(crate) fn deserialize_js_config(
    mut value: serde_json::Value,
    path: &Path,
) -> Result<Oxlintrc, OxcDiagnostic> {
    let Some(map) = value.as_object_mut() else {
        return Err(OxcDiagnostic::error(
            "Configuration file must have a default export that is an object.",
        ));
    };

    let extends_value = map.remove("extends");
    let extends_configs = if let Some(extends_value) = extends_value {
        let serde_json::Value::Array(items) = extends_value else {
            return Err(OxcDiagnostic::error(
                "`extends` must be an array of config objects (strings/paths are not supported).",
            ));
        };

        items
            .into_iter()
            .enumerate()
            .map(|(idx, item)| {
                if !item.is_object() {
                    return Err(OxcDiagnostic::error(format!(
                        "`extends[{idx}]` must be a config object (strings/paths are not supported).",
                    )));
                }
                deserialize_js_config(item, path)
            })
            .collect::<Result<Vec<_>, _>>()?
    } else {
        Vec::new()
    };

    let mut oxlintrc: Oxlintrc =
        serde_json::from_value(value).map_err(|err| OxcDiagnostic::error(err.to_string()))?;
    oxlintrc.extends_configs = extends_configs;

    oxlintrc.path = path.to_path_buf();
    if let Some(config_dir) = path.parent() {
        oxlintrc.set_config_dir(&config_dir.to_path_buf());
    }

    Ok(oxlintrc)
}

use std::path::Path;

/// Parse the JSON response from JS side into raw config results.
fn parse_js_config_response(json: &str) -> Result<Vec<JsRawConfigResult>, Vec<OxcDiagnostic>> {
    let response: LoadJsConfigsResponse = serde_json::from_str(json).map_err(|e| {
        vec![OxcDiagnostic::error(format!("Failed to parse JS config response: {e}"))]
    })?;

    match response {
        LoadJsConfigsResponse::Success { success } => Ok(success
            .into_iter()
            .map(|entry| JsRawConfigResult { path: PathBuf::from(entry.path), value: entry.config })
            .collect()),
        LoadJsConfigsResponse::Failure { failures } => Err(failures
            .into_iter()
            .map(|failure| {
                OxcDiagnostic::error(format!(
                    "Failed to load config: {}\n\n{}",
                    failure.path, failure.error
                ))
            })
            .collect()),
        LoadJsConfigsResponse::Error { error } => {
            Err(vec![OxcDiagnostic::error(format!("Failed to load config files:\n\n{error}"))])
        }
    }
}
