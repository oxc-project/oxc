//! Generator for TypeScript type definitions for all AST types.

use std::{borrow::Cow, fmt::Write};

use itertools::Itertools;

use crate::{
    Codegen, Generator, TYPESCRIPT_DEFINITIONS_PATH,
    derives::estree::{
        get_fieldless_variant_value, get_struct_field_name, should_flatten_field, should_skip_field,
    },
    output::Output,
    schema::{Def, EnumDef, FieldDef, Schema, StructDef, TypeDef},
};

use super::define_generator;

/// Generator for TypeScript type definitions.
pub struct TypescriptGenerator;

define_generator!(TypescriptGenerator);

impl Generator for TypescriptGenerator {
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

        if let Some(ts_def) = ts_def {
            write!(code, "{ts_def};\n\n").unwrap();
        }
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
fn generate_ts_type_def_for_struct(struct_def: &StructDef, schema: &Schema) -> Option<String> {
    // If struct marked with `#[estree(ts_alias = "...")]`, then it needs no type def
    if struct_def.estree.ts_alias.is_some() {
        return None;
    }

    // If struct is marked as `#[estree(flatten)]`, and only has a single field which isn't skipped,
    // don't generate a type def. That single field will be inserted inline into structs which include
    // this one rather than them extending this type.
    if struct_def.estree.flatten && get_single_field(struct_def, schema).is_some() {
        return None;
    }

    let type_name = struct_def.name();
    let mut fields_str = String::new();
    let mut extends = vec![];

    if should_add_type_field_to_struct(struct_def) {
        let type_name = struct_def.estree.rename.as_deref().unwrap_or_else(|| struct_def.name());
        fields_str.push_str(&format!("\n\ttype: '{type_name}';"));
    }

    let mut output_as_type = false;

    if let Some(field_indices) = &struct_def.estree.field_indices {
        // Specified field order - output in this order
        for &field_index in field_indices {
            let field_index = field_index as usize;
            if let Some(field) = struct_def.fields.get(field_index) {
                generate_ts_type_def_for_struct_field(
                    struct_def,
                    field,
                    &mut fields_str,
                    &mut extends,
                    &mut output_as_type,
                    schema,
                );
            } else {
                let (field_name, converter_name) =
                    &struct_def.estree.add_fields[field_index - struct_def.fields.len()];
                generate_ts_type_def_for_added_struct_field(
                    field_name,
                    converter_name,
                    &mut fields_str,
                    schema,
                );
            }
        }
    } else {
        // No specified field order - output in original order
        for field in &struct_def.fields {
            generate_ts_type_def_for_struct_field(
                struct_def,
                field,
                &mut fields_str,
                &mut extends,
                &mut output_as_type,
                schema,
            );
        }

        for (field_name, converter_name) in &struct_def.estree.add_fields {
            generate_ts_type_def_for_added_struct_field(
                field_name,
                converter_name,
                &mut fields_str,
                schema,
            );
        }
    }

    let ts_def = if extends.is_empty() {
        format!("export interface {type_name} {{{fields_str}\n}}")
    } else if output_as_type {
        format!("export type {type_name} = ({{{fields_str}\n}}) & {};", extends.join(" & "))
    } else {
        format!("export interface {type_name} extends {} {{{fields_str}\n}}", extends.join(", "))
    };
    Some(ts_def)
}

/// Generate Typescript type definition for a struct field.
///
/// Field definition is appended to `fields_str` or `extends`.
fn generate_ts_type_def_for_struct_field<'s>(
    struct_def: &StructDef,
    field: &'s FieldDef,
    fields_str: &mut String,
    extends: &mut Vec<Cow<'s, str>>,
    output_as_type: &mut bool,
    schema: &'s Schema,
) {
    if should_skip_field(field, schema) {
        return;
    }

    generate_ts_type_def_for_struct_field_impl(
        struct_def,
        field,
        fields_str,
        extends,
        output_as_type,
        schema,
    );
}

