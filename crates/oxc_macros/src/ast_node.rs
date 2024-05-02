use proc_macro2::TokenStream as TokenStream2;

use quote::{format_ident, quote};
use syn::{
    parse_quote, punctuated::Punctuated, AngleBracketedGenericArguments, Attribute, Expr,
    ExprGroup, ExprLit, Field, Fields, GenericArgument, Ident, ImplItemFn, Item, ItemEnum,
    ItemStruct, Lit, Meta, Path, PathArguments, PathSegment, Token, Type, TypePath, TypeReference,
    Variant,
};

const TRAVERSABLE: &str = "Traversable";

pub fn ast_node(mut item: Item) -> TokenStream2 {
    let result = match &mut item {
        Item::Struct(it) => modify_struct(it),
        Item::Enum(it) => modify_enum(it),
        _ => panic!("ast_node attribute can only be used on enums and structure types!"),
    };

    let ident = result.ident;

    let traversable_mod = format_ident!("traversable_{}", ident.to_string().to_lowercase());

    let traversable = result.traversable;

    let output = quote! {
        #[derive(layout_inspect::Inspect)]
        #item

        pub mod #traversable_mod {
            use super::*;

            #traversable
        }
    };

    // dbg!(&output.to_string());
    output
}

fn modify_struct(item: &mut ItemStruct) -> NodeData {
    let mut has_repr_attr = false;
    for attr in &item.attrs {
        if attr.path().is_ident("repr") {
            // TODO: Check is `#[repr(C)]`
            has_repr_attr = true;
        }
    }

    item.fields.iter().for_each(validate_field);
    // add the correct representation
    if !has_repr_attr {
        item.attrs.push(parse_quote!(#[repr(C)]));
    }
    NodeData { ident: &item.ident, traversable: generate_traversable_struct(item) }
}

fn modify_enum(item: &mut ItemEnum) -> NodeData {
    let mut has_repr_attr = false;
    for attr in &item.attrs {
        if attr.path().is_ident("repr") {
            // TODO: Check is `#[repr(C, u8)]`
            has_repr_attr = true;
        }
    }

    assert!(
        item.variants.len() <= 128,
        "`ast_node` enums are limited to a maximum of 128 variants."
    );
    item.variants.iter().for_each(validate_variant);
    // add the correct representation
    if !has_repr_attr {
        item.attrs.push(parse_quote!(#[repr(C, u8)]));
    }

    // add explicit discriminants to all variants
    let mut next_discriminant = 0u8;
    item.variants.iter_mut().for_each(|var| {
        if let Some((.., expr)) = &var.discriminant {
            // Explicit discriminant
            let discriminant = match expr {
                Expr::Lit(ExprLit { lit: Lit::Int(lit), .. }) => {
                    Some(lit.base10_parse::<u8>().unwrap())
                }
                Expr::Group(ExprGroup { expr, .. }) => {
                    if let Expr::Lit(ExprLit { lit: Lit::Int(lit), .. }) = &**expr {
                        Some(lit.base10_parse::<u8>().unwrap())
                    } else {
                        None
                    }
                }
                _ => None,
            };

            if let Some(discriminant) = discriminant {
                next_discriminant = discriminant + 1;
            } else {
                panic!("`ast_node` attribute only supports integers as explicit discriminators");
            }
        } else {
            // No explicit discriminant - create discriminant following last
            var.discriminant = Some((parse_quote!(=), parse_quote!(#next_discriminant)));
            next_discriminant += 1;
        };
    });

    assert!(
        next_discriminant <= 128,
        "Explicit discriminants must be less than 128 and must not cause later unspecified discriminants to reach 128"
    );

    // add the dummy variant.
    // using 128 as discriminant so other discriminants can be compared with cheap AND operations.
    item.variants.push(parse_quote!(Dummy = 128));

    NodeData { ident: &item.ident, traversable: generate_traversable_enum(item) }
}

// validators
// there are only here for early errors we still do some in-depth checks while generating the
// traversable modules.

fn validate_field(field: &Field) {
    assert!(
        match &field.ty {
            Type::Path(ty) if ty.path.segments.len() == 1 => true,
            Type::Reference(_) => true,
            _ => false,
        },
        "Currently `ast_node` attribute only supports single segment type paths and references."
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
        matches!(var.fields, Fields::Unnamed(_) | Fields::Unit),
        "Currently, `ast_node` attribute only supports unnamed and unit enum variants."
    );
}

// generators

fn generate_traversable_struct(item: &ItemStruct) -> TokenStream2 {
    let ident = format_ident!("{TRAVERSABLE}{}", item.ident);
    let generics = &item.generics;

    let fields = transform_fields(&item.fields);
    let methods: Vec<_> = fields.iter().flat_map(generate_traversable_methods).collect();

    // TODO: traits like serialization, Debug and Hash fail with `GCell`;
    // But we may want to keep other attributes.
    let output = quote! {
        #[repr(C)] // TODO: we can derive attributes from `item` if we filter invalid attributes.
        pub struct #ident #generics {
            #fields
        }

        impl #generics #ident #generics {
            #(#methods)*
        }

        impl #generics GCell<#ident #generics> {

        }
    };

    output
}

fn generate_traversable_enum(item: &ItemEnum) -> TokenStream2 {
    let ident = format_ident!("{TRAVERSABLE}{}", item.ident);
    let generics = &item.generics;

    let variants = transform_variants(&item.variants);

    // TODO: traits like serialization, Debug and Hash fail with `GCell`;
    // But we may want to keep other attributes.
    let output = quote! {
        #[repr(C, u8)]
        // #(#attributes)*
        pub enum #ident #generics {
            #variants
        }
    };

    output
}

fn generate_traversable_methods(field: &Field) -> Vec<ImplItemFn> {
    let type_name = &type_name(&field.ty);
    let mut methods = Vec::new();

    // Early reaturn if we are visiting a non traversable/shared type.
    if !is_traversable_type_name(type_name) && !is_shared_type_name(type_name) {
        return methods;
    }

    let v = &mut methods;

    if is_collection(type_name) {
        generate_traversable_vec_methods(v, field);
    }

    if is_ast_enum_type_name(type_name) {
        generate_traversable_enum_method(v, field);
    } else {
        generate_traversable_struct_method(v, field);
    }

    methods
}

fn generate_traversable_vec_methods(v: &mut Vec<ImplItemFn>, field: &Field) {
    let ident =
        field.ident.as_ref().expect("`ast_node` attribute only supports named struct fields.");

    debug_assert_eq!(ident, "SharedVec");

    let ty = &field.ty;
    let (_lifetime, ref generic_ty) = {
        let generics = type_generics(ty).expect("We only accept generic collections.");
        let mut iter = (&generics.args).into_iter();
        let GenericArgument::Lifetime(lifetime) =
            iter.next().expect("`ast_node` only support's arena vectors")
        else {
            panic!("`ast_node` expected the first argument to the `Vec` to be a lifetime.");
        };
        let GenericArgument::Type(generic_ty) =
            iter.next().expect("`ast_node` only accepts arena vectors.")
        else {
            panic!("`ast_node` expected the second argument to the `Vec` to be a generic type argument.");
        };

        assert!(iter.next().is_none(), "`ast_node` only accepts arena vectors.");

        (lifetime, transform_type(generic_ty.clone()))
    };

    TraversableStructVecMethodsGenerator { ident, generic_ty }.generate(v);
}

struct TraversableStructVecMethodsGenerator<'a> {
    ident: &'a Ident,
    generic_ty: &'a Type,
}

impl<'a> TraversableStructVecMethodsGenerator<'a> {
    fn generate_as_struct(self, _: &mut Vec<ImplItemFn>) {
        #![allow(clippy::unused_self)]
        // TODO: implement me when we stabilized the struct version of these methods
    }

    fn generate_as_enum(self, v: &mut Vec<ImplItemFn>) {
        macro_rules! vquote {
            ($($tt:tt)*) => {{
                v.push(parse_quote!($($tt)*))
            }};
        }
        let ident = self.ident;
        let generic_ty = self.generic_ty;

        let ident_len = format_ident!("{ident}_len");
        let ident_item = format_ident!("{ident}_item");
        let ident_item_get = format_ident!("{ident}_item_get");

        vquote! {
            /// Get length of #ident.
            fn #ident_len(&self) -> usize {
                self.#ident.len()
            }
        }

        vquote! {
            /// Get #ident item.
            /// # Panic
            /// Panics if `index` is out of bounds.
            fn #ident_item(&self, index: usize) -> #generic_ty {
                self.#ident[index]
            }
        }

        vquote! {
            /// Get #ident item.
            /// Returns `None` if `index` is out of bounds.
            fn #ident_item_get(&self, index: usize) -> #generic_ty {
                self.#ident.get(index).copied()
            }
        }
    }

    fn generate(self, v: &mut Vec<ImplItemFn>) {
        if is_ast_enum_type_name(&type_name(self.generic_ty)) {
            self.generate_as_enum(v);
        } else {
            self.generate_as_struct(v);
        }
    }
}

