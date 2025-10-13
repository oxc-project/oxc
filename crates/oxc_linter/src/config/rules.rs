use std::{borrow::Cow, fmt};

use itertools::Itertools;
use rustc_hash::FxHashMap;
use schemars::{JsonSchema, r#gen::SchemaGenerator, schema::Schema};
use serde::{
    Deserialize, Serialize, Serializer,
    de::{self, Deserializer, Visitor},
    ser::SerializeMap,
};

use oxc_diagnostics::{Error, OxcDiagnostic};

use crate::{
    AllowWarnDeny, ExternalPluginStore, LintPlugins,
    external_plugin_store::{ExternalRuleId, ExternalRuleLookupError},
    rules::{RULES, RuleEnum},
    utils::{is_eslint_rule_adapted_to_typescript, is_jest_rule_adapted_to_vitest},
};

type RuleSet = FxHashMap<RuleEnum, AllowWarnDeny>;

// TS type is `Record<string, RuleConf>`
//   - type SeverityConf = 0 | 1 | 2 | "off" | "warn" | "error";
//   - type RuleConf = SeverityConf | [SeverityConf, ...any[]];
// <https://github.com/eslint/eslint/blob/ce838adc3b673e52a151f36da0eedf5876977514/lib/shared/types.js#L12>
// Note: when update document comment, also update `DummyRuleMap`'s description in this file.
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct OxlintRules {
    /// List of all configured rules
    pub(crate) rules: Vec<ESLintRule>,
}

impl OxlintRules {
    pub fn new(rules: Vec<ESLintRule>) -> Self {
        Self { rules }
    }

    /// Returns `true` if there are no rules.
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }
}

/// A fully qualified rule name.
///
/// e.g. `eslint/no-console` or `react/rule-of-hooks`.
/// Includes the plugin name, the rule name, and the configuration for the rule (if any).
/// This does not imply the rule is known to the linter as that, only that it is configured.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ESLintRule {
    /// Name of the plugin: `eslint`, `react`, etc.
    pub plugin_name: String,
    /// Name of the rule: `no-console`, `prefer-const`, etc.
    pub rule_name: String,
    /// Severity of the rule: `off`, `warn`, `error`, etc.
    pub severity: AllowWarnDeny,
    /// JSON configuration for the rule, if any.
    pub config: Option<serde_json::Value>,
}

impl OxlintRules {
    pub(crate) fn override_rules(
        &self,
        rules_for_override: &mut RuleSet,
        external_rules_for_override: &mut FxHashMap<ExternalRuleId, AllowWarnDeny>,
        all_rules: &[RuleEnum],
        external_plugin_store: &ExternalPluginStore,
    ) -> Result<(), ExternalRuleLookupError> {
        let mut rules_to_replace = vec![];

        let lookup = self.rules.iter().into_group_map_by(|r| r.rule_name.as_str());

        for (name, rule_configs) in &lookup {
            let rules_map = rules_for_override
                .iter()
                .filter(|&(r, _)| r.name() == *name)
                .map(|(r, _)| (r.plugin_name(), r))
                .collect::<FxHashMap<_, _>>();

            for rule_config in rule_configs {
                let (rule_name, plugin_name) = transform_rule_and_plugin_name(
                    &rule_config.rule_name,
                    &rule_config.plugin_name,
                );
                let config = rule_config.config.clone().unwrap_or_default();
                let severity = rule_config.severity;

                if LintPlugins::try_from(plugin_name).is_ok() {
                    let rule = rules_map.get(&plugin_name).copied().or_else(|| {
                        all_rules
                            .iter()
                            .find(|r| r.name() == rule_name && r.plugin_name() == plugin_name)
                    });
                    if let Some(rule) = rule {
                        rules_to_replace.push((rule.read_json(config), severity));
                    }
                } else {
                    // If JS plugins are disabled (language server), assume plugin name refers to a JS plugin,
                    // and that rule name is valid for that plugin.
                    // But language server doesn't support JS plugins, so ignore the rule.
                    //
                    // This unfortunately means we can't catch genuinely invalid plugin names in language server
                    // (e.g. typos like `unicon/filename-case`). But we can't avoid this as the name of a JS plugin
                    // can only be known by loading it, which language server can't do at present.
                    if external_plugin_store.is_enabled() {
                        let external_rule_id =
                            external_plugin_store.lookup_rule_id(plugin_name, rule_name)?;
                        external_rules_for_override
                            .entry(external_rule_id)
                            .and_modify(|sev| *sev = severity)
                            .or_insert(severity);
                    }
                }
            }
        }

        for (rule, severity) in rules_to_replace {
            let _ = rules_for_override.remove(&rule);
            rules_for_override.insert(rule, severity);
        }

        Ok(())
    }
}

fn transform_rule_and_plugin_name<'a>(
    rule_name: &'a str,
    plugin_name: &'a str,
) -> (&'a str, &'a str) {
    let plugin_name = match plugin_name {
        "vitest" if is_jest_rule_adapted_to_vitest(rule_name) => "jest",
        "unicorn" if rule_name == "no-negated-condition" => "eslint",
        "typescript" if is_eslint_rule_adapted_to_typescript(rule_name) => "eslint",
        _ => plugin_name,
    };

    (rule_name, plugin_name)
}

