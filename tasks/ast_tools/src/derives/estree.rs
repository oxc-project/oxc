//! Derive for `ESTree` impls, which serialize AST to ESTree format in JSON.

use std::borrow::Cow;

use proc_macro2::TokenStream;
use quote::quote;

use crate::{
    Result,
    codegen::{Codegen, DeriveId},
    schema::{Def, EnumDef, FieldDef, File, Schema, StructDef, TypeDef, VariantDef, Visibility},
    utils::create_safe_ident,
};

use super::{
    AttrLocation, AttrPart, AttrPositions, Derive, StructOrEnum, attr_positions, define_derive,
};

/// Derive for `ESTree` impls, which serialize AST to ESTree format in JSON.
pub struct DeriveESTree;

define_derive!(DeriveESTree);

impl Derive for DeriveESTree {
    fn trait_name(&self) -> &'static str {
        "ESTree"
    }

    fn crate_name(&self) -> &'static str {
        "oxc_estree"
    }

    fn snake_name(&self) -> String {
        "estree".to_string()
    }

    /// Register that accept `#[estree]` attr on structs, enums, struct fields, enum variants,
    /// or meta types.
    /// Allow attr on structs and enums which don't derive this trait.
    /// Also accept `#[ts]` and `#[js_only]` attrs on struct fields and meta types.
    fn attrs(&self) -> &[(&'static str, AttrPositions)] {
        &[
            (
                "estree",
                attr_positions!(
                    StructMaybeDerived | EnumMaybeDerived | StructField | EnumVariant | Meta
                ),
            ),
            ("ts", attr_positions!(StructField | Meta)),
            ("js_only", attr_positions!(StructField | Meta)),
        ]
    }

    /// Parse `#[estree]` and `#[ts]` attrs.
    fn parse_attr(&self, attr_name: &str, location: AttrLocation, part: AttrPart) -> Result<()> {
        match attr_name {
            "estree" => parse_estree_attr(location, part),
            "ts" => parse_ts_attr(location, &part),
            "js_only" => parse_js_only_attr(location, &part),
            _ => unreachable!(),
        }
    }

    /// Initialize `estree.field_order` on structs.
    fn prepare(&self, schema: &mut Schema, codegen: &Codegen) {
        let derive_id = codegen.get_derive_id_by_name(self.trait_name());
        prepare_field_orders(schema, derive_id);
    }

    fn prelude(&self) -> TokenStream {
        quote! {
            #![allow(unused_imports, clippy::match_same_arms, clippy::semicolon_if_nothing_returned)]

            ///@@line_break
            use oxc_estree::{
                Concat2, Concat3, ESTree, FlatStructSerializer,
                JsonSafeString, Serializer, StructSerializer,
            };
        }
    }

    /// Generate implementation of `ESTree` for a struct or enum.
    fn derive(&self, type_def: StructOrEnum, schema: &Schema) -> TokenStream {
        generate_impl_for_type(type_def, schema)
    }
}

