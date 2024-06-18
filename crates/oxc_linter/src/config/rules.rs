use std::{borrow::Cow, fmt, ops::Deref};

use oxc_diagnostics::{Error, OxcDiagnostic};
use rustc_hash::FxHashMap;
use schemars::{gen::SchemaGenerator, schema::Schema, JsonSchema};
use serde::{
    de::{self, Deserializer, Visitor},
    Deserialize,
};

use crate::AllowWarnDeny;

// TS type is `Record<string, RuleConf>`
//   - type SeverityConf = 0 | 1 | 2 | "off" | "warn" | "error";
//   - type RuleConf = SeverityConf | [SeverityConf, ...any[]];
// <https://github.com/eslint/eslint/blob/ce838adc3b673e52a151f36da0eedf5876977514/lib/shared/types.js#L12>
#[derive(Debug, Clone, Default)]
pub struct OxlintRules(Vec<ESLintRule>);

#[derive(Debug, Clone)]
pub struct ESLintRule {
    pub plugin_name: String,
    pub rule_name: String,
    pub severity: AllowWarnDeny,
    pub config: Option<serde_json::Value>,
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
            #[schemars(range(min = 0, max = 2.0))]
            Number(usize),
            String(String),
            Array(Vec<serde_json::Value>),
        }
        gen.subschema_for::<FxHashMap<String, DummyRule>>()
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
        return ("eslint".to_string(), name.to_string());
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

impl Deref for OxlintRules {
    type Target = Vec<ESLintRule>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn failed_to_parse_rule_value(value: &str, err: &str) -> OxcDiagnostic {
    OxcDiagnostic::error(format!("Failed to rule value {value:?} with error {err:?}"))
}

#[cfg(test)]
mod test {
    use serde::Deserialize;

    use super::OxlintRules;

    #[test]
    fn test_parse_rules() {
        let rules = OxlintRules::deserialize(&serde_json::json!({
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
}
