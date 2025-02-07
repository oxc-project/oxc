//! Derive for `Serialize` impls, which serialize AST to ESTree format in JSON.

use std::borrow::Cow;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_str, Type};

use crate::{
    schema::{Def, EnumDef, FieldDef, Schema, StructDef, TypeDef, VariantDef, Visibility},
    utils::number_lit,
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

    fn crate_name(&self) -> &'static str {
        "oxc_estree"
    }

    fn snake_name(&self) -> String {
        "estree".to_string()
    }

    /// Register that accept `#[estree]` attr on structs, enums, struct fields, or enum variants.
    /// Allow attr on structs and enums which don't derive this trait.
    fn attrs(&self) -> &[(&'static str, AttrPositions)] {
        &[(
            "estree",
            attr_positions!(StructMaybeDerived | EnumMaybeDerived | StructField | EnumVariant),
        )]
    }

    /// Parse `#[estree]` attr.
    fn parse_attr(&self, _attr_name: &str, location: AttrLocation, part: AttrPart) -> Result<()> {
        // No need to check attr name is `estree`, because that's the only attribute this derive handles
        parse_estree_attr(location, part)
    }

    fn prelude(&self) -> TokenStream {
        quote! {
            #![allow(unused_imports, clippy::match_same_arms)]

            ///@@line_break
            use serde::{
                __private::ser::FlatMapSerializer,
                ser::SerializeMap,
                Serialize, Serializer
            };

            ///@@line_break
            use oxc_estree::ser::AppendTo;
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
            StructOrEnum::Enum(enum_def) => {
                if enum_def.estree.custom_serialize {
                    return quote!();
                }
                generate_body_for_enum(enum_def, schema)
            }
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
            AttrPart::String("add_ts", value) => struct_def.estree.add_ts = Some(value),
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

    let krate = struct_def.file(schema).krate();
    let mut gen = StructSerializerGenerator::new(!struct_def.estree.no_type, krate, schema);
    gen.generate_stmts_for_struct(struct_def, &quote!(self));

    let type_field = if gen.add_type_field {
        let type_name = struct_def.estree.rename.as_deref().unwrap_or_else(|| struct_def.name());
        quote! {
            map.serialize_entry("type", #type_name)?;
        }
    } else {
        quote!()
    };

    // Add any additional manually-defined fields
    let add_fields = struct_def.estree.add_fields.iter().map(|(name, value)| {
        let value = parse_str::<syn::Expr>(value).unwrap();
        quote!( map.serialize_entry(#name, &#value)?; )
    });

    let stmts = gen.stmts;
    quote! {
        let mut map = serializer.serialize_map(None)?;
        #type_field
        #stmts
        #(#add_fields)*
        map.end()
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
    /// Crate in which the `Serialize` impl for the type will be generated
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
        for field in &struct_def.fields {
            self.generate_stmts_for_field(field, struct_def, self_path);
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
                #self_path.#field_name_ident.serialize(FlatMapSerializer(&mut map))?;
            });
            return;
        }

        let field_camel_name = get_struct_field_name(field);

        if field_camel_name == "type" {
            self.add_type_field = false;
        }

        let mut value = quote!( #self_path.#field_name_ident );
        if let Some(via_str) = field.estree.via.as_deref() {
            let via_ty = parse_str::<Type>(via_str).unwrap();
            value = quote!( #via_ty::from(&#value) );
        } else if let Some(append_field_index) = field.estree.append_field_index {
            let append_from_ident = struct_def.fields[append_field_index].ident();
            value = quote! {
                AppendTo { array: &#value, after: &#self_path.#append_from_ident }
            };
        }

        self.stmts.extend(quote! {
            map.serialize_entry(#field_camel_name, &#value)?;
        });
    }
}

/// Generate body of `serialize` method for an enum.
fn generate_body_for_enum(enum_def: &EnumDef, schema: &Schema) -> TokenStream {
    let enum_ident = enum_def.ident();

    let match_branches = enum_def.all_variants(schema).map(|variant| {
        let variant_ident = variant.ident();
        if variant.is_fieldless() {
            let enum_name = enum_def.name();
            let discriminant = number_lit(variant.discriminant);
            let value = get_fieldless_variant_value(enum_def, variant);
            quote! {
                #enum_ident::#variant_ident => serializer.serialize_unit_variant(#enum_name, #discriminant, #value),
            }
        } else {
            quote! {
                #enum_ident::#variant_ident(it) => it.serialize(serializer),
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
/// the `Serialize` impl will be generated, and one of the flattened fields is not public.
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
