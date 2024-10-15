use std::{borrow::Cow, fmt, ops::Deref};

use oxc_diagnostics::{Error, OxcDiagnostic};
use rustc_hash::{FxHashMap, FxHashSet};
use schemars::{gen::SchemaGenerator, schema::Schema, JsonSchema};
use serde::{
    de::{self, Deserializer, Visitor},
    ser::SerializeMap,
    Deserialize, Serialize,
};

use crate::{
    rules::{RuleEnum, RULES},
    utils::is_jest_rule_adapted_to_vitest,
    AllowWarnDeny, RuleWithSeverity,
};

type RuleSet = FxHashSet<RuleWithSeverity>;

// TS type is `Record<string, RuleConf>`
//   - type SeverityConf = 0 | 1 | 2 | "off" | "warn" | "error";
//   - type RuleConf = SeverityConf | [SeverityConf, ...any[]];
// <https://github.com/eslint/eslint/blob/ce838adc3b673e52a151f36da0eedf5876977514/lib/shared/types.js#L12>
// Note: when update document comment, also update `DummyRuleMap`'s description in this file.
#[derive(Debug, Clone, Default)]
#[cfg_attr(test, derive(PartialEq))]
pub struct OxlintRules(Vec<ESLintRule>);

#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub struct ESLintRule {
    pub plugin_name: String,
    pub rule_name: String,
    pub severity: AllowWarnDeny,
    pub config: Option<serde_json::Value>,
}

impl Deref for OxlintRules {
    type Target = Vec<ESLintRule>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl IntoIterator for OxlintRules {
    type Item = ESLintRule;
    type IntoIter = <Vec<ESLintRule> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl OxlintRules {
    #[allow(clippy::option_if_let_else)]
    pub(crate) fn override_rules(&self, rules_for_override: &mut RuleSet, all_rules: &[RuleEnum]) {
        use itertools::Itertools;
        let mut rules_to_replace: Vec<RuleWithSeverity> = vec![];
        let mut rules_to_remove: Vec<RuleWithSeverity> = vec![];

        // Rules can have the same name but different plugin names
        let lookup = self.iter().into_group_map_by(|r| r.rule_name.as_str());

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

impl JsonSchema for OxlintRules {
    fn schema_name() -> String {
        "OxlintRules".to_owned()
    }

    fn schema_id() -> Cow<'static, str> {
        Cow::Borrowed("OxlintRules")
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        #[allow(unused)]
        #[derive(Debug, Clone, JsonSchema)]
        #[serde(untagged)]
        enum DummyRule {
            Toggle(AllowWarnDeny),
            ToggleAndConfig(Vec<serde_json::Value>),
        }

        #[allow(unused)]
        #[derive(Debug, JsonSchema)]
        #[schemars(
            description = "See [Oxlint Rules](https://oxc.rs/docs/guide/usage/linter/rules.html)"
        )]
        struct DummyRuleMap(pub FxHashMap<String, DummyRule>);

        gen.subschema_for::<DummyRuleMap>()
    }
}

impl Serialize for OxlintRules {
    fn serialize<S>(&self, s: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut rules = s.serialize_map(Some(self.len()))?;

        for rule in &self.0 {
            let key = rule.full_name();
            match rule.config.as_ref() {
                // e.g. unicorn/some-rule: ["warn", { foo: "bar" }]
                Some(config) if !config.is_null() => {
                    let value = (rule.severity.as_str(), config);
                    rules.serialize_entry(&key, &value)?;
                }
                // e.g. unicorn/some-rule: "warn"
                _ => {
                    rules.serialize_entry(&key, rule.severity.as_str())?;
                }
            }
        }

        rules.end()
    }
}

// Manually implement Deserialize because the type is a bit complex...
// - Handle single value form and array form
// - SeverityConf into AllowWarnDeny
// - Align plugin names
impl<'de> Deserialize<'de> for OxlintRules {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct OxlintRulesVisitor;

        impl<'de> Visitor<'de> for OxlintRulesVisitor {
            type Value = OxlintRules;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("Record<string, SeverityConf | [SeverityConf, ...any[]]>")
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: de::MapAccess<'de>,
            {
                let mut rules = vec![];
                while let Some((key, value)) = map.next_entry::<String, serde_json::Value>()? {
                    let (plugin_name, rule_name) = parse_rule_key(&key);
                    let (severity, config) = parse_rule_value(&value).map_err(de::Error::custom)?;
                    rules.push(ESLintRule { plugin_name, rule_name, severity, config });
                }

                Ok(OxlintRules(rules))
            }
        }