/// Parse `#[estree]` attr.
fn parse_estree_attr(location: AttrLocation, part: AttrPart) -> Result<()> {
    match location {
        // `#[estree]` attr on struct
        AttrLocation::Struct(struct_def) => match part {
            AttrPart::Tag("skip") => struct_def.estree.skip = true,
            AttrPart::Tag("flatten") => struct_def.estree.flatten = true,
            AttrPart::Tag("no_type") => struct_def.estree.no_type = true,
            AttrPart::Tag("no_ts_def") => struct_def.estree.no_ts_def = true,
            AttrPart::Tag("no_parent") => struct_def.estree.no_parent = true,
            AttrPart::List("add_fields", list) => {
                for list_element in list {
                    let (name, value) = list_element.try_into_string()?;
                    struct_def.estree.add_fields.push((name, value));
                }
            }
            AttrPart::List("field_order", list) => {
                // Get iterator over all field names (including added fields)
                let all_field_names = struct_def.fields.iter().map(FieldDef::name).chain(
                    struct_def.estree.add_fields.iter().map(|(field_name, _)| field_name.as_str()),
                );

                // Convert field names to indexes.
                // Added fields (`#[estree(add_fields(...))]`) get indexes after the real fields.
                // Error if same field name included more than once.
                let mut field_indices = vec![];
                for list_element in list {
                    let field_name = list_element.try_into_tag()?;
                    let field_index = all_field_names
                        .clone()
                        .position(|this_field_name| this_field_name == field_name)
                        .ok_or(())?;
                    let field_index = u8::try_from(field_index).map_err(|_| ())?;
                    if field_indices.contains(&field_index) {
                        return Err(());
                    }
                    field_indices.push(field_index);
                }
                struct_def.estree.field_indices = field_indices;
            }
            AttrPart::String("ts_alias", value) => struct_def.estree.ts_alias = Some(value),
            AttrPart::String("add_ts_def", value) => struct_def.estree.add_ts_def = Some(value),
            AttrPart::String("rename", value) => struct_def.estree.rename = Some(value),
            AttrPart::String("via", value) => struct_def.estree.via = Some(value),
            _ => return Err(()),
        },
        // `#[estree]` attr on enum
        AttrLocation::Enum(enum_def) => match part {
            AttrPart::Tag("skip") => enum_def.estree.skip = true,
            AttrPart::Tag("no_rename_variants") => enum_def.estree.no_rename_variants = true,
            AttrPart::Tag("no_ts_def") => enum_def.estree.no_ts_def = true,
            AttrPart::String("ts_alias", value) => enum_def.estree.ts_alias = Some(value),
            AttrPart::String("add_ts_def", value) => {
                enum_def.estree.add_ts_def = Some(value);
            }
            AttrPart::String("via", value) => enum_def.estree.via = Some(value),
            _ => return Err(()),
        },
        // `#[estree]` attr on struct field
        AttrLocation::StructField(struct_def, field_index) => match part {
            AttrPart::Tag("skip") => struct_def.fields[field_index].estree.skip = true,
            AttrPart::Tag("flatten") => struct_def.fields[field_index].estree.flatten = true,
            AttrPart::Tag("no_flatten") => struct_def.fields[field_index].estree.no_flatten = true,
            AttrPart::Tag("json_safe") => struct_def.fields[field_index].estree.json_safe = true,
            AttrPart::String("rename", value) => {
                struct_def.fields[field_index].estree.rename = Some(value);
            }
            AttrPart::String("via", value) => {
                struct_def.fields[field_index].estree.via = Some(value);
            }
            AttrPart::String(attr @ ("prepend_to" | "append_to"), value) => {
                // Find field this field is to be prepended/appended to
                let target_field_index = struct_def
                    .fields
                    .iter()
                    .enumerate()
                    .find(|(_, other_field)| other_field.name() == value)
                    .map(|(field_index, _)| field_index)
                    .ok_or(())?;
                if target_field_index == field_index {
                    // Can't prepend/append field to itself
                    return Err(());
                }
                let target_field = &mut struct_def.fields[target_field_index];
                let other_field_index_mut = if attr == "prepend_to" {
                    &mut target_field.estree.prepend_field_index
                } else {
                    &mut target_field.estree.append_field_index
                };
                if other_field_index_mut.is_some() {
                    // Can't prepend/append twice to same field
                    return Err(());
                }
                *other_field_index_mut = Some(field_index);
                struct_def.fields[field_index].estree.skip = true;
            }
            AttrPart::String("ts_type", value) => {
                struct_def.fields[field_index].estree.ts_type = Some(value);
            }
            _ => return Err(()),
        },
        // `#[estree]` attr on enum variant
        AttrLocation::EnumVariant(enum_def, variant_index) => match part {
            AttrPart::Tag("skip") => {
                enum_def.variants[variant_index].estree.skip = true;
            }
            AttrPart::String("rename", value) => {
                enum_def.variants[variant_index].estree.rename = Some(value);
            }
            AttrPart::String("via", value) => {
                enum_def.variants[variant_index].estree.via = Some(value);
            }
            _ => return Err(()),
        },
        // `#[estree]` attr on meta type
        AttrLocation::Meta(meta) => match part {
            AttrPart::String("ts_type", ts_type) => meta.estree.ts_type = Some(ts_type),
            AttrPart::String("raw_deser", raw_deser) => meta.estree.raw_deser = Some(raw_deser),
            _ => return Err(()),
        },
        _ => unreachable!(),
    }

    Ok(())
}

/// Parse `#[ts]` attr on struct field or meta type.
fn parse_ts_attr(location: AttrLocation, part: &AttrPart) -> Result<()> {
    if !matches!(part, AttrPart::None) {
        return Err(());
    }

    // Location can only be `StructField` or `Meta`
    match location {
        AttrLocation::StructField(struct_def, field_index) => {
            struct_def.fields[field_index].estree.is_ts = true;
        }
        AttrLocation::Meta(meta) => meta.estree.is_ts = true,
        _ => unreachable!(),
    }

    Ok(())
}

