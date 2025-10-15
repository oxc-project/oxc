//! Generator for TypeScript type definitions for all AST types.

use std::borrow::Cow;

use itertools::Itertools;
use lazy_regex::{Captures, Lazy, Regex, lazy_regex, regex::Replacer};

use crate::{
    Codegen, Generator, OXLINT_APP_PATH, TYPESCRIPT_DEFINITIONS_PATH,
    derives::estree::{
        get_fieldless_variant_value, get_struct_field_name, should_flatten_field,
        should_skip_enum_variant, should_skip_field,
    },
    output::Output,
    schema::{Def, EnumDef, FieldDef, Schema, StructDef, TypeDef, TypeId},
    utils::{FxIndexSet, format_cow, write_it},
};

use super::define_generator;

/// Generator for TypeScript type definitions.
pub struct TypescriptGenerator;

define_generator!(TypescriptGenerator);

impl Generator for TypescriptGenerator {
    /// Generate Typescript type definitions for all AST types.
    fn generate_many(&self, schema: &Schema, codegen: &Codegen) -> Vec<Output> {
        let code = generate_ts_type_defs(schema, codegen);
        let oxlint_code = amend_oxlint_types(&code);

        vec![
            Output::Javascript { path: TYPESCRIPT_DEFINITIONS_PATH.to_string(), code },
            Output::Javascript {
                path: format!("{OXLINT_APP_PATH}/src-js/generated/types.d.ts"),
                code: oxlint_code,
            },
        ]
    }
}

/// Generate Typescript type definitions for all types.
fn generate_ts_type_defs(schema: &Schema, codegen: &Codegen) -> String {
    let estree_derive_id = codegen.get_derive_id_by_name("ESTree");
    let program_type_id = schema.type_names["Program"];

    let mut code = String::new();
    let mut ast_node_names: Vec<String> = vec![];
    for type_def in &schema.types {
        if type_def.generates_derive(estree_derive_id) {
            generate_ts_type_def(type_def, &mut code, &mut ast_node_names, program_type_id, schema);
        }
    }

    // Manually append `ParamPattern`, which is generated via `add_ts_def`.
    // `ParamPattern` is a union type of other `add_ts_def`ed types.
    // TODO: Should not be hard-coded here.
    let ast_node_union = ast_node_names.join(" | ");
    write_it!(code, "export type Node = {ast_node_union} | ParamPattern;\n\n");

    code
}

/// Generate Typescript type definition for a struct or enum.
///
/// Push type defs to `code`.
fn generate_ts_type_def(
    type_def: &TypeDef,
    code: &mut String,
    ast_node_names: &mut Vec<String>,
    program_type_id: TypeId,
    schema: &Schema,
) {
    // Skip TS def generation if `#[estree(no_ts_def)]` attribute
    let no_ts_def = match type_def {
        TypeDef::Struct(struct_def) => &struct_def.estree.no_ts_def,
        TypeDef::Enum(enum_def) => &enum_def.estree.no_ts_def,
        _ => unreachable!(),
    };

    if !no_ts_def {
        let ts_def = match type_def {
            TypeDef::Struct(struct_def) => {
                generate_ts_type_def_for_struct(struct_def, ast_node_names, program_type_id, schema)
            }
            TypeDef::Enum(enum_def) => generate_ts_type_def_for_enum(enum_def, schema),
            _ => unreachable!(),
        };

        if let Some(ts_def) = ts_def {
            write_it!(code, "{ts_def};\n\n");
        }
    }

    // Add additional custom TS def if provided via `#[estree(add_ts_def = "...")]` attribute
    let add_ts_def = match type_def {
        TypeDef::Struct(struct_def) => &struct_def.estree.add_ts_def,
        TypeDef::Enum(enum_def) => &enum_def.estree.add_ts_def,
        _ => unreachable!(),
    };
    if let Some(add_ts_def) = add_ts_def {
        write_it!(code, "export {add_ts_def};\n\n");
    }
}

