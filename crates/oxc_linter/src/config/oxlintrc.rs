use oxc_diagnostics::OxcDiagnostic;
use schemars::{schema::SingleOrVec, JsonSchema};
use serde::{Deserialize, Serialize};
use std::{fs::read_to_string, path::Path};

use crate::{
    config::{OxlintEnv, OxlintGlobals, OxlintRules, OxlintSettings},
    LintPlugins,
};

/// Oxlint Configuration File
///
/// This configuration is aligned with ESLint v8's configuration schema (`eslintrc.json`).
///
/// Usage: `oxlint -c oxlintrc.json`
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
///   "env": {
///       "browser": true
///   },
///   "globals": {
///     "foo": "readonly"
///   },
///   "settings": {
///   },
///   "rules": {
///       "eqeqeq": "warn"
///   }
///  }
/// ```
#[derive(Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
#[non_exhaustive]
pub struct Oxlintrc {
    // pub plugins: Vec<String>,
    pub plugins: LintPlugins,
    pub extends: Option<SingleOrVec<String>>,
    pub root: bool,
    pub settings: OxlintSettings,
    /// Environments enable and disable collections of global variables.
    pub env: OxlintEnv,
    /// Enabled or disabled specific global variables.
    pub globals: OxlintGlobals,
    /// See [Oxlint Rules](https://oxc.rs/docs/guide/usage/linter/rules.html).
    pub rules: OxlintRules,
}

impl Oxlintrc {
    /// # Errors
    ///
    /// * Parse Failure
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, OxcDiagnostic> {
        let path = path.as_ref();
        let mut string = read_to_string(path).map_err(|e| {
            OxcDiagnostic::error(format!("Failed to parse config {path:?} with error {e:?}"))
        })?;

        // jsonc support
        json_strip_comments::strip(&mut string).map_err(|err| {
            OxcDiagnostic::error(format!("Failed to parse jsonc file {path:?}: {err:?}"))
        })?;

        let json = serde_json::from_str::<serde_json::Value>(&string).map_err(|err| {
            let guess = mime_guess::from_path(path);
            let err = match guess.first() {
                // syntax error
                Some(mime) if mime.subtype() == "json" => err.to_string(),
                Some(_) => "Only json configuration is supported".to_string(),
                None => {
                    format!(
                        "{err}, if the configuration is not a json file, please use json instead."
                    )
                }
            };
            OxcDiagnostic::error(format!("Failed to parse eslint config {path:?}.\n{err}"))
        })?;

        let config = Self::deserialize(&json).map_err(|err| {
            OxcDiagnostic::error(format!("Failed to parse config with error {err:?}"))
        })?;

        Ok(config)
    }
}

#[cfg(test)]
mod test {
    use std::env;

    use oxc_span::CompactStr;

    use serde::Deserialize;

    use crate::LintPlugins;

    use super::Oxlintrc;

    #[test]
    fn test_from_file() {
        let fixture_path = env::current_dir().unwrap().join("fixtures/eslint_config.json");
        let config = Oxlintrc::from_file(&fixture_path).unwrap();
        assert!(!config.rules.is_empty());
        assert!(config.plugins.contains(LintPlugins::ESLINT));
        assert!(config.plugins.contains(LintPlugins::REACT));
        assert!(config.plugins.contains(LintPlugins::TYPESCRIPT));
        assert!(config.plugins.contains(LintPlugins::UNICORN));
        assert!(config.plugins.contains(LintPlugins::JSX_A11Y));
        assert!(!config.plugins.contains(LintPlugins::REACT_PERF));
    }

    #[test]
    fn test_deserialize() {
        let config = Oxlintrc::deserialize(&serde_json::json!({
            "rules": {
                "no-console": "off",
                "no-debugger": 2,
                "no-bitwise": [
                    "error",
                    { "allow": ["~"] }
                ],
                "eqeqeq": [
                    "error",
                    "always", { "null": "ignore" }, "foo"
                ],
                "@typescript-eslint/ban-types": "error",
                "jsx-a11y/alt-text": "warn",
                "@next/next/noop": [1]
            },
            "settings": {
                "jsx-a11y": {
                    "polymorphicPropName": "role",
                    "components": {
                        "Link": "Anchor",
                        "Link2": "Anchor2"
                    }
                },
            },
            "env": { "browser": true, },
            "globals": { "foo": "readonly", }
        }));
        assert!(config.is_ok());

        let Oxlintrc { rules, settings, env, globals, .. } = config.unwrap();
        assert!(!rules.is_empty());
        assert_eq!(
            settings.jsx_a11y.polymorphic_prop_name.as_ref().map(CompactStr::as_str),
            Some("role")
        );
        assert_eq!(env.iter().count(), 1);
        assert!(globals.is_enabled("foo"));
    }

    // #[test]
    // fn test_vitest_rule_replace() {
    //     let fixture_path: std::path::PathBuf =
    //         env::current_dir().unwrap().join("fixtures/eslint_config_vitest_replace.json");
    //     let config = Oxlintrc::from_file(&fixture_path).unwrap();
    //     let mut set = FxHashSet::default();
    //     config.override_rules(&mut set, &RULES);

    //     let rule = set.into_iter().next().unwrap();
    //     assert_eq!(rule.name(), "no-disabled-tests");
    //     assert_eq!(rule.plugin_name(), "jest");
    // }
}
