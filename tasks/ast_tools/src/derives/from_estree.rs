//! Derive for `FromESTree` impls, which deserialize ESTree JSON into oxc AST.

use proc_macro2::TokenStream;
use quote::quote;

use crate::schema::{Def, EnumDef, FieldDef, Schema, StructDef, TypeDef, VariantDef};

use super::estree::{
    get_fieldless_variant_value, get_struct_field_name, should_skip_enum_variant, should_skip_field,
};
use super::{Derive, StructOrEnum, define_derive};

/// Derive for `FromESTree` impls, which deserialize ESTree JSON into oxc AST.
pub struct DeriveFromESTree;

define_derive!(DeriveFromESTree);

impl Derive for DeriveFromESTree {
    fn trait_name(&self) -> &'static str {
        "FromESTree"
    }

    fn trait_has_lifetime(&self) -> bool {
        true
    }

    fn crate_name(&self) -> &'static str {
        "oxc_ast"
    }

    fn snake_name(&self) -> String {
        "from_estree".to_string()
    }

    // We don't declare any attrs here because we reuse the ESTree schema extension
    // data which is already populated by DeriveESTree's attribute parsing.
    // The `#[estree]` attrs are owned by DeriveESTree.

    fn prelude(&self) -> TokenStream {
        quote! {
            #![allow(
                unused_imports,
                clippy::match_same_arms,
                clippy::semicolon_if_nothing_returned,
                clippy::too_many_lines
            )]

            ///@@line_break
            use oxc_allocator::{Allocator, Box as ABox, Vec as AVec};
            use crate::deserialize::{
                DeserError, DeserResult, ESTreeField, ESTreeType, FromESTree,
                parse_span, parse_span_or_empty,
            };
        }
    }

    /// Generate implementation of `FromESTree` for a struct or enum.
    fn derive(&self, type_def: StructOrEnum, schema: &Schema) -> TokenStream {
        match type_def {
            StructOrEnum::Struct(struct_def) => {
                if struct_def.estree.skip {
                    return quote!();
                }
                generate_impl_for_struct(struct_def, schema)
            }
            StructOrEnum::Enum(enum_def) => {
                if enum_def.estree.skip {
                    return quote!();
                }
                generate_impl_for_enum(enum_def, schema)
            }
        }
    }
}

/// Generate `FromESTree` implementation for a struct.
fn generate_impl_for_struct(struct_def: &StructDef, schema: &Schema) -> TokenStream {
    // Use concrete lifetime 'a (not anonymous '_) since we return data with that lifetime
    let ty = struct_def.ty_with_lifetime(schema, false);
    let (body, uses_allocator) = generate_body_for_struct(struct_def, schema);

    // Use underscore prefix for allocator if it's not used
    let allocator_param = if uses_allocator { quote!(allocator) } else { quote!(_allocator) };

    quote! {
        impl<'a> FromESTree<'a> for #ty {
            fn from_estree(value: &serde_json::Value, #allocator_param: &'a Allocator) -> DeserResult<Self> {
                #body
            }
        }
    }
}

/// Generate body of `from_estree` method for a struct.
/// Returns (body TokenStream, uses_allocator bool).
fn generate_body_for_struct(struct_def: &StructDef, schema: &Schema) -> (TokenStream, bool) {
    // Generate field deserializations and track if allocator is used
    let field_results: Vec<(TokenStream, bool)> = struct_def
        .fields
        .iter()
        .map(|field| generate_field_deserialization(field, struct_def, schema))
        .collect();

    let field_deserializations: Vec<_> = field_results.iter().map(|(ts, _)| ts.clone()).collect();
    let uses_allocator = field_results.iter().any(|(_, uses)| *uses);

    // Generate struct construction
    let field_assignments = struct_def
        .fields
        .iter()
        .map(|field| {
            let field_ident = field.ident();
            quote! { #field_ident }
        })
        .collect::<Vec<_>>();

    let struct_ident = struct_def.ident();

    let body = quote! {
        #(#field_deserializations)*
        Ok(#struct_ident {
            #(#field_assignments,)*
        })
    };

    (body, uses_allocator)
}

/// Generate deserialization for a single struct field.
/// Returns (TokenStream, uses_allocator bool).
fn generate_field_deserialization(
    field: &FieldDef,
    _struct_def: &StructDef,
    schema: &Schema,
) -> (TokenStream, bool) {
    let field_ident = field.ident();
    let field_name = field.name();

    // Special handling for span field
    if field_name == "span" {
        return (
            quote! {
                let span = parse_span_or_empty(value);
            },
            false, // span parsing doesn't use allocator
        );
    }

    // Skip fields marked with #[estree(skip)]
    if should_skip_field(field, schema) {
        return generate_default_field(field, schema);
    }

    // Get the ESTree field name (might be renamed)
    let estree_name = get_struct_field_name(field);
    let estree_name_str = estree_name.as_ref();

    // Get field type
    let field_type = field.type_def(schema);

    // Determine if field is optional
    let is_optional = matches!(field_type, TypeDef::Option(_));

    if is_optional {
        // For Option<T>, use estree_field_opt
        (
            quote! {
                let #field_ident = match value.estree_field_opt(#estree_name_str) {
                    Some(field_value) if !field_value.is_null() => {
                        Some(FromESTree::from_estree(field_value, allocator)?)
                    }
                    _ => None,
                };
            },
            true, // allocator is passed to FromESTree
        )
    } else {
        // For required fields
        (
            quote! {
                let #field_ident = FromESTree::from_estree(
                    value.estree_field(#estree_name_str)?,
                    allocator
                )?;
            },
            true, // allocator is passed to FromESTree
        )
    }
}

