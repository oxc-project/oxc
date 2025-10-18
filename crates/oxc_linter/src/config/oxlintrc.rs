use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use rustc_hash::{FxHashMap, FxHashSet};
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use oxc_diagnostics::OxcDiagnostic;

use crate::{LintPlugins, utils::read_to_string};

use super::{
    categories::OxlintCategories, env::OxlintEnv, globals::OxlintGlobals,
    overrides::OxlintOverrides, rules::OxlintRules, settings::OxlintSettings,
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
///     "import/no-cycle": "error",
///     "react/self-closing-comp": ["error", { "html": false }]
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
    pub plugins: Option<LintPlugins>,
    /// JS plugins.
    ///
    /// Note: JS plugins are experimental and not subject to semver.
    /// They are not supported in language server at present.
    #[serde(
        rename = "jsPlugins",
        deserialize_with = "deserialize_external_plugins",
        serialize_with = "serialize_external_plugins",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    #[schemars(with = "Option<FxHashSet<String>>")]
    pub external_plugins: Option<FxHashSet<(PathBuf, String)>>,
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
    /// Paths of configuration files that this configuration file extends (inherits from). The files
    /// are resolved relative to the location of the configuration file that contains the `extends`
    /// property. The configuration files are merged from the first to the last, with the last file
    /// overriding the previous ones.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub extends: Vec<PathBuf>,
}

impl Oxlintrc {
    /// # Errors
    ///
    /// * Parse Failure
    pub fn from_file(path: &Path) -> Result<Self, OxcDiagnostic> {
        let mut string = read_to_string(path).map_err(|e| {
            OxcDiagnostic::error(format!(
                "Failed to parse config {} with error {e:?}",
                path.display()
            ))
        })?;

        // jsonc support
        json_strip_comments::strip(&mut string).map_err(|err| {
            OxcDiagnostic::error(format!("Failed to parse jsonc file {}: {err:?}", path.display()))
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
            OxcDiagnostic::error(format!(
                "Failed to parse eslint config {}.\n{err}",
                path.display()
            ))
        })?;

        let mut config = Self::deserialize(&json).map_err(|err| {
            OxcDiagnostic::error(format!("Failed to parse config with error {err:?}"))
        })?;

        config.path = path.to_path_buf();

        #[expect(clippy::missing_panics_doc)]
        let config_dir = config.path.parent().unwrap();
        if let Some(external_plugins) = &mut config.external_plugins {
            *external_plugins = std::mem::take(external_plugins)
                .into_iter()
                .map(|(_, specifier)| (config_dir.to_path_buf(), specifier))
                .collect();
        }

        for override_config in config.overrides.iter_mut() {
            if let Some(external_plugins) = &mut override_config.external_plugins {
                *external_plugins = std::mem::take(external_plugins)
                    .into_iter()
                    .map(|(_, specifier)| (config_dir.to_path_buf(), specifier))
                    .collect();
            }
        }

        Ok(config)
    }

    /// # Errors
    ///
    /// * Parse Failure
    pub fn from_string(json_string: &str) -> Result<Self, OxcDiagnostic> {
        let json = serde_json::from_str::<serde_json::Value>(json_string)
            .unwrap_or(serde_json::Value::Null);

        Self::deserialize(&json).map_err(|err| {
            OxcDiagnostic::error(format!("Failed to parse config with error {err:?}"))
        })
    }

    /// Merges two [Oxlintrc] files together
    /// [Self] takes priority over `other`
    #[must_use]
    pub fn merge(&self, other: Oxlintrc) -> Oxlintrc {
        let mut categories = other.categories.clone();
        categories.extend(self.categories.iter());

        let rules = self
            .rules
            .rules
            .iter()
            .chain(&other.rules.rules)
            .fold(FxHashMap::default(), |mut rules_set, rule| {
                if rules_set.contains_key(&(&rule.plugin_name, &rule.rule_name)) {
                    return rules_set;
                }
                rules_set.insert((&rule.plugin_name, &rule.rule_name), rule);
                rules_set
            })
            .values()
            .map(|rule| (**rule).clone())
            .collect::<Vec<_>>();

        let settings = self.settings.clone();
        let env = self.env.clone();
        let globals = self.globals.clone();

        let mut overrides = self.overrides.clone();
        overrides.extend(other.overrides);

        let plugins = match (self.plugins, other.plugins) {
            (Some(self_plugins), Some(other_plugins)) => Some(self_plugins | other_plugins),
            (Some(self_plugins), None) => Some(self_plugins),
            (None, other_plugins) => other_plugins,
        };

        let external_plugins = match (&self.external_plugins, &other.external_plugins) {
            (Some(self_external), Some(other_external)) => {
                Some(self_external.iter().chain(other_external).cloned().collect())
            }
            (Some(self_external), None) => Some(self_external.clone()),
            (None, Some(other_external)) => Some(other_external.clone()),
            (None, None) => None,
        };

        Oxlintrc {
            plugins,
            external_plugins,
            categories,
            rules: OxlintRules::new(rules),
            settings,
            env,
            globals,
            overrides,
            path: self.path.clone(),
            ignore_patterns: self.ignore_patterns.clone(),
            extends: self.extends.clone(),
        }
    }
}