fn generate_traversable_struct_method(_: &mut Vec<ImplItemFn>, _: &Field) {}

fn generate_traversable_enum_method(_: &mut Vec<ImplItemFn>, _: &Field) {}

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

    fields.unnamed = fields.unnamed.into_iter().map(transform_field).collect();

    variant.fields = Fields::Unnamed(fields);
    variant.attrs.clear();
    variant
}

fn transform_type(ty: Type) -> Type {
    match ty {
        Type::Path(ty) => transform_type_path(ty),
        Type::Reference(ty) => transform_type_reference(ty),
        _ => ty,
    }
}

fn transform_type_path(ty: TypePath) -> Type {
    let ty = transform_generic_type(ty);

    Type::Path(ty)
}

fn transform_type_reference(mut ty: TypeReference) -> Type {
    let elem = transform_type(*ty.elem);
    ty.elem = Box::from(elem);

    Type::Reference(ty)
}

fn transform_generic_type(mut ty: TypePath) -> TypePath {
    fn recreate_original_path(mut path: Path, ident: Ident, arguments: PathArguments) -> Path {
        path.segments.push(PathSegment { ident, arguments });
        path
    }

    fn recreate_original_type(
        mut ty: TypePath,
        ident: Ident,
        arguments: PathArguments,
    ) -> TypePath {
        ty.path = recreate_original_path(ty.path, ident, arguments);
        ty
    }

    fn transform_args(
        args: Punctuated<GenericArgument, Token![,]>,
    ) -> Punctuated<GenericArgument, Token![,]> {
        args.into_iter()
            .map(|arg| match arg {
                GenericArgument::Type(ty) => GenericArgument::Type(transform_type(ty)),
                _ => arg,
            })
            .collect()
    }

    assert!(!ty.path.segments.is_empty());
    let seg = ty
        .path
        .segments
        .pop()
        .expect("Expected generic type with one or more path segments.")
        .into_value();

    match seg.arguments {
        // as the rule of thumb; if a type has lifetimes we should transform it to a traversable type.
        PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. })
            if args.iter().any(|arg| matches!(arg, GenericArgument::Lifetime(_))) =>
        {
            let path = if seg.ident == "Vec" {
                parse_quote!(SharedVec)
            } else if seg.ident == "Box" {
                parse_quote!(SharedBox)
            } else if !is_special_type_name(&seg.ident) {
                let new_ident = format_ident!("{TRAVERSABLE}{}", seg.ident);
                parse_quote!(#new_ident)
            } else {
                recreate_original_path(ty.path, seg.ident, PathArguments::None)
            };

            let args = transform_args(args);

            parse_quote!(#path <#args>)
        }
        PathArguments::AngleBracketed(angle_args @ AngleBracketedGenericArguments { .. }) => {
            recreate_original_type(
                ty,
                seg.ident,
                PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                    args: transform_args(angle_args.args),
                    ..angle_args
                }),
            )
        }
        PathArguments::Parenthesized(_) => {
            panic!("`ast_node` does not support parenthesized types(eg. `Fn(u32) -> u32)`.");
        }
        PathArguments::None => recreate_original_type(ty, seg.ident, PathArguments::None),
    }
}

