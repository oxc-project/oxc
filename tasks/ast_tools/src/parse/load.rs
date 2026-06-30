use std::{fs, path::Path};

use quote::quote;
use syn::{
    Attribute, Ident, Item, ItemEnum, ItemMacro, ItemStruct, Meta, Token, Visibility,
    parse::ParseBuffer, parse_file, parse_quote, punctuated::Punctuated,
};

use crate::{schema::FileId, utils::ident_name};

use super::{
    parse::convert_expr_to_string,
    skeleton::{EnumSkeleton, Skeleton, StructSkeleton},
};

/// Load file and extract structs and enums with `#[ast]` or `#[ast_meta]` attributes.
///
/// Only parses enough to get the name of each type.
///
/// Returns a list of [`Skeleton`]s found in the file.
///
/// This is the bare minimum to be able to "link up" types to each other in next pass.
pub fn load_file(file_id: FileId, file_path: &str, root_path: &Path) -> Vec<Skeleton> {
    let content = fs::read_to_string(root_path.join(file_path)).unwrap();

    let file = parse_file(content.as_str()).unwrap();
    file.items.into_iter().filter_map(|item| parse_item(item, file_id)).collect()
}

fn parse_item(item: Item, file_id: FileId) -> Option<Skeleton> {
    match item {
        Item::Struct(item) => parse_struct(item, file_id),
        Item::Enum(item) => parse_enum(item, file_id),
        Item::Macro(item) => parse_macro(&item, file_id),
        _ => None,
    }
}

fn parse_struct(item: ItemStruct, file_id: FileId) -> Option<Skeleton> {
    let (name, is_foreign, is_meta) = get_type_name(&item.attrs, &item.ident)?;
    let skeleton = StructSkeleton { name, file_id, is_foreign, is_meta, item };
    Some(Skeleton::Struct(skeleton))
}

fn parse_enum(item: ItemEnum, file_id: FileId) -> Option<Skeleton> {
    let (name, is_foreign, is_meta) = get_type_name(&item.attrs, &item.ident)?;
    let skeleton = EnumSkeleton { name, file_id, is_foreign, is_meta, item };
    Some(Skeleton::Enum(skeleton))
}

fn parse_macro(item: &ItemMacro, file_id: FileId) -> Option<Skeleton> {
    item.mac.path.get_ident().and_then(|macro_name| match macro_name.to_string().as_str() {
        "inherit_variants" => parse_inherit_variants_macro(item, file_id),
        "define_nonmax_u32_index_type" => parse_index_type_macro(item, file_id, true),
        "define_index_type" => parse_index_type_macro(item, file_id, false),
        _ => None,
    })
}

fn parse_inherit_variants_macro(item: &ItemMacro, file_id: FileId) -> Option<Skeleton> {
    // The body of `inherit_variants!` is a normal enum.
    // Inheritance is expressed as `INHERIT(EnumName<'a>)` marker variants.
    let item = item
        .mac
        .parse_body::<ItemEnum>()
        .expect("Failed to parse contents of `inherit_variants!` macro");
    parse_enum(item, file_id)
}

fn parse_index_type_macro(
    item: &ItemMacro,
    file_id: FileId,
    is_nonmax_u32: bool,
) -> Option<Skeleton> {
    item.mac
        .parse_body_with(|input: &ParseBuffer| {
            let attrs = input.call(Attribute::parse_outer)?;
            let vis = input.parse::<Visibility>().ok();
            let _ = input.parse::<Token![struct]>()?;
            let ident = input.parse::<Ident>()?;

            // `define_nonmax_u32_index_type`'s inner type is always `NonMaxU32`.
            // `define_index_type` specifies inner type with `= Type`.
            let ty_ident = if is_nonmax_u32 {
                quote!(NonMaxU32)
            } else {
                let _ = input.parse::<Token![=]>()?;
                let ty_ident = input.parse::<Ident>()?;
                quote!(#ty_ident)
            };
            let _ = input.parse::<Token![;]>()?;

            let Some((name, is_foreign, is_meta)) = get_type_name(&attrs, &ident) else {
                return Ok(None);
            };

            let item = parse_quote! {
                #(#attrs)*
                #vis struct #ident (#ty_ident);
            };

            Ok(Some(Skeleton::Struct(StructSkeleton { name, file_id, is_foreign, is_meta, item })))
        })
        .unwrap()
}

/// Get name of type, and whether it has an `#[ast_meta]` attribute on it.
///
/// Parse attributes and find `#[ast]`, `#[ast(foreign = ForeignType)]`, or `#[ast_meta]`.
///
/// If no `#[ast]` or `#[ast_meta]` attr is present, returns `None`.
///
/// Otherwise, returns foreign name if provided with `#[ast(foreign = ForeignType)]`,
/// or otherwise name of the `ident`.
///
/// # Panics
/// Panics if cannot parse attributes.
fn get_type_name(
    attrs: &[Attribute],
    ident: &Ident,
) -> Option<(/* type name */ String, /* is_foreign */ bool, /* is_meta */ bool)> {
    let mut has_ast_attr = false;
    let mut has_meta_attr = false;
    let mut foreign_name = None;
    for attr in attrs {
        let Some(attr_ident) = attr.path().get_ident() else { continue };

        if attr_ident == "ast" {
            has_ast_attr = true;
            if let Some(this_foreign_name) = parse_ast_attr_foreign_name(attr, ident) {
                assert!(
                    foreign_name.is_none(),
                    "Multiple `#[ast(foreign)]` attributes on type: `{ident}`"
                );
                foreign_name = Some(this_foreign_name);
            }
        } else if attr_ident == "ast_meta" {
            assert!(
                matches!(&attr.meta, Meta::Path(_)),
                "Unable to parse `#[ast_meta]` attribute on type: `{ident}`"
            );
            assert!(!has_meta_attr, "Multiple `#[ast_meta]` attributes on type: `{ident}`");
            has_meta_attr = true;
        }
    }

    if has_meta_attr {
        assert!(
            !has_ast_attr,
            "Type cannot be tagged with both `#[ast]` and `#[ast_meta]`: `{ident}`"
        );
        Some((ident_name(ident), false, true))
    } else if has_ast_attr {
        if let Some(foreign_name) = foreign_name {
            Some((foreign_name, true, false))
        } else {
            Some((ident_name(ident), false, false))
        }
    } else {
        None
    }
}

fn parse_ast_attr_foreign_name(attr: &Attribute, ident: &Ident) -> Option<String> {
    let meta_list = match &attr.meta {
        Meta::Path(_) => return None,
        Meta::List(meta_list) => meta_list,
        Meta::NameValue(_) => panic!("Failed to parse `#[ast]` attribute"),
    };
    let metas = meta_list
        .parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
        .expect("Unable to parse `#[ast]` attribute");

    let mut foreign_name = None;
    for meta in &metas {
        if let Meta::NameValue(name_value) = meta
            && name_value.path.is_ident("foreign")
        {
            assert!(
                foreign_name.is_none(),
                "Multiple `#[ast(foreign)]` attributes on type: `{ident}`"
            );
            foreign_name = Some(convert_expr_to_string(&name_value.value));
        }
    }
    foreign_name
}