/// Generate Typescript type definition for a struct.
fn generate_ts_type_def_for_struct(
    struct_def: &StructDef,
    ast_node_names: &mut Vec<String>,
    program_type_id: TypeId,
    schema: &Schema,
) -> Option<String> {
    // If struct marked with `#[estree(ts_alias = "...")]`, then it needs no type def
    if struct_def.estree.ts_alias.is_some() {
        return None;
    }

    // If struct has a converter defined with `#[estree(via = Converter)]` and that converter defines
    // a type alias, then it needs no type def
    if let Some(converter_name) = &struct_def.estree.via
        && get_ts_type_for_converter(converter_name, schema).is_some()
    {
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
        write_it!(fields_str, "\n\ttype: '{type_name}';");
    }

    if !struct_def.estree.no_type {
        ast_node_names.push(type_name.to_string());
    }

    let mut output_as_type = false;

    for &field_index in &struct_def.estree.field_indices {
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

    if !struct_def.estree.no_type {
        let parent_type = if struct_def.id == program_type_id { "null" } else { "Node" };
        write_it!(fields_str, "\n\tparent?: {parent_type};");
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
    // Get fields to concatenate
    // (if fields marked `#[estree(prepend_to)]` or `#[estree(append_to)]` targeting this field)
    let mut concat_fields = [field; 3];
    let mut concat_field_count = 1;
    if let Some(prepend_field_index) = field.estree.prepend_field_index {
        concat_fields[0] = &struct_def.fields[prepend_field_index];
        concat_field_count = 2;
    }
    if let Some(append_field_index) = field.estree.append_field_index {
        concat_fields[concat_field_count] = &struct_def.fields[append_field_index];
        concat_field_count += 1;
    }

    let field_type_name = if concat_field_count > 1 {
        // Combine types of concatenated fields
        let mut field_type_name = "Array<".to_string();
        let mut include_null = false;
        for (index, &field) in concat_fields[..concat_field_count].iter().enumerate() {
            let field_type = match field.type_def(schema) {
                TypeDef::Option(option_def) => option_def.inner_type(schema),
                TypeDef::Vec(vec_def) => match vec_def.inner_type(schema) {
                    TypeDef::Option(option_def) => {
                        include_null = true;
                        option_def.inner_type(schema)
                    }
                    field_type => field_type,
                },
                _ => panic!(
                    "Appended field must be `Option<T>` or `Vec<T>`: `{}::{}`",
                    struct_def.name(),
                    field.name()
                ),
            };

            if index > 0 {
                field_type_name.push_str(" | ");
            }
            field_type_name.push_str(&ts_type_name(field_type, schema));
        }

        if include_null {
            field_type_name.push_str(" | null");
        }
        field_type_name.push('>');
        Cow::Owned(field_type_name)
    } else if let Some(converter_name) = &field.estree.via {
        let Some(ts_type) = get_ts_type_for_converter(converter_name, schema) else {
            panic!("No `ts_type` provided for ESTree converter `{converter_name}`");
        };
        Cow::Borrowed(ts_type)
    } else {
        get_field_type_name(field, schema)
    };

    if should_flatten_field(field, schema) {
        if let TypeDef::Struct(field_type) = field.type_def(schema)
            && let Some(flatten_field) = get_single_field(field_type, schema)
        {
            // Only one field to flatten. Add it as a field on the parent type, instead of extending.
            generate_ts_type_def_for_struct_field_impl(
                field_type,
                flatten_field,
                fields_str,
                extends,
                output_as_type,
                schema,
            );
        } else {
            // Need `type` instead of `interface` when flattening BindingPattern
            if field_type_name.contains('|') || field_type_name == "BindingPattern" {
                *output_as_type = true;
            }
            extends.push(field_type_name);
        }
        return;
    }

    let field_camel_name = get_struct_field_name(field);
    let question_mark = if field.estree.is_js || field.estree.is_ts { "?" } else { "" };
    write_it!(fields_str, "\n\t{field_camel_name}{question_mark}: {field_type_name};");
}

/// Generate Typescript type definition for an extra struct field
/// specified with `#[estree(add_fields(...))]`.
fn generate_ts_type_def_for_added_struct_field(
    field_name: &str,
    converter_name: &str,
    fields_str: &mut String,
    schema: &Schema,
) {
    let converter = schema.meta_by_name(converter_name);
    let Some(ts_type) = converter.estree.ts_type.as_deref() else {
        panic!("No `ts_type` provided for ESTree converter `{converter_name}`");
    };
    let question_mark = if converter.estree.is_js || converter.estree.is_ts { "?" } else { "" };
    write_it!(fields_str, "\n\t{field_name}{question_mark}: {ts_type};");
}

/// Get the TS type definition for a converter.
///
/// Converters are specified with `#[estree(add_fields(field_name = converter_name))]`
/// and `#[estree(via = converter_name)]`.
fn get_ts_type_for_converter<'s>(converter_name: &str, schema: &'s Schema) -> Option<&'s str> {
    let converter = schema.meta_by_name(converter_name);
    converter.estree.ts_type.as_deref()
}

