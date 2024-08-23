mod env;
mod globals;
mod rules;
mod settings;

use std::path::Path;

use oxc_diagnostics::OxcDiagnostic;
use rustc_hash::FxHashSet;
use schemars::JsonSchema;
use serde::Deserialize;

pub use self::{
    env::OxlintEnv,
    globals::OxlintGlobals,
    rules::OxlintRules,
    settings::{jsdoc::JSDocPluginSettings, OxlintSettings},
};
use crate::{
    rules::RuleEnum,
    utils::{is_jest_rule_adapted_to_vitest, read_to_string},
    AllowWarnDeny, RuleWithSeverity,
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
#[derive(Debug, Default, Deserialize, JsonSchema)]
#[serde(default)]
pub struct OxlintConfig {
    /// See [Oxlint Rules](https://oxc.rs/docs/guide/usage/linter/rules.html).
    pub rules: OxlintRules,
    pub settings: OxlintSettings,
    /// Environments enable and disable collections of global variables.
    pub env: OxlintEnv,
    /// Enabled or disabled specific global variables.
    pub globals: OxlintGlobals,
}

/// Configuration used by the linter, fixer, and rules.
///
/// This is a mapping from the public [`OxlintConfig`] API to a trimmed down
/// version that is also better suited for internal use. Do not expose this
/// struct outside this crate.
#[derive(Debug, Default)]
pub(crate) struct LintConfig {
    pub(crate) settings: OxlintSettings,
    /// Environments enable and disable collections of global variables.
    pub(crate) env: OxlintEnv,
    /// Enabled or disabled specific global variables.
    pub(crate) globals: OxlintGlobals,
}

impl From<OxlintConfig> for LintConfig {
    fn from(config: OxlintConfig) -> Self {
        Self { settings: config.settings, env: config.env, globals: config.globals }
    }
}

impl OxlintConfig {
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

    #[allow(clippy::option_if_let_else)]
    pub fn override_rules(
        &self,
        rules_for_override: &mut FxHashSet<RuleWithSeverity>,
        all_rules: &[RuleEnum],
    ) {
        use itertools::Itertools;
        let mut rules_to_replace: Vec<RuleWithSeverity> = vec![];
        let mut rules_to_remove: Vec<RuleWithSeverity> = vec![];

        // Rules can have the same name but different plugin names
        let lookup = self.rules.iter().into_group_map_by(|r| r.rule_name.as_str());

        for (name, rule_configs) in &lookup {
            match rule_configs.len() {
                0 => unreachable!(),
                1 => {
                    let rule_config = &rule_configs[0];
                    let (rule_name, plugin_name) = transform_rule_and_plugin_name(
                        &rule_config.rule_name,
                        &rule_config.plugin_name,
                    );
                    let severity = rule_config.severity;
                    match severity {
                        AllowWarnDeny::Warn | AllowWarnDeny::Deny => {
                            if let Some(rule) = all_rules
                                .iter()
                                .find(|r| r.name() == rule_name && r.plugin_name() == plugin_name)
                            {
                                let config = rule_config.config.clone().unwrap_or_default();
                                let rule = rule.read_json(config);
                                rules_to_replace.push(RuleWithSeverity::new(rule, severity));
                            }
                        }
                        AllowWarnDeny::Allow => {
                            if let Some(rule) = rules_for_override
                                .iter()
                                .find(|r| r.name() == rule_name && r.plugin_name() == plugin_name)
                            {
                                let rule = rule.clone();
                                rules_to_remove.push(rule);
                            }
                        }
                    }
                }
                _ => {
                    // For overlapping rule names, use the "error" one
                    // "no-loss-of-precision": "off",
                    // "@typescript-eslint/no-loss-of-precision": "error"
                    if let Some(rule_config) =
                        rule_configs.iter().find(|r| r.severity.is_warn_deny())
                    {
                        if let Some(rule) = rules_for_override.iter().find(|r| r.name() == *name) {
                            let config = rule_config.config.clone().unwrap_or_default();
                            rules_to_replace
                                .push(RuleWithSeverity::new(rule.read_json(config), rule.severity));
                        }
                    } else if rule_configs.iter().all(|r| r.severity.is_allow()) {
                        if let Some(rule) = rules_for_override.iter().find(|r| r.name() == *name) {
                            rules_to_remove.push(rule.clone());
                        }
                    }
                }
            }
        }

        for rule in rules_to_remove {
            rules_for_override.remove(&rule);
        }
        for rule in rules_to_replace {
            rules_for_override.replace(rule);
        }
    }
}

fn transform_rule_and_plugin_name<'a>(
    rule_name: &'a str,
    plugin_name: &'a str,
) -> (&'a str, &'a str) {
    if plugin_name == "vitest" && is_jest_rule_adapted_to_vitest(rule_name) {
        return (rule_name, "jest");
    }

    (rule_name, plugin_name)
}

#[cfg(test)]
mod test {
    use std::env;

    use oxc_span::CompactStr;
    use rustc_hash::FxHashSet;
    use serde::Deserialize;

    use crate::rules::RULES;

    use super::OxlintConfig;

    #[test]
    fn test_from_file() {
        let fixture_path = env::current_dir().unwrap().join("fixtures/eslint_config.json");
        let config = OxlintConfig::from_file(&fixture_path).unwrap();
        assert!(!config.rules.is_empty());
    }

    #[test]
    fn test_deserialize() {
        let config = OxlintConfig::deserialize(&serde_json::json!({
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

        let OxlintConfig { rules, settings, env, globals } = config.unwrap();
        assert!(!rules.is_empty());
        assert_eq!(
            settings.jsx_a11y.polymorphic_prop_name.as_ref().map(CompactStr::as_str),
            Some("role")
        );
        assert_eq!(env.iter().count(), 1);
        assert!(globals.is_enabled("foo"));
    }

    #[test]
    fn test_vitest_rule_replace() {
        let fixture_path: std::path::PathBuf =
            env::current_dir().unwrap().join("fixtures/eslint_config_vitest_replace.json");
        let config = OxlintConfig::from_file(&fixture_path).unwrap();
        let mut set = FxHashSet::default();
        config.override_rules(&mut set, &RULES);

        let rule = set.into_iter().next().unwrap();
        assert_eq!(rule.name(), "no-disabled-tests");
        assert_eq!(rule.plugin_name(), "jest");
    }
}
