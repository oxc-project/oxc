mod rules;
mod settings;

use std::path::Path;

use indexmap::IndexMap;
use oxc_diagnostics::OxcDiagnostic;
use rustc_hash::{FxHashMap, FxHashSet};
use schematic::{
    schema::{JsonSchemaRenderer, SchemaGenerator},
    Config, ConfigEnum, ConfigLoader,
};
use serde::Deserialize;

use crate::{rules::RuleEnum, AllowWarnDeny, RuleWithSeverity};

pub use self::{
    rules::{parse_eslint_rules_config, ESLintRule, ESLintRules},
    settings::{jsdoc::JSDocPluginSettings, ESLintSettings},
};

// TODO: map out all the rule names
// https://github.com/SchemaStore/schemastore/blob/11399c32e4ae7dd7d4004e97e2fa0caede5a1273/src/schemas/json/eslintrc.json#L1414
pub type ESLintRulesConfig = IndexMap<String, ESLintRuleConfig>;

// TODO: add deprecated `false`
// <https://eslint.org/docs/v8.x/use/configure/language-options#using-configuration-files-1>
pub type ESLintGlobals = FxHashMap<String, GlobalValue>;

// TODO: list out the keys
// https://github.com/SchemaStore/schemastore/blob/11399c32e4ae7dd7d4004e97e2fa0caede5a1273/src/schemas/json/eslintrc.json#L1221
pub type ESLintEnv = FxHashMap<String, bool>;

#[derive(Debug, Eq, PartialEq, Clone, Copy, Deserialize, ConfigEnum)]
#[serde(rename_all = "lowercase")]
pub enum GlobalValue {
    Readonly,
    Writeable,
    Off,
}

impl GlobalValue {
    pub fn is_on(&self) -> bool {
        matches!(self, Self::Readonly | Self::Writeable)
    }

    pub fn is_off(&self) -> bool {
        matches!(self, Self::Off)
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, ConfigEnum)]
#[serde(rename_all = "lowercase")]
pub enum ESLintSeverityString {
    Off,
    Warn,
    #[serde(rename = "error")]
    E,
}

impl From<ESLintSeverityString> for AllowWarnDeny {
    fn from(value: ESLintSeverityString) -> Self {
        match value {
            ESLintSeverityString::Off => Self::Allow,
            ESLintSeverityString::Warn => Self::Warn,
            ESLintSeverityString::E => Self::Deny,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Deserialize, Config)]
pub enum ESLintRuleConfig {
    // TODO: validate 0 1 2
    Number(u8),
    #[serde(skip_serializing)]
    String(ESLintSeverityString),
    Vec(Vec<serde_json::Value>),
}

/// Oxlint Configuration
#[derive(Debug, Deserialize, Config)]
pub struct OxlintJsonConfig {
    #[serde(skip_serializing)]
    pub(crate) rules: ESLintRulesConfig,

    #[serde(skip_serializing)]
    pub(crate) settings: ESLintSettings,

    /// An environment defines global variables that are predefined.
    #[serde(skip_serializing)]
    pub(crate) env: ESLintEnv,

    /// Set each global variable name equal to true to allow the variable to be overwritten or false to disallow overwriting.
    #[serde(skip_serializing)]
    pub(crate) globals: ESLintGlobals,
}

#[derive(Debug, Default)]
pub struct ESLintConfig {
    pub(crate) rules: ESLintRules,
    pub(crate) settings: ESLintSettings,
    pub(crate) env: ESLintEnv,
    pub(crate) globals: ESLintGlobals,
}

impl ESLintConfig {
    /// # Errors
    ///
    /// * Parse Failure
    pub fn from_file(path: &Path) -> Result<Self, OxcDiagnostic> {
        let config = ConfigLoader::<OxlintJsonConfig>::new()
            .file(path)
            .and_then(|config| config.load())
            .map_err(|err| OxcDiagnostic::error(format!("{err}")))?
            .config;

        // let mut generator = SchemaGenerator::default();
        // generator.add::<OxlintJsonConfig>();
        // generator.generate("schema.json", JsonSchemaRenderer::default()).unwrap();

        Ok(ESLintConfig {
            rules: parse_eslint_rules_config(config.rules),
            settings: config.settings,
            env: config.env,
            globals: config.globals,
        })
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
                    let rule_name = &rule_config.rule_name;
                    let plugin_name = &rule_config.plugin_name;
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
            "env": { "browser": true, },
            "globals": { "foo": "readonly", }
        }));
        assert!(config.is_ok());

        let ESLintConfig { rules, settings, env, globals } = config.unwrap();
        assert!(!rules.is_empty());
        assert_eq!(settings.jsx_a11y.polymorphic_prop_name, Some("role".to_string()));
        assert_eq!(env.iter().count(), 1);
        assert!(globals.is_enabled("foo"));
    }
}
