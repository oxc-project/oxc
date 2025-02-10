use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use oxc_diagnostics::OxcDiagnostic;

use crate::utils::read_to_string;

use super::{
    categories::OxlintCategories, env::OxlintEnv, globals::OxlintGlobals,
    overrides::OxlintOverrides, plugins::LintPlugins, rules::OxlintRules, settings::OxlintSettings,
};

/// Oxlint Configuration File
///
/// This configuration is aligned with ESLint v8's configuration schema (`eslintrc.json`).
///
/// Usage: `oxlint -c oxlintrc.json --import-plugin`
///
/// ::: danger NOTE
///
/// Only the `.json` format is supported. You can use comments in configuration files.
///
/// :::
///
/// Example
///
/// `.oxlintrc.json`
///
/// ```json
/// {
///   "$schema": "./node_modules/oxlint/configuration_schema.json",
///   "plugins": ["import", "typescript", "unicorn"],
///   "env": {
///     "browser": true
///   },
///   "globals": {
///     "foo": "readonly"
///   },
///   "settings": {
///   },
///   "rules": {
///     "eqeqeq": "warn",
///     "import/no-cycle": "error"
///   },
///   "overrides": [
///     {
///       "files": ["*.test.ts", "*.spec.ts"],
///       "rules": {
///         "@typescript-eslint/no-explicit-any": "off"
///       }
///     }
///   ]
///  }
/// ```
#[derive(Debug, Default, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(default)]
#[non_exhaustive]
pub struct Oxlintrc {
    pub plugins: LintPlugins,
    pub categories: OxlintCategories,
    /// Example
    ///
    /// `.oxlintrc.json`
    ///
    /// ```json
    /// {
    ///   "$schema": "./node_modules/oxlint/configuration_schema.json",
    ///   "rules": {
    ///     "eqeqeq": "warn",
    ///     "import/no-cycle": "error",
    ///     "prefer-const": ["error", { "ignoreReadBeforeAssign": true }]
    ///   }
    ///  }
    /// ```
    ///
    /// See [Oxlint Rules](https://oxc.rs/docs/guide/usage/linter/rules.html) for the list of
    /// rules.
    pub rules: OxlintRules,
    pub settings: OxlintSettings,
    /// Environments enable and disable collections of global variables.
    pub env: OxlintEnv,
    /// Enabled or disabled specific global variables.
    pub globals: OxlintGlobals,
    /// Add, remove, or otherwise reconfigure rules for specific files or groups of files.
    #[serde(skip_serializing_if = "OxlintOverrides::is_empty")]
    pub overrides: OxlintOverrides,
    /// Absolute path to the configuration file.
    #[serde(skip)]
    pub path: PathBuf,
    /// Globs to ignore during linting. These are resolved from the configuration file path.
    #[serde(rename = "ignorePatterns")]
    pub ignore_patterns: Vec<String>,
}

impl Oxlintrc {
    /// # Errors
    ///
    /// * Parse Failure
    pub fn from_file(path: &Path) -> Result<Self, OxcDiagnostic> {
        let mut string = read_to_string(path).map_err(|e| {
            OxcDiagnostic::error(format!("Failed to parse config {path:?} with error {e:?}"))
        })?;

        // jsonc support
        json_strip_comments::strip(&mut string).map_err(|err| {
            OxcDiagnostic::error(format!("Failed to parse jsonc file {path:?}: {err:?}"))
        })?;

        let json = serde_json::from_str::<serde_json::Value>(&string).map_err(|err| {
            let ext = path.extension().and_then(OsStr::to_str);
            let err = match ext {
                // syntax error
                Some(ext) if is_json_ext(ext) => err.to_string(),
                Some(_) => "Only JSON configuration files are supported".to_string(),
                None => {
                    format!(
                        "{err}, if the configuration is not a JSON file, please use JSON instead."
                    )
                }
            };
            OxcDiagnostic::error(format!("Failed to parse eslint config {path:?}.\n{err}"))
        })?;

        let mut config = Self::deserialize(&json).map_err(|err| {
            OxcDiagnostic::error(format!("Failed to parse config with error {err:?}"))
        })?;

        config.path = path.to_path_buf();

        Ok(config)
    }
}

fn is_json_ext(ext: &str) -> bool {
    ext == "json" || ext == "jsonc"
}

#[cfg(test)]
mod test {
    use serde_json::json;

    use super::*;

    #[test]
    fn test_oxlintrc_de_empty() {
        let config: Oxlintrc = serde_json::from_value(json!({})).unwrap();
        assert_eq!(config.plugins, LintPlugins::default());
        assert_eq!(config.rules, OxlintRules::default());
        assert!(config.rules.is_empty());
        assert_eq!(config.settings, OxlintSettings::default());
        assert_eq!(config.env, OxlintEnv::default());
        assert_eq!(config.path, PathBuf::default());
    }

    #[test]
    fn test_oxlintrc_de_plugins_empty_array() {
        let config: Oxlintrc = serde_json::from_value(json!({ "plugins": [] })).unwrap();
        assert_eq!(config.plugins, LintPlugins::empty());
    }

    #[test]
    fn test_oxlintrc_empty_config_plugins() {
        let config: Oxlintrc = serde_json::from_str(r"{}").unwrap();
        assert_eq!(config.plugins, LintPlugins::default());
    }

    #[test]
    fn test_oxlintrc_specifying_plugins_will_override() {
        let config: Oxlintrc = serde_json::from_str(r#"{ "plugins": ["react", "oxc"] }"#).unwrap();
        assert_eq!(config.plugins, LintPlugins::REACT.union(LintPlugins::OXC));
        let config: Oxlintrc =
            serde_json::from_str(r#"{ "plugins": ["typescript", "unicorn"] }"#).unwrap();
        assert_eq!(config.plugins, LintPlugins::TYPESCRIPT.union(LintPlugins::UNICORN));
        let config: Oxlintrc =
            serde_json::from_str(r#"{ "plugins": ["typescript", "unicorn", "react", "oxc", "import", "jsdoc", "jest", "vitest", "jsx-a11y", "nextjs", "react-perf", "promise", "node"] }"#).unwrap();
        assert_eq!(config.plugins, LintPlugins::all());

        let config: Oxlintrc =
            serde_json::from_str(r#"{ "plugins": ["typescript", "@typescript-eslint"] }"#).unwrap();
        assert_eq!(config.plugins, LintPlugins::TYPESCRIPT);
    }
}
