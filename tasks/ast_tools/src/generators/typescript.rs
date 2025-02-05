//! Generator for TypeScript type definitions for all AST types.

use std::{borrow::Cow, fmt::Write};

use itertools::Itertools;

use crate::{
    derives::estree::{
        get_fieldless_variant_value, get_struct_field_name, should_add_type_field_to_struct,
        should_flatten_field, should_skip_field,
    },
    output::Output,
    schema::{Def, EnumDef, FieldDef, Schema, StructDef, TypeDef},
    Codegen, Generator, Result, TYPESCRIPT_DEFINITIONS_PATH,
};

use super::{attr_positions, define_generator, AttrLocation, AttrPart, AttrPositions};

/// Generator for TypeScript type definitions.
pub struct TypescriptGenerator;

define_generator!(TypescriptGenerator);

impl Generator for TypescriptGenerator {
    /// Register that accept `#[ts]` attr on struct fields and enum variants.
    fn attrs(&self) -> &[(&'static str, AttrPositions)] {
        &[("ts", attr_positions!(StructField | EnumVariant))]
    }

    /// Parse `#[ts]` on struct field or enum variant.
    fn parse_attr(&self, _attr_name: &str, location: AttrLocation, part: AttrPart) -> Result<()> {
        // No need to check attr name is `ts`, because that's the only attribute this derive handles.
        if !matches!(part, AttrPart::None) {
            return Err(());
        }

        // Location can only be `StructField` or `EnumVariant`
        match location {
            AttrLocation::StructField(struct_def, field_index) => {
                struct_def.fields[field_index].estree.is_ts = true;
            }
            AttrLocation::EnumVariant(enum_def, variant_index) => {
                enum_def.variants[variant_index].estree.is_ts = true;
            }
            _ => unreachable!(),
        }

        Ok(())
    }

    /// Generate Typescript type definitions for all AST types.
    fn generate(&self, schema: &Schema, codegen: &Codegen) -> Output {
        let estree_derive_id = codegen.get_derive_id_by_name("ESTree");

        let mut code = String::new();
        for type_def in &schema.types {
            if type_def.generates_derive(estree_derive_id) {
                generate_ts_type_def(type_def, &mut code, schema);
            }
        }

        Output::Javascript { path: TYPESCRIPT_DEFINITIONS_PATH.to_string(), code }
    }
}

/// Generate Typescript type definition for a struct or enum.
///
/// Push type defs to `code`.
fn generate_ts_type_def(type_def: &TypeDef, code: &mut String, schema: &Schema) {
    // Use custom TS def if provided via `#[estree(custom_ts_def = "...")]` attribute
    let custom_ts_def = match type_def {
        TypeDef::Struct(struct_def) => &struct_def.estree.custom_ts_def,
        TypeDef::Enum(enum_def) => &enum_def.estree.custom_ts_def,
        _ => unreachable!(),
    };

    if let Some(custom_ts_def) = custom_ts_def {
        // Empty string means don't output any TS def at all for this type
        if !custom_ts_def.is_empty() {
            write!(code, "export {custom_ts_def};\n\n").unwrap();
        }
    } else {
        // No custom definition. Generate one.
        let ts_def = match type_def {
            TypeDef::Struct(struct_def) => generate_ts_type_def_for_struct(struct_def, schema),
            TypeDef::Enum(enum_def) => generate_ts_type_def_for_enum(enum_def, schema),
            _ => unreachable!(),
        };
        write!(code, "{ts_def};\n\n").unwrap();
    };

    // Add additional custom TS def if provided via `#[estree(add_ts_def = "...")]` attribute
    let add_ts_def = match type_def {
        TypeDef::Struct(struct_def) => &struct_def.estree.add_ts_def,
        TypeDef::Enum(enum_def) => &enum_def.estree.add_ts_def,
        _ => unreachable!(),
    };
    if let Some(add_ts_def) = add_ts_def {
        write!(code, "export {add_ts_def};\n\n").unwrap();
    }
}

/// Generate Typescript type definition for a struct.
fn generate_ts_type_def_for_struct(struct_def: &StructDef, schema: &Schema) -> String {
    let type_name = struct_def.name();
    let mut fields_str = String::new();
    let mut extends = vec![];

    if should_add_type_field_to_struct(struct_def) {
        let type_name = struct_def.estree.rename.as_deref().unwrap_or_else(|| struct_def.name());
        fields_str.push_str(&format!("\n\ttype: '{type_name}';"));
    }

    let mut output_as_type = false;
    for field in &struct_def.fields {
        if should_skip_field(field, schema) {
            continue;
        }

        let field_type_name = if let Some(append_field_index) = field.estree.append_field_index {
            let appended_field = struct_def.fields[append_field_index].type_def(schema);
            let appended_field = appended_field.as_option().unwrap();
            let appended_type_name = ts_type_name(appended_field.inner_type(schema), schema);

            let field_type = field.type_def(schema);
            let (vec_def, is_option) = match field_type {
                TypeDef::Vec(vec_def) => (vec_def, false),
                TypeDef::Option(option_def) => {
                    let vec_def = option_def.inner_type(schema).as_vec().unwrap();
                    (vec_def, true)
                }
                _ => panic!(
                    "Can only append a field to a `Vec<T>` or `Option<Vec<T>>`: `{}::{}`",
                    type_name,
                    field.name()
                ),
            };

            let mut inner_type = vec_def.inner_type(schema);
            let mut inner_is_option = false;
            if let TypeDef::Option(option_def) = inner_type {
                inner_is_option = true;
                inner_type = option_def.inner_type(schema);
            }
            let inner_type_name = ts_type_name(inner_type, schema);
            let mut field_type_name = format!("Array<{inner_type_name} | {appended_type_name}");
            if inner_is_option {
                field_type_name.push_str(" | null");
            }
            field_type_name.push('>');
            if is_option {
                field_type_name.push_str(" | null");
            }

            Cow::Owned(field_type_name)
        } else {
            get_field_type_name(field, schema)
        };

        if should_flatten_field(field, schema) {
            if !output_as_type && field_type_name.contains('|') {
                output_as_type = true;
            }
            extends.push(field_type_name);
            continue;
        }

        let field_camel_name = get_struct_field_name(field);
        fields_str.push_str(&format!("\n\t{field_camel_name}: {field_type_name};"));
    }

    if let Some(add_ts) = struct_def.estree.add_ts.as_deref() {
        fields_str.push_str(&format!("\n\t{add_ts};"));
    }

    if extends.is_empty() {
        format!("export interface {type_name} {{{fields_str}\n}}")
    } else if output_as_type {
        format!("export type {type_name} = ({{{fields_str}\n}}) & {};", extends.join(" & "))
    } else {
        format!("export interface {type_name} extends {} {{{fields_str}\n}}", extends.join(", "))
    }
}

/// Generate Typescript type definition for an enum.
fn generate_ts_type_def_for_enum(enum_def: &EnumDef, schema: &Schema) -> String {
    let union = if enum_def.is_fieldless() {
        enum_def
            .all_variants(schema)
            .map(|variant| format!("'{}'", get_fieldless_variant_value(enum_def, variant)))
            .join(" | ")
    } else {
        enum_def
            .all_variants(schema)
            .map(|variant| ts_type_name(variant.field_type(schema).unwrap(), schema))
            .join(" | ")
    };

    let enum_name = enum_def.name();
    format!("export type {enum_name} = {union};")
}

/// Get TS type name for a type.
fn ts_type_name<'s>(type_def: &'s TypeDef, schema: &'s Schema) -> Cow<'s, str> {
    match type_def {
        TypeDef::Struct(struct_def) => Cow::Borrowed(struct_def.name()),
        TypeDef::Enum(enum_def) => Cow::Borrowed(enum_def.name()),
        TypeDef::Primitive(primitive_def) => Cow::Borrowed(match primitive_def.name() {
            #[rustfmt::skip]
            "u8" | "u16" | "u32" | "u64" | "u128" | "usize"
            | "i8" | "i16" | "i32" | "i64" | "i128" | "isize"
            | "f32" | "f64" => "number",
            "bool" => "boolean",
            "&str" | "Atom" => "string",
            name => name,
        }),
        TypeDef::Option(option_def) => {
            Cow::Owned(format!("{} | null", ts_type_name(option_def.inner_type(schema), schema)))
        }
        TypeDef::Vec(vec_def) => {
            Cow::Owned(format!("Array<{}>", ts_type_name(vec_def.inner_type(schema), schema)))
        }
        TypeDef::Box(box_def) => ts_type_name(box_def.inner_type(schema), schema),
        TypeDef::Cell(cell_def) => ts_type_name(cell_def.inner_type(schema), schema),
    }
}

/// Get type name for a field.
fn get_field_type_name<'s>(field: &'s FieldDef, schema: &'s Schema) -> Cow<'s, str> {
    if let Some(ts_type) = field.estree.ts_type.as_deref() {
        Cow::Borrowed(ts_type)
    } else {
        let field_type = field.type_def(schema);
        ts_type_name(field_type, schema)
    }
}
