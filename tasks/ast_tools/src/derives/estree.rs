//! Derive for `Serialize` impls, which serialize AST to ESTree format in JSON.

use std::borrow::Cow;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_str, Type};

use crate::{
    schema::{Def, EnumDef, FieldDef, Schema, StructDef, TypeDef, VariantDef},
    Result,
};

use super::{
    attr_positions, define_derive, AttrLocation, AttrPart, AttrPositions, Derive, StructOrEnum,
};

/// Derive for `Serialize` impls, which serialize AST to ESTree format in JSON.
pub struct DeriveESTree;

define_derive!(DeriveESTree);

impl Derive for DeriveESTree {
    fn trait_name(&self) -> &'static str {
        "ESTree"
    }

    fn snake_name(&self) -> String {
        "estree".to_string()
    }

    /// Register that accept `#[estree]` attr on structs, enums, struct fields, or enum variants.
    fn attrs(&self) -> &[(&'static str, AttrPositions)] {
        &[("estree", attr_positions!(Struct | Enum | StructField | EnumVariant))]
    }

    /// Parse `#[estree]` attr.
    fn parse_attr(&self, _attr_name: &str, location: AttrLocation, part: AttrPart) -> Result<()> {
        // No need to check attr name is `estree`, because that's the only attribute this derive handles
        parse_estree_attr(location, part)
    }

    fn prelude(&self) -> TokenStream {
        quote! {
            #![allow(unused_imports, unused_mut, clippy::match_same_arms)]

            ///@@line_break
            use serde::{Serialize, Serializer, ser::SerializeMap};
        }
    }

    /// Generate implementation of `Serialize` for a struct or enum.
    fn derive(&self, type_def: StructOrEnum, schema: &Schema) -> TokenStream {
        let body = match type_def {
            StructOrEnum::Struct(struct_def) => {
                if struct_def.estree.custom_serialize {
                    return quote!();
                }
                generate_body_for_struct(struct_def, schema)
            }
            StructOrEnum::Enum(enum_def) => generate_body_for_enum(enum_def, schema),
        };

        let ty = type_def.ty_anon(schema);

        quote! {
            impl Serialize for #ty {
                fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
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
            AttrPart::Tag("always_flatten") => struct_def.estree.always_flatten = true,
            AttrPart::Tag("no_type") => struct_def.estree.no_type = true,
            AttrPart::Tag("custom_serialize") => struct_def.estree.custom_serialize = true,
            AttrPart::String("rename", value) => struct_def.estree.rename = Some(value),
            AttrPart::String("via", value) => struct_def.estree.via = Some(value),
            AttrPart::String("add_ts", value) => struct_def.estree.add_ts = Some(value),
            _ => return Err(()),
        },
        // `#[estree]` attr on enum
        AttrLocation::Enum(enum_def) => match part {
            AttrPart::Tag("no_rename_variants") => enum_def.estree.no_rename_variants = true,
            AttrPart::Tag("custom_ts_def") => enum_def.estree.custom_ts_def = true,
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
        _ => unreachable!(),
    }

    Ok(())
}

/// Generate body of `serialize` method for a struct.
fn generate_body_for_struct(struct_def: &StructDef, schema: &Schema) -> TokenStream {
    if let Some(via_str) = struct_def.estree.via.as_deref() {
        let via_ty = parse_str::<Type>(via_str).unwrap();
        return quote! {
            #via_ty::from(self).serialize(serializer)
        };
    }

    let mut stmts = quote!();

    if should_add_type_field_to_struct(struct_def) {
        let type_name = struct_def.estree.rename.as_deref().unwrap_or_else(|| struct_def.name());
        stmts.extend(quote!( map.serialize_entry("type", #type_name)?; ));
    }

    for field in &struct_def.fields {
        if !field.estree.skip {
            stmts.extend(generate_stmt_for_struct_field(field, struct_def, schema));
        }
    }

    quote! {
        let mut map = serializer.serialize_map(None)?;
        #stmts
        map.end()
    }
}

/// Generate code to serialize a struct field.
fn generate_stmt_for_struct_field(
    field: &FieldDef,
    struct_def: &StructDef,
    schema: &Schema,
) -> TokenStream {
    let field_name_ident = field.ident();

    if should_flatten_field(field, schema) {
        return quote! {
            self.#field_name_ident.serialize(serde::__private::ser::FlatMapSerializer(&mut map))?;
        };
    }

    let field_camel_name = get_struct_field_name(field);

    let mut value = quote!( &self.#field_name_ident );
    if let Some(via_str) = field.estree.via.as_deref() {
        let via_ty = parse_str::<Type>(via_str).unwrap();
        value = quote!( &#via_ty(#value) );
    } else if let Some(append_field_index) = field.estree.append_field_index {
        let append_from_ident = struct_def.fields[append_field_index].ident();
        value = quote! {
            &oxc_estree::ser::AppendTo { array: #value, after: &self.#append_from_ident }
        };
    }

    quote! {
        map.serialize_entry(#field_camel_name, #value)?;
    }
}

/// Generate body of `serialize` method for an enum.
fn generate_body_for_enum(enum_def: &EnumDef, schema: &Schema) -> TokenStream {
    let enum_ident = enum_def.ident();

    if enum_def.is_fieldless() {
        let enum_name = enum_def.name();
        let match_branches = enum_def.all_variants(schema).map(|variant| {
            let variant_ident = variant.ident();
            // TODO: Don't print numbers as `0u32` - just `0` is fine
            let discriminant = u32::from(variant.discriminant);
            let value = get_fieldless_variant_value(enum_def, variant);

            quote! {
                #enum_ident::#variant_ident => {
                    serializer.serialize_unit_variant(#enum_name, #discriminant, #value)
                }
            }
        });

        quote! {
            match *self {
                #(#match_branches)*
            }
        }
    } else {
        let match_branches = enum_def.all_variants(schema).map(|variant| {
            let variant_ident = variant.ident();
            // TODO: Rename `x` to `it` to match other generated code
            quote! {
                #enum_ident::#variant_ident(x) => {
                    Serialize::serialize(x, serializer)
                }
            }
        });

        quote! {
            match self {
                #(#match_branches)*
            }
        }
    }
}

/// Get if should generate a `type` field.
///
/// Type field should be added unless struct has an `#[estree(no_type)]` attr
/// or struct has an existing field called `type`.
///
/// This function also used by Typescript generator.
pub fn should_add_type_field_to_struct(struct_def: &StructDef) -> bool {
    if struct_def.estree.no_type {
        false
    } else {
        !struct_def.fields.iter().any(|field| matches!(field.name(), "type"))
    }
}

/// Get if should flatten a struct field.
///
/// Returns `true` if either the field has an `#[estree(flatten)]` attr on it,
/// or the type that the field contains has an `#[estree(always_flatten)]` attr.
///
/// This function also used by Typescript generator.
pub fn should_flatten_field(field: &FieldDef, schema: &Schema) -> bool {
    if field.estree.flatten {
        true
    } else {
        let field_type = field.type_def(schema);
        matches!(field_type, TypeDef::Struct(field_struct_def) if field_struct_def.estree.always_flatten)
    }
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
