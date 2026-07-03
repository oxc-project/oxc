use proc_macro2::{TokenStream, TokenTree};
use quote::{quote, quote_spanned};
use syn::{
    Attribute, Fields, FieldsNamed, Ident, Item, ItemEnum, ItemStruct, PathArguments, PathSegment,
    Type, Variant, parse_quote, punctuated::Punctuated, spanned::Spanned, token::Comma,
};

use crate::{
    generated::{
        derived_traits::get_trait_crate_and_generics,
        enums::{ENUMS, INHERITED_ENUMS},
        structs::STRUCTS,
    },
    variant::make_inherited_variant,
};

/// `#[ast]` macro.
pub fn ast(item: &mut Item, args: TokenStream) -> TokenStream {
    match item {
        Item::Enum(item) => modify_enum(item),
        Item::Struct(item) => modify_struct(item, args),
        _ => unreachable!(),
    }
}

/// Details of how `#[ast]` macro should modify an enum.
pub struct EnumDetails {
    /// `true` if all variants are fieldless (unit) - including any inherited variants.
    /// Determines whether the enum is `#[repr(u8)]` (fieldless) or `#[repr(C, u8)]` (has fields).
    pub is_fieldless: bool,

    /// Indexes into `INHERITED_ENUMS` of the enums whose variants this enum inherits.
    ///
    /// `#[ast]` macro removes the `INHERIT` marker variants from the enum, and inserts the
    /// variants of these enums in their place. Empty if the enum doesn't inherit any variants.
    pub inherits: &'static [u32],
}

/// An enum whose variants are inherited by other enums via `INHERIT`.
///
/// `oxc_ast_tools` generates these in `INHERITED_ENUMS` (see generated `enums.rs`).
pub struct InheritedEnum {
    /// Doc comment added to each inherited variant, e.g. `" Inherited from [`Expression`]"`.
    pub doc: &'static str,
    /// The enum's (flattened) variants.
    pub variants: &'static [EnumVariant],
}

/// Data describing a variant inherited via `INHERIT`.
///
/// [`make_inherited_variant`] turns one into a `syn::Variant`.
pub struct EnumVariant {
    /// Variant name, e.g. `"BooleanLiteral"`.
    pub name: &'static str,
    /// Name of the variant's field type (the inner type if boxed),
    /// e.g. `"BooleanLiteral"` in `Box<'a, BooleanLiteral>`.
    pub inner_name: &'static str,
    /// `true` if the field type has a lifetime (`Inner<'a>` vs `Inner`).
    pub inner_has_lifetime: bool,
    /// `true` if the field type is `Box`-wrapped (`Box<'a, Inner>` vs `Inner`).
    pub is_boxed: bool,
    /// Variant discriminant.
    pub discriminant: u8,
}

/// Add `#[repr(...)]` and `#[derive(::oxc_ast_macros::Ast)]` to enum, insert inherited variants,
/// and static assertions for `#[generate_derive]`.
fn modify_enum(item: &mut ItemEnum) -> TokenStream {
    modify_enum_impl(item).unwrap_or_else(|message| {
        let error = compile_error(&item.ident, message);
        quote! {
            #[derive(::oxc_ast_macros::Ast)]
            #item
            #error
        }
    })
}