impl JsonSchema for OxlintRules {
    fn schema_name() -> String {
        "OxlintRules".to_owned()
    }

    fn schema_id() -> Cow<'static, str> {
        Cow::Borrowed("OxlintRules")
    }

    fn json_schema(r#gen: &mut SchemaGenerator) -> Schema {
        #[expect(unused)]
        #[derive(Debug, Clone, JsonSchema)]
        #[serde(untagged)]
        enum DummyRule {
            Toggle(AllowWarnDeny),
            ToggleAndConfig(Vec<serde_json::Value>),
        }

        #[expect(unused)]
        #[derive(Debug, JsonSchema)]
        #[schemars(
            description = "See [Oxlint Rules](https://oxc.rs/docs/guide/usage/linter/rules.html)"
        )]
        struct DummyRuleMap(pub FxHashMap<String, DummyRule>);

        r#gen.subschema_for::<DummyRuleMap>()
    }
}

impl Serialize for OxlintRules {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut rules = s.serialize_map(Some(self.rules.len()))?;

        for rule in &self.rules {
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

                Ok(OxlintRules { rules })
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
                // plugins under the `eslint` scope are the only rules that are supported
                // to exist in the config file under just the rule name (no plugin)
                .map_or("eslint", RuleEnum::plugin_name)
                .to_string(),
            name.to_string(),
        );
    };

    let (oxlint_plugin_name, rule_name) = match plugin_name {
        "@typescript-eslint" => ("typescript", rule_name),
        // import-x has the same rules but better performance
        "import-x" => ("import", rule_name),
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
    /// `<rule_name>`.
    // This is effectively the inverse operation for `parse_rule_key`.
    pub fn full_name(&self) -> Cow<'_, str> {
        if self.plugin_name == "eslint" {
            Cow::Borrowed(self.rule_name.as_str())
        } else {
            Cow::Owned(format!("{}/{}", self.plugin_name, self.rule_name))
        }
    }
}

#[cfg(test)]
#[expect(clippy::default_trait_access)]
mod test {
    use rustc_hash::FxHashMap;
    use serde::Deserialize;
    use serde_json::{Value, json};

    use crate::{
        AllowWarnDeny, ExternalPluginStore,
        rules::{RULES, RuleEnum},
    };

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
        let mut rules = rules.rules.iter();

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
        assert_eq!(r3.plugin_name, "eslint");
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
        let mut external_rules_for_override = FxHashMap::default();
        let external_linter_store = ExternalPluginStore::default();
        rules_config
            .override_rules(rules, &mut external_rules_for_override, &RULES, &external_linter_store)
            .unwrap();
    }

    #[test]
    fn test_override_empty() {
        let mut rules = RuleSet::default();
        let configs = [json!({ "no-console": "error" }), json!({ "eslint/no-console": "error" })];

        for config in configs {
            rules.clear();
            r#override(&mut rules, &config);

            assert_eq!(rules.len(), 1, "{config:?}");
            let (rule, severity) = rules.iter().next().unwrap();
            assert_eq!(rule.name(), "no-console", "{config:?}");
            assert_eq!(severity, &AllowWarnDeny::Deny, "{config:?}");
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

        assert_eq!(
            rules.len(),
            1,
            "eslint rules should be configurable by their typescript-eslint reimplementations: {config:?}"
        );
        let (rule, severity) = rules.iter().next().unwrap();
        assert_eq!(
            rule.name(),
            "no-console",
            "eslint rules should be configurable by their typescript-eslint reimplementations: {config:?}"
        );
        assert_eq!(
            severity,
            &AllowWarnDeny::Deny,
            "eslint rules should be configurable by their typescript-eslint reimplementations: {config:?}"
        );
    }

    #[test]
    fn test_override_allow() {
        let mut rules = RuleSet::default();
        rules.insert(RuleEnum::EslintNoConsole(Default::default()), AllowWarnDeny::Deny);
        r#override(&mut rules, &json!({ "eslint/no-console": "off" }));

        assert!(!rules.iter().any(|(_, severity)| severity.is_warn_deny()));
    }

    #[test]
    fn test_override_plugin_prefix_duplicates() {
        let configs = [
            json!({ "@typescript-eslint/no-unused-vars": "error" }),
            json!({ "no-unused-vars": "off", "typescript/no-unused-vars": "error" }),
            json!({ "no-unused-vars": "off", "@typescript-eslint/no-unused-vars": "error" }),
        ];

        for config in &configs {
            let mut rules = RuleSet::default();
            r#override(&mut rules, config);

            assert_eq!(rules.len(), 1, "{config:?}");
            let (rule, severity) = rules.iter().next().unwrap();
            assert_eq!(rule.name(), "no-unused-vars", "{config:?}");
            assert_eq!(severity, &AllowWarnDeny::Deny, "{config:?}");
        }

        for config in &configs {
            let mut rules = RuleSet::default();
            rules.insert(RuleEnum::EslintNoUnusedVars(Default::default()), AllowWarnDeny::Warn);
            r#override(&mut rules, config);

            assert_eq!(rules.len(), 1, "{config:?}");
            let (rule, severity) = rules.iter().next().unwrap();
            assert_eq!(rule.name(), "no-unused-vars", "{config:?}");
            assert_eq!(severity, &AllowWarnDeny::Deny, "{config:?}");
        }
    }
}