/// Generate Typescript type definition for an enum.
fn generate_ts_type_def_for_enum(enum_def: &EnumDef, schema: &Schema) -> Option<String> {
    // If enum marked with `#[estree(ts_alias = "...")]`, then it needs no type def
    if enum_def.estree.ts_alias.is_some() {
        return None;
    }

    // If enum has a converter defined with `#[estree(via = Converter)]` and that converter defines
    // a type alias, then it needs no type def
    if let Some(converter_name) = &enum_def.estree.via
        && get_ts_type_for_converter(converter_name, schema).is_some()
    {
        return None;
    }

    // Get variant type names.
    // Collect into `FxIndexSet` to filter out duplicates.
    let mut variant_type_names = enum_def
        .variants
        .iter()
        .filter(|variant| !should_skip_enum_variant(variant))
        .map(|variant| {
            if let Some(converter_name) = &variant.estree.via {
                Cow::Borrowed(get_ts_type_for_converter(converter_name, schema).unwrap())
            } else if let Some(variant_type) = variant.field_type(schema) {
                ts_type_name(variant_type, schema)
            } else {
                format_cow!("'{}'", get_fieldless_variant_value(enum_def, variant))
            }
        })
        .collect::<FxIndexSet<_>>();

    variant_type_names.extend(
        enum_def.inherits_types(schema).map(|inherited_type| ts_type_name(inherited_type, schema)),
    );

    let union = variant_type_names.iter().join(" | ");

    let enum_name = enum_def.name();
    Some(format!("export type {enum_name} = {union};"))
}

/// Get TS type name for a type.
fn ts_type_name<'s>(type_def: &'s TypeDef, schema: &'s Schema) -> Cow<'s, str> {
    match type_def {
        TypeDef::Struct(struct_def) => {
            if let Some(ts_alias) = &struct_def.estree.ts_alias {
                Cow::Borrowed(ts_alias)
            } else if let Some(converter_name) = &struct_def.estree.via
                && let Some(type_name) = get_ts_type_for_converter(converter_name, schema)
            {
                Cow::Borrowed(type_name)
            } else {
                Cow::Borrowed(struct_def.name())
            }
        }
        TypeDef::Enum(enum_def) => {
            if let Some(ts_alias) = &enum_def.estree.ts_alias {
                Cow::Borrowed(ts_alias)
            } else if let Some(converter_name) = &enum_def.estree.via
                && let Some(type_name) = get_ts_type_for_converter(converter_name, schema)
            {
                Cow::Borrowed(type_name)
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
            format_cow!("{} | null", ts_type_name(option_def.inner_type(schema), schema))
        }
        TypeDef::Vec(vec_def) => {
            format_cow!("Array<{}>", ts_type_name(vec_def.inner_type(schema), schema))
        }
        TypeDef::Box(box_def) => ts_type_name(box_def.inner_type(schema), schema),
        TypeDef::Cell(cell_def) => ts_type_name(cell_def.inner_type(schema), schema),
        TypeDef::Pointer(pointer_def) => ts_type_name(pointer_def.inner_type(schema), schema),
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
        !struct_def.fields.iter().any(|field| {
            let field_name = field.estree.rename.as_deref().unwrap_or_else(|| field.name());
            field_name == "type"
        })
    }
}

/// If struct has only a single unskipped field, return it.
///
/// If no fields, or more than 1 unskipped field, returns `None`.
fn get_single_field<'s>(struct_def: &'s StructDef, schema: &Schema) -> Option<&'s FieldDef> {
    let mut fields_which_are_not_skipped =
        struct_def.fields.iter().filter(|field| !should_skip_field(field, schema));

    if let Some(field) = fields_which_are_not_skipped.next()
        && fields_which_are_not_skipped.next().is_none()
    {
        Some(field)
    } else {
        None
    }
}

/// Amend version of types for Oxlint.
fn amend_oxlint_types(code: &str) -> String {
    // Remove `export interface Span`, and instead import local version of same interface,
    // which includes non-optional `range` and `loc` fields.
    static SPAN_REGEX: Lazy<Regex> = lazy_regex!(r"export interface Span \{.+?\}");

    struct SpanReplacer;
    impl Replacer for SpanReplacer {
        fn replace_append(&mut self, _caps: &Captures, _dst: &mut String) {
            // Remove it
        }
    }

    let mut code = SPAN_REGEX.replace(code, SpanReplacer).into_owned();

    // Add `comments` field to `Program`
    #[expect(clippy::items_after_statements)]
    const HASHBANG_FIELD: &str = "hashbang: Hashbang | null;";
    let index = code.find(HASHBANG_FIELD).unwrap();
    code.insert_str(index + HASHBANG_FIELD.len(), "comments: Comment[];");

    #[rustfmt::skip]
    code.insert_str(0, "
        import { Span, Comment } from '../plugins/types.ts';
        export { Span, Comment };

    ");

    code
}