fn is_json_ext(ext: &str) -> bool {
    ext == "json" || ext == "jsonc"
}

fn deserialize_external_plugins<'de, D>(
    deserializer: D,
) -> Result<Option<FxHashSet<(PathBuf, String)>>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt_set: Option<FxHashSet<String>> = Option::deserialize(deserializer)?;
    Ok(opt_set
        .map(|set| set.into_iter().map(|specifier| (PathBuf::default(), specifier)).collect()))
}

#[expect(clippy::ref_option)]
fn serialize_external_plugins<S>(
    plugins: &Option<FxHashSet<(PathBuf, String)>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    // Serialize as an array of original specifiers (the values in the map)
    match plugins {
        Some(set) => serializer.collect_seq(set.iter().map(|(_, specifier)| specifier)),
        None => serializer.serialize_none(),
    }
}

#[cfg(test)]
mod test {
    use serde_json::json;

    use crate::config::plugins::LintPlugins;

    use super::*;

    #[test]
    fn test_oxlintrc_de_empty() {
        let config: Oxlintrc = serde_json::from_value(json!({})).unwrap();
        assert_eq!(config.plugins, None);
        assert_eq!(config.rules, OxlintRules::default());
        assert!(config.rules.is_empty());
        assert_eq!(config.settings, OxlintSettings::default());
        assert_eq!(config.env, OxlintEnv::default());
        assert_eq!(config.path, PathBuf::default());
        assert_eq!(config.extends, Vec::<PathBuf>::default());
    }

    #[test]
    fn test_oxlintrc_de_plugins_empty_array() {
        let config: Oxlintrc = serde_json::from_value(json!({ "plugins": [] })).unwrap();
        assert_eq!(config.plugins, Some(LintPlugins::empty()));
    }

    #[test]
    fn test_oxlintrc_empty_config_plugins() {
        let config: Oxlintrc = serde_json::from_str(r"{}").unwrap();
        assert_eq!(config.plugins, None);
    }

    #[test]
    fn test_oxlintrc_specifying_plugins_will_override() {
        let config: Oxlintrc = serde_json::from_str(r#"{ "plugins": ["react", "oxc"] }"#).unwrap();

        assert_eq!(config.plugins, Some(LintPlugins::REACT | LintPlugins::OXC));
        let config: Oxlintrc =
            serde_json::from_str(r#"{ "plugins": ["typescript", "unicorn"] }"#).unwrap();
        assert_eq!(config.plugins, Some(LintPlugins::TYPESCRIPT | LintPlugins::UNICORN));
        let config: Oxlintrc =
            serde_json::from_str(r#"{ "plugins": ["typescript", "unicorn", "react", "oxc", "import", "jsdoc", "jest", "vitest", "jsx-a11y", "nextjs", "react-perf", "promise", "node", "regex", "vue"] }"#).unwrap();
        assert_eq!(config.plugins, Some(LintPlugins::all()));

        let config: Oxlintrc =
            serde_json::from_str(r#"{ "plugins": ["typescript", "@typescript-eslint"] }"#).unwrap();
        assert_eq!(config.plugins, Some(LintPlugins::TYPESCRIPT));
    }

    #[test]
    fn test_oxlintrc_extends() {
        let config: Oxlintrc = serde_json::from_str(
            r#"{"extends": [".oxlintrc.json", "./oxlint-ts.json", "../.config/.oxlintrc.json"]}"#,
        )
        .unwrap();
        assert_eq!(
            config.extends,
            vec![
                PathBuf::from(".oxlintrc.json"),
                PathBuf::from("./oxlint-ts.json"),
                PathBuf::from("../.config/.oxlintrc.json")
            ]
        );

        let config: Oxlintrc = serde_json::from_str(r#"{"extends": []}"#).unwrap();
        assert_eq!(0, config.extends.len());
    }
}
