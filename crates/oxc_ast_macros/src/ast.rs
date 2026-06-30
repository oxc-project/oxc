use proc_macro2::{TokenStream, TokenTree};
use quote::{quote, quote_spanned};
use syn::{
    Attribute, Fields, FieldsNamed, Ident, Item, ItemEnum, ItemStruct, parse_quote,
    punctuated::Punctuated, spanned::Spanned, token::Comma,
};

use crate::generated::{
    derived_traits::get_trait_crate_and_generics, enums::ENUMS, structs::STRUCTS,
};

/// `#[ast]` macro.
pub fn ast(item: &mut Item, args: TokenStream) -> TokenStream {
    match item {
        Item::Enum(item) => modify_enum(item),
        Item::Struct(item) => modify_struct(item, args),
        _ => unreachable!(),
    }
}

/// Details of how `#[ast]` macro should modify an enum.
pub struct EnumDetails {
    /// `true` if all variants are fieldless (unit) - including any inherited variants.
    /// Determines whether the enum is `#[repr(u8)]` (fieldless) or `#[repr(C, u8)]` (has fields).
    pub is_fieldless: bool,
}

/// Add `#[repr(...)]` and `#[derive(::oxc_ast_macros::Ast)]` to enum,
/// and static assertions for `#[generate_derive]`.
fn modify_enum(item: &ItemEnum) -> TokenStream {
    modify_enum_impl(item).unwrap_or_else(|message| {
        let error = compile_error(&item.ident, message);
        quote! {
            #[derive(::oxc_ast_macros::Ast)]
            #item
            #error
        }
    })
}

fn modify_enum_impl(item: &ItemEnum) -> Result<TokenStream, &'static str> {
    // Get enum data.
    // Whether enum has any non-unit variants is calculated by `oxc_ast_tools`,
    // rather than calculating it here on every compilation.
    let enum_name = item.ident.to_string();
    let Some(enum_details) = ENUMS.get(&enum_name) else {
        return Err("Enum is unknown. Run `just ast` to re-run the codegen.");
    };

    // Fieldless enums are `#[repr(u8)]`. Enums with any non-unit variant are `#[repr(C, u8)]`.
    let repr = if enum_details.is_fieldless { quote!(#[repr(u8)]) } else { quote!(#[repr(C, u8)]) };

    let assertions = assert_generated_derives(&item.attrs);

    Ok(quote! {
        #repr
        #[derive(::oxc_ast_macros::Ast)]
        #item
        #assertions
    })
}

/// Details of how `#[ast]` macro should modify a struct.
pub struct StructDetails {
    pub field_order: Option<&'static [u8]>,
    pub is_node: bool,
    pub is_transparent: bool,
}

/// Add `#[repr(C)]` / `#[repr(transparent)]`, and `#[derive(::oxc_ast_macros::Ast)]` to struct,
/// and static assertions for `#[generate_derive]`.
/// If is an AST node (has a `NodeId`), add `#[non_exhaustive]` attr.
/// Re-order struct fields if instructed by `STRUCTS` data.
fn modify_struct(item: &mut ItemStruct, args: TokenStream) -> TokenStream {
    modify_struct_impl(item, args).unwrap_or_else(|message| {
        let error = compile_error(&item.ident, message);
        quote! {
            #[derive(::oxc_ast_macros::Ast)]
            #item
            #error
        }
    })
}

fn modify_struct_impl(
    item: &mut ItemStruct,
    args: TokenStream,
) -> Result<TokenStream, &'static str> {
    // Skip foreign types
    if let Some(TokenTree::Ident(ident)) = args.into_iter().next()
        && ident == "foreign"
    {
        return Ok(quote! {
            #[derive(::oxc_ast_macros::Ast)]
            #item
        });
    }

    // Get struct data
    let struct_name = item.ident.to_string();
    let Some(struct_details) = STRUCTS.get(&struct_name) else {
        return Err("Struct is unknown. Run `just ast` to re-run the codegen.");
    };

    let assertions = assert_generated_derives(&item.attrs);

    reorder_struct_fields(item, struct_details)?;

    // `#[repr(transparent)]` for structs with at most 1 non-zero-sized field, `#[repr(C)]` otherwise
    let repr = if struct_details.is_transparent {
        quote!(#[repr(transparent)])
    } else {
        quote!(#[repr(C)])
    };

    // `#[non_exhaustive]` on AST node types
    let non_exhaustive =
        if struct_details.is_node { Some(quote!(#[non_exhaustive])) } else { None };

    Ok(quote! {
        #repr
        #non_exhaustive
        #[derive(::oxc_ast_macros::Ast)]
        #item
        #assertions
    })
}

/// Re-order struct fields, depending on instructions in `STRUCTS` (which is codegen-ed).
///
/// Mutates `item` in place, re-ordering its fields.
fn reorder_struct_fields(
    item: &mut ItemStruct,
    struct_details: &StructDetails,
) -> Result<(), &'static str> {
    // Exit if fields don't need re-ordering
    let Some(field_order) = struct_details.field_order else {
        return Ok(());
    };

    // Re-order fields.
    // `field_order` contains indexes of fields in the order they should be.
    let named = match &mut item.fields {
        Fields::Named(FieldsNamed { named, .. }) if named.len() == field_order.len() => named,
        _ => {
            return Err("Struct has been altered. Run `just ast` to re-run the codegen.");
        }
    };

    // Create 2 sets of fields.
    // 1st set are the fields in original order, each prefixed with `#[cfg(doc)]`.
    // 2nd set are the fields in new order, each prefixed with `#[cfg(not(doc))]`.
    // This is necessary so that fields are listed in original source order in docs.
    let mut fields = named.clone().into_pairs().zip(field_order).collect::<Vec<_>>();
    fields.sort_unstable_by_key(|(_, index)| **index);

    for field in named.iter_mut() {
        field.attrs.insert(0, parse_quote!( #[cfg(doc)]));
    }

    named.extend(fields.into_iter().map(|(mut pair, _)| {
        pair.value_mut().attrs.insert(0, parse_quote!( #[cfg(not(doc))]));
        pair
    }));

    Ok(())
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
///
/// If any errors e.g. cannot parse `#[generate_derive]`, or unknown traits, just skip them.
/// It is the responsibility of `oxc_ast_tools` to raise errors for those.
fn assert_generated_derives(attrs: &[Attribute]) -> TokenStream {
    let mut assertions = quote!();
    for attr in attrs {
        if !attr.path().is_ident("generate_derive") {
            continue;
        }

        let Ok(parsed) = attr.parse_args_with(Punctuated::<Ident, Comma>::parse_terminated) else {
            continue;
        };

        for trait_ident in parsed {
            let trait_name = trait_ident.to_string();
            let Some((trait_path, generics)) = get_trait_crate_and_generics(&trait_name) else {
                continue;
            };

            // These are wrapped in a scope to avoid the need for unique identifiers
            assertions.extend(quote! {{
                trait AssertionTrait: #trait_path #generics {}
                impl<T: #trait_ident #generics> AssertionTrait for T {}
            }});
        }
    }

    quote! {
        const _: () = { #assertions };
    }
}

/// Generate a `compile_error!` macro invocation with the given message, and the span of `spanned`.
fn compile_error<S: Spanned>(spanned: &S, message: &str) -> TokenStream {
    quote_spanned! { spanned.span() => compile_error!(#message); }
}
