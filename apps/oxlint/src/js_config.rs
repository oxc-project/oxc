use serde::Deserialize;
use std::path::PathBuf;

use oxc_diagnostics::OxcDiagnostic;
use oxc_linter::{Oxlintrc, OxlintrcExtendsEntry};

use crate::run::JsLoadJsConfigsCb;

/// Callback type for loading JavaScript/TypeScript config files.
pub type JsConfigLoaderCb =
    Box<dyn Fn(Vec<String>) -> Result<Vec<JsConfigResult>, Vec<OxcDiagnostic>> + Send + Sync>;

/// Result of loading a single JavaScript/TypeScript config file.
/// `config` is `None` when the JS side signals "skip" (e.g., vite.config.ts without `.lint` field).
#[derive(Debug, Clone)]
pub struct JsConfigResult {
    pub path: PathBuf,
    pub config: Option<Oxlintrc>,
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
    let (extends, extends_configs, extends_entries) = if let Some(extends_value) = extends_value {
        let serde_json::Value::Array(items) = extends_value else {
            return Err(OxcDiagnostic::error(
                "`extends` must be an array of config objects or strings.",
            ));
        };

        let mut extends = Vec::new();
        let mut extends_configs = Vec::new();
        let mut extends_entries = Vec::new();
        for (idx, item) in items.into_iter().enumerate() {
            match item {
                serde_json::Value::String(path) => {
                    let path = PathBuf::from(path);
                    extends.push(path.clone());
                    extends_entries.push(OxlintrcExtendsEntry::Path(path));
                }
                value if value.is_object() => {
                    let config = parse_js_oxlintrc(value)?;
                    extends_configs.push(config.clone());
                    extends_entries.push(OxlintrcExtendsEntry::Config(config));
                }
                _ => {
                    return Err(OxcDiagnostic::error(format!(
                        "`extends[{idx}]` must be a config object or string.",
                    )));
                }
            }
        }

        (extends, extends_configs, extends_entries)
    } else {
        (Vec::new(), Vec::new(), Vec::new())
    };

    let mut oxlintrc: Oxlintrc =
        serde_json::from_value(value).map_err(|err| OxcDiagnostic::error(err.to_string()))?;
    oxlintrc.extends = extends;
    oxlintrc.extends_configs = extends_configs;
    oxlintrc.extends_entries = extends_entries;
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

                    // `config: null` signals that this config should be skipped
                    // (e.g., vite.config.ts without a `.lint` field)
                    if entry.config.is_null() {
                        configs.push(JsConfigResult { path, config: None });
                        return (configs, errors);
                    }

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
                    configs.push(JsConfigResult { path, config: Some(oxlintrc) });

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

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use oxc_linter::{
        AllowWarnDeny, ConfigStoreBuilder, ExternalPluginStore, RuleCategory, rules::RULES,
    };
    use serde_json::json;

    use super::{parse_js_config_response, parse_js_oxlintrc};

    #[test]
    fn test_parse_js_oxlintrc_allows_string_and_object_extends() {
        let config = parse_js_oxlintrc(json!({
            "extends": [
                "oxlint-standard",
                { "rules": { "no-console": "error" } }
            ],
            "rules": { "no-debugger": "warn" }
        }))
        .unwrap();

        assert_eq!(config.extends, vec![PathBuf::from("oxlint-standard")]);
        assert_eq!(config.extends_configs.len(), 1);
        assert!(!config.rules.is_empty());
        assert!(!config.extends_configs[0].rules.is_empty());
    }

    #[test]
    fn test_parse_js_oxlintrc_rejects_non_string_non_object_extends() {
        let err = parse_js_oxlintrc(json!({ "extends": [42] })).unwrap_err();
        assert!(err.to_string().contains("`extends[0]` must be a config object or string."));
    }

    #[test]
    fn test_parse_js_oxlintrc_preserves_language_options_ids() {
        let config = parse_js_oxlintrc(json!({
            "_languageOptionsId": 7,
            "overrides": [{
                "files": ["**/*.svelte"],
                "_languageOptionsId": 9
            }]
        }))
        .unwrap();

        assert_eq!(config.language_options_ids, vec![7]);
        assert_eq!(config.overrides.iter().next().unwrap().language_options_id, Some(9));
    }

    #[test]
    fn test_parse_js_oxlintrc_preserves_custom_parser_flags() {
        let config = parse_js_oxlintrc(json!({
            "_languageOptionsHasParser": true,
            "overrides": [{
                "files": ["**/*.svelte"],
                "_languageOptionsHasParser": false
            }]
        }))
        .unwrap();

        assert_eq!(config.language_options_has_parser, Some(true));
        assert_eq!(
            config.overrides.iter().next().unwrap().language_options_has_parser,
            Some(false)
        );
    }