        deserializer.deserialize_any(OxlintRulesVisitor)
    }
}

fn parse_rule_key(name: &str) -> (String, String) {
    let Some((plugin_name, rule_name)) = name.split_once('/') else {
        return (
            RULES
                .iter()
                .find(|r| r.name() == name)
                .map_or("unknown_plugin", RuleEnum::plugin_name)
                .to_string(),
            name.to_string(),
        );
    };

    let (oxlint_plugin_name, rule_name) = match plugin_name {
        "@typescript-eslint" => ("typescript", rule_name),
        "jsx-a11y" => ("jsx_a11y", rule_name),
        "react-perf" => ("react_perf", rule_name),
        // e.g. "@next/next/google-font-display"
        "@next" => ("nextjs", rule_name.trim_start_matches("next/")),
        // For backwards compatibility, react hook rules reside in the react plugin.
        "react-hooks" => ("react", rule_name),
        // For backwards compatibility, deepscan rules reside in the oxc plugin.
        "deepscan" => ("oxc", rule_name),
        _ => (plugin_name, rule_name),
    };

    (oxlint_plugin_name.to_string(), rule_name.to_string())
}

fn parse_rule_value(
    value: &serde_json::Value,
) -> Result<(AllowWarnDeny, Option<serde_json::Value>), Error> {
    match value {
        serde_json::Value::String(_) | serde_json::Value::Number(_) => {
            let severity = AllowWarnDeny::try_from(value)?;
            Ok((severity, None))
        }

        serde_json::Value::Array(v) => {
            if v.is_empty() {
                return Err(failed_to_parse_rule_value(
                    &value.to_string(),
                    "Type should be `[SeverityConf, ...any[]`",
                )
                .into());
            }

            // The first item should be SeverityConf
            let severity = AllowWarnDeny::try_from(v.first().unwrap())?;
            // e.g. ["warn"], [0]
            let config = if v.len() == 1 {
                None
            // e.g. ["error", "args", { type: "whatever" }, ["len", "also"]]
            } else {
                Some(serde_json::Value::Array(v.iter().skip(1).cloned().collect::<Vec<_>>()))
            };

            Ok((severity, config))
        }

        _ => Err(failed_to_parse_rule_value(
            &value.to_string(),
            "Type should be `SeverityConf | [SeverityConf, ...any[]]`",
        )
        .into()),
    }
}

fn failed_to_parse_rule_value(value: &str, err: &str) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("Failed to rule value {value:?} with error {err:?}"))
}

impl ESLintRule {
    /// Returns `<plugin_name>/<rule_name>` for non-eslint rules. For eslint rules, returns
    /// `<rule_name>`. This is effectively the inverse operation for [`parse_rule_key`].
    fn full_name(&self) -> Cow<'_, str> {
        if self.plugin_name == "eslint" {
            Cow::Borrowed(self.rule_name.as_str())
        } else {
            Cow::Owned(format!("{}/{}", self.plugin_name, self.rule_name))
        }
    }
}

#[cfg(test)]
#[allow(clippy::default_trait_access)]
mod test {
    use crate::{
        rules::{RuleEnum, RULES},
        AllowWarnDeny, RuleWithSeverity,
    };
    use serde::Deserialize;
    use serde_json::{json, Value};

    use super::{OxlintRules, RuleSet};

