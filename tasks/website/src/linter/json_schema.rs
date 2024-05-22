use handlebars::Handlebars;
use schemars::{
    schema::{RootSchema, Schema, SchemaObject},
    schema_for,
};
use serde::Serialize;

use oxc_linter::ESLintConfig;

pub fn generate_schema_json() {
    let schema = schema_for!(ESLintConfig);
    println!("{}", serde_json::to_string_pretty(&schema).unwrap());
}

pub fn generate_schema_markdown() {
    let rendered = Renderer::new().render();
    println!("{rendered}");
}

const ROOT: &str = "
# {{title}}

{{> section}}
";

const SECTION: &str = "
{{#each sections}}
{{level}} `{{title}}`

type: `{{instance_type}}`

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
    instance_type: String,
    description: String,
    sections: Vec<Section>,
}

struct Renderer {
    handlebars: Handlebars<'static>,
    root_schema: RootSchema,
}

impl Renderer {
    fn new() -> Self {
        let mut handlebars = Handlebars::new();
        handlebars.register_escape_fn(handlebars::no_escape);
        assert!(handlebars.register_template_string("root", ROOT).is_ok());
        assert!(handlebars.register_template_string("section", SECTION).is_ok());
        let root_schema = schema_for!(ESLintConfig);
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

    fn get_referenced_schema(&self, reference: &str) -> &SchemaObject {
        let definitions = &self.root_schema.definitions;
        let definition = definitions.get(reference.trim_start_matches("#/definitions/"));
        definition.map(Self::get_schema_object).unwrap()
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
        let Some(object) = &schema.object else { return vec![] };
        object
            .properties
            .iter()
            .map(|(key, schema)| {
                let key = parent_key.map_or_else(|| key.clone(), |k| format!("{k}.{key}"));
                self.render_schema(depth + 1, &key, Self::get_schema_object(schema))
            })
            .collect::<Vec<_>>()
    }

    fn render_schema(&self, depth: usize, key: &str, schema: &SchemaObject) -> Section {
        let schema = schema.reference.as_ref().map_or(schema, |r| self.get_referenced_schema(r));
        Section {
            level: "#".repeat(depth),
            title: key.into(),
            instance_type: serde_json::to_string(&schema.instance_type).unwrap().replace('"', ""),
            description: schema
                .metadata
                .as_ref()
                .and_then(|m| m.description.clone())
                .unwrap_or_default(),
            sections: self.render_properties(depth, Some(key), schema),
        }
    }
}
