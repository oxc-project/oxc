//! Derive for `ContentEq` trait.

use proc_macro2::TokenStream;
use quote::quote;

use crate::{
    Result,
    schema::{Def, EnumDef, Schema, StructDef, TypeDef},
    utils::create_safe_ident,
};

use super::{
    AttrLocation, AttrPart, AttrPositions, Derive, StructOrEnum, attr_positions, define_derive,
};

/// Derive for `ContentEq` trait.
pub struct DeriveContentEq;

define_derive!(DeriveContentEq);

impl Derive for DeriveContentEq {
    fn trait_name(&self) -> &'static str {
        "ContentEq"
    }

    fn crate_name(&self) -> &'static str {
        "oxc_span"
    }

    /// Register that accept `#[content_eq]` attr on structs, enums, or struct fields.
    /// Allow attr on structs and enums which don't derive this trait.
    fn attrs(&self) -> &[(&'static str, AttrPositions)] {
        &[("content_eq", attr_positions!(StructMaybeDerived | EnumMaybeDerived | StructField))]
    }

    /// Parse `#[content_eq(skip)]` attr.
    fn parse_attr(&self, _attr_name: &str, location: AttrLocation, part: AttrPart) -> Result<()> {
        // No need to check attr name is `content_eq`, because that's the only attribute this derive handles.
        if !matches!(part, AttrPart::Tag("skip")) {
            return Err(());
        }

        match location {
            AttrLocation::Struct(struct_def) => struct_def.content_eq.skip = true,
            AttrLocation::Enum(enum_def) => enum_def.content_eq.skip = true,
            AttrLocation::StructField(struct_def, field_index) => {
                struct_def.fields[field_index].content_eq.skip = true;
            }
            _ => return Err(()),
        }

        Ok(())
    }

    fn prelude(&self) -> TokenStream {
        quote! {
            #![allow(clippy::match_same_arms)]

            ///@@line_break
            use oxc_span::ContentEq;
        }
    }

    fn derive(&self, type_def: StructOrEnum, schema: &Schema) -> TokenStream {
        match type_def {
            StructOrEnum::Struct(struct_def) => derive_struct(struct_def, schema),
            StructOrEnum::Enum(enum_def) => derive_enum(enum_def, schema),
        }
    }
}

fn derive_struct(struct_def: &StructDef, schema: &Schema) -> TokenStream {
    let mut uses_other = true;

    let body = if struct_def.content_eq.skip {
        // Struct has `#[content_eq(skip)]` attr. So `content_eq` always returns true.
        uses_other = false;
        quote!(true)
    } else {
        let fields = struct_def
            .fields
            .iter()
            // Skip node_id field - it's internal and not part of content equality
            .filter(|field| field.name() != "node_id")
            .filter(|field| !field.content_eq.skip)
            .filter(|field| {
                let innermost_type = field.type_def(schema).innermost_type(schema);
                match innermost_type {
                    TypeDef::Struct(struct_def) => !struct_def.content_eq.skip,
                    TypeDef::Enum(enum_def) => !enum_def.content_eq.skip,
                    _ => true,
                }
            })
            .map(|field| {
                let ident = field.ident();
                quote!( ContentEq::content_eq(&self.#ident, &other.#ident) )
            });

        let mut body = quote!( #(#fields)&&* );
        if body.is_empty() {
            body = quote!(true);
            uses_other = false;
        }
        body
    };

    generate_impl(&struct_def.ty_anon(schema), &body, uses_other)
}

fn derive_enum(enum_def: &EnumDef, schema: &Schema) -> TokenStream {
    let mut uses_other = true;

    let body = if enum_def.content_eq.skip {
        // Enum has `#[content_eq(skip)]` attr. So `content_eq` always returns true.
        uses_other = false;
        quote!(true)
    } else if enum_def.is_fieldless() {
        // We assume fieldless enums implement `PartialEq`
        quote!(self == other)
    } else {
        let matches = enum_def.all_variants(schema).map(|variant| {
            let ident = variant.ident();
            if variant.is_fieldless() {
                quote!( (Self::#ident, Self::#ident) => true )
            } else {
                quote!( (Self::#ident(a), Self::#ident(b)) => a.content_eq(b) )
            }
        });

        quote! {
            match (self, other) {
                #(#matches,)*
                _ => false,
            }
        }
    };

    generate_impl(&enum_def.ty_anon(schema), &body, uses_other)
}

fn generate_impl(ty: &TokenStream, body: &TokenStream, uses_other: bool) -> TokenStream {
    let other_ident = create_safe_ident(if uses_other { "other" } else { "_" });
    quote! {
        impl ContentEq for #ty {
            fn content_eq(&self, #other_ident: &Self) -> bool {
                #body
            }
        }
    }
}