    #[test]
    fn test_parse_rules() {
        let rules = OxlintRules::deserialize(&json!({
            "no-console": "off",
            "foo/no-unused-vars": [1],
            "dummy": ["error", "arg1", "args2"],
            "@next/next/noop": 2,
        }))
        .unwrap();
        let mut rules = rules.iter();

        let r1 = rules.next().unwrap();
        assert_eq!(r1.rule_name, "no-console");
        assert_eq!(r1.plugin_name, "eslint");
        assert!(r1.severity.is_allow());
        assert!(r1.config.is_none());

        let r2 = rules.next().unwrap();
        assert_eq!(r2.rule_name, "no-unused-vars");
        assert_eq!(r2.plugin_name, "foo");
        assert!(r2.severity.is_warn_deny());
        assert!(r2.config.is_none());

        let r3 = rules.next().unwrap();
        assert_eq!(r3.rule_name, "dummy");
        assert_eq!(r3.plugin_name, "unknown_plugin");
        assert!(r3.severity.is_warn_deny());
        assert_eq!(r3.config, Some(serde_json::json!(["arg1", "args2"])));

        let r4 = rules.next().unwrap();
        assert_eq!(r4.rule_name, "noop");
        assert_eq!(r4.plugin_name, "nextjs");
        assert!(r4.severity.is_warn_deny());
        assert!(r4.config.is_none());
    }

    #[test]
    fn test_parse_rules_default() {
        let rules = OxlintRules::default();
        assert!(rules.is_empty());
    }

    fn r#override(rules: &mut RuleSet, rules_rc: &Value) {
        let rules_config = OxlintRules::deserialize(rules_rc).unwrap();
        rules_config.override_rules(rules, &RULES);
    }

    #[test]
    fn test_override_empty() {
        let mut rules = RuleSet::default();
        let configs = [json!({ "no-console": "error" }), json!({ "eslint/no-console": "error" })];

        for config in configs {
            rules.clear();
            r#override(&mut rules, &config);

            assert_eq!(rules.len(), 1, "{config:?}");
            let rule = rules.iter().next().unwrap();
            assert_eq!(rule.name(), "no-console", "{config:?}");
            assert_eq!(rule.severity, AllowWarnDeny::Deny, "{config:?}");
        }
    }

    // FIXME
    #[test]
    #[should_panic(
        expected = "eslint rules should be configurable by their typescript-eslint reimplementations:"
    )]
    fn test_override_empty_fixme() {
        let config = json!({ "@typescript-eslint/no-console": "error" });
        let mut rules = RuleSet::default();

        rules.clear();
        r#override(&mut rules, &config);

        assert_eq!(rules.len(), 1, "eslint rules should be configurable by their typescript-eslint reimplementations: {config:?}");
        let rule = rules.iter().next().unwrap();
        assert_eq!(rule.name(), "no-console", "eslint rules should be configurable by their typescript-eslint reimplementations: {config:?}");
        assert_eq!(rule.severity, AllowWarnDeny::Deny, "eslint rules should be configurable by their typescript-eslint reimplementations: {config:?}");
    }

    #[test]
    fn test_override_allow() {
        let mut rules = RuleSet::default();
        rules.insert(RuleWithSeverity {
            rule: RuleEnum::NoConsole(Default::default()),
            severity: AllowWarnDeny::Deny,
        });
        r#override(&mut rules, &json!({ "eslint/no-console": "off" }));

        assert!(rules.is_empty());
    }

    #[test]
    fn test_override_plugin_prefix_duplicates() {
        let configs = [
            // json!({ "@typescript-eslint/no-unused-vars": "error" }),
            json!({ "no-unused-vars": "off", "typescript/no-unused-vars": "error" }),
            json!({ "no-unused-vars": "off", "@typescript-eslint/no-unused-vars": "error" }),
        ];

        for config in configs {
            let mut rules = RuleSet::default();
            r#override(&mut rules, &config);

            // FIXME: this fails, meaning the behavior with two rules (in different plugins) does
            // not match the behavior of a single rule in a oxlintrc.
            // assert_eq!(rules.len(), 1, "{config:?}");
            // let rule = rules.iter().next().unwrap();
            // assert_eq!(rule.name(), "no-unused-vars", "{config:?}");
            // assert_eq!(rule.severity, AllowWarnDeny::Deny, "{config:?}");

            // rules = RuleSet::default();
            rules.insert(RuleWithSeverity {
                rule: RuleEnum::NoUnusedVars(Default::default()),
                severity: AllowWarnDeny::Warn,
            });
            r#override(&mut rules, &config);

            assert_eq!(rules.len(), 1, "{config:?}");
            let rule = rules.iter().next().unwrap();
            assert_eq!(rule.name(), "no-unused-vars", "{config:?}");
            assert_eq!(rule.severity, AllowWarnDeny::Warn, "{config:?}");
        }
    }
}
