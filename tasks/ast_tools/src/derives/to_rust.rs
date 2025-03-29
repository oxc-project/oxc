//! Derive for `ToRust` trait.

use proc_macro2::TokenStream;
use quote::quote;

use crate::{
    derives::{AttrPositions, Derive, StructOrEnum, attr_positions, define_derive},
    schema::{Def, EnumDef, Schema, StructDef, TypeDef},
    utils::create_ident,
};

/// Derive for `ToRust` trait.
pub struct DeriveToRust;

define_derive!(DeriveToRust);

impl Derive for DeriveToRust {
    fn trait_name(&self) -> &'static str {
        "ToRust"
    }

    fn trait_has_lifetime(&self) -> bool {
        false
    }

    fn crate_name(&self) -> &'static str {
        "oxc_quote_types"
    }

    fn prelude(&self) -> TokenStream {
        quote! {
            #![allow(clippy::redundant_closure_for_method_calls)]
        }
    }

    fn attrs(&self) -> &[(&'static str, AttrPositions)] {
        &[("to_rust", attr_positions!(Struct | Enum))]
    }

    fn derive(&self, type_def: StructOrEnum, schema: &Schema) -> TokenStream {
        match type_def {
            StructOrEnum::Struct(struct_def) => derive_struct(struct_def, schema),
            StructOrEnum::Enum(enum_def) => derive_enum(enum_def, schema),
        }
    }
}

#[expect(clippy::needless_pass_by_value)]
fn encode_type_stream(field_type: &TypeDef, schema: &Schema, target: TokenStream) -> TokenStream {
    if let Some(vec_type) = field_type.as_vec() {
        let inner_type = vec_type.inner_type(schema);
        let encoded_inner_type = encode_type_stream(inner_type, schema, quote!(v));
        quote!(::oxc_quote_types::Node::Vec(#target.iter().map(|v| #encoded_inner_type).collect()))
    } else if let Some(opt_type) = field_type.as_option() {
        let inner_type = opt_type.inner_type(schema);
        let encoded_inner_type = encode_type_stream(inner_type, schema, quote!(v));
        quote!(::oxc_quote_types::Node::Option(#target.as_ref().map(|v| ::std::boxed::Box::new(#encoded_inner_type))))
    } else if field_type.is_box() {
        quote!(::oxc_quote_types::Node::Box(::std::boxed::Box::new(#target.to_rust())))
    } else if let Some(cell_type) = field_type.as_cell() {
        if cell_type.inner_type(schema).is_option() {
            quote!(::oxc_quote_types::Node::CellOption)
        } else {
            let inner_type = cell_type.inner_type(schema);
            let encoded_inner_type =
                encode_type_stream(inner_type, schema, quote!(#target.clone().into_inner()));
            quote!(::oxc_quote_types::Node::Cell(#encoded_inner_type))
        }
    } else {
        quote!(#target.to_rust())
    }
}

fn derive_struct(struct_def: &StructDef, schema: &Schema) -> TokenStream {
    let ty = struct_def.ty_anon(schema);

    let name = &struct_def.name;

    // We have to do this since we must encode the *exact* type specifications
    // of each field (including Deref types, which would otherwise lose their
    // type information by simply calling `.to_rust()` on them).
    let fields = struct_def.fields.iter().map(|field| {
        let field_type = field.type_def(schema);
        let name_str = &field.name;
        let name = create_ident(name_str);
        let value = encode_type_stream(field_type, schema, quote!(self.#name));
        quote! { (#name_str, #value) }
    });

    quote! {
        impl ::oxc_quote_types::ToRust for #ty {
            fn to_rust(&self) -> ::oxc_quote_types::Node {
                ::oxc_quote_types::Node::Struct(::std::boxed::Box::new(::oxc_quote_types::Struct {
                    name: #name,
                    fields: ::std::vec![#(#fields),*]
                }))
            }
        }
    }
}

fn derive_enum(enum_def: &EnumDef, schema: &Schema) -> TokenStream {
    let ty = enum_def.ty_anon(schema);
    let name = &enum_def.name;

    let variants = enum_def.all_variants(schema).map(|variant| {
        let variant_name_str = &variant.name;
        let variant_name = create_ident(variant_name_str);

        if let Some(field) = variant.field_type(schema) {
            let encoded_field = encode_type_stream(field, schema, quote!(item));

            quote! {
                Self::#variant_name (item) => ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: #name,
                    variant: #variant_name_str,
                    field: ::std::option::Option::Some(#encoded_field)
                }))
            }
        } else {
            quote! {
                Self::#variant_name => ::oxc_quote_types::Node::Enum(::std::boxed::Box::new(::oxc_quote_types::Enum {
                    name: #name,
                    variant: #variant_name_str,
                    field: ::std::option::Option::None
                }))
            }
        }
    });

    quote! {
        impl ::oxc_quote_types::ToRust for #ty {
            fn to_rust(&self) -> ::oxc_quote_types::Node {
                match self {
                    #(#variants),*
                }
            }
        }
    }
}