/// Parse `#[js_only]` attr on struct field or meta type.
fn parse_js_only_attr(location: AttrLocation, part: &AttrPart) -> Result<()> {
    if !matches!(part, AttrPart::None) {
        return Err(());
    }

    // Location can only be `StructField` or `Meta`
    match location {
        AttrLocation::StructField(struct_def, field_index) => {
            struct_def.fields[field_index].estree.is_js = true;
        }
        AttrLocation::Meta(meta) => meta.estree.is_js = true,
        _ => unreachable!(),
    }

    Ok(())
}

/// Initialize `estree.field_order` on all structs.
fn prepare_field_orders(schema: &mut Schema, estree_derive_id: DeriveId) {
    // Note: Outside the loop to avoid allocating temporary `Vec`s on each turn of the loop.
    // Instead, reuse this `Vec` over and over.
    let mut unskipped_field_indices = vec![];

    for type_id in schema.types.indices() {
        let Some(struct_def) = schema.types[type_id].as_struct() else { continue };
        if !struct_def.generates_derive(estree_derive_id) {
            continue;
        }

        if struct_def.estree.field_indices.is_empty() {
            // No field order specified with `#[estree(field_order(...))]`.
            // Default field order is:
            // 1. `type` field (if present)
            // 2. Struct fields, in definition order.
            // 3. Extra fields (`#[estree(add_fields(...)]`), in order.
            // 4. `span` field (if present)
            let mut field_indices = vec![];
            let mut type_field_index = None;
            let mut span_field_index = None;
            for (field_index, field) in struct_def.fields.iter().enumerate() {
                if !should_skip_field(field, schema) {
                    let field_index = u8::try_from(field_index).unwrap();
                    match field.name() {
                        "type" => type_field_index = Some(field_index),
                        "span" => span_field_index = Some(field_index),
                        _ => field_indices.push(field_index),
                    }
                }
            }

            if let Some(type_field_index) = type_field_index {
                field_indices.insert(0, type_field_index);
            }

            if !struct_def.estree.add_fields.is_empty() {
                let first_index = u8::try_from(struct_def.fields.len()).unwrap();
                let last_index =
                    u8::try_from(struct_def.fields.len() + struct_def.estree.add_fields.len() - 1)
                        .unwrap();
                field_indices.extend(first_index..=last_index);
            }

            if let Some(span_field_index) = span_field_index {
                field_indices.push(span_field_index);
            }

            let struct_def = schema.struct_def_mut(type_id);
            struct_def.estree.field_indices = field_indices;
        } else {
            // Custom field order specified with `#[estree(field_order(...))]`.
            // Verify does not miss any fields, no fields marked `#[estree(skip)]` are included.
            for (field_index, field) in struct_def.fields.iter().enumerate() {
                if !should_skip_field(field, schema) {
                    let field_index = u8::try_from(field_index).unwrap();
                    unskipped_field_indices.push(field_index);
                }
            }

            let fields_len = struct_def.fields.len();
            for &field_index in &struct_def.estree.field_indices {
                if (field_index as usize) < fields_len {
                    assert!(
                        unskipped_field_indices.contains(&field_index),
                        "Skipped field `{}` included in `#[estree(field_order)]`: `{}`",
                        struct_def.fields[field_index as usize].name(),
                        struct_def.name()
                    );
                }
            }

            assert!(
                struct_def.estree.field_indices.len()
                    == unskipped_field_indices.len() + struct_def.estree.add_fields.len(),
                "`#[estree(field_order)]` misses fields: `{}`",
                struct_def.name()
            );

            unskipped_field_indices.clear();
        }
    }
}

/// Generate implementation of `ESTree` for a struct or enum.
fn generate_impl_for_type(type_def: StructOrEnum, schema: &Schema) -> TokenStream {
    let body = match type_def {
        StructOrEnum::Struct(struct_def) => {
            if let Some(converter_name) = &struct_def.estree.via {
                generate_body_for_via_override(converter_name, struct_def.file(schema), schema)
            } else {
                generate_body_for_struct(struct_def, schema)
            }
        }
        StructOrEnum::Enum(enum_def) => {
            if let Some(converter_name) = &enum_def.estree.via {
                generate_body_for_via_override(converter_name, enum_def.file(schema), schema)
            } else {
                generate_body_for_enum(enum_def, schema)
            }
        }
    };

    let ty = type_def.ty_anon(schema);

    quote! {
        impl ESTree for #ty {
            fn serialize<S: Serializer>(&self, serializer: S) {
                #body
            }
        }
    }
}

