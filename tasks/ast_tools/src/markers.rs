use proc_macro2::TokenStream;
use serde::Serialize;
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
#[derive(Debug, Clone)]
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
#[derive(Debug, Default, Clone)]
pub struct VisitArgs(Punctuated<VisitArg, Token![,]>);

impl IntoIterator for VisitArgs {
    type IntoIter = syn::punctuated::IntoIter<Self::Item>;
    type Item = VisitArg;

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
#[derive(Default, Debug)]
pub struct VisitMarkers {
    pub visit_as: Option<Ident>,
    pub visit_args: Option<VisitArgs>,
    pub enter_before: bool,
    pub ignore: bool,
}

/// A struct representing `#[scope(...)]` markers
#[derive(Default, Debug)]
pub struct ScopeMarkers {
    /// `#[scope(enter_before)]`
    pub enter_before: bool,
    /// `#[scope(exit_before)]`
    pub exit_before: bool,
}

/// A struct representing all the helper attributes that might be used with `#[generate_derive(...)]`
#[derive(Debug, Default, Serialize)]
pub struct DeriveAttributes {
    pub clone_in: CloneInAttribute,
}

/// A enum representing the value passed in `#[clone_in(...)]` derive helper attribute.
#[derive(Debug, Default, Serialize)]
pub enum CloneInAttribute {
    #[default]
    None,
    Default,
}

impl From<&Ident> for CloneInAttribute {
    fn from(ident: &Ident) -> Self {
        if ident == "default" {
            Self::Default
        } else {
            panic!("Invalid argument used in `#[clone_in(...)]` attribute.");
        }
    }
}

/// A struct representing the `#[scope(...)]` attribute.
#[derive(Debug, Default)]
pub struct ScopeAttribute {
    pub flags: Option<Expr>,
    pub strict_if: Option<Expr>,
}

impl Parse for ScopeAttribute {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        let parsed = input.parse_terminated(CommonAttribute::parse, Token![,])?;
        Ok(parsed.into_iter().fold(Self::default(), |mut acc, CommonAttribute { ident, args }| {
            let expr = parse2(args).expect("Invalid `#[scope]` input.");
            match ident.to_string().as_str() {
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

pub fn get_visit_markers<'a, I>(attrs: I) -> crate::Result<VisitMarkers>
where
    I: IntoIterator<Item = &'a Attribute>,
{
    #[expect(clippy::trivially_copy_pass_by_ref)]
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

    attr.map_or_else(
        || Ok(VisitMarkers::default()),
        |attr| {
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
        },
    )
}

pub fn get_scope_markers<'a, I>(attrs: I) -> crate::Result<ScopeMarkers>
where
    I: IntoIterator<Item = &'a Attribute>,
{
    #[expect(clippy::trivially_copy_pass_by_ref)]
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

    attr.map_or_else(
        || Ok(ScopeMarkers::default()),
        |attr| {
            attr.parse_args_with(Ident::parse)
                .map(|id| ScopeMarkers {
                    enter_before: id == "enter_before",
                    exit_before: id == "exit_before",
                })
                .normalize()
        },
    )
}

pub fn get_derive_attributes<'a, I>(attrs: I) -> crate::Result<DeriveAttributes>
where
    I: IntoIterator<Item = &'a Attribute>,
{
    fn try_parse_clone_in(attr: &Attribute) -> crate::Result<Option<CloneInAttribute>> {
        if attr.path().is_ident("clone_in") {
            let arg = attr.parse_args_with(Ident::parse).normalize()?;
            Ok(Some(CloneInAttribute::from(&arg)))
        } else {
            Ok(None)
        }
    }
    let mut clone_in = None;
    for attr in attrs {
        if let Some(attr) = try_parse_clone_in(attr)? {
            assert!(clone_in.replace(attr).is_none(), "Duplicate `#[clone_in(...)]` attribute.");
        }
    }
    Ok(DeriveAttributes { clone_in: clone_in.unwrap_or_default() })
}

pub fn get_scope_attribute<'a, I>(attrs: I) -> Option<crate::Result<ScopeAttribute>>
where
    I: IntoIterator<Item = &'a Attribute>,
{
    let attr = attrs.into_iter().find(|it| it.path().is_ident("scope"));
    attr.map(|attr| {
        debug_assert!(attr.path().is_ident("scope"));
        let result = if matches!(attr.meta, Meta::Path(_)) {
            // empty `#[scope]`.
            Ok(ScopeAttribute::default())
        } else {
            attr.parse_args_with(ScopeAttribute::parse)
        };

        result.normalize()
    })
}
