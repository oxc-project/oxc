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
                    let mut oxlintrc: Oxlintrc = match serde_json::from_value(entry.config) {
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

                    // Check if extends is used - not yet supported
                    if !oxlintrc.extends.is_empty() {
                        errors.push(OxcDiagnostic::error(format!(
                            "`extends` in JavaScript configs is not yet supported (found in {})",
                            entry.path
                        )));
                        return (configs, errors);
                    }

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
