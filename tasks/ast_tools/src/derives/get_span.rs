//! Derive for `GetSpan` trait.

use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use crate::{
    Result,
    schema::{Def, EnumDef, Schema, StructDef},
    utils::create_safe_ident,
};

use super::{AttrLocation, AttrPart, AttrPositions, Derive, StructOrEnum, define_derive};

/// Derive for `GetSpan` trait.
pub struct DeriveGetSpan;

define_derive!(DeriveGetSpan);

impl Derive for DeriveGetSpan {
    fn trait_name(&self) -> &'static str {
        "GetSpan"
    }

    fn crate_name(&self) -> &'static str {
        "oxc_span"
    }

    /// Register that accept `#[span]` attr on struct fields.
    fn attrs(&self) -> &[(&'static str, AttrPositions)] {
        &[("span", AttrPositions::StructField)]
    }

    /// Parse `#[span]` on struct field.
    fn parse_attr(&self, _attr_name: &str, location: AttrLocation, part: AttrPart) -> Result<()> {
        // No need to check attr name is `span`, because that's the only attribute this derive handles.
        // Ditto location can only be `StructField`.
        let AttrLocation::StructField(struct_def, field_index) = location else { unreachable!() };

        if matches!(part, AttrPart::None) {
            struct_def.span.span_field_index = Some(field_index);
            Ok(())
        } else {
            Err(())
        }
    }

    fn prelude(&self) -> TokenStream {
        quote! {
            #![expect(clippy::match_same_arms)]

            ///@@line_break
            use oxc_span::{Span, GetSpan};
        }
    }

    fn derive(&self, type_def: StructOrEnum, schema: &Schema) -> TokenStream {
        let trait_ident = create_safe_ident("GetSpan");
        let method_ident = create_safe_ident("span");
        let self_ty = quote!(&self);
        let result_ty = quote!(Span);
        let result_expr = quote!(self.span);
        let reference = quote!( & );

        derive_type(
            type_def,
            &trait_ident,
            &method_ident,
            &self_ty,
            &result_ty,
            &result_expr,
            &reference,
            schema,
        )
    }
}

pub struct DeriveGetSpanMut;

define_derive!(DeriveGetSpanMut);

impl Derive for DeriveGetSpanMut {
    fn trait_name(&self) -> &'static str {
        "GetSpanMut"
    }

    /// Get crate trait is defined in.
    fn crate_name(&self) -> &'static str {
        "oxc_span"
    }

    fn prelude(&self) -> TokenStream {
        quote! {
            #![expect(clippy::match_same_arms)]

            ///@@line_break
            use oxc_span::{Span, GetSpanMut};
        }
    }

    fn derive(&self, type_def: StructOrEnum, schema: &Schema) -> TokenStream {
        let trait_ident = create_safe_ident("GetSpanMut");
        let method_ident = create_safe_ident("span_mut");
        let self_ty = quote!(&mut self);
        let result_ty = quote!(&mut Span);
        let result_expr = quote!(&mut self.span);
        let reference = quote!( &mut );

        derive_type(
            type_def,
            &trait_ident,
            &method_ident,
            &self_ty,
            &result_ty,
            &result_expr,
            &reference,
            schema,
        )
    }
}

/// Generate `GetSpan` / `GetSpanMut` trait implementation for a type.
fn derive_type(
    type_def: StructOrEnum,
    trait_ident: &Ident,
    method_ident: &Ident,
    self_ty: &TokenStream,
    result_ty: &TokenStream,
    result_expr: &TokenStream,
    reference: &TokenStream,
    schema: &Schema,
) -> TokenStream {
    match type_def {
        StructOrEnum::Struct(struct_def) => derive_struct(
            struct_def,
            trait_ident,
            method_ident,
            self_ty,
            result_ty,
            result_expr,
            reference,
            schema,
        ),
        StructOrEnum::Enum(enum_def) => {
            derive_enum(enum_def, trait_ident, method_ident, self_ty, result_ty, reference, schema)
        }
    }
}

/// Generate `GetSpan` / `GetSpanMut` trait implementation for a struct.
fn derive_struct(
    struct_def: &StructDef,
    trait_ident: &Ident,
    method_ident: &Ident,
    self_ty: &TokenStream,
    result_ty: &TokenStream,
    result_expr: &TokenStream,
    reference: &TokenStream,
    schema: &Schema,
) -> TokenStream {
    let ty = struct_def.ty_anon(schema);

    let result_expr = if let Some(field_index) = struct_def.span.span_field_index {
        let field_ident = struct_def.fields[field_index].ident();
        &quote!( #trait_ident::#method_ident(#reference self.#field_ident) )
    } else {
        result_expr
    };

    quote! {
        impl #trait_ident for #ty {
            #[inline]
            fn #method_ident(#self_ty) -> #result_ty {
                #result_expr
            }
        }
    }
}

/// Generate `GetSpan` / `GetSpanMut` trait implementation for an enum.
fn derive_enum(
    enum_def: &EnumDef,
    trait_ident: &Ident,
    method_ident: &Ident,
    self_ty: &TokenStream,
    result_ty: &TokenStream,
    reference: &TokenStream,
    schema: &Schema,
) -> TokenStream {
    let ty = enum_def.ty_anon(schema);

    let matches = enum_def.all_variants(schema).map(|variant| {
        let variant_ident = variant.ident();
        let variant_type = variant.field_type(schema).unwrap();
        if variant_type.is_box() {
            quote!( Self::#variant_ident(it) => #trait_ident::#method_ident(#reference **it) )
        } else {
            quote!( Self::#variant_ident(it) => #trait_ident::#method_ident(it) )
        }
    });

    quote! {
        impl #trait_ident for #ty {
            fn #method_ident(#self_ty) -> #result_ty {
                match self {
                    #(#matches),*
                }
            }
        }
    }
}
