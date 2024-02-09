mod env;
pub mod errors;
mod rules;
mod settings;

use std::path::Path;

use oxc_diagnostics::{Error, FailedToOpenFileError, Report};
use rustc_hash::FxHashSet;
use serde::Deserialize;

use crate::{rules::RuleEnum, AllowWarnDeny};

use self::errors::{
    FailedToParseConfigError, FailedToParseConfigJsonError, FailedToParseConfigPropertyError,
    FailedToParseJsonc,
};
pub use self::{env::ESLintEnv, rules::ESLintRules, settings::ESLintSettings};

/// ESLint Config
/// <https://eslint.org/docs/latest/use/configure/configuration-files-new#configuration-objects>
#[derive(Debug, Deserialize)]
pub struct ESLintConfig {
    #[serde(default)]
    rules: ESLintRules,
    #[serde(default)]
    settings: ESLintSettings,
    #[serde(default)]
    env: ESLintEnv,
}

impl ESLintConfig {
    pub fn from_file(path: &Path) -> Result<Self, Report> {
        let mut string = std::fs::read_to_string(path).map_err(|e| {
            FailedToParseConfigError(vec![Error::new(FailedToOpenFileError(path.to_path_buf(), e))])
        })?;

        // jsonc support
        json_strip_comments::strip(&mut string)
            .map_err(|_| FailedToParseJsonc(path.to_path_buf()))?;

        let json = serde_json::from_str::<serde_json::Value>(&string).map_err(|err| {
            let guess = mime_guess::from_path(path);
            let err = match guess.first() {
                // syntax error
                Some(mime) if mime.subtype() == "json" => err.to_string(),
                Some(_) => "only json configuration is supported".to_string(),
                None => {
                    format!(
                        "{err}, if the configuration is not a json file, please use json instead."
                    )
                }
            };
            FailedToParseConfigError(vec![Error::new(FailedToParseConfigJsonError(
                path.to_path_buf(),
                err,
            ))])
        })?;

        let config = Self::deserialize(&json).map_err(|err| {
            FailedToParseConfigError(vec![Error::new(FailedToParseConfigPropertyError(
                err.to_string(),
            ))])
        })?;

        Ok(config)
    }

    pub fn properties(self) -> (ESLintSettings, ESLintEnv) {
        (self.settings, self.env)
    }

    #[allow(clippy::option_if_let_else)]
    pub fn override_rules(
        &self,
        rules_for_override: &mut FxHashSet<RuleEnum>,
        all_rules: &[RuleEnum],
    ) {
        use itertools::Itertools;
        let mut rules_to_replace = vec![];
        let mut rules_to_remove = vec![];

        // Rules can have the same name but different plugin names
        let lookup = self.rules.iter().into_group_map_by(|r| r.rule_name.as_str());

        for (name, rule_configs) in &lookup {
            match rule_configs.len() {
                0 => unreachable!(),
                1 => {
                    let rule_config = &rule_configs[0];
                    let rule_name = &rule_config.rule_name;
                    let plugin_name = &rule_config.plugin_name;
                    match rule_config.severity {
                        AllowWarnDeny::Warn | AllowWarnDeny::Deny => {
                            if let Some(rule) = all_rules
                                .iter()
                                .find(|r| r.name() == rule_name && r.plugin_name() == plugin_name)
                            {
                                rules_to_replace.push(rule.read_json(rule_config.config.clone()));
                            }
                        }
                        AllowWarnDeny::Allow => {
                            if let Some(rule) = rules_for_override
                                .iter()
                                .find(|r| r.name() == rule_name && r.plugin_name() == plugin_name)
                            {
                                rules_to_remove.push(rule.clone());
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
                            rules_to_replace.push(rule.read_json(rule_config.config.clone()));
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

#[cfg(test)]
mod test {
    use super::ESLintConfig;
    use serde::Deserialize;
    use std::env;

    #[test]
    fn test_from_file() {
        let fixture_path = env::current_dir().unwrap().join("fixtures/eslint_config.json");
        let config = ESLintConfig::from_file(&fixture_path).unwrap();
        assert!(!config.rules.is_empty());
    }

    #[test]
    fn test_deserialize() {
        let config = ESLintConfig::deserialize(&serde_json::json!({
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
            "env": { "browser": true, }
        }));
        assert!(config.is_ok());

        let ESLintConfig { rules, settings, env } = config.unwrap();
        assert!(!rules.is_empty());
        assert_eq!(settings.jsx_a11y.polymorphic_prop_name, Some("role".to_string()));
        assert_eq!(env.iter().count(), 1);
    }
}
