#![expect(clippy::disallowed_methods)]

use handlebars::Handlebars;
use itertools::Itertools;
use schemars::schema::{
    InstanceType, RootSchema, Schema, SchemaObject, SingleOrVec, SubschemaValidation,
};
use serde::Serialize;

const ROOT: &str = "\
---
search: false
---

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

/// Data passed to `SECTION` hbs template
#[derive(Serialize)]
pub struct Section {
    level: String,
    title: String,
    pub instance_type: Option<String>,
    pub description: String,
    pub default: Option<String>,
    sections: Vec<Section>,
}

impl Section {
    /// Renders this section to markdown using the provided renderer.
    ///
    /// # Panics
    /// Panics if handlebars template rendering fails.
    pub fn to_md(&self, renderer: &Renderer) -> String {
        renderer.handlebars.render("section", self).unwrap()
    }
}

#[derive(Debug)]
pub struct Renderer {
    handlebars: Handlebars<'static>,
    root_schema: RootSchema,
}

impl Renderer {
    /// Creates a new renderer from a JSON schema.
    ///
    /// # Panics
    /// Panics if handlebars template registration fails.
    pub fn new(root_schema: RootSchema) -> Self {
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

    /// Renders the schema to markdown documentation.
    ///
    /// # Panics
    /// Panics if handlebars template rendering fails.
    pub fn render(self) -> String {
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
                .flat_map(|item| match item {
                    // array
                    SingleOrVec::Single(schema) => {
                        let schema_object = Self::get_schema_object(schema);
                        let key = parent_key.map_or_else(String::new, |k| format!("{k}[n]"));
                        vec![self.render_schema_impl(depth + 1, &key, schema_object)]
                    }
                    // tuple
                    SingleOrVec::Vec(schema) => schema
                        .iter()
                        .enumerate()
                        .map(|(i, schema)| {
                            let schema_object = Self::get_schema_object(schema);
                            let key = parent_key.map_or_else(String::new, |k| format!("{k}[{i}]"));
                            self.render_schema_impl(depth + 1, &key, schema_object)
                        })
                        .collect(),
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
                        return self.render_sub_schema(depth + 1, &key, subschemas, schema_object);
                    }

                    vec![self.render_schema_impl(depth + 1, &key, schema_object)]
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
                    let mut section = self.render_schema_impl(depth, key, subschema);
                    if section.default.is_none() && !subschema.has_type(InstanceType::Object) {
                        section.default = Self::render_default(schema);
                    }
                    section.sanitize();
                    section
                })
                .collect::<Vec<Section>>();
        }
        if let Some(schemas) = &subschemas.any_of {
            // Check if anyOf contains a complex object (with properties) that should be expanded
            let mut object_schemas: Vec<&SchemaObject> = vec![];
            let mut primitive_types: Vec<String> = vec![];

            for subschema in schemas {
                let subschema = Self::get_schema_object(subschema);
                let subschema = self.get_referenced_schema(subschema);

                if subschema.object.is_some() || subschema.array.is_some() {
                    // This is a complex object or array - should be expanded
                    object_schemas.push(subschema);
                } else if let Some(instance_type) = &subschema.instance_type {
                    // This is a primitive type
                    primitive_types
                        .push(serde_json::to_string(instance_type).unwrap().replace('"', ""));
                }
            }

            // If we have object schemas, render them with their properties
            if !object_schemas.is_empty() {
                // Build the union type string including both objects and primitives
                let mut type_parts: Vec<String> = vec![];
                for schema in &object_schemas {
                    if schema.array.is_some() {
                        type_parts.push("array".to_string());
                    } else if schema.object.is_some() {
                        type_parts.push("object".to_string());
                    }
                }
                type_parts.extend(primitive_types);
                let type_parts = type_parts.into_iter().unique().collect::<Vec<_>>();

                let union_type =
                    if type_parts.is_empty() { None } else { Some(type_parts.join(" | ")) };

                // Render properties from the first object schema
                let sections = if let Some(first_object) = object_schemas.first() {
                    self.render_properties(depth, Some(key), first_object)
                } else {
                    vec![]
                };

                return vec![Section {
                    level: "#".repeat(depth),
                    title: key.into(),
                    instance_type: union_type,
                    default: Self::render_default(schema),
                    description: schema
                        .metadata
                        .as_ref()
                        .and_then(|m| m.description.clone())
                        .unwrap_or_default(),
                    sections,
                }];
            }

            // Otherwise, just render as a union of primitive types
            if !primitive_types.is_empty() {
                let union_type = primitive_types.join(" | ");
                return vec![Section {
                    level: "#".repeat(depth),
                    title: key.into(),
                    instance_type: Some(union_type),
                    default: Self::render_default(schema),
                    description: schema
                        .metadata
                        .as_ref()
                        .and_then(|m| m.description.clone())
                        .unwrap_or_default(),
                    sections: vec![],
                }];
            }
        }
        vec![]
    }

    pub fn render_schema(&self, depth: usize, key: &str, schema: &SchemaObject) -> Section {
        let mut section = self.render_schema_impl(depth, key, schema);
        section.sanitize();
        section
    }

    fn render_schema_impl(&self, depth: usize, key: &str, schema: &SchemaObject) -> Section {
        let ref_schema = schema;
        let schema = self.get_referenced_schema(schema);

        // Handle subschemas (anyOf, allOf, etc.) at the top level
        if let Some(subschemas) = &schema.subschemas {
            // Special case: if this is a oneOf with enum variants (enum-based config)
            // render them as subsections with their descriptions
            if let Some(one_of) = &subschemas.one_of {
                let enum_variants: Vec<_> = one_of
                    .iter()
                    .filter_map(|variant| {
                        let obj = Self::get_schema_object(variant);
                        let obj = self.get_referenced_schema(obj);
                        // Check if this is an enum variant (single enum value)
                        obj.enum_values
                            .as_ref()
                            .filter(|vals| vals.len() == 1)
                            .and_then(|vals| vals.first())
                            .and_then(|val| val.as_str())
                            .map(|value| {
                                let description = obj
                                    .metadata
                                    .as_ref()
                                    .and_then(|m| m.description.clone())
                                    .unwrap_or_default();
                                (value, description)
                            })
                    })
                    .collect();

                // If all oneOf variants are single-value enums, treat as enum config
                if !enum_variants.is_empty() && enum_variants.len() == one_of.len() {
                    let instance_type = Some(
                        enum_variants
                            .iter()
                            .map(|(val, _)| format!(r#""{val}""#))
                            .collect::<Vec<_>>()
                            .join(" | "),
                    );

                    let sections: Vec<Section> = enum_variants
                        .into_iter()
                        .map(|(value, description)| Section {
                            level: "#".repeat(depth + 1),
                            title: format!(r#"`"{value}"`"#),
                            instance_type: None,
                            default: None,
                            description,
                            sections: vec![],
                        })
                        .collect();

                    return Section {
                        level: "#".repeat(depth),
                        title: key.into(),
                        instance_type,
                        default: Self::render_default(ref_schema)
                            .or_else(|| Self::render_default(schema)),
                        description: schema
                            .metadata
                            .as_ref()
                            .and_then(|m| m.description.clone())
                            .or_else(|| {
                                ref_schema.metadata.as_ref().and_then(|m| m.description.clone())
                            })
                            .unwrap_or_default(),
                        sections,
                    };
                }
            }

            let subsections = self.render_sub_schema(depth, key, subschemas, schema);
            if !subsections.is_empty() {
                return subsections.into_iter().next().unwrap();
            }
        }

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
            let mut instance_type = schema.instance_type.as_ref().map(|t| match t {
                SingleOrVec::Single(single) => {
                    serde_json::to_string(single).unwrap().replace('"', "")
                }
                SingleOrVec::Vec(types) => types
                    .iter()
                    .map(|ty| serde_json::to_string(ty).unwrap().replace('"', ""))
                    .join(" | "),
            });
            let key_to_render = if key.is_empty() { None } else { Some(key) };
            let sections = self.render_properties(depth, key_to_render, schema);

            if instance_type.as_ref().is_some_and(|it| it == "string")
                && let Some(r#enum) = &schema.enum_values
            {
                instance_type = Some(
                    r#enum
                        .iter()
                        .map(|v| v.as_str().unwrap())
                        .map(|s| format!(r#""{s}""#))
                        .join(" | "),
                );
            }

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
                .or_else(|| ref_schema.metadata.as_ref().and_then(|m| m.description.clone()))
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
