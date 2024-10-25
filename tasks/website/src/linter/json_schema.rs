use handlebars::Handlebars;
use oxc_linter::Oxlintrc;
use schemars::{
    schema::{InstanceType, RootSchema, Schema, SchemaObject, SingleOrVec, SubschemaValidation},
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

{{#if default}}
default: `{{default}}`
{{/if}}

{{description}}

{{> section}}
{{/each}}
";

/// Data passed to [`ROOT`] hbs template
#[derive(Serialize)]
struct Root {
    title: String,
    sections: Vec<Section>,
}

/// Data passed to [`SECTION`] hbs template
#[derive(Serialize)]
struct Section {
    level: String,
    title: String,
    instance_type: Option<String>,
    description: String,
    default: Option<String>,
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
        handlebars
            .register_template_string("root", ROOT)
            .expect("Failed to register root template.");
        handlebars
            .register_template_string("section", SECTION)
            .expect("Failed to register section template.");
        Self { handlebars, root_schema }
    }

    fn render(self) -> String {
        let mut root = self.render_root_schema();
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

    fn render_root_schema(&self) -> Root {
        let title = self
            .root_schema
            .schema
            .metadata
            .as_ref()
            .and_then(|m| m.description.clone())
            .unwrap_or_default();
        let sections = self.render_properties(1, None, &self.root_schema.schema);
        Root { title, sections }
    }

    /// Recursively render a subschema's properties into sections.
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
                        return self.render_sub_schema(depth, &key, subschemas, schema_object);
                    }

                    vec![self.render_schema(depth + 1, &key, schema_object)]
                })
                .collect::<Vec<_>>();
        }
        if let Some(subschemas) = &schema.subschemas {
            let key = parent_key.unwrap_or("");
            self.render_sub_schema(depth, key, subschemas, schema);
        }
        vec![]
    }

    fn render_sub_schema(
        &self,
        depth: usize,
        key: &str,
        subschemas: &SubschemaValidation,
        schema: &SchemaObject,
    ) -> Vec<Section> {
        if let Some(schemas) = &subschemas.all_of {
            return schemas
                .iter()
                .map(|subschema| {
                    let subschema = Self::get_schema_object(subschema);
                    let subschema = self.get_referenced_schema(subschema);
                    let mut section = self.render_schema(depth + 1, key, subschema);
                    if section.default.is_none() && !subschema.has_type(InstanceType::Object) {
                        section.default = Self::render_default(schema);
                    }
                    section.sanitize();
                    section
                })
                .collect::<Vec<Section>>();
        }
        vec![]
    }

    fn render_schema(&self, depth: usize, key: &str, schema: &SchemaObject) -> Section {
        let ref_schema = schema;
        let schema = self.get_referenced_schema(schema);

        let (instance_type, sections) = if let Some(item) = as_primitive_array(schema) {
            // e.g. "string[]" instead of "array"
            let instance_type = serde_json::to_string_pretty(item.instance_type.as_ref().unwrap())
                .unwrap()
                .replace('"', "")
                + "[]";

            // Do not render subsections for primitive arrays
            (Some(instance_type), vec![])
        } else if let Some(values) = as_mapped_type(schema) {
            // Mapped types have empty `properties` and instead use
            // `additionalProperties`. Try to render a better type than `object`
            let values = self.get_referenced_schema(values);
            let value_type = values
                .instance_type
                .as_ref()
                .map(|t| serde_json::to_string(t).unwrap().replace('"', ""));
            let instance_type =
                value_type.map_or_else(|| "object".into(), |v| format!("Record<string, {v}>"));

            (Some(instance_type), vec![])
        } else {
            let instance_type = schema
                .instance_type
                .as_ref()
                .map(|t| serde_json::to_string_pretty(t).unwrap().replace('"', ""));
            let sections = self.render_properties(depth, Some(key), schema);

            (instance_type, sections)
        };

        Section {
            level: "#".repeat(depth),
            title: key.into(),
            instance_type,
            default: Self::render_default(ref_schema).or_else(|| Self::render_default(schema)),
            description: schema
                .metadata
                .as_ref()
                .and_then(|m| m.description.clone())
                .unwrap_or_default(),
            sections,
        }
    }

    fn render_default(schema: &SchemaObject) -> Option<String> {
        let m = schema.metadata.as_ref()?;
        let default = m.default.as_ref()?;
        let rendered = serde_json::to_string(default).unwrap_or_else(|_| {
            panic!(
                "Failed to serialize `default` field for schema: {}",
                m.title.as_deref().unwrap_or("<unknown>")
            )
        });
        Some(rendered.replace(',', ", "))
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

fn as_mapped_type(schema: &SchemaObject) -> Option<&SchemaObject> {
    let obj = schema.object.as_ref()?;
    obj.properties
        .is_empty()
        .then_some(obj.additional_properties.as_ref())
        .flatten()
        .map(|ap| Renderer::get_schema_object(ap))
}
/// If `schema` is an array of primitive data types, returns [`Some`] with the
/// primitive schema (i.e. the array item schema).
fn as_primitive_array(schema: &SchemaObject) -> Option<&SchemaObject> {
    let arr = schema.array.as_ref()?;
    let SingleOrVec::Single(item) = arr.items.as_ref()? else {
        return None;
    };
    let Schema::Object(item) = &**item else {
        return None;
    };

    as_primitive(item)
}

/// If `schema` has a primitive data type (e.g. `string`, `integer`), returns
/// `Some(schema)`.
fn as_primitive(schema: &SchemaObject) -> Option<&SchemaObject> {
    // null intentionally omitted
    const PRIMITIVE_TYPES: [InstanceType; 4] =
        [InstanceType::Boolean, InstanceType::Integer, InstanceType::Number, InstanceType::String];

    let is_primitive = !schema.is_ref()
        && PRIMITIVE_TYPES.iter().any(|t| {
            // schema.has_type(*t)
            schema.instance_type.as_ref().is_some_and(|ty| {
                let SingleOrVec::Single(single) = ty else { return false };
                single.as_ref() == t
            })
        });

    is_primitive.then_some(schema)
}