/// Generate body of `serialize` method for a struct.
fn generate_body_for_struct(struct_def: &StructDef, schema: &Schema) -> TokenStream {
    let krate = struct_def.file(schema).krate();
    let mut g = StructSerializerGenerator::new(!struct_def.estree.no_type, krate, schema);
    g.generate_stmts_for_struct(struct_def, &quote!(self));

    let type_field = if g.add_type_field {
        let type_name = struct_def.estree.rename.as_deref().unwrap_or_else(|| struct_def.name());
        let type_name = string_to_tokens(type_name, true);
        quote!( state.serialize_field("type", #type_name); )
    } else {
        quote!()
    };

    let stmts = g.stmts;
    quote! {
        let mut state = serializer.serialize_struct();
        #type_field
        #stmts
        state.end();
    }
}

/// Generator for stmts to serialize fields of a struct.
///
/// Recursively enters any flattened fields which contain a struct,
/// and generates statements for each of the flattened struct's fields too.
///
/// If a field called `type` is found, `add_type_field` is set to `false`.
struct StructSerializerGenerator<'s> {
    /// `serialize` statements
    stmts: TokenStream,
    /// `true` if a `type` field should be added.
    /// `false` one already exists (or if `#[estree(no_type)]` attr on struct).
    add_type_field: bool,
    /// Crate in which the `ESTree` impl for the type will be generated
    krate: &'s str,
    /// Schema
    schema: &'s Schema,
}

impl<'s> StructSerializerGenerator<'s> {
    /// Create new [`StructSerializerGenerator`].
    fn new(add_type_field: bool, krate: &'s str, schema: &'s Schema) -> Self {
        Self { stmts: quote!(), add_type_field, krate, schema }
    }

    /// Generate code to serialize all fields in a struct.
    fn generate_stmts_for_struct(&mut self, struct_def: &StructDef, self_path: &TokenStream) {
        for &field_index in &struct_def.estree.field_indices {
            let field_index = field_index as usize;
            if let Some(field) = struct_def.fields.get(field_index) {
                self.generate_stmts_for_field(field, struct_def, self_path);
            } else {
                let (field_name, converter_name) =
                    &struct_def.estree.add_fields[field_index - struct_def.fields.len()];
                self.generate_stmt_for_added_field(field_name, converter_name, self_path);
            }
        }
    }

    /// Generate code to serialize a struct field.
    fn generate_stmts_for_field(
        &mut self,
        field: &FieldDef,
        struct_def: &StructDef,
        self_path: &TokenStream,
    ) {
        if should_skip_field(field, self.schema) {
            return;
        }

        if field.name() == "span" {
            self.stmts.extend(quote! {
                state.serialize_span(#self_path.span);
            });
            return;
        }

        let field_name_ident = field.ident();

        if should_flatten_field(field, self.schema) {
            if can_flatten_field_inline(field, self.krate, self.schema) {
                let inner_struct_def = field.type_def(self.schema).as_struct().unwrap();
                self.generate_stmts_for_struct(
                    inner_struct_def,
                    &quote!(#self_path.#field_name_ident),
                );
                return;
            }

            let field_type = field.type_def(self.schema);
            assert!(
                field_type.is_struct() || field_type.is_enum(),
                "Cannot flatten a field which is not a struct or enum: `{}::{}`",
                struct_def.name(),
                field_type.name(),
            );

            self.stmts.extend(quote! {
                #self_path.#field_name_ident.serialize(FlatStructSerializer(&mut state));
            });
            return;
        }

        let field_camel_name = get_struct_field_name(field);

        if field_camel_name == "type" {
            self.add_type_field = false;
        }

        let value = if let Some(converter_name) = &field.estree.via {
            let converter_path = get_converter_path(converter_name, self.krate, self.schema);
            quote!( #converter_path(#self_path) )
        } else if let Some(prepend_field_index) = field.estree.prepend_field_index {
            let prepend_from_ident = struct_def.fields[prepend_field_index].ident();
            if let Some(append_field_index) = field.estree.append_field_index {
                let append_from_ident = struct_def.fields[append_field_index].ident();
                quote! {
                    Concat3(&#self_path.#prepend_from_ident, &#self_path.#field_name_ident, &#self_path.#append_from_ident)
                }
            } else {
                quote! {
                    Concat2(&#self_path.#prepend_from_ident, &#self_path.#field_name_ident)
                }
            }
        } else if let Some(append_field_index) = field.estree.append_field_index {
            let append_from_ident = struct_def.fields[append_field_index].ident();
            quote! {
                Concat2(&#self_path.#field_name_ident, &#self_path.#append_from_ident)
            }
        } else if field.estree.json_safe {
            // Wrap value in `JsonSafeString(...)` if field is tagged `#[estree(json_safe)]`
            let value = match field.type_def(self.schema) {
                TypeDef::Primitive(primitive_def) => match primitive_def.name() {
                    "&str" => Some(quote!( JsonSafeString(#self_path.#field_name_ident) )),
                    "Atom" => Some(quote!( JsonSafeString(#self_path.#field_name_ident.as_str()) )),
                    _ => None,
                },
                TypeDef::Option(option_def) => option_def
                    .inner_type(self.schema)
                    .as_primitive()
                    .and_then(|primitive_def| match primitive_def.name() {
                        "&str" => Some(quote! {
                            #self_path.#field_name_ident.map(|s| JsonSafeString(s))
                        }),
                        "Atom" => Some(quote! {
                            #self_path.#field_name_ident.map(|s| JsonSafeString(s.as_str()))
                        }),
                        _ => None,
                    }),
                _ => None,
            };

            value.unwrap_or_else(|| {
                panic!(
                    "`#[estree(json_safe)]` is only valid on struct fields containing a `&str` or `Atom`: {}::{}",
                    struct_def.name(),
                    field.name(),
                )
            })
        } else {
            quote!( #self_path.#field_name_ident )
        };

        let serialize_method_ident = create_safe_ident(if field.estree.is_js {
            "serialize_js_field"
        } else if field.estree.is_ts {
            "serialize_ts_field"
        } else {
            "serialize_field"
        });

        self.stmts.extend(quote! {
            state.#serialize_method_ident(#field_camel_name, &#value);
        });
    }

    fn generate_stmt_for_added_field(
        &mut self,
        field_name: &str,
        converter_name: &str,
        self_path: &TokenStream,
    ) {
        let converter = self.schema.meta_by_name(converter_name);
        let converter_path = converter.import_path_from_crate(self.krate, self.schema);
        let serialize_method_ident = create_safe_ident(if converter.estree.is_js {
            "serialize_js_field"
        } else if converter.estree.is_ts {
            "serialize_ts_field"
        } else {
            "serialize_field"
        });
        self.stmts.extend(quote! {
            state.#serialize_method_ident(#field_name, &#converter_path(#self_path));
        });
    }
}

/// Generate body of `serialize` method for an enum.
fn generate_body_for_enum(enum_def: &EnumDef, schema: &Schema) -> TokenStream {
    let match_branches = enum_def.all_variants(schema).map(|variant| {
        let variant_ident = variant.ident();

        if should_skip_enum_variant(variant) {
            let pattern = if variant.is_fieldless() { quote!() } else { quote!((_)) };
            return quote! {
                Self::#variant_ident #pattern => unreachable!("This enum variant is skipped."),
            };
        }

        let converter_path = variant.estree.via.as_deref().map(|converter_name| {
            get_converter_path(converter_name, enum_def.file(schema).krate(), schema)
        });

        if variant.is_fieldless() {
            let value = if let Some(converter_path) = converter_path {
                quote!( #converter_path(()) )
            } else {
                let value = get_fieldless_variant_value(enum_def, variant);
                string_to_tokens(value.as_ref(), false)
            };

            quote! {
                Self::#variant_ident => #value.serialize(serializer),
            }
        } else {
            let value = if let Some(converter_path) = converter_path {
                quote!( #converter_path(it) )
            } else {
                quote!(it)
            };

            quote! {
                Self::#variant_ident(it) => #value.serialize(serializer),
            }
        }
    });

    quote! {
        match self {
            #(#match_branches)*
        }
    }
}

/// Generate body of `serialize` method for a struct or enum with `#[estree(via = ...)]` attribute.
fn generate_body_for_via_override(
    converter_name: &str,
    file: &File,
    schema: &Schema,
) -> TokenStream {
    let converter_path = get_converter_path(converter_name, file.krate(), schema);
    quote!( #converter_path(self).serialize(serializer) )
}

/// Get path to converter from crate `from_krate`.
/// e.g. `oxc_ast::serialize::Null` or `crate::serialize::Null`.
fn get_converter_path(converter_name: &str, from_krate: &str, schema: &Schema) -> TokenStream {
    let converter = schema.meta_by_name(converter_name);
    converter.import_path_from_crate(from_krate, schema)
}

/// Get if a struct field should be skipped when serializing.
///
/// Returns `true` if either the field has an `#[estree(skip)]` attr on it,
/// or the type that the field contains has an `#[estree(skip)]` attr.
///
/// This function also used by Typescript and raw transfer generators.
pub fn should_skip_field(field: &FieldDef, schema: &Schema) -> bool {
    // Always skip node_id field - it's internal and not part of ESTree serialization
    if field.name() == "node_id" {
        return true;
    }
    if field.estree.skip {
        true
    } else {
        let innermost_type = field.type_def(schema).innermost_type(schema);
        match innermost_type {
            TypeDef::Struct(struct_def) => struct_def.estree.skip,
            TypeDef::Enum(enum_def) => enum_def.estree.skip,
            _ => false,
        }
    }
}

/// Get if an enum variant should be skipped when serializing.
///
/// Returns `true` if the variant has an `#[estree(skip)]` attr on it.
///
/// This function also used by Typescript and raw transfer generators.
pub fn should_skip_enum_variant(variant: &VariantDef) -> bool {
    variant.estree.skip
}

/// Get if should flatten a struct field.
///
/// Returns `true` if either the field has an `#[estree(flatten)]` attr on it,
/// or the type that the field contains has an `#[estree(flatten)]` attr.
///
/// This function also used by Typescript generator.
pub fn should_flatten_field(field: &FieldDef, schema: &Schema) -> bool {
    if field.estree.flatten {
        true
    } else if field.estree.no_flatten {
        false
    } else {
        let field_type = field.type_def(schema);
        matches!(field_type, TypeDef::Struct(field_struct_def) if field_struct_def.estree.flatten)
    }
}

/// Get if struct field can be flattened inline.
///
/// If the field's type is an enum, then it can't.
///
/// If the field's type is a struct, then usually it can.
/// But it can't in the case where that type is defined in a different crate from where
/// the `ESTree` impl will be generated, and one of the flattened fields is not public.
pub fn can_flatten_field_inline(field: &FieldDef, krate: &str, schema: &Schema) -> bool {
    let field_type = field.type_def(schema);
    let TypeDef::Struct(struct_def) = field_type else { return false };

    struct_def.fields.iter().all(|field| {
        if should_skip_field(field, schema) {
            true
        } else {
            match field.visibility {
                Visibility::Public => true,
                Visibility::Restricted => struct_def.file(schema).krate() == krate,
                Visibility::Private => false,
            }
        }
    })
}

/// Get value of a fieldless enum variant.
///
/// Value is determined by:
/// * `#[estree(rename)]` attr on variant.
/// * `#[estree(no_rename_variants)]` attr on enum.
///
/// This function also used by Typescript and raw transfer generators.
pub fn get_fieldless_variant_value<'s>(
    enum_def: &'s EnumDef,
    variant: &'s VariantDef,
) -> Cow<'s, str> {
    if let Some(variant_name) = variant.estree.rename.as_deref() {
        Cow::Borrowed(variant_name)
    } else if enum_def.estree.no_rename_variants {
        Cow::Borrowed(variant.name())
    } else {
        Cow::Owned(variant.camel_name())
    }
}

/// Get ESTree name for struct field.
///
/// This function also used by Typescript and raw transfer generators.
pub fn get_struct_field_name(field: &FieldDef) -> Cow<'_, str> {
    if let Some(field_name) = field.estree.rename.as_deref() {
        Cow::Borrowed(field_name)
    } else {
        Cow::Owned(field.camel_name())
    }
}

/// Convert string to [`TokenStream`] representing string literal.
///
/// If the string contains no characters which need escaping in JSON,
/// returns tokens for `JsonSafeString("string")`, which is faster to serialize.
///
/// If `as_ref` is `true`, and string is JSON-safe, returns tokens for `&JsonSafeString("string")`.
fn string_to_tokens(str: &str, as_ref: bool) -> TokenStream {
    let contains_chars_needing_escaping =
        str.as_bytes().iter().any(|&b| b < 32 || b == b'"' || b == b'\\');
    if contains_chars_needing_escaping {
        quote!(#str)
    } else if as_ref {
        quote!( &JsonSafeString(#str) )
    } else {
        quote!( JsonSafeString(#str) )
    }
}
