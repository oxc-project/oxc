use handlebars::Handlebars;
use schemars::{
    schema::{RootSchema, Schema, SchemaObject, SingleOrVec},
    schema_for,
};
use serde::Serialize;

use oxc_linter::ESLintConfig;

pub fn generate_schema_json() {
    let schema = schema_for!(ESLintConfig);
    println!("{}", serde_json::to_string_pretty(&schema).unwrap());
}

pub fn generate_schema_markdown() {
    let root_schema = schema_for!(ESLintConfig);
    let rendered = Renderer::new(root_schema).render();
    println!("{rendered}");
}

const ROOT: &str = "
# {{title}}

{{> section}}
";

const SECTION: &str = "
{{#each sections}}
{{level}} {{title}}

{{#if instance_type}}
type: `{{instance_type}}`
{{/if}}

{{description}}

{{> section}}

{{/each}}
";

#[derive(Serialize)]
struct Root {
    title: String,
    sections: Vec<Section>,
}

#[derive(Serialize)]
struct Section {
    level: String,
    title: String,
    instance_type: Option<String>,
    description: String,
    sections: Vec<Section>,
}

struct Renderer {
    handlebars: Handlebars<'static>,
    root_schema: RootSchema,
}

impl Renderer {
    fn new(root_schema: RootSchema) -> Self {
        let mut handlebars = Handlebars::new();
        handlebars.register_escape_fn(handlebars::no_escape);
        assert!(handlebars.register_template_string("root", ROOT).is_ok());
        assert!(handlebars.register_template_string("section", SECTION).is_ok());
        Self { handlebars, root_schema }
    }

    fn render(self) -> String {
        let root = self.render_root_schema(&self.root_schema);
        self.handlebars.render("root", &root).unwrap()
    }

    fn get_schema_object(schema: &Schema) -> &SchemaObject {
        match schema {
            Schema::Object(schema_object) => schema_object,
            Schema::Bool(_) => panic!("definition must be an object."),
        }
    }

    fn get_referenced_schema<'a>(&'a self, object: &'a SchemaObject) -> &'a SchemaObject {
        if let Some(reference) = &object.reference {
            let definitions = &self.root_schema.definitions;
            let definition = definitions.get(reference.trim_start_matches("#/definitions/"));
            definition.map(Self::get_schema_object).unwrap()
        } else {
            object
        }
    }

    fn render_root_schema(&self, root_schema: &RootSchema) -> Root {
        let title = root_schema
            .schema
            .metadata
            .as_ref()
            .and_then(|m| m.description.clone())
            .unwrap_or_default();
        let sections = self.render_properties(1, None, &root_schema.schema);
        Root { title, sections }
    }

    fn render_properties(
        &self,
        depth: usize,
        parent_key: Option<&str>,
        schema: &SchemaObject,
    ) -> Vec<Section> {
        if let Some(array) = &schema.array {
            return array
                .items
                .iter()
                .map(|item| match item {
                    SingleOrVec::Single(schema) => {
                        let schema_object = Self::get_schema_object(schema);
                        let key = parent_key.map_or_else(String::new, |k| format!("{k}[n]"));
                        self.render_schema(depth + 1, &key, schema_object)
                    }
                    SingleOrVec::Vec(_) => panic!(),
                })
                .collect();
        }
        if let Some(object) = &schema.object {
            return object
                .properties
                .iter()
                .map(|(key, schema)| {
                    let key = parent_key.map_or_else(|| key.clone(), |k| format!("{k}.{key}"));
                    self.render_schema(depth + 1, &key, Self::get_schema_object(schema))
                })
                .collect::<Vec<_>>();
        }
        vec![]
    }

    fn render_schema(&self, depth: usize, key: &str, schema: &SchemaObject) -> Section {
        let schema = self.get_referenced_schema(schema);
        Section {
            level: "#".repeat(depth),
            title: key.into(),
            instance_type: schema
                .instance_type
                .as_ref()
                .map(|t| serde_json::to_string_pretty(t).unwrap().replace('"', "")),
            description: schema
                .metadata
                .as_ref()
                .and_then(|m| m.description.clone())
                .unwrap_or_default(),
            sections: self.render_properties(depth, Some(key), schema),
        }
    }
}
