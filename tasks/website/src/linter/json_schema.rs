use handlebars::Handlebars;
use oxc_linter::Oxlintrc;
use schemars::{
    schema::{RootSchema, Schema, SchemaObject, SingleOrVec, SubschemaValidation},
    schema_for,
};
use serde::Serialize;

pub fn print_schema_json() {
    println!("{}", generate_schema_json());
}

fn generate_schema_json() -> String {
    let schema = schema_for!(Oxlintrc);
    serde_json::to_string_pretty(&schema).unwrap()
}

#[test]
fn test_schema_markdown() {
    let snapshot = generate_schema_markdown();
    insta::with_settings!({ prepend_module_to_snapshot => false }, {
        insta::assert_snapshot!(snapshot);
    });
}

pub fn print_schema_markdown() {
    println!("{}", generate_schema_markdown());
}

fn generate_schema_markdown() -> String {
    let root_schema = schema_for!(Oxlintrc);
    Renderer::new(root_schema).render()
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
        let mut root = self.render_root_schema(&self.root_schema);
        root.sanitize();
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
                .flat_map(|(key, schema)| {
                    let key = parent_key.map_or_else(|| key.clone(), |k| format!("{k}.{key}"));
                    let schema_object = Self::get_schema_object(schema);

                    if let Some(subschemas) = &schema_object.subschemas {
                        return self.render_sub_schema(depth, &key, subschemas);
                    }

                    vec![self.render_schema(depth + 1, &key, schema_object)]
                })
                .collect::<Vec<_>>();
        }
        if let Some(subschemas) = &schema.subschemas {
            let key = parent_key.unwrap_or("");
            self.render_sub_schema(depth, key, subschemas);
        }
        vec![]
    }

    fn render_sub_schema(
        &self,
        depth: usize,
        key: &str,
        subschemas: &SubschemaValidation,
    ) -> Vec<Section> {
        if let Some(schemas) = &subschemas.all_of {
            return schemas
                .iter()
                .map(|schema| {
                    let schema = Self::get_schema_object(schema);
                    let schema = self.get_referenced_schema(schema);
                    let mut section = self.render_schema(depth + 1, key, schema);
                    section.sanitize();
                    section
                })
                .collect::<Vec<Section>>();
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

impl Root {
    fn sanitize(&mut self) {
        sanitize(&mut self.title);
    }
}

impl Section {
    fn sanitize(&mut self) {
        sanitize(&mut self.description);
    }
}

fn sanitize(s: &mut String) {
    let Some(start) = s.find("```json") else { return };
    let start = start + 7;
    let end = s[start..].find("```").unwrap();
    let json: serde_json::Value = serde_json::from_str(&s[start..start + end]).unwrap();
    let json = serde_json::to_string_pretty(&json).unwrap();
    let json = format!("\n{json}\n");
    s.replace_range(start..start + end, &json);
}
