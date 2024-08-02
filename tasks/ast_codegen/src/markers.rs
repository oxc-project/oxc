use proc_macro2::TokenStream;
use syn::{
    ext::IdentExt,
    parenthesized,
    parse::{Parse, ParseStream},
    parse2,
    punctuated::Punctuated,
    spanned::Spanned,
    token, Attribute, Expr, Ident, Meta, MetaNameValue, Token,
};

use crate::util::NormalizeError;

/// A single visit argument passed via `#[visit(args(...))]`
#[derive(Debug)]
pub struct VisitArg {
    pub ident: Ident,
    pub value: Expr,
}

impl Parse for VisitArg {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        let nv: MetaNameValue = input.parse()?;
        Ok(Self {
            ident: nv.path.get_ident().map_or_else(
                || Err(syn::Error::new(nv.span(), "Invalid `visit_args` input!")),
                |it| Ok(it.clone()),
            )?,
            value: nv.value,
        })
    }
}

/// A struct containing `#[visit(args(...))]` items
///                              ^^^^^^^^^
#[derive(Debug, Default)]
pub struct VisitArgs(Punctuated<VisitArg, Token![,]>);

impl IntoIterator for VisitArgs {
    type Item = VisitArg;
    type IntoIter = syn::punctuated::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl Parse for VisitArgs {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        input.parse_terminated(VisitArg::parse, Token![,]).map(Self)
    }
}

/// A struct representing `#[visit(...)]` markers
#[derive(Debug)]
pub struct VisitMarkers {
    pub visit_as: Option<Ident>,
    pub visit_args: Option<VisitArgs>,
    pub enter_before: bool,
    pub ignore: bool,
}

/// A struct representing `#[scope(...)]` markers
pub struct ScopeMarkers {
    pub enter_before: bool,
}

/// A struct representing the `#[scope(...)]` attribute.
#[derive(Debug, Default)]
pub struct ScopeAttr {
    pub r#if: Option<Expr>,
    pub flags: Option<Expr>,
    pub strict_if: Option<Expr>,
}

impl Parse for ScopeAttr {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        let parsed = input.parse_terminated(CommonAttribute::parse, Token![,])?;
        Ok(parsed.into_iter().fold(Self::default(), |mut acc, CommonAttribute { ident, args }| {
            let expr = parse2(args).expect("Invalid `#[scope]` input.");
            match ident.to_string().as_str() {
                "if" => acc.r#if = Some(expr),
                "flags" => acc.flags = Some(expr),
                "strict_if" => acc.strict_if = Some(expr),
                _ => {}
            }
            acc
        }))
    }
}

#[derive(Debug)]
struct CommonAttribute {
    ident: Ident,
    args: TokenStream,
}

impl Parse for CommonAttribute {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        let ident = input.call(Ident::parse_any).unwrap();
        let args =
            if input.peek(token::Paren) || input.peek(token::Bracket) || input.peek(token::Brace) {
                let content;
                parenthesized!(content in input);
                content.parse()?
            } else {
                TokenStream::default()
            };
        Ok(CommonAttribute { ident, args })
    }
}

pub fn get_visit_markers<'a, I>(attrs: I) -> Option<crate::Result<VisitMarkers>>
where
    I: IntoIterator<Item = &'a Attribute>,
{
    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn predicate(it: &&Attribute) -> bool {
        it.path().is_ident("visit")
    }

    let mut iter = attrs.into_iter();
    let attr = iter.find(predicate);
    debug_assert_eq!(
        iter.find(predicate),
        None,
        "For now we only accept one `#[visit]` marker per field/variant, Please merge them together!"
    );

    attr.map(|attr| {
        let mut visit_as = None;
        let mut visit_args = None;
        let mut enter_before = false;
        let mut ignore = false;
        let nested =
            attr.parse_args_with(Punctuated::<CommonAttribute, Token![,]>::parse_terminated);
        nested
            .map(|nested| {
                for com in nested {
                    if com.ident == "args" {
                        visit_args = Some(parse2(com.args).unwrap());
                    } else if com.ident == "as" {
                        visit_as =
                            Some(parse2(com.args).expect("Invalid `#[visit[as(...)]]` input!"));
                    } else if com.ident == "enter_before" {
                        enter_before = true;
                    } else if com.ident == "ignore" {
                        ignore = true;
                    } else {
                        panic!("Invalid `#[visit(...)]` input!")
                    }
                }
            })
            .map(|()| VisitMarkers { visit_as, visit_args, enter_before, ignore })
            .normalize()
    })
}

pub fn get_scope_markers<'a, I>(attrs: I) -> Option<crate::Result<ScopeMarkers>>
where
    I: IntoIterator<Item = &'a Attribute>,
{
    #[allow(clippy::trivially_copy_pass_by_ref)]
    fn predicate(it: &&Attribute) -> bool {
        it.path().is_ident("scope")
    }

    let mut iter = attrs.into_iter();
    let attr = iter.find(predicate);
    debug_assert_eq!(
        iter.find(predicate),
        None,
        "For now we only accept one `#[scope]` marker per field/variant, Please merge them together!"
    );

    attr.map(|attr| {
        attr.parse_args_with(Ident::parse)
            .map(|id| ScopeMarkers { enter_before: id == "enter_before" })
            .normalize()
    })
}

pub fn get_scope_attr<'a, I>(attrs: I) -> Option<crate::Result<ScopeAttr>>
where
    I: IntoIterator<Item = &'a Attribute>,
{
    let attr = attrs.into_iter().find(|it| it.path().is_ident("scope"));
    attr.map(|attr| {
        debug_assert!(attr.path().is_ident("scope"));
        let result = if matches!(attr.meta, Meta::Path(_)) {
            // empty `#[scope]`.
            Ok(ScopeAttr::default())
        } else {
            attr.parse_args_with(ScopeAttr::parse)
        };

        result.normalize()
    })
}
