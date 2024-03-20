use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::Item;

pub fn ast_node(item: &Item) -> TokenStream {
    match item {
        Item::Struct(item) => modify(item),
        Item::Enum(item) => modify(item),
        _ => panic!("Only use `ast_node` attribute on structs and enums"),
    }
}

fn modify<T: ToTokens>(item: &T) -> TokenStream {
    quote! {
        #[cfg_attr(feature = "raw", ::oxc_macros::ser_raw)]
        #item
    }
}
