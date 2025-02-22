//! Derive for `ESTree` impls, which serialize AST to ESTree format in JSON.

use std::borrow::Cow;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Expr, parse_str};

use crate::{
    Result,
    schema::{Def, EnumDef, FieldDef, Schema, StructDef, TypeDef, VariantDef, Visibility},
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
    /// Also accept `#[ts]` attr on struct fields and enum variants.
    fn attrs(&self) -> &[(&'static str, AttrPositions)] {
        &[
            (
                "estree",
                attr_positions!(
                    StructMaybeDerived | EnumMaybeDerived | StructField | EnumVariant | Meta
                ),
            ),
            ("ts", attr_positions!(StructField | EnumVariant)),
        ]
    }

    /// Parse `#[estree]` and `#[ts]` attrs.
    fn parse_attr(&self, attr_name: &str, location: AttrLocation, part: AttrPart) -> Result<()> {
        match attr_name {
            "estree" => parse_estree_attr(location, part),
            "ts" => parse_ts_attr(location, &part),
            _ => unreachable!(),
        }
    }

    fn prelude(&self) -> TokenStream {
        quote! {
            #![allow(unused_imports, clippy::match_same_arms, clippy::semicolon_if_nothing_returned)]

            ///@@line_break
            use oxc_estree::{
                ser::{AppendTo, AppendToConcat},
                ESTree, FlatStructSerializer, Serializer, StructSerializer,
            };
        }
    }

    /// Generate implementation of `ESTree` for a struct or enum.
    fn derive(&self, type_def: StructOrEnum, schema: &Schema) -> TokenStream {
        let body = match type_def {
            StructOrEnum::Struct(struct_def) => {
                if struct_def.estree.custom_serialize {
                    return quote!();
                }
                generate_body_for_struct(struct_def, schema)
            }
            StructOrEnum::Enum(enum_def) => {
                if enum_def.estree.custom_serialize {
                    return quote!();
                }
                generate_body_for_enum(enum_def, schema)
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
}

/// Parse `#[estree]` attr.
fn parse_estree_attr(location: AttrLocation, part: AttrPart) -> Result<()> {
    // No need to check attr name is `estree`, because that's the only attribute this derive handles
    match location {
        // `#[estree]` attr on struct
        AttrLocation::Struct(struct_def) => match part {
            AttrPart::Tag("skip") => struct_def.estree.skip = true,
            AttrPart::Tag("flatten") => struct_def.estree.flatten = true,
            AttrPart::Tag("no_type") => struct_def.estree.no_type = true,
            AttrPart::Tag("custom_serialize") => struct_def.estree.custom_serialize = true,
            AttrPart::Tag("no_ts_def") => struct_def.estree.custom_ts_def = Some(String::new()),
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
                let field_indices = list
                    .into_iter()
                    .map(|list_element| {
                        let field_name = list_element.try_into_tag()?;
                        let field_name = field_name.trim_start_matches("r#");
                        all_field_names
                            .clone()
                            .position(|this_field_name| this_field_name == field_name)
                            .map(|index| u8::try_from(index).map_err(|_| ()))
                            .ok_or(())?
                    })
                    .collect::<Result<Vec<_>>>()?;
                struct_def.estree.field_indices = Some(field_indices);
            }
            AttrPart::String("custom_ts_def", value) => {
                struct_def.estree.custom_ts_def = Some(value);
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
            AttrPart::Tag("custom_serialize") => enum_def.estree.custom_serialize = true,
            AttrPart::Tag("no_ts_def") => enum_def.estree.custom_ts_def = Some(String::new()),
            AttrPart::String("custom_ts_def", value) => enum_def.estree.custom_ts_def = Some(value),
            AttrPart::String("ts_alias", value) => enum_def.estree.ts_alias = Some(value),
            AttrPart::String("add_ts_def", value) => {
                enum_def.estree.add_ts_def = Some(value);
            }
            _ => return Err(()),
        },
        // `#[estree]` attr on struct field
        AttrLocation::StructField(struct_def, field_index) => match part {
            AttrPart::Tag("skip") => struct_def.fields[field_index].estree.skip = true,
            AttrPart::Tag("flatten") => struct_def.fields[field_index].estree.flatten = true,
            AttrPart::String("rename", value) => {
                struct_def.fields[field_index].estree.rename = Some(value);
            }
            AttrPart::String("via", value) => {
                struct_def.fields[field_index].estree.via = Some(value);
            }
            AttrPart::String("append_to", value) => {
                // Find field this field is to be appended to
                let target_field_index = struct_def
                    .fields
                    .iter()
                    .enumerate()
                    .find(|(_, other_field)| other_field.name() == value)
                    .map(|(field_index, _)| field_index)
                    .ok_or(())?;
                if target_field_index == field_index {
                    // Can't append field to itself
                    return Err(());
                }
                let target_field = &mut struct_def.fields[target_field_index];
                if target_field.estree.append_field_index.is_some() {
                    // Can't append twice to same field
                    return Err(());
                }
                target_field.estree.append_field_index = Some(field_index);
                struct_def.fields[field_index].estree.skip = true;
            }
            AttrPart::String("ts_type", value) => {
                struct_def.fields[field_index].estree.ts_type = Some(value);
            }
            _ => return Err(()),
        },
        // `#[estree]` attr on enum variant
        AttrLocation::EnumVariant(enum_def, variant_index) => match part {
            AttrPart::String("rename", value) => {
                enum_def.variants[variant_index].estree.rename = Some(value);
            }
            _ => return Err(()),
        },
        // `#[estree]` attr on meta type
        AttrLocation::Meta(meta) => match part {
            AttrPart::String("ts_type", ts_type) => meta.estree.ts_type = Some(ts_type),
            _ => return Err(()),
        },
        _ => unreachable!(),
    }

    Ok(())
}

/// Parse `#[ts]` attr on struct field or enum variant.
fn parse_ts_attr(location: AttrLocation, part: &AttrPart) -> Result<()> {
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

/// Generate body of `serialize` method for a struct.
fn generate_body_for_struct(struct_def: &StructDef, schema: &Schema) -> TokenStream {
    if let Some(via_str) = struct_def.estree.via.as_deref() {
        let via_expr = parse_str::<Expr>(via_str).unwrap();
        return quote! {
            #via_expr.serialize(serializer)
        };
    }

    let krate = struct_def.file(schema).krate();
    let mut g = StructSerializerGenerator::new(!struct_def.estree.no_type, krate, schema);
    g.generate_stmts_for_struct(struct_def, &quote!(self));

    let type_field = if g.add_type_field {
        let type_name = struct_def.estree.rename.as_deref().unwrap_or_else(|| struct_def.name());
        quote! {
            state.serialize_field("type", #type_name);
        }
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
        if let Some(field_indices) = &struct_def.estree.field_indices {
            // Specified field order - serialize in this order
            for &field_index in field_indices {
                let field_index = field_index as usize;
                if let Some(field) = struct_def.fields.get(field_index) {
                    self.generate_stmts_for_field(field, struct_def, self_path);
                } else {
                    let (field_name, converter_name) =
                        &struct_def.estree.add_fields[field_index - struct_def.fields.len()];
                    self.generate_stmt_for_added_field(field_name, converter_name, self_path);
                }
            }
        } else {
            // No specified field order - serialize in original order
            for field in &struct_def.fields {
                self.generate_stmts_for_field(field, struct_def, self_path);
            }

            for (field_name, converter_name) in &struct_def.estree.add_fields {
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
            let converter = self.schema.meta_by_name(converter_name);
            let converter_path = converter.import_path_from_crate(self.krate, self.schema);
            quote!( #converter_path(#self_path) )
        } else if let Some(append_field_index) = field.estree.append_field_index {
            let append_field = &struct_def.fields[append_field_index];
            let append_from_ident = append_field.ident();
            let wrapper_name = if append_field.type_def(self.schema).is_option() {
                "AppendTo"
            } else {
                "AppendToConcat"
            };
            let wrapper_ident = create_safe_ident(wrapper_name);
            quote! {
                #wrapper_ident { array: &#self_path.#field_name_ident, after: &#self_path.#append_from_ident  }
            }
        } else {
            quote!( #self_path.#field_name_ident )
        };

        let serialize_method_ident = create_safe_ident(if field.estree.is_ts {
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
        self.stmts.extend(quote! {
            state.serialize_field(#field_name, &#converter_path(#self_path));
        });
    }
}

/// Generate body of `serialize` method for an enum.
fn generate_body_for_enum(enum_def: &EnumDef, schema: &Schema) -> TokenStream {
    let match_branches = enum_def.all_variants(schema).map(|variant| {
        let variant_ident = variant.ident();
        if variant.is_fieldless() {
            let value = get_fieldless_variant_value(enum_def, variant);
            quote! {
                Self::#variant_ident => #value.serialize(serializer),
            }
        } else {
            quote! {
                Self::#variant_ident(it) => it.serialize(serializer),
            }
        }
    });

    quote! {
        match self {
            #(#match_branches)*
        }
    }
}

/// Get if a struct field should be skipped when serializing.
///
/// Returns `true` if either the field has an `#[estree(skip)]` attr on it,
/// or the type that the field contains has an `#[estree(skip)]` attr.
///
/// This function also used by Typescript generator.
pub fn should_skip_field(field: &FieldDef, schema: &Schema) -> bool {
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

/// Get if should flatten a struct field.
///
/// Returns `true` if either the field has an `#[estree(flatten)]` attr on it,
/// or the type that the field contains has an `#[estree(flatten)]` attr.
///
/// This function also used by Typescript generator.
pub fn should_flatten_field(field: &FieldDef, schema: &Schema) -> bool {
    if field.estree.flatten {
        true
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
/// This function also used by Typescript generator.
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
/// This function also used by Typescript generator.
pub fn get_struct_field_name(field: &FieldDef) -> Cow<'_, str> {
    if let Some(field_name) = field.estree.rename.as_deref() {
        Cow::Borrowed(field_name)
    } else {
        Cow::Owned(field.camel_name())
    }
}
