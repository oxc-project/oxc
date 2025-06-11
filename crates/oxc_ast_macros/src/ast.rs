use std::mem;

use proc_macro2::{TokenStream, TokenTree};
use quote::quote;
use syn::{
    Attribute, Fields, FieldsNamed, Ident, Item, ItemEnum, ItemStruct, parse_quote,
    punctuated::Punctuated, token::Comma,
};

use crate::generated::{derived_traits::get_trait_crate_and_generics, structs::STRUCTS};

/// `#[ast]` macro.
pub fn ast(item: &mut Item, args: TokenStream) -> TokenStream {
    match item {
        Item::Enum(item) => modify_enum(item),
        Item::Struct(item) => modify_struct(item, args),
        _ => unreachable!(),
    }
}

/// Add `#[repr(...)]` and `#[derive(::oxc_ast_macros::Ast)]` to enum,
/// and static assertions for `#[generate_derive]`.
fn modify_enum(item: &ItemEnum) -> TokenStream {
    // If enum has any non-unit variant, `#[repr(C, u8)]`, otherwise `#[repr(u8)]`
    let repr = if item.variants.iter().any(|var| !matches!(var.fields, Fields::Unit)) {
        quote!(#[repr(C, u8)])
    } else {
        quote!(#[repr(u8)])
    };

    let assertions = assert_generated_derives(&item.attrs);

    quote! {
        #repr
        #[derive(::oxc_ast_macros::Ast)]
        #item
        #assertions
    }
}

/// Details of how `#[ast]` macro should modify a struct.
pub struct StructDetails {
    pub field_order: Option<&'static [u8]>,
}

/// Add `#[repr(C)]` and `#[derive(::oxc_ast_macros::Ast)]` to struct,
/// and static assertions for `#[generate_derive]`.
/// Re-order struct fields if instructed by `STRUCTS` data.
fn modify_struct(item: &mut ItemStruct, args: TokenStream) -> TokenStream {
    let assertions = assert_generated_derives(&item.attrs);

    let item = reorder_struct_fields(item, args).unwrap_or_else(|| quote!(#item));

    quote! {
        #[repr(C)]
        #[derive(::oxc_ast_macros::Ast)]
        #item
        #assertions
    }
}

/// Re-order struct fields, depending on instructions in `STRUCTS` (which is codegen-ed).
fn reorder_struct_fields(item: &mut ItemStruct, args: TokenStream) -> Option<TokenStream> {
    // Skip foreign types
    if let Some(TokenTree::Ident(ident)) = args.into_iter().next() {
        if ident == "foreign" {
            return None;
        }
    }

    // Get struct data. Exit if no fields need re-ordering.
    let struct_name = item.ident.to_string();
    let field_order = STRUCTS[&struct_name].field_order?;

    // Re-order fields.
    // `field_order` contains indexes of fields in the order they should be.
    let fields = mem::replace(&mut item.fields, Fields::Unit);
    let Fields::Named(FieldsNamed { brace_token, mut named }) = fields else { unreachable!() };

    assert!(
        named.len() == field_order.len(),
        "Wrong number of fields for `{struct_name}` in `STRUCTS`"
    );

    // Create 2 sets of fields.
    // 1st set are the fields in original order, each prefixed with `#[cfg(doc)]`.
    // 2nd set are the fields in new order, each prefixed with `#[cfg(not(doc))]`.
    // This is necessary so that fields are listed in original source order in docs.
    let mut fields = named.clone().into_pairs().zip(field_order).collect::<Vec<_>>();
    fields.sort_unstable_by_key(|(_, index)| **index);

    for field in &mut named {
        field.attrs.insert(0, parse_quote!( #[cfg(doc)]));
    }

    named.extend(fields.into_iter().map(|(mut pair, _)| {
        pair.value_mut().attrs.insert(0, parse_quote!( #[cfg(not(doc))]));
        pair
    }));

    item.fields = Fields::Named(FieldsNamed { brace_token, named });

    Some(quote!( #item ))
}

/// Generate assertions that traits used in `#[generate_derive]` are in scope.
///
/// e.g. for `#[generate_derive(GetSpan)]`, it generates:
///
/// ```rs
/// const _: () = {
///     {
///         trait AssertionTrait: ::oxc_span::GetSpan {}
///         impl<T: GetSpan> AssertionTrait for T {}
///     }
/// };
/// ```
///
/// If `GetSpan` is not in scope, or it is not the correct `oxc_span::GetSpan`,
/// this will raise a compilation error.
fn assert_generated_derives(attrs: &[Attribute]) -> TokenStream {
    // We don't care here if a trait is derived multiple times.
    // It is the responsibility of `oxc_ast_tools` to raise errors for those.
    let assertions = attrs
        .iter()
        .filter(|attr| attr.path().is_ident("generate_derive"))
        .flat_map(parse_attr)
        .map(|trait_ident| {
            let trait_name = trait_ident.to_string();
            let Some((trait_path, generics)) = get_trait_crate_and_generics(&trait_name) else {
                panic!("Invalid derive trait(generate_derive): {trait_name}");
            };

            // These are wrapped in a scope to avoid the need for unique identifiers
            quote! {{
                trait AssertionTrait: #trait_path #generics {}
                impl<T: #trait_ident #generics> AssertionTrait for T {}
            }}
        });
    quote!( const _: () = { #(#assertions)* }; )
}

#[inline]
fn parse_attr(attr: &Attribute) -> impl Iterator<Item = Ident> + use<> {
    attr.parse_args_with(Punctuated::<Ident, Comma>::parse_terminated)
        .expect("`#[generate_derive]` only accepts traits as single segment paths. Found an invalid argument.")
        .into_iter()
}