/// Generate default value for a skipped field.
/// Returns (TokenStream, uses_allocator bool).
fn generate_default_field(field: &FieldDef, schema: &Schema) -> (TokenStream, bool) {
    let field_ident = field.ident();
    let field_type = field.type_def(schema);

    // Try to generate a sensible default based on type
    let (default_value, uses_allocator) = match field_type {
        TypeDef::Option(_) => (quote!(None), false),
        TypeDef::Vec(_) => (quote!(AVec::new_in(allocator)), true),
        TypeDef::Cell(_) => (quote!(std::cell::Cell::default()), false),
        TypeDef::Primitive(p) => {
            let default = match p.name() {
                "bool" => quote!(false),
                "u8" | "u16" | "u32" | "u64" | "usize" => quote!(0),
                "i8" | "i16" | "i32" | "i64" | "isize" => quote!(0),
                "f32" | "f64" => quote!(0.0),
                _ => quote!(Default::default()),
            };
            (default, false)
        }
        _ => (quote!(Default::default()), false),
    };

    (
        quote! {
            let #field_ident = #default_value;
        },
        uses_allocator,
    )
}

/// Generate `FromESTree` implementation for an enum.
fn generate_impl_for_enum(enum_def: &EnumDef, schema: &Schema) -> TokenStream {
    // Use concrete lifetime 'a (not anonymous '_) since we return data with that lifetime
    let ty = enum_def.ty_with_lifetime(schema, false);

    // Check if this is a fieldless enum (like PropertyKind)
    if enum_def.is_fieldless() {
        return generate_impl_for_fieldless_enum(enum_def, schema);
    }

    // For enums with fields, we need to dispatch based on the `type` field
    let body = generate_body_for_enum_with_fields(enum_def, schema);

    quote! {
        impl<'a> FromESTree<'a> for #ty {
            fn from_estree(value: &serde_json::Value, allocator: &'a Allocator) -> DeserResult<Self> {
                #body
            }
        }
    }
}

/// Generate `FromESTree` implementation for a fieldless enum (string value).
fn generate_impl_for_fieldless_enum(enum_def: &EnumDef, schema: &Schema) -> TokenStream {
    // Use concrete lifetime 'a (not anonymous '_) since we return data with that lifetime
    let ty = enum_def.ty_with_lifetime(schema, false);

    let match_arms = enum_def
        .all_variants(schema)
        .filter(|variant| !should_skip_enum_variant(variant))
        .map(|variant| {
            let variant_ident = variant.ident();
            let value = get_fieldless_variant_value(enum_def, variant);
            let value_str = value.as_ref();
            quote! {
                #value_str => Ok(Self::#variant_ident),
            }
        })
        .collect::<Vec<_>>();

    let enum_name = enum_def.name();

    quote! {
        impl<'a> FromESTree<'a> for #ty {
            fn from_estree(value: &serde_json::Value, _allocator: &'a Allocator) -> DeserResult<Self> {
                let s = value.as_str().ok_or(DeserError::ExpectedString)?;
                match s {
                    #(#match_arms)*
                    other => Err(DeserError::InvalidFieldValue(
                        #enum_name,
                        other.to_string()
                    )),
                }
            }
        }
    }
}

/// Generate body for enum with fields - dispatch based on `type` field.
fn generate_body_for_enum_with_fields(enum_def: &EnumDef, schema: &Schema) -> TokenStream {
    let match_arms = enum_def
        .all_variants(schema)
        .filter(|variant| !should_skip_enum_variant(variant))
        .filter_map(|variant| generate_enum_variant_arm(variant, enum_def, schema))
        .collect::<Vec<_>>();

    quote! {
        let type_name = value.estree_type()?;
        match type_name {
            #(#match_arms)*
            other => Err(DeserError::UnknownNodeType(other.to_string())),
        }
    }
}

/// Generate a match arm for an enum variant.
fn generate_enum_variant_arm(
    variant: &VariantDef,
    _enum_def: &EnumDef,
    schema: &Schema,
) -> Option<TokenStream> {
    let variant_ident = variant.ident();

    // Get the inner type of the variant
    let inner_type = variant.field_type(schema)?;

    // Get the ESTree type name for this variant
    let estree_type_name = get_estree_type_name_for_type(inner_type, schema);

    // Check if this is a Box type (most enum variants are Box<T>)
    let is_boxed = matches!(inner_type, TypeDef::Box(_));

    if is_boxed {
        Some(quote! {
            #estree_type_name => {
                Ok(Self::#variant_ident(ABox::new_in(FromESTree::from_estree(value, allocator)?, allocator)))
            }
        })
    } else {
        Some(quote! {
            #estree_type_name => {
                Ok(Self::#variant_ident(FromESTree::from_estree(value, allocator)?))
            }
        })
    }
}

/// Get the ESTree `type` field value for a type.
fn get_estree_type_name_for_type<'a>(type_def: &'a TypeDef, schema: &'a Schema) -> &'a str {
    // Unwrap Box/Vec/Option to get to the actual type
    let inner_type = type_def.innermost_type(schema);

    match inner_type {
        TypeDef::Struct(struct_def) => {
            // Use the renamed name if specified, otherwise use the struct name
            struct_def.estree.rename.as_deref().unwrap_or_else(|| struct_def.name())
        }
        TypeDef::Enum(enum_def) => {
            // Enums don't have a single type name - this shouldn't happen for enum variants
            enum_def.name()
        }
        _ => {
            // Primitives don't have type names
            "unknown"
        }
    }
}
