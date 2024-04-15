use proc_macro2::TokenStream as TokenStream2;

use quote::quote;
use syn::{
    parse_quote, punctuated::Punctuated, Attribute, Generics, Ident, Item, ItemEnum, ItemStruct,
    Meta, Token, Variant,
};

pub fn ast_node(mut item: Item) -> TokenStream2 {
    let result = match &mut item {
        Item::Struct(it) => modify_struct(it),
        Item::Enum(it) => modify_enum(it),
        _ => panic!("ast_node attribute can only be used on enums and structure types!"),
    };

    let traversable_test_trait = impl_traversable_test_trait(&result);

    quote! {
        #item

        #traversable_test_trait
    }
}

fn modify_struct(item: &mut ItemStruct) -> NodeData {
    item.attrs.iter().for_each(validate_struct_attribute);
    // add the correct representation
    item.attrs.push(parse_quote!(#[repr(C)]));
    NodeData { ident: &item.ident, generics: &item.generics }
}

fn modify_enum(item: &mut ItemEnum) -> NodeData {
    item.attrs.iter().for_each(validate_enum_attribute);

    assert!(
        item.variants.len() < 256,
        "`ast_node` enums are limited to the maximum of 256 variants."
    );
    item.variants.iter().for_each(validate_enum_variant);

    // add the correct representation
    item.attrs.push(parse_quote!(#[repr(C, u8)]));

    // add the dummy variant
    item.variants.insert(0, parse_quote!(Dummy));
    // add explicit discriminants to all variants
    item.variants
        .iter_mut()
        .enumerate()
        .for_each(|(i, var)| var.discriminant = Some((parse_quote!(=), parse_quote!(#i as u8))));

    NodeData { ident: &item.ident, generics: &item.generics }
}

// validators

fn validate_struct_attribute(attr: &Attribute) {
    // make sure that no structure derives Clone/Copy traits.
    // TODO: It will fail if there is a manual Clone/Copy traits implemented for the struct.
    // Negative traits (!Copy and !Clone) are nightly so I'm not sure how we can fully enforce it.
    assert!(
        !{
            let args = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated);
            attr.path().is_ident("derive")
                && args.is_ok_and(|args| {
                    args.iter()
                        .any(|arg| arg.path().is_ident("Clone") || arg.path().is_ident("Copy"))
                })
        },
        "`ast_node` can't have Clone or Copy traits"
    );

    validate_attribute(attr);
}

fn validate_enum_attribute(attr: &Attribute) {
    // TODO: Later on we may want to enforce deriving clone and copy traits for all enum types
    validate_attribute(attr);
}

fn validate_attribute(attr: &Attribute) {
    assert!(
        !attr.path().is_ident("repr"),
        "using `repr` attribute is not allowed with `ast_node`."
    );
}

fn validate_enum_variant(var: &Variant) {
    assert_ne!(
        var.ident, "Dummy",
        r#"Found a variant called `Dummy`,\
           Please use another name,\
           This variant identifier is reserved by `ast_node` attribute."#
    );
    assert!(
        var.discriminant.is_none(),
        "Using explicit enum discriminants is not allowed with `ast_node` attribute."
    )
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