fn generate_ts_type_def_for_struct_field_impl<'s>(
    struct_def: &StructDef,
    field: &'s FieldDef,
    fields_str: &mut String,
    extends: &mut Vec<Cow<'s, str>>,
    output_as_type: &mut bool,
    schema: &'s Schema,
) {
    let field_type_name = if let Some(append_field_index) = field.estree.append_field_index {
        let appended_field = &struct_def.fields[append_field_index];
        let appended_type = appended_field.type_def(schema);
        let appended_type = match appended_type {
            TypeDef::Option(option_def) => option_def.inner_type(schema),
            TypeDef::Vec(vec_def) => vec_def.inner_type(schema),
            _ => panic!(
                "Appended field must be `Option<T>` or `Vec<T>`: `{}::{}`",
                struct_def.name(),
                appended_field.name()
            ),
        };
        let appended_type_name = ts_type_name(appended_type, schema);

        let field_type = field.type_def(schema);
        let (vec_def, is_option) = match field_type {
            TypeDef::Vec(vec_def) => (vec_def, false),
            TypeDef::Option(option_def) => {
                let vec_def = option_def.inner_type(schema).as_vec().unwrap();
                (vec_def, true)
            }
            _ => panic!(
                "Can only append a field to a `Vec<T>` or `Option<Vec<T>>`: `{}::{}`",
                struct_def.name(),
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
    } else if let Some(converter_name) = &field.estree.via {
        Cow::Borrowed(get_ts_type_for_converter(converter_name, schema))
    } else {
        get_field_type_name(field, schema)
    };

    if should_flatten_field(field, schema) {
        if let TypeDef::Struct(field_type) = field.type_def(schema) {
            if let Some(flatten_field) = get_single_field(field_type, schema) {
                // Only one field to flatten. Add it as a field on the parent type, instead of extending.
                generate_ts_type_def_for_struct_field_impl(
                    field_type,
                    flatten_field,
                    fields_str,
                    extends,
                    output_as_type,
                    schema,
                );
                return;
            }
        }

        // need `type` instead of `interface` when flattening BindingPattern
        if field_type_name.contains('|') || field_type_name == "BindingPattern" {
            *output_as_type = true;
        }
        extends.push(field_type_name);
        return;
    }

    let field_camel_name = get_struct_field_name(field);
    fields_str.push_str(&format!("\n\t{field_camel_name}: {field_type_name};"));
}

/// Generate Typescript type definition for an extra struct field
/// specified with `#[estree(add_fields(...))]`.
fn generate_ts_type_def_for_added_struct_field(
    field_name: &str,
    converter_name: &str,
    fields_str: &mut String,
    schema: &Schema,
) {
    let ts_type = get_ts_type_for_converter(converter_name, schema);
    fields_str.push_str(&format!("\n\t{field_name}: {ts_type};"));
}

/// Get the TS type definition for a converter.
///
/// Converters are specified with `#[estree(add_fields(field_name = converter_name))]`
/// and `#[estree(via = converter_name)]`.
fn get_ts_type_for_converter<'s>(converter_name: &str, schema: &'s Schema) -> &'s str {
    let converter = schema.meta_by_name(converter_name);
    let Some(ts_type) = &converter.estree.ts_type else {
        panic!("No `ts_type` provided for ESTree converter `{}`", converter.name());
    };
    ts_type
}

/// Generate Typescript type definition for an enum.
fn generate_ts_type_def_for_enum(enum_def: &EnumDef, schema: &Schema) -> Option<String> {
    // If enum marked with `#[estree(ts_alias = "...")]`, then it needs no type def
    if enum_def.estree.ts_alias.is_some() {
        return None;
    }

    let own_variants_type_names = enum_def.variants.iter().map(|variant| {
        if let Some(variant_type) = variant.field_type(schema) {
            ts_type_name(variant_type, schema)
        } else {
            Cow::Owned(format!("'{}'", get_fieldless_variant_value(enum_def, variant)))
        }
    });

    let inherits_type_names =
        enum_def.inherits_types(schema).map(|inherited_type| ts_type_name(inherited_type, schema));

    let union = own_variants_type_names.chain(inherits_type_names).join(" | ");

    let enum_name = enum_def.name();
    Some(format!("export type {enum_name} = {union};"))
}

/// Get TS type name for a type.
fn ts_type_name<'s>(type_def: &'s TypeDef, schema: &'s Schema) -> Cow<'s, str> {
    match type_def {
        TypeDef::Struct(struct_def) => {
            if let Some(ts_alias) = &struct_def.estree.ts_alias {
                Cow::Borrowed(ts_alias)
            } else {
                Cow::Borrowed(struct_def.name())
            }
        }
        TypeDef::Enum(enum_def) => {
            if let Some(ts_alias) = &enum_def.estree.ts_alias {
                Cow::Borrowed(ts_alias)
            } else {
                Cow::Borrowed(enum_def.name())
            }
        }
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

/// Get if should generate a `type` field.
///
/// Type field should be added unless struct has an `#[estree(no_type)]` attr
/// or struct has an existing field called `type`.
fn should_add_type_field_to_struct(struct_def: &StructDef) -> bool {
    if struct_def.estree.no_type {
        false
    } else {
        !struct_def.fields.iter().any(|field| matches!(field.name(), "type"))
    }
}

/// If struct has only a single unskipped field, return it.
///
/// If no fields, or more than 1 unskipped field, returns `None`.
fn get_single_field<'s>(struct_def: &'s StructDef, schema: &Schema) -> Option<&'s FieldDef> {
    let mut fields_which_are_not_skipped =
        struct_def.fields.iter().filter(|field| !should_skip_field(field, schema));

    if let Some(field) = fields_which_are_not_skipped.next() {
        if fields_which_are_not_skipped.next().is_none() {
            return Some(field);
        }
    }
    None
}