    #[test]
    fn test_parse_js_oxlintrc_allows_recommended_categories() {
        let config = parse_js_oxlintrc(json!({
            "categories": {
                "suspicious": "recommended"
            }
        }))
        .unwrap();

        assert_eq!(
            serde_json::to_value(&config.categories).unwrap(),
            json!({ "suspicious": "recommended" })
        );
    }

    #[test]
    fn test_parse_js_config_response_resolves_string_extends_in_builder() {
        let root_dir = tempfile::tempdir().unwrap();
        let config_path = root_dir.path().join("oxlint.config.ts");
        let base_path = root_dir.path().join("base.json");
        fs::write(&config_path, "export default {};\n").unwrap();
        fs::write(
            &base_path,
            r#"{
                "rules": {
                    "eqeqeq": "warn"
                }
            }"#,
        )
        .unwrap();

        let response = json!({
            "Success": [{
                "path": config_path,
                "config": {
                    "extends": ["./base.json"],
                    "rules": {
                        "no-debugger": "error"
                    }
                }
            }]
        });

        let parsed = parse_js_config_response(&response.to_string()).unwrap();
        let config = parsed.into_iter().next().unwrap().config.unwrap();
        assert_eq!(config.extends, vec![PathBuf::from("./base.json")]);

        let mut external_plugin_store = ExternalPluginStore::default();
        let builder =
            ConfigStoreBuilder::from_oxlintrc(true, config, None, &mut external_plugin_store, None)
                .unwrap();
        let config = builder.build(&mut external_plugin_store).unwrap();

