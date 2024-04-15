use std::slice;

use proc_macro2::TokenStream as TokenStream2;

use quote::quote;
use syn::{
    punctuated::Punctuated, Attribute, Generics, Ident, Item, ItemEnum, ItemStruct, Meta, Token,
};

pub fn ast_node(mut item: Item) -> TokenStream2 {
    let result = match &mut item {
        Item::Struct(it) => modify_struct(it),
        Item::Enum(it) => modify_enum(it),
        _ => panic!("ast_node attribute can only be used on enums and structure types!"),
    };

    let traversable_test_trait = impl_traversable_test_trait(&result);

    // let ident = item;
    quote! {
        #item

        #traversable_test_trait
    }
}

fn modify_struct(item: &mut ItemStruct) -> NodeData {
    validate_struct_attributes(item.attrs.iter());
    NodeData { ident: &item.ident, generics: &item.generics }
}

fn modify_enum(item: &mut ItemEnum) -> NodeData {
    validate_enum_attributes(item.attrs.iter());
    NodeData { ident: &item.ident, generics: &item.generics }
}

fn validate_struct_attributes<'a>(mut attrs: slice::Iter<'a, Attribute>) {
    // make sure that no structure derives Clone/Copy trait.
    // TODO: It might fail if there is a manual Clone/Copy trait implemented for the struct.
    assert!(!attrs.any(|attr| {
        let args = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated);
        attr.path().is_ident("derive")
            && args.is_ok_and(|args| {
                args.iter().any(|arg| arg.path().is_ident("Clone") || arg.path().is_ident("Copy"))
            })
    }));
}

fn validate_enum_attributes<'a>(_: slice::Iter<'a, Attribute>) {
    // TODO: Later on we may want to enforce deriving clone and copy traits for all enum types
}

fn impl_traversable_test_trait(node: &NodeData) -> TokenStream2 {
    let ident = node.ident;
    let generics = node.generics;
    quote! {
        impl #generics crate::traverse::TraversableTest for #ident #generics { }
    }
}

struct NodeData<'a> {
    ident: &'a Ident,
    generics: &'a Generics,
}
