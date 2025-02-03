//! Derive for `GetSpan` trait.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

use crate::{
    schema::{Def, EnumDef, Schema, StructDef},
    Result,
};

use super::{define_derive, AttrLocation, AttrPart, AttrPositions, Derive, StructOrEnum};

/// Derive for `GetSpan` trait.
pub struct DeriveGetSpan;

define_derive!(DeriveGetSpan);

impl Derive for DeriveGetSpan {
    fn trait_name(&self) -> &'static str {
        "GetSpan"
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
            #![allow(clippy::match_same_arms)]

            ///@@line_break
            use oxc_span::{Span, GetSpan};
        }
    }

    fn derive(&self, type_def: StructOrEnum, schema: &Schema) -> TokenStream {
        let self_ty = quote!(&self);
        let result_ty = quote!(Span);
        let result_expr = quote!(self.span);
        let reference = quote!( & );
        let unboxed_ref = quote!(it.as_ref());

        derive_type(
            type_def,
            "GetSpan",
            "span",
            &self_ty,
            &result_ty,
            &result_expr,
            &reference,
            &unboxed_ref,
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

    fn prelude(&self) -> TokenStream {
        quote! {
            #![allow(clippy::match_same_arms)]

            ///@@line_break
            use oxc_span::{Span, GetSpanMut};
        }
    }

    fn derive(&self, type_def: StructOrEnum, schema: &Schema) -> TokenStream {
        let self_ty = quote!(&mut self);
        let result_ty = quote!(&mut Span);
        let result_expr = quote!(&mut self.span);
        let reference = quote!( &mut );
        let unboxed_ref = quote!(&mut **it);

        derive_type(
            type_def,
            "GetSpanMut",
            "span_mut",
            &self_ty,
            &result_ty,
            &result_expr,
            &reference,
            &unboxed_ref,
            schema,
        )
    }
}

/// Generate `GetSpan` / `GetSpanMut` trait implementation for a type.
#[expect(clippy::too_many_arguments)]
fn derive_type(
    type_def: StructOrEnum,
    trait_name: &str,
    method_name: &str,
    self_ty: &TokenStream,
    result_ty: &TokenStream,
    result_expr: &TokenStream,
    reference: &TokenStream,
    unboxed_ref: &TokenStream,
    schema: &Schema,
) -> TokenStream {
    let trait_ident = format_ident!("{trait_name}");
    let method_ident = format_ident!("{method_name}");
    match type_def {
        StructOrEnum::Struct(struct_def) => derive_struct(
            struct_def,
            &trait_ident,
            &method_ident,
            self_ty,
            result_ty,
            result_expr,
            reference,
            schema,
        ),
        StructOrEnum::Enum(enum_def) => derive_enum(
            enum_def,
            &trait_ident,
            &method_ident,
            self_ty,
            result_ty,
            unboxed_ref,
            schema,
        ),
    }
}

/// Generate `GetSpan` / `GetSpanMut` trait implementation for a struct.
#[expect(clippy::too_many_arguments)]
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
    unboxed_ref: &TokenStream,
    schema: &Schema,
) -> TokenStream {
    let ty = enum_def.ty_anon(schema);

    let matches = enum_def.all_variants(schema).map(|variant| {
        let variant_ident = variant.ident();
        let variant_type = variant.field_type(schema).unwrap();
        // TODO: Just generate `it.span()` or `it.span_mut()`.
        // Then output is the same whether variant is boxed or not, and `unboxed_ref` is not needed.
        if variant_type.is_box() {
            quote!( Self::#variant_ident(it) => #trait_ident::#method_ident(#unboxed_ref) )
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