        assert_eq!(
            config
                .rules()
                .iter()
                .find(|(rule, _)| rule.plugin_name() == "eslint" && rule.name() == "no-debugger")
                .map(|(_, severity)| *severity),
            Some(AllowWarnDeny::Deny)
        );
        assert_eq!(
            config
                .rules()
                .iter()
                .find(|(rule, _)| rule.plugin_name() == "eslint" && rule.name() == "eqeqeq")
                .map(|(_, severity)| *severity),
            Some(AllowWarnDeny::Warn)
        );
    }

    #[test]
    fn test_parse_js_config_response_preserves_recommended_categories_in_builder() {
        let root_dir = tempfile::tempdir().unwrap();
        let config_path = root_dir.path().join("oxlint.config.ts");
        fs::write(&config_path, "export default {};\n").unwrap();

        let response = json!({
            "Success": [{
                "path": config_path,
                "config": {
                    "plugins": ["react"],
                    "categories": {
                        "suspicious": "recommended"
                    }
                }
            }]
        });

        let parsed = parse_js_config_response(&response.to_string()).unwrap();
        let config = parsed.into_iter().next().unwrap().config.unwrap();
        assert_eq!(
            serde_json::to_value(&config.categories).unwrap(),
            json!({ "suspicious": "recommended" })
        );

        let mut external_plugin_store = ExternalPluginStore::default();
        let builder =
            ConfigStoreBuilder::from_oxlintrc(true, config, None, &mut external_plugin_store, None)
                .unwrap();
        let config = builder.build(&mut external_plugin_store).unwrap();

        assert!(config.rules().iter().any(|(rule, severity)| {
            rule.category() == RuleCategory::Suspicious && *severity == AllowWarnDeny::Warn
        }));
        assert_eq!(
            config
                .rules()
                .iter()
                .find(|(rule, _)| {
                    rule.category() == RuleCategory::Suspicious && rule.plugin_name() == "react"
                })
                .map(|(_, severity)| *severity),
            None
        );

        let react_suspicious_rule = RULES
            .iter()
            .find(|rule| {
                rule.category() == RuleCategory::Suspicious && rule.plugin_name() == "react"
            })
            .unwrap();
        assert!(
            !config.rules().iter().any(|(rule, _)| {
                rule.plugin_name() == react_suspicious_rule.plugin_name()
                    && rule.name() == react_suspicious_rule.name()
            }),
            "plugin-only suspicious rules should stay disabled for category-level recommended"
        );
    }

    #[test]
    fn test_parse_js_config_response_preserves_mixed_extends_order() {
        let root_dir = tempfile::tempdir().unwrap();
        let config_path = root_dir.path().join("oxlint.config.ts");
        let base_path = root_dir.path().join("base.json");
        fs::write(&config_path, "export default {};\n").unwrap();
        fs::write(
            &base_path,
            r#"{
                "rules": {
                    "no-debugger": "error"
                }
            }"#,
        )
        .unwrap();

        let response = json!({
            "Success": [{
                "path": config_path,
                "config": {
                    "extends": [
                        "./base.json",
                        { "rules": { "no-debugger": "warn" } }
                    ]
                }
            }]
        });

        let parsed = parse_js_config_response(&response.to_string()).unwrap();
        let config = parsed.into_iter().next().unwrap().config.unwrap();

        let mut external_plugin_store = ExternalPluginStore::default();
        let builder =
            ConfigStoreBuilder::from_oxlintrc(true, config, None, &mut external_plugin_store, None)
                .unwrap();
        let config = builder.build(&mut external_plugin_store).unwrap();

        assert_eq!(
            config
                .rules()
                .iter()
                .find(|(rule, _)| rule.plugin_name() == "eslint" && rule.name() == "no-debugger")
                .map(|(_, severity)| *severity),
            Some(AllowWarnDeny::Warn),
            "later inline object extends should override earlier string extends"
        );
    }

    #[test]
    fn test_parse_js_config_response_preserves_flat_config_compat_entries() {
        let root_dir = tempfile::tempdir().unwrap();
        let config_path = root_dir.path().join("oxlint.config.ts");
        fs::write(&config_path, "export default {};\n").unwrap();

        let response = json!({
            "Success": [{
                "path": config_path,
                "config": {
                    "extends": [
                        {
                            "jsPlugins": [{
                                "name": "svelte",
                                "specifier": "eslint-plugin-svelte"
                            }]
                        },
                        {
                            "overrides": [{
                                "files": ["**/*.svelte"],
                                "_languageOptionsId": 11,
                                "_languageOptionsHasParser": true,
                                "rules": {
                                    "svelte/no-useless-mustaches": "error"
                                }
                            }]
                        }
                    ]
                }
            }]
        });

        let parsed = parse_js_config_response(&response.to_string()).unwrap();
        let config = parsed.into_iter().next().unwrap().config.unwrap();

        assert_eq!(config.extends_configs.len(), 2);
        let first_extends = &config.extends_configs[0];
        let first_plugins = first_extends.external_plugins.as_ref().unwrap();
        assert!(first_plugins.iter().any(|entry| {
            entry.name.as_deref() == Some("svelte")
                && entry.specifier == "eslint-plugin-svelte"
        }));

        let second_extends = &config.extends_configs[1];
        let override_config = second_extends.overrides.iter().next().unwrap();
        assert_eq!(override_config.language_options_id, Some(11));
        assert_eq!(override_config.language_options_has_parser, Some(true));
        assert!(override_config.files.is_match("src/routes/App.svelte"));
        assert_eq!(
            serde_json::to_value(&override_config.rules).unwrap(),
            json!({
                "svelte/no-useless-mustaches": "deny"
            }),
            "Rust canonicalizes JS-config severity aliases like \"error\" to Oxlint's \"deny\" form after parsing"
        );
    }

    #[test]
    fn test_parse_js_config_response_preserves_flat_config_ignore_patterns() {
        let root_dir = tempfile::tempdir().unwrap();
        let config_path = root_dir.path().join("oxlint.config.ts");
        fs::write(&config_path, "export default {};\n").unwrap();

        let response = json!({
            "Success": [{
                "path": config_path,
                "config": {
                    "extends": [
                        {
                            "ignorePatterns": [".svelte-kit/**", "build/**"]
                        }
                    ]
                }
            }]
        });

        let parsed = parse_js_config_response(&response.to_string()).unwrap();
        let config = parsed.into_iter().next().unwrap().config.unwrap();

        assert_eq!(config.extends_configs.len(), 1);
        assert_eq!(
            config.extends_configs[0].ignore_patterns,
            vec![".svelte-kit/**".to_string(), "build/**".to_string()]
        );
    }

    #[test]
    fn test_parse_js_config_response_resolves_nested_object_extends_from_config_dir() {
        let root_dir = tempfile::tempdir().unwrap();
        let config_dir = root_dir.path().join("configs");
        fs::create_dir_all(&config_dir).unwrap();

        let config_path = config_dir.join("oxlint.config.ts");
        fs::write(&config_path, "export default {};\n").unwrap();

        fs::write(
            config_dir.join("base.json"),
            r#"{
                "rules": {
                    "no-alert": "warn"
                }
            }"#,
        )
        .unwrap();

        let response = json!({
            "Success": [{
                "path": config_path,
                "config": {
                    "extends": [
                        { "extends": ["./base.json"] }
                    ]
                }
            }]
        });

        let parsed = parse_js_config_response(&response.to_string()).unwrap();
        let config = parsed.into_iter().next().unwrap().config.unwrap();

        let mut external_plugin_store = ExternalPluginStore::default();
        let builder =
            ConfigStoreBuilder::from_oxlintrc(true, config, None, &mut external_plugin_store, None)
                .unwrap();
        let config = builder.build(&mut external_plugin_store).unwrap();

        assert_eq!(
            config
                .rules()
                .iter()
                .find(|(rule, _)| rule.plugin_name() == "eslint" && rule.name() == "no-alert")
                .map(|(_, severity)| *severity),
            Some(AllowWarnDeny::Warn)
        );
    }
}
