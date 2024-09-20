use std::path::Path;

use oxc_diagnostics::OxcDiagnostic;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{env::OxlintEnv, globals::OxlintGlobals, rules::OxlintRules, settings::OxlintSettings};

use crate::{options::LintPlugins, utils::read_to_string};

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
///   }
///  }
/// ```
#[derive(Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(default)]
#[non_exhaustive]
pub struct Oxlintrc {
    pub plugins: LintPlugins,
    /// See [Oxlint Rules](https://oxc.rs/docs/guide/usage/linter/rules.html).
    pub rules: OxlintRules,
    pub settings: OxlintSettings,
    /// Environments enable and disable collections of global variables.
    pub env: OxlintEnv,
    /// Enabled or disabled specific global variables.
    pub globals: OxlintGlobals,
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

    // #[allow(clippy::option_if_let_else)]
    // pub fn override_rules(
    //     &self,
    //     rules_for_override: &mut FxHashSet<RuleWithSeverity>,
    //     all_rules: &[RuleEnum],
    // ) {
    //     use itertools::Itertools;
    //     let mut rules_to_replace: Vec<RuleWithSeverity> = vec![];
    //     let mut rules_to_remove: Vec<RuleWithSeverity> = vec![];

    //     // Rules can have the same name but different plugin names
    //     let lookup = self.rules.iter().into_group_map_by(|r| r.rule_name.as_str());

    //     for (name, rule_configs) in &lookup {
    //         match rule_configs.len() {
    //             0 => unreachable!(),
    //             1 => {
    //                 let rule_config = &rule_configs[0];
    //                 let (rule_name, plugin_name) = transform_rule_and_plugin_name(
    //                     &rule_config.rule_name,
    //                     &rule_config.plugin_name,
    //                 );
    //                 let severity = rule_config.severity;
    //                 match severity {
    //                     AllowWarnDeny::Warn | AllowWarnDeny::Deny => {
    //                         if let Some(rule) = all_rules
    //                             .iter()
    //                             .find(|r| r.name() == rule_name && r.plugin_name() == plugin_name)
    //                         {
    //                             let config = rule_config.config.clone().unwrap_or_default();
    //                             let rule = rule.read_json(config);
    //                             rules_to_replace.push(RuleWithSeverity::new(rule, severity));
    //                         }
    //                     }
    //                     AllowWarnDeny::Allow => {
    //                         if let Some(rule) = rules_for_override
    //                             .iter()
    //                             .find(|r| r.name() == rule_name && r.plugin_name() == plugin_name)
    //                         {
    //                             let rule = rule.clone();
    //                             rules_to_remove.push(rule);
    //                         }
    //                     }
    //                 }
    //             }
    //             _ => {
    //                 // For overlapping rule names, use the "error" one
    //                 // "no-loss-of-precision": "off",
    //                 // "@typescript-eslint/no-loss-of-precision": "error"
    //                 if let Some(rule_config) =
    //                     rule_configs.iter().find(|r| r.severity.is_warn_deny())
    //                 {
    //                     if let Some(rule) = rules_for_override.iter().find(|r| r.name() == *name) {
    //                         let config = rule_config.config.clone().unwrap_or_default();
    //                         rules_to_replace
    //                             .push(RuleWithSeverity::new(rule.read_json(config), rule.severity));
    //                     }
    //                 } else if rule_configs.iter().all(|r| r.severity.is_allow()) {
    //                     if let Some(rule) = rules_for_override.iter().find(|r| r.name() == *name) {
    //                         rules_to_remove.push(rule.clone());
    //                     }
    //                 }
    //             }
    //         }
    //     }

    //     for rule in rules_to_remove {
    //         rules_for_override.remove(&rule);
    //     }
    //     for rule in rules_to_replace {
    //         rules_for_override.replace(rule);
    //     }
    // }
}