fn modify_enum_impl(item: &mut ItemEnum) -> Result<TokenStream, &'static str> {
    // Get enum data. Calculated by `oxc_ast_tools`, rather than re-calculating here on every compilation.
    let enum_name = item.ident.to_string();
    let Some(enum_details) = ENUMS.get(&enum_name) else {
        return Err("Enum is unknown. Run `just ast` to re-run the codegen.");
    };

    // Replace `INHERIT` marker variants with the variants of the enums they inherit from
    let mut inherit_links = TokenStream::new();
    if !enum_details.inherits.is_empty() {
        // Remove the `INHERIT(EnumName<'a>)` markers, capturing for each (in source order):
        // * The whole marker variant's span.
        // * The `EnumName` ident from the field type (which carries `EnumName`'s span in source).
        // * Whether the field type has a lifetime (`EnumName<'a>` vs `EnumName`).
        let mut markers = Vec::with_capacity(enum_details.inherits.len());
        let variants = std::mem::take(&mut item.variants);
        item.variants = variants
            .into_iter()
            .filter(|variant| {
                if is_inherit_marker(variant) {
                    let segment = inherit_marker_inner_segment(variant);
                    let has_lifetime = !matches!(segment.arguments, PathArguments::None);
                    markers.push((variant.span(), segment.ident.clone(), has_lifetime));
                    false
                } else {
                    true
                }
            })
            .collect();

        // `inherits` and `markers` are both in source order, so they line up
        for (&index, (marker_span, inner_ident, has_lifetime)) in
            enum_details.inherits.iter().zip(markers)
        {
            let inherited = &INHERITED_ENUMS[index as usize];

            // Insert the inherited enum's variants, spanned to the `INHERIT` marker variant they replace
            // (so "go to definition" on the inserted variants jumps to the marker).
            item.variants.extend(
                inherited
                    .variants
                    .iter()
                    .map(|variant| make_inherited_variant(variant, inherited.doc, marker_span)),
            );

            // Emit a reference to the inherited enum as a type, using the `EnumName` ident from the
            // marker's field type (so it carries the source span of `EnumName` in `INHERIT(EnumName<'a>)`).
            // This makes "go to definition" on that `EnumName` resolve to the enum - otherwise the
            // marker variant is removed, so the source token would resolve to nothing.
            let generics = if has_lifetime { quote!(<'static>) } else { quote!() };
            inherit_links.extend(quote! {
                const _: Option<#inner_ident #generics> = None;
            });
        }
    }

    // Fieldless enums are `#[repr(u8)]`. Enums with any non-unit variant are `#[repr(C, u8)]`.
    let repr = if enum_details.is_fieldless { quote!(#[repr(u8)]) } else { quote!(#[repr(C, u8)]) };

    let assertions = assert_generated_derives(&item.attrs);

    Ok(quote! {
        #repr
        #[derive(::oxc_ast_macros::Ast)]
        #item
        #assertions
        #inherit_links
    })
}

/// Check if an enum variant is an `INHERIT` marker (e.g. `INHERIT(MemberExpression<'a>)`).
fn is_inherit_marker(variant: &Variant) -> bool {
    variant.ident == "INHERIT"
}

/// Get the inherited enum's type path segment from an `INHERIT` marker variant,
/// e.g. `MemberExpression<'a>` from `INHERIT(MemberExpression<'a>)`.
///
/// The segment's `ident` carries the span of `MemberExpression` in the original source,
/// and its `arguments` indicate whether the type has a lifetime.
fn inherit_marker_inner_segment(variant: &Variant) -> &PathSegment {
    let Fields::Unnamed(fields) = &variant.fields else {
        unreachable!("`INHERIT` marker must be a tuple variant: `{}`", variant.ident);
    };
    let Some(Type::Path(type_path)) = fields.unnamed.first().map(|field| &field.ty) else {
        unreachable!("`INHERIT` marker must contain a single path type: `{}`", variant.ident);
    };
    type_path.path.segments.last().expect("`INHERIT` marker type has empty path")
}

/// Details of how `#[ast]` macro should modify a struct.
pub struct StructDetails {
    /// Memory order of the struct's fields.
    ///
    /// `field_order[n]` is the position in memory of the field which is written `n`th in source.
    /// `#[ast]` macro re-orders the fields into this order, so the struct is packed with minimal padding.
    ///
    /// `None` if the fields are already in optimal order, and don't need re-ordering.
    pub field_order: Option<&'static [u8]>,

    /// Whether the struct is an AST node.
    /// i.e. it has a `node_id: Cell<NodeId>` field (and therefore an `AstKind`).
    ///
    /// `#[ast]` macro adds `#[non_exhaustive]` to AST node structs. That prevents them being constructed
    /// with a struct literal outside of `oxc_ast` - consumers must use `AstBuilder` instead.
    pub is_node: bool,

    /// `true` if struct has at most 1 field with non-zero size, so can be `#[repr(transparent)]`.
    pub is_transparent: bool,
}

/// Add `#[repr(C)]` / `#[repr(transparent)]`, and `#[derive(::oxc_ast_macros::Ast)]` to struct,
/// and static assertions for `#[generate_derive]`.
/// If is an AST node (has a `NodeId`), add `#[non_exhaustive]` attr.
/// Re-order struct fields if instructed by `STRUCTS` data.
fn modify_struct(item: &mut ItemStruct, args: TokenStream) -> TokenStream {
    modify_struct_impl(item, args).unwrap_or_else(|message| {
        let error = compile_error(&item.ident, message);
        quote! {
            #[derive(::oxc_ast_macros::Ast)]
            #item
            #error
        }
    })
}

fn modify_struct_impl(
    item: &mut ItemStruct,
    args: TokenStream,
) -> Result<TokenStream, &'static str> {
    // Skip foreign types
    if let Some(TokenTree::Ident(ident)) = args.into_iter().next()
        && ident == "foreign"
    {
        return Ok(quote! {
            #[derive(::oxc_ast_macros::Ast)]
            #item
        });
    }

    // Get struct data
    let struct_name = item.ident.to_string();
    let Some(struct_details) = STRUCTS.get(&struct_name) else {
        return Err("Struct is unknown. Run `just ast` to re-run the codegen.");
    };

    let assertions = assert_generated_derives(&item.attrs);

    reorder_struct_fields(item, struct_details)?;

    // `#[repr(transparent)]` for structs with at most 1 non-zero-sized field, `#[repr(C)]` otherwise
    let repr = if struct_details.is_transparent {
        quote!(#[repr(transparent)])
    } else {
        quote!(#[repr(C)])
    };

    // `#[non_exhaustive]` on AST node types
    let non_exhaustive =
        if struct_details.is_node { Some(quote!(#[non_exhaustive])) } else { None };

    Ok(quote! {
        #repr
        #non_exhaustive
        #[derive(::oxc_ast_macros::Ast)]
        #item
        #assertions
    })
}

/// Re-order struct fields, depending on instructions in `STRUCTS` (which is codegen-ed).
///
/// Mutates `item` in place, re-ordering its fields.
fn reorder_struct_fields(
    item: &mut ItemStruct,
    struct_details: &StructDetails,
) -> Result<(), &'static str> {
    // Exit if fields don't need re-ordering
    let Some(field_order) = struct_details.field_order else {
        return Ok(());
    };

    // Re-order fields.
    // `field_order` contains indexes of fields in the order they should be.
    let named = match &mut item.fields {
        Fields::Named(FieldsNamed { named, .. }) if named.len() == field_order.len() => named,
        _ => {
            return Err("Struct has been altered. Run `just ast` to re-run the codegen.");
        }
    };

    // Create 2 sets of fields.
    // 1st set are the fields in original order, each prefixed with `#[cfg(doc)]`.
    // 2nd set are the fields in new order, each prefixed with `#[cfg(not(doc))]`.
    // This is necessary so that fields are listed in original source order in docs.
    let mut fields = named.clone().into_pairs().zip(field_order).collect::<Vec<_>>();
    fields.sort_unstable_by_key(|(_, index)| **index);

    for field in named.iter_mut() {
        field.attrs.insert(0, parse_quote!( #[cfg(doc)]));
    }

    named.extend(fields.into_iter().map(|(mut pair, _)| {
        pair.value_mut().attrs.insert(0, parse_quote!( #[cfg(not(doc))]));
        pair
    }));

    Ok(())
}

/// Generate assertions that traits used in `#[generate_derive]` are in scope.
///
/// e.g. for `#[generate_derive(GetSpan)]`, it generates:
///
/// ```rs
/// const _: () = {
///     {
///         trait AssertionTrait: ::oxc_span::GetSpan {}
///         impl<T: GetSpan> AssertionTrait for T {}
///     }
/// };
/// ```
///
/// If `GetSpan` is not in scope, or it is not the correct `oxc_span::GetSpan`,
/// this will raise a compilation error.
///
/// If any errors e.g. cannot parse `#[generate_derive]`, or unknown traits, just skip them.
/// It is the responsibility of `oxc_ast_tools` to raise errors for those.
fn assert_generated_derives(attrs: &[Attribute]) -> TokenStream {
    let mut assertions = quote!();
    for attr in attrs {
        if !attr.path().is_ident("generate_derive") {
            continue;
        }

        let Ok(parsed) = attr.parse_args_with(Punctuated::<Ident, Comma>::parse_terminated) else {
            continue;
        };

        for trait_ident in parsed {
            let trait_name = trait_ident.to_string();
            let Some((trait_path, generics)) = get_trait_crate_and_generics(&trait_name) else {
                continue;
            };

            // These are wrapped in a scope to avoid the need for unique identifiers
            assertions.extend(quote! {{
                trait AssertionTrait: #trait_path #generics {}
                impl<T: #trait_ident #generics> AssertionTrait for T {}
            }});
        }
    }

    quote! {
        const _: () = { #assertions };
    }
}

/// Generate a `compile_error!` macro invocation with the given message, and the span of `spanned`.
fn compile_error<S: Spanned>(spanned: &S, message: &str) -> TokenStream {
    quote_spanned! { spanned.span() => compile_error!(#message); }
}
