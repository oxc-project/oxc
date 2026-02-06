use serde::Deserialize;
use std::path::PathBuf;

use oxc_diagnostics::OxcDiagnostic;
use oxc_linter::Oxlintrc;

use crate::run::JsLoadJsConfigsCb;

/// Callback type for loading JavaScript/TypeScript config files.
pub type JsConfigLoaderCb =
    Box<dyn Fn(Vec<String>) -> Result<Vec<JsConfigResult>, Vec<OxcDiagnostic>> + Send + Sync>;

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

fn parse_js_oxlintrc(mut value: serde_json::Value) -> Result<Oxlintrc, OxcDiagnostic> {
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
                parse_js_oxlintrc(item)
            })
            .collect::<Result<Vec<_>, _>>()?
    } else {
        Vec::new()
    };

    let mut oxlintrc: Oxlintrc =
        serde_json::from_value(value).map_err(|err| OxcDiagnostic::error(err.to_string()))?;
    oxlintrc.extends_configs = extends_configs;
    Ok(oxlintrc)
}

/// Parse the JSON response from JS side into `JsConfigResult` structs.
fn parse_js_config_response(json: &str) -> Result<Vec<JsConfigResult>, Vec<OxcDiagnostic>> {
    let response: LoadJsConfigsResponse = serde_json::from_str(json).map_err(|e| {
        vec![OxcDiagnostic::error(format!("Failed to parse JS config response: {e}"))]
    })?;

    match response {
        LoadJsConfigsResponse::Success { success } => {
            let count = success.len();
            let (configs, errors) = success.into_iter().fold(
                (Vec::with_capacity(count), Vec::new()),
                |(mut configs, mut errors), entry| {
                    let path = PathBuf::from(&entry.path);
                    let mut oxlintrc = match parse_js_oxlintrc(entry.config) {
                        Ok(config) => config,
                        Err(err) => {
                            errors.push(
                                OxcDiagnostic::error(format!(
                                    "Failed to parse config from {}",
                                    entry.path
                                ))
                                .with_note(err.to_string()),
                            );
                            return (configs, errors);
                        }
                    };
                    oxlintrc.path.clone_from(&path);

                    let Some(config_dir_parent) = oxlintrc.path.parent() else {
                        errors.push(OxcDiagnostic::error(format!(
                            "Config path has no parent directory: {}",
                            entry.path
                        )));
                        return (configs, errors);
                    };
                    let config_dir = config_dir_parent.to_path_buf();
                    oxlintrc.set_config_dir(&config_dir);
                    configs.push(JsConfigResult { path, config: oxlintrc });

                    (configs, errors)
                },
            );

            if errors.is_empty() { Ok(configs) } else { Err(errors) }
        }
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
