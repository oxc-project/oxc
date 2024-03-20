use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_quote, token::Eq, Expr, ExprLit, Item, ItemEnum, ItemStruct, Lit};

pub fn ser_raw(item: &mut Item) -> TokenStream {
    match item {
        Item::Struct(item) => modify_struct(item),
        Item::Enum(item) => modify_enum(item),
        _ => panic!("Only use `ser_raw` attribute on structs and enums"),
    }
}

fn modify_struct(item: &mut ItemStruct) -> TokenStream {
    quote! {
        #[derive(::layout_inspect::Inspect)]
        #item
    }
}

fn modify_enum(item: &mut ItemEnum) -> TokenStream {
    // Add explicit discriminant to all variants.
    // Existing discriminants are retained, and numbering continues to increment after them.
    // Last variant discriminant defined as 254, to leave only 1 niche value (255).
    // This means only 1 possible value for the `None` variant of `Option<T>`.
    let len = item.variants.len();
    assert!(len <= 254, "Too many enum variants");

    let mut next_discriminant = 0u8;
    for (index, variant) in item.variants.iter_mut().enumerate() {
        let discriminant = if let Some((.., expr)) = &variant.discriminant {
            // Explicit discriminant
            if let Expr::Lit(ExprLit { lit: Lit::Int(lit), .. }) = expr {
                lit.base10_parse::<u8>().unwrap()
            } else {
                panic!("`ser_raw` attribute only supports integers as explicit discriminators");
            }
        } else {
            // No explicit discriminant - create discriminant following last
            let discriminant = if index < len - 1 {
                next_discriminant
            } else {
                assert!(next_discriminant <= 254);
                254
            };
            variant.discriminant = Some((
                Eq { spans: [variant.ident.span()] },
                Expr::Lit(ExprLit { attrs: vec![], lit: parse_quote!(#discriminant) }),
            ));
            discriminant
        };

        next_discriminant = discriminant + 1;
    }

    quote! {
        #[derive(::layout_inspect::Inspect)]
        #[repr(u8)]
        #item
    }
}
