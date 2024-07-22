mod reorder_fields;
mod generated {
    pub mod ast_field_order_data;
}

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use reorder_fields::reorder_fields;

/// returns `#[repr(C, u8)]` if `enum_` has any non-unit variant,
/// Otherwise it would return `#[repr(u8)]`.
fn enum_repr(enum_: &syn::ItemEnum) -> TokenStream2 {
    if enum_.variants.iter().any(|var| !matches!(var.fields, syn::Fields::Unit)) {
        quote!(#[repr(C, u8)])
    } else {
        quote!(#[repr(u8)])
    }
}

/// This attribute serves two purposes,
/// First, it is a marker for our codegen to detect AST types. Furthermore.
/// It is also a lightweight macro; All of its computation is cached and
/// it only applies the following changes without any complex operation:
///
/// * Prepend `#[repr(C)]` to structs
/// * Prepend `#[repr(C, u8)]` to fieldful enums e.g. `enum E { X: u32, Y: u8 }`
/// * Prepend `#[repr(u8)]` to unit (fieldless) enums e.g. `enum E { X, Y, Z, }`
/// * Prepend `#[derive(oxc_ast_macros::Ast)]` to all structs and enums
///
#[proc_macro_attribute]
#[allow(clippy::missing_panics_doc)]
pub fn ast(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::Item);
    let (repr, item) = match input {
        syn::Item::Enum(enum_) => (enum_repr(&enum_), syn::Item::Enum(enum_)),
        syn::Item::Struct(mut struct_) => {
            let id = struct_.ident.to_string();
            // if we have field ordering data for this type use it to reorder.
            if let Some(data) = generated::ast_field_order_data::get(id.as_str()) {
                reorder_fields(&mut struct_, data);
            };
            (quote!(#[repr(C)]), syn::Item::Struct(struct_))
        }

        _ => {
            unreachable!()
        }
    };

    let expanded = quote! {
        #[derive(::oxc_ast_macros::Ast)]
        #repr
        #item
    };
    TokenStream::from(expanded)
}

/// Dummy derive macro for a non-existent trait `Ast`.
///
/// Does not generate any code.
/// Only purpose is to allow using `#[scope]`, `#[visit]`, and other attrs in the AST node type defs.
/// These "marker" attributes are used in codegen.
#[proc_macro_derive(Ast, attributes(scope, visit, span, serde, tsify))]
pub fn ast_derive(_item: TokenStream) -> TokenStream {
    TokenStream::new()
}