// fn transform_derive_attribute(attr: Attribute) -> Attribute {
//     attr
// }

fn is_special_type_name(ident: &Ident) -> bool {
    ident == "Atom"
        || ident == "RegExp"
        || ident == "TemplateElementValue"
        || ident == "IdentifierName"
        || ident == "Modifiers"
}

fn is_traversable_type_name(ident: &str) -> bool {
    ident.starts_with(TRAVERSABLE)
}

fn is_shared_type_name(ident: &str) -> bool {
    ident.starts_with("Shared")
}

fn is_ast_enum_type_name(ident: &str) -> bool {
    if ident.len() <= TRAVERSABLE.len() {
        return false;
    }
    let ident = &ident[TRAVERSABLE.len()..];
    matches! {
        ident,
        | "Statement"
    }
}

fn is_collection(ident: &str) -> bool {
    ident == "Vec"
}

fn type_name(ty: &Type) -> String {
    fn type_path_name(ty: &TypePath) -> String {
        assert!(!ty.path.segments.is_empty());
        let seg = &ty
            .path
            .segments
            .last()
            .expect("Expected generic type with one or more path segments.");

        // TODO: is there any way to get this as `&str`?
        seg.ident.to_string()
    }

    fn type_ref_name(ty: &TypeReference) -> String {
        type_name(ty.elem.as_ref())
    }

    match ty {
        Type::Path(ty) => type_path_name(ty),
        Type::Reference(ty) => type_ref_name(ty),
        _ => panic!("Unsupported type!"),
    }
}

fn type_generics(ty: &Type) -> Option<&AngleBracketedGenericArguments> {
    fn type_path_generics(ty: &TypePath) -> Option<&AngleBracketedGenericArguments> {
        assert!(!ty.path.segments.is_empty());
        let seg = &ty
            .path
            .segments
            .last()
            .expect("Expected generic type with one or more path segments.");

        match &seg.arguments {
            PathArguments::AngleBracketed(args) => Some(args),
            PathArguments::Parenthesized(_) => {
                panic!("Parenthesized type arguments are not allowed with `ast_node` attribute.")
            }
            PathArguments::None => None,
        }
    }

    match ty {
        Type::Path(ty) => type_path_generics(ty),
        _ => panic!("Unsupported type!"),
    }
}

#[allow(dead_code)]
fn has_clone(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        let args = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated);
        attr.path().is_ident("derive")
            && args.is_ok_and(|args| args.iter().any(|arg| arg.path().is_ident("Clone")))
    })
}

struct NodeData<'a> {
    ident: &'a Ident,
    traversable: TokenStream2,
}
