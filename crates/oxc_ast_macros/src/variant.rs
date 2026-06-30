//! Construct `syn::Variant`s for variants inherited via `INHERIT`, for the `#[ast]` macro to insert.
//!
//! These are built by hand, rather than with `parse_quote!`, to avoid the cost of lexing + parsing
//! on every compilation. `oxc_ast_tools` generates calls to [`make_inherited_variant`] in
//! `INHERITED_ENUMS` (see generated `enums.rs`).

use proc_macro2::Span;
use syn::{
    AngleBracketedGenericArguments, AttrStyle, Attribute, Expr, ExprLit, Field, FieldMutability,
    Fields, FieldsUnnamed, GenericArgument, Ident, Lifetime, Lit, LitInt, LitStr, Meta,
    MetaNameValue, Path, PathArguments, PathSegment, Type, TypePath, Variant, Visibility,
    punctuated::Punctuated, token,
};

use crate::ast::EnumVariant;

/// Construct a `syn::Variant` from an [`EnumVariant`] and its enum's `doc`, equivalent to:
///
/// ```ignore
/// #[doc = <doc>]
/// <name>(<inner>) = <discriminant>          // if `!is_boxed`
/// #[doc = <doc>]
/// <name>(Box<'a, <inner>>) = <discriminant> // if `is_boxed`
/// ```
///
/// where `<inner>` is `<inner_name>` or `<inner_name><'a>` (depending on `inner_has_lifetime`).
///
/// All spans are set to `span` (the `INHERIT` marker variant's span), so "go to definition"
/// on the inserted variant jumps to the marker in source.
pub fn make_inherited_variant(variant: &EnumVariant, doc: &str, span: Span) -> Variant {
    let mut ty = named_type(variant.inner_name, variant.inner_has_lifetime, span);
    if variant.is_boxed {
        ty = boxed(ty, span);
    }

    Variant {
        attrs: vec![doc_attr(doc, span)],
        ident: Ident::new(variant.name, span),
        fields: Fields::Unnamed(FieldsUnnamed {
            paren_token: token::Paren::default(),
            unnamed: once_punctuated(Field {
                attrs: vec![],
                vis: Visibility::Inherited,
                mutability: FieldMutability::None,
                ident: None,
                colon_token: None,
                ty,
            }),
        }),
        discriminant: Some((token::Eq::default(), int_expr(variant.discriminant, span))),
    }
}

/// `#[doc = <doc>]`
fn doc_attr(doc: &str, span: Span) -> Attribute {
    Attribute {
        pound_token: token::Pound::default(),
        style: AttrStyle::Outer,
        bracket_token: token::Bracket::default(),
        meta: Meta::NameValue(MetaNameValue {
            path: Path::from(Ident::new("doc", span)),
            eq_token: token::Eq::default(),
            value: Expr::Lit(ExprLit { attrs: vec![], lit: Lit::Str(LitStr::new(doc, span)) }),
        }),
    }
}

/// Integer literal expression e.g. `8`.
fn int_expr(value: u8, span: Span) -> Expr {
    Expr::Lit(ExprLit { attrs: vec![], lit: Lit::Int(LitInt::new(&value.to_string(), span)) })
}

/// `<name>` or `<name><'a>`.
fn named_type(name: &str, has_lifetime: bool, span: Span) -> Type {
    let arguments = if has_lifetime {
        angle_bracketed(once_punctuated(lifetime(span)))
    } else {
        PathArguments::None
    };
    path_type(Ident::new(name, span), arguments)
}

/// `Box<'a, <inner>>`.
fn boxed(inner: Type, span: Span) -> Type {
    let mut args = Punctuated::new();
    args.push(lifetime(span));
    args.push(GenericArgument::Type(inner));
    path_type(Ident::new("Box", span), angle_bracketed(args))
}

/// `'a` lifetime generic argument.
fn lifetime(span: Span) -> GenericArgument {
    GenericArgument::Lifetime(Lifetime::new("'a", span))
}

/// Wrap generic arguments in `<...>`.
fn angle_bracketed(args: Punctuated<GenericArgument, token::Comma>) -> PathArguments {
    PathArguments::AngleBracketed(AngleBracketedGenericArguments {
        colon2_token: None,
        lt_token: token::Lt::default(),
        args,
        gt_token: token::Gt::default(),
    })
}

/// Single-segment path type, e.g. `Foo` or `Foo<'a>`.
fn path_type(ident: Ident, arguments: PathArguments) -> Type {
    Type::Path(TypePath {
        qself: None,
        path: Path {
            leading_colon: None,
            segments: once_punctuated(PathSegment { ident, arguments }),
        },
    })
}

/// Create a `Punctuated` containing a single element.
fn once_punctuated<T, P>(value: T) -> Punctuated<T, P> {
    let mut punctuated = Punctuated::new();
    punctuated.push_value(value);
    punctuated
}
