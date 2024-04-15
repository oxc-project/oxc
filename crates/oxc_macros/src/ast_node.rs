use proc_macro2::TokenStream as TokenStream2;

use quote::quote;
use syn::{
    parse_quote, punctuated::Punctuated, Attribute, Generics, Ident, Item, ItemEnum, ItemStruct,
    Meta, Token,
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
    validate_struct_attributes(&item.attrs);
    item.attrs.push(parse_quote!(#[repr(C)]));
    NodeData { ident: &item.ident, generics: &item.generics }
}

fn modify_enum(item: &mut ItemEnum) -> NodeData {
    validate_enum_attributes(&item.attrs);
    item.attrs.push(parse_quote!(#[repr(C, u8)]));
    NodeData { ident: &item.ident, generics: &item.generics }
}

fn validate_struct_attributes<I>(attrs: &I)
where
    for<'a> &'a I: IntoIterator<Item = &'a Attribute>,
{
    // make sure that no structure derives Clone/Copy traits.
    // TODO: It will fail if there is a manual Clone/Copy traits implemented for the struct.
    // Negative traits (!Copy and !Clone) are nightly so I'm not sure how we can fully enforce it.
    assert!(
        !attrs.into_iter().any(|attr| {
            let args = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated);
            attr.path().is_ident("derive")
                && args.is_ok_and(|args| {
                    args.iter()
                        .any(|arg| arg.path().is_ident("Clone") || arg.path().is_ident("Copy"))
                })
        }),
        "`ast_node` can't have Clone or Copy traits"
    );

    validate_attributes(attrs);
}

fn validate_enum_attributes<I>(attrs: &I)
where
    for<'a> &'a I: IntoIterator<Item = &'a Attribute>,
{
    // TODO: Later on we may want to enforce deriving clone and copy traits for all enum types
    validate_attributes(attrs);
}

fn validate_attributes<I>(attrs: &I)
where
    for<'a> &'a I: IntoIterator<Item = &'a Attribute>,
{
    assert!(
        !attrs.into_iter().any(|attr| attr.path().is_ident("repr")),
        "using `repr` attribute is not allowed with `ast_node`."
    );
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
