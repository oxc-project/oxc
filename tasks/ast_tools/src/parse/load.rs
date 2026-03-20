use std::{fs, path::Path};

use syn::{
    Attribute, Generics, Ident, Item, ItemEnum, ItemMacro, ItemStruct, Meta, Token, Variant,
    Visibility, WhereClause, braced,
    parse::{Parse, ParseBuffer},
    parse_file,
    punctuated::Punctuated,
};

use crate::{schema::FileId, utils::ident_name};

use super::{
    parse::convert_expr_to_string,
    skeleton::{EnumSkeleton, Skeleton, StructSkeleton},
};

/// Load file and extract structs and enums with `#[ast]` or `#[ast_meta]` attributes.
///
/// Only parses enough to get:
/// * Name of type.
/// * Inherits of enums wrapped in `inherit_variants!` macro.
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
    let skeleton = EnumSkeleton { name, file_id, is_foreign, is_meta, item, inherits: vec![] };
    Some(Skeleton::Enum(skeleton))
}

fn parse_macro(item: &ItemMacro, file_id: FileId) -> Option<Skeleton> {
    if item.mac.path.is_ident("inherit_variants") {
        return Some(parse_inherit_variants_macro(item, file_id));
    }
    if item.mac.path.is_ident("define_nonmax_u32_index_type")
        || item.mac.path.is_ident("define_index_type")
    {
        return parse_index_type_macro(item, file_id);
    }
    None
}

fn parse_inherit_variants_macro(item: &ItemMacro, file_id: FileId) -> Skeleton {
    item.mac
        .parse_body_with(|input: &ParseBuffer| {
            // Because of `@inherit`s we can't use the actual `ItemEnum` parse.
            // This closure is similar to how `ItemEnum` parser works, with the exception
            // of how we approach parsing variants.
            // First we try to parse as a `Variant`. If that fails, we try parsing as `@inherits`.
            // We raise an error only if both of these fail.
            let attrs = input.call(Attribute::parse_outer)?;

            let vis = input.parse::<Visibility>()?;
            let enum_token = input.parse::<Token![enum]>()?;
            let ident = input.parse::<Ident>()?;
            let generics = input.parse::<Generics>()?;

            let where_clause = input.parse::<Option<WhereClause>>()?;
            assert!(where_clause.is_none(), "Types with `where` clauses are not supported");

            let Some((name, false, false)) = get_type_name(&attrs, &ident) else {
                panic!("Enum in `inherit_variants!` macro must have `#[ast]` attr: {ident}");
            };

            let content;
            let brace_token = braced!(content in input);
            let mut variants = Punctuated::new();
            let mut inherits = vec![];
            while !content.is_empty() {
                match Variant::parse(&content) {
                    Ok(variant) => {
                        variants.push_value(variant);
                        let punct = content.parse()?;
                        variants.push_punct(punct);
                    }
                    _ => {
                        if content.parse::<Token![@]>().is_ok()
                            && content.parse::<Ident>().is_ok_and(|id| id == "inherit")
                        {
                            let inherit_ident =
                                content.parse::<Ident>().expect("Invalid `@inherits`");
                            inherits.push(ident_name(&inherit_ident));
                        } else {
                            panic!("Invalid `inherit_variants!` macro usage");
                        }
                    }
                }
            }

            let item = ItemEnum { attrs, vis, enum_token, ident, generics, brace_token, variants };
            let skeleton =
                EnumSkeleton { name, file_id, is_foreign: false, is_meta: false, item, inherits };
            Ok(Skeleton::Enum(skeleton))
        })
        .expect("Failed to parse contents of `inherit_variants!` macro")
}

fn parse_index_type_macro(item: &ItemMacro, file_id: FileId) -> Option<Skeleton> {
    item.mac
        .parse_body_with(|input: &ParseBuffer| {
            let attrs = input.call(Attribute::parse_outer)?;
            let vis = input.parse::<Visibility>()?;
            let _ = input.parse::<Token![struct]>()?;
            let ident = input.parse::<Ident>()?;
            // Handle optional `= Type` for define_index_type
            if input.parse::<Token![=]>().is_ok() {
                let _ = input.parse::<Ident>()?;
            }
            let _ = input.parse::<Token![;]>()?;

            let Some((name, is_foreign, is_meta)) = get_type_name(&attrs, &ident) else {
                return Ok(None);
            };

            // Create minimal ItemStruct - the field type doesn't matter since these are primitives
            let item_struct = ItemStruct {
                attrs,
                vis,
                struct_token: <_>::default(),
                ident,
                generics: <_>::default(),
                fields: syn::Fields::Unnamed(syn::FieldsUnnamed {
                    paren_token: <_>::default(),
                    unnamed: {
                        let mut punct = Punctuated::new();
                        punct.push(syn::Field {
                            attrs: vec![],
                            vis: syn::Visibility::Inherited,
                            mutability: syn::FieldMutability::None,
                            ident: None,
                            colon_token: None,
                            ty: syn::parse_quote!(NonMaxU32),
                        });
                        punct
                    },
                }),
                semi_token: None,
            };

            let skeleton = StructSkeleton { name, file_id, is_foreign, is_meta, item: item_struct };
            Ok(Some(Skeleton::Struct(skeleton)))
        })
        .ok()?
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
