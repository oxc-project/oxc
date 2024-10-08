use std::{borrow::Cow, ops::Deref};

use rustc_hash::FxHashMap;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{AllowWarnDeny, LintFilter, RuleCategory};

/// Configure an entire category of rules all at once.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct OxlintCategories(FxHashMap<RuleCategory, AllowWarnDeny>);

impl Deref for OxlintCategories {
    type Target = FxHashMap<RuleCategory, AllowWarnDeny>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl OxlintCategories {
    pub fn filters(&self) -> impl Iterator<Item = LintFilter> + '_ {
        self.iter().map(|(category, severity)| LintFilter::new(*severity, *category).unwrap())
    }
}

impl JsonSchema for OxlintCategories {
    fn schema_id() -> Cow<'static, str> {
        Cow::Borrowed("OxlintCategories")
    }

    fn schema_name() -> String {
        "OxlintCategories".to_string()
    }

    fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        let severity = gen.subschema_for::<AllowWarnDeny>();
        let mut schema =
            gen.subschema_for::<FxHashMap<RuleCategory, AllowWarnDeny>>().into_object();

        {
            schema.object().additional_properties = None;
            let properties = &mut schema.object().properties;

            properties.insert(RuleCategory::Correctness.as_str().to_string(), severity.clone());
            properties.insert(RuleCategory::Suspicious.as_str().to_string(), severity.clone());
            properties.insert(RuleCategory::Pedantic.as_str().to_string(), severity.clone());
            properties.insert(RuleCategory::Perf.as_str().to_string(), severity.clone());
            properties.insert(RuleCategory::Style.as_str().to_string(), severity.clone());
            properties.insert(RuleCategory::Restriction.as_str().to_string(), severity.clone());
            properties.insert(RuleCategory::Nursery.as_str().to_string(), severity.clone());
        }

        {
            let metadata = schema.metadata();
            metadata.title = Some("Rule Categories".to_string());

            metadata.description = Some(
                r#"
Configure an entire category of rules all at once.

Rules enabled or disabled this way will be overwritten by individual rules in the `rules` field.

# Example
```json
{
    "categories": {
        "correctness": "warn"
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

            metadata.examples = vec![serde_json::json!({ "correctness": "warn" })];
        }

        schema.into()
    }
}
