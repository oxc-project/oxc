use crate::{rules::RULES, AllowWarnDeny};

use super::OxlintRules;
use schemars::{
    gen::SchemaGenerator,
    schema::{Schema, SchemaObject},
    JsonSchema,
};
// use crate::RuleMeta;
use std::borrow::Cow;

impl JsonSchema for OxlintRules {
    fn schema_name() -> String {
        "OxlintRules".to_owned()
    }

    fn schema_id() -> Cow<'static, str> {
        Cow::Borrowed("OxlintRules")
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        let mut schema: SchemaObject = SchemaObject::default();
        schema.object().properties.extend(RULES.iter().map(|rule| {
            let rule_config_schema = rule.schema(gen);
            let mut rule_schema = rule_property_schema(gen, rule_config_schema);

            let docs = rule.documentation();
            rule_schema.metadata().description = docs.map(Into::into);
            if let Some(docs) = docs {
                // markdownDescription is a non-standard property that VSCode
                // uses in intellisense. It lets us show the rule's
                // documentation with markdown formatting.
                rule_schema
                    .extensions
                    .insert("markdownDescription".into(), docs.to_string().into());
            }

            // Don't scope eslint rules, only plugins.
            let scoped_name = if rule.plugin_name() == "eslint" {
                rule.name().into()
            } else {
                rule.plugin_name().to_string() + "/" + rule.name()
            };

            (scoped_name, rule_schema.into())
        }));

        schema.into()
    }
}

fn rule_property_schema(gen: &mut SchemaGenerator, config_schema: Schema) -> SchemaObject {
    let any_list = gen.subschema_for::<Vec<serde_json::Value>>().into_object();
    let toggle_schema = gen.subschema_for::<AllowWarnDeny>().into_object();
    let mut toggle_and_config = SchemaObject::default();
    toggle_and_config.array().items = Some(Schema::Object(toggle_schema.clone()).into());
    toggle_and_config.array().additional_items = Some(Box::new(config_schema));

    let mut schema = SchemaObject::default();
    schema.subschemas().any_of =
        Some(vec![toggle_schema.into(), toggle_and_config.into(), any_list.into()]);

    schema
}
