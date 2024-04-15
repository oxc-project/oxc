use proc_macro2::TokenStream as TokenStream2;

use quote::{format_ident, quote};
use syn::{
    parse_quote, punctuated::Punctuated, AngleBracketedGenericArguments, Attribute, Field, Fields,
    GenericArgument, Generics, Ident, Item, ItemEnum, ItemStruct, Meta, PathArguments, Token, Type,
    TypePath, Variant,
};

pub fn ast_node(mut item: Item) -> TokenStream2 {
    let result = match &mut item {
        Item::Struct(it) => modify_struct(it),
        Item::Enum(it) => modify_enum(it),
        _ => panic!("ast_node attribute can only be used on enums and structure types!"),
    };

    let traversable_test_trait = impl_traversable_test_trait(&result);

    let ident = result.ident;

    let traversable_mod = format_ident!("traversable_{}", ident.to_string().to_lowercase());

    let traversable = result.traversable;

    quote! {
        #item

        #traversable_test_trait

        mod #traversable_mod {
            use super::*;

            #traversable
        }
    }
}

fn modify_struct(item: &mut ItemStruct) -> NodeData {
    item.attrs.iter().for_each(validate_struct_attribute);
    item.fields.iter().for_each(validate_field);
    // add the correct representation
    item.attrs.push(parse_quote!(#[repr(C)]));
    NodeData {
        ident: &item.ident,
        generics: &item.generics,
        traversable: generate_traversable_struct(item),
    }
}

fn modify_enum(item: &mut ItemEnum) -> NodeData {
    item.attrs.iter().for_each(validate_enum_attribute);

    assert!(
        item.variants.len() < 256,
        "`ast_node` enums are limited to the maximum of 256 variants."
    );
    item.variants.iter().for_each(validate_variant);

    // add the correct representation
    item.attrs.push(parse_quote!(#[repr(C, u8)]));

    // add the dummy variant
    item.variants.insert(0, parse_quote!(Dummy));
    // add explicit discriminants to all variants
    item.variants
        .iter_mut()
        .enumerate()
        .for_each(|(i, var)| var.discriminant = Some((parse_quote!(=), parse_quote!(#i as u8))));

    NodeData {
        ident: &item.ident,
        generics: &item.generics,
        traversable: generate_traversable_enum(item),
    }
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
                    args.iter() // To be honest the Copy trait check here is redundant.
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

fn validate_field(field: &Field) {
    assert!(
        matches!(&field.ty, Type::Path(ty) if ty.path.segments.len() == 1),
        "Currently `ast_node` attribute only supports single segment type paths."
    );
}

fn validate_variant(var: &Variant) {
    assert_ne!(
        var.ident, "Dummy",
        r#"Found a variant called `Dummy`,\
           Please use another name,\
           This variant identifier is reserved by `ast_node` attribute."#
    );

    assert!(
        var.discriminant.is_none(),
        "Using explicit enum discriminants is not allowed with `ast_node` attribute."
    );

    assert!(
        matches!(var.fields, Fields::Unnamed(_) | Fields::Unit),
        "Currently, `ast_node` attribute only supports unamed and unit enum variants."
    );
}

// generators

fn generate_traversable_struct(item: &ItemStruct) -> TokenStream2 {
    let ident = format_ident!("Traversable{}", item.ident);
    let generics = &item.generics;

    // TODO: traits like serialization, Debug and Hash fail with `GCell`;
    // But we may want to keep other attributes.
    // let (outer_attrs, inner_attrs) =
    //     item.attrs.iter().fold((Vec::new(), Vec::new()), |mut acc, attr| {
    //         match &attr.style {
    //             AttrStyle::Outer => acc.0.push(attr),
    //             AttrStyle::Inner(_) => acc.1.push(attr),
    //         }
    //
    //         acc
    //     });

    let fields = transform_fields(&item.fields);

    let output = quote! {
        #[repr(C)] // TODO: we can replace it with outer_attrs if we fix the issues with it.
        // #(#outer_attrs)*
        pub struct #ident #generics {
            // #(#inner_attrs)*
            #fields
        }

    };

    output
}

fn generate_traversable_enum(item: &ItemEnum) -> TokenStream2 {
    let ident = format_ident!("Traversable{}", item.ident);
    let generics = &item.generics;

    // TODO: traits like serialization, Debug and Hash fail with `GCell`;
    // But we may want to keep other attributes.

    let variants = transform_variants(&item.variants);

    let output = quote! {
        #[repr(C, u8)]
        pub enum #ident #generics {
            #variants
        }

    };

    output
}

// transformers

fn transform_fields(fields: &Fields) -> Punctuated<Field, Token![,]> {
    let Fields::Named(fields) = fields else {
        panic!("`ast_node` attribute only works with named structure fields");
    };
    fields.named.iter().map(ToOwned::to_owned).map(transform_field).collect()
}

fn transform_field(mut field: Field) -> Field {
    field.ty = transform_type(field.ty);
    field.attrs.clear();

    field
}

fn transform_variants(variants: &Punctuated<Variant, Token![,]>) -> Punctuated<Variant, Token![,]> {
    variants.into_iter().map(ToOwned::to_owned).map(transform_variant).collect()
}

fn transform_variant(mut variant: Variant) -> Variant {
    let Fields::Unnamed(mut fields) = variant.fields else {
        return variant;
    };

    fields.unnamed = fields
        .unnamed
        .into_iter()
        .map(|mut field| {
            field.ty = transform_type(field.ty);
            field
        })
        .collect();

    variant.fields = Fields::Unnamed(fields);
    variant
}

fn transform_type(ty: Type) -> Type {
    let Type::Path(ty) = ty else {
        unreachable!("Should have been asserted at this point.");
    };

    let ty = transform_generic_type(ty);

    Type::Path(ty)
}

fn transform_generic_type(ty: TypePath) -> TypePath {
    let first_seg = ty.path.segments.first().expect("already asserted");
    let ident = &first_seg.ident;
    let args = &first_seg.arguments;

    if ident == "Vec" {
        return parse_quote!(crate::traverse::SharedVec #args);
    } else if ident == "Box" {
        return parse_quote!(crate::traverse::SharedBox #args);
    }

    let ty = match args {
        // as the rule of thumb; if a type has lifetimes we should transform it to a traversable type.
        PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. })
            if args.iter().any(|arg| matches!(arg, GenericArgument::Lifetime(_))) =>
        {
            println!("EWE");
            ty
        }
        _ => ty,
    };

    ty
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
    traversable: TokenStream2,
}
