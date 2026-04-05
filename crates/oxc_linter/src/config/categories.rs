use std::{
    borrow::Cow,
    ops::{Deref, DerefMut},
};

use rustc_hash::FxHashMap;
use schemars::{
    JsonSchema,
    schema::{InstanceType, Schema, SchemaObject, SingleOrVec, SubschemaValidation},
};
use serde::{Deserialize, Serialize, de};

use crate::{AllowWarnDeny, RuleCategory, RuleEnum};

use super::plugins::LintPlugins;

const RECOMMENDED_CATEGORY_SEVERITY: AllowWarnDeny = AllowWarnDeny::Warn;

/// Configure a category as either a full severity or Oxlint's built-in category-default subset.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum CategoryConfig {
    Severity(AllowWarnDeny),
    Recommended,
}

impl From<AllowWarnDeny> for CategoryConfig {
    fn from(value: AllowWarnDeny) -> Self {
        Self::Severity(value)
    }
}

impl Serialize for CategoryConfig {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Severity(severity) => severity.serialize(serializer),
            Self::Recommended => serializer.serialize_str("recommended"),
        }
    }
}

impl CategoryConfig {
    pub fn severity_for_rule(self, rule: &RuleEnum) -> Option<AllowWarnDeny> {
        match self {
            Self::Severity(severity) => Some(severity),
            Self::Recommended => {
                is_category_default_rule(rule).then_some(RECOMMENDED_CATEGORY_SEVERITY)
            }
        }
    }
}

impl<'de> Deserialize<'de> for CategoryConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Repr<'a> {
            Int(u8),
            Str(Cow<'a, str>),
        }

        match Repr::deserialize(deserializer)? {
            Repr::Int(value) => AllowWarnDeny::try_from(u64::from(value))
                .map(Self::Severity)
                .map_err(de::Error::custom),
            Repr::Str(value) => {
                if value == "recommended" {
                    Ok(Self::Recommended)
                } else {
                    AllowWarnDeny::try_from(value.as_ref())
                        .map(Self::Severity)
                        .map_err(de::Error::custom)
                }
            }
        }
    }
}

impl JsonSchema for CategoryConfig {
    fn schema_name() -> String {
        "CategoryConfig".to_string()
    }

    fn schema_id() -> Cow<'static, str> {
        Cow::Borrowed("CategoryConfig")
    }

    fn json_schema(r#gen: &mut schemars::r#gen::SchemaGenerator) -> Schema {
        let severity = r#gen.subschema_for::<AllowWarnDeny>();

        let mut recommended = SchemaObject::default();
        recommended.instance_type = Some(SingleOrVec::Single(Box::new(InstanceType::String)));
        recommended.enum_values = Some(vec![serde_json::Value::String("recommended".into())]);

        let mut schema = SchemaObject::default();
        schema.subschemas = Some(Box::new(SubschemaValidation {
            any_of: Some(vec![severity, recommended.into()]),
            ..Default::default()
        }));
        schema.into()
    }
}

/// Configure an entire category of rules all at once.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct OxlintCategories(FxHashMap<RuleCategory, CategoryConfig>);

impl Deref for OxlintCategories {
    type Target = FxHashMap<RuleCategory, CategoryConfig>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for OxlintCategories {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl OxlintCategories {
    pub fn insert<C: Into<CategoryConfig>>(
        &mut self,
        category: RuleCategory,
        config: C,
    ) -> Option<CategoryConfig> {
        self.0.insert(category, config.into())
    }

    pub fn severity_for_rule(&self, rule: &RuleEnum) -> Option<AllowWarnDeny> {
        self.get(&rule.category()).and_then(|config| config.severity_for_rule(rule))
    }
}

pub(crate) fn is_category_default_rule(rule: &RuleEnum) -> bool {
    // Category-level `recommended` currently maps to Oxlint's built-in category defaults:
    // core ESLint rules plus rules from Oxlint's default built-in plugins.
    LintPlugins::try_from(rule.plugin_name())
        .is_ok_and(|plugin| plugin.is_empty() || LintPlugins::default().contains(plugin))
}

impl JsonSchema for OxlintCategories {
    fn schema_id() -> Cow<'static, str> {
        Cow::Borrowed("OxlintCategories")
    }

    fn schema_name() -> String {
        "OxlintCategories".to_string()
    }

    fn json_schema(r#gen: &mut schemars::r#gen::SchemaGenerator) -> schemars::schema::Schema {
        let category_config = CategoryConfig::json_schema(r#gen);
        let mut schema = SchemaObject::default();
        schema.instance_type = Some(SingleOrVec::Single(Box::new(InstanceType::Object)));

        {
            schema.object().additional_properties = Some(Box::new(false.into()));
            let properties = &mut schema.object().properties;

            properties
                .insert(RuleCategory::Correctness.as_str().to_string(), category_config.clone());
            properties
                .insert(RuleCategory::Suspicious.as_str().to_string(), category_config.clone());
            properties.insert(RuleCategory::Pedantic.as_str().to_string(), category_config.clone());
            properties.insert(RuleCategory::Perf.as_str().to_string(), category_config.clone());
            properties.insert(RuleCategory::Style.as_str().to_string(), category_config.clone());
            properties
                .insert(RuleCategory::Restriction.as_str().to_string(), category_config.clone());
            properties.insert(RuleCategory::Nursery.as_str().to_string(), category_config);
        }

        {
            let metadata = schema.metadata();
            metadata.title = Some("Rule Categories".to_string());

            metadata.description = Some(
                r#"
Configure an entire category of rules all at once.

Use a severity such as `"warn"` or `"error"` to apply every enabled rule in that category.
Use `"recommended"` to apply Oxlint's built-in category-default subset for that category (core ESLint plus default built-in plugins) at `"warn"` severity.

Rules enabled or disabled this way will be overwritten by individual rules in the `rules` field.

Example
```json
{
    "$schema": "./node_modules/oxlint/configuration_schema.json",
    "categories": {
        "correctness": "warn",
        "suspicious": "recommended"
    },
    "rules": {
        "eslint/no-unused-vars": "error"
    }
}
```
"#
                .trim()
                .to_string(),
            );

            metadata.examples = vec![serde_json::json!({
                "correctness": "warn",
                "suspicious": "recommended"
            })];
        }

        schema.into()
    }
}

#[cfg(test)]
mod test {
    use serde_json::json;

    use super::{CategoryConfig, OxlintCategories};
    use crate::{AllowWarnDeny, RuleCategory};

    #[test]
    fn category_config_recommended_round_trips() {
        let value = serde_json::to_value(CategoryConfig::Recommended).unwrap();
        assert_eq!(value, json!("recommended"));

        let parsed: CategoryConfig = serde_json::from_value(json!("recommended")).unwrap();
        assert_eq!(parsed, CategoryConfig::Recommended);
    }

    #[test]
    fn category_config_severity_round_trips() {
        let value = serde_json::to_value(CategoryConfig::Severity(AllowWarnDeny::Deny)).unwrap();
        assert_eq!(value, json!("deny"));

        let parsed: CategoryConfig = serde_json::from_value(json!("warn")).unwrap();
        assert_eq!(parsed, CategoryConfig::Severity(AllowWarnDeny::Warn));
    }

    #[test]
    fn oxlint_categories_insert_coerces_severities() {
        let mut categories = OxlintCategories::default();
        categories.insert(RuleCategory::Suspicious, AllowWarnDeny::Warn);

        assert_eq!(
            categories.get(&RuleCategory::Suspicious),
            Some(&CategoryConfig::Severity(AllowWarnDeny::Warn))
        );
    }

    #[test]
    fn category_config_recommended_only_enables_category_defaults() {
        let builtin_suspicious = crate::rules::RULES
            .iter()
            .find(|rule| {
                rule.category() == RuleCategory::Suspicious && super::is_category_default_rule(rule)
            })
            .unwrap();
        let plugin_only_suspicious = crate::rules::RULES
            .iter()
            .find(|rule| {
                rule.category() == RuleCategory::Suspicious && rule.plugin_name() == "react"
            })
            .unwrap();

        assert_eq!(
            CategoryConfig::Recommended.severity_for_rule(builtin_suspicious),
            Some(AllowWarnDeny::Warn)
        );
        assert_eq!(CategoryConfig::Recommended.severity_for_rule(plugin_only_suspicious), None);
    }
}
