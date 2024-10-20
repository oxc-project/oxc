use proc_macro2::TokenStream;
use serde::Serialize;
use syn::{
    ext::IdentExt,
    parenthesized,
    parse::{Parse, ParseStream},
    parse2,
    punctuated::Punctuated,
    spanned::Spanned,
    token, Attribute, Expr, Ident, LitStr, Meta, MetaNameValue, Token,
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
    pub estree: ESTreeFieldAttribute,
    pub tsify_type: Option<String>,
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

/// An enum representing the `#[estree(...)]` attributes that we implement for structs.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub enum ESTreeStructAttribute {
    CustomSerialize,
    NoType,
    Type(String),
}

impl Parse for ESTreeStructAttribute {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        let is_type = input.peek(Token![type]);
        if is_type {
            input.parse::<Token![type]>()?;
            input.parse::<Token![=]>()?;
            Ok(Self::Type(input.parse::<LitStr>()?.value()))
        } else {
            let ident = input.call(Ident::parse_any).unwrap().to_string();
            match ident.as_str() {
                "no_type" => Ok(Self::NoType),
                "custom_serialize" => Ok(Self::CustomSerialize),
                _ => panic!("Unsupported #[estree(...)] argument: {ident}"),
            }
        }
    }
}

/// A struct representing the `#[estree(...)]` attributes that we implement for enums.
#[derive(Debug, Serialize, Default)]
pub struct ESTreeEnumAttribute {
    pub rename_all: Option<String>,
    pub untagged: bool,
    pub custom_ts_def: bool,
}

impl Parse for ESTreeEnumAttribute {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        let mut rename_all = None;
        let mut untagged = false;
        let mut custom_ts_def = false;

        loop {
            let ident = input.call(Ident::parse_any).unwrap().to_string();
            match ident.as_str() {
                "rename_all" => {
                    input.parse::<Token![=]>()?;
                    assert!(
                        rename_all.replace(input.parse::<LitStr>()?.value()).is_none(),
                        "Duplicate estree(rename_all)"
                    );
                }
                "untagged" => {
                    if untagged {
                        panic!("Duplicate estree(untagged)");
                    } else {
                        untagged = true;
                    }
                }
                "custom_ts_def" => {
                    if custom_ts_def {
                        panic!("Duplicate estree(custom_ts_def)");
                    } else {
                        custom_ts_def = true;
                    }
                }
                arg => panic!("Unsupported #[estree(...)] argument: {arg}"),
            }
            let comma = input.peek(Token![,]);
            if comma {
                input.parse::<Token![,]>().unwrap();
            } else {
                break;
            }
        }
        Ok(Self { rename_all, untagged, custom_ts_def })
    }
}

/// A struct representing the `#[estree(...)]` attributes that we implement for fields.
#[derive(Debug, Serialize, Default)]
pub struct ESTreeFieldAttribute {
    pub flatten: bool,
    pub skip: bool,
    pub rename: Option<String>,
}

impl Parse for ESTreeFieldAttribute {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        let mut flatten = false;
        let mut skip = false;
        let mut rename = None;

        loop {
            let ident = input.call(Ident::parse_any).unwrap().to_string();
            match ident.as_str() {
                "rename" => {
                    input.parse::<Token![=]>()?;
                    assert!(
                        rename.replace(input.parse::<LitStr>()?.value()).is_none(),
                        "Duplicate estree(rename)"
                    );
                }
                "flatten" => {
                    if flatten {
                        panic!("Duplicate estree(flatten)");
                    } else {
                        flatten = true;
                    }
                }
                "skip" => {
                    if skip {
                        panic!("Duplicate estree(skip)");
                    } else {
                        skip = true;
                    }
                }
                arg => panic!("Unsupported #[estree(...)] argument: {arg}"),
            }
            let comma = input.peek(Token![,]);
            if comma {
                input.parse::<Token![,]>().unwrap();
            } else {
                break;
            }
        }
        Ok(Self { flatten, skip, rename })
    }
}

/// A struct representing the `#[tsify(type = "...")]` attribute.
pub struct TsifyAttribute(String);

impl Parse for TsifyAttribute {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        input.parse::<Token![type]>()?;
        input.parse::<Token![=]>()?;
        let type_ = input.parse::<LitStr>()?;
        Ok(Self(type_.value()))
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
    fn try_parse_estree(attr: &Attribute) -> crate::Result<Option<ESTreeFieldAttribute>> {
        if attr.path().is_ident("estree") {
            let arg = attr.parse_args_with(ESTreeFieldAttribute::parse).normalize()?;
            Ok(Some(arg))
        } else {
            Ok(None)
        }
    }
    fn try_parse_tsify_type(attr: &Attribute) -> crate::Result<Option<String>> {
        if attr.path().is_ident("tsify") {
            let arg = attr.parse_args_with(TsifyAttribute::parse).normalize()?;
            Ok(Some(arg.0))
        } else {
            Ok(None)
        }
    }
    let mut clone_in = None;
    let mut estree = None;
    let mut tsify_type = None;
    for attr in attrs {
        if let Some(attr) = try_parse_clone_in(attr)? {
            assert!(clone_in.replace(attr).is_none(), "Duplicate `#[clone_in(...)]` attribute.");
        }
        if let Some(attr) = try_parse_estree(attr)? {
            assert!(estree.replace(attr).is_none(), "Duplicate `#[estree(...)]` attribute.");
        }
        if let Some(attr) = try_parse_tsify_type(attr)? {
            assert!(
                tsify_type.replace(attr).is_none(),
                "Duplicate `#[tsify(type = \"...\")]` attribute."
            );
        }
    }
    Ok(DeriveAttributes {
        clone_in: clone_in.unwrap_or_default(),
        estree: estree.unwrap_or_default(),
        tsify_type,
    })
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

pub fn get_estree_attribute<'a, T, I>(attrs: I) -> Option<crate::Result<T>>
where
    I: IntoIterator<Item = &'a Attribute>,
    T: Parse,
{
    let attr = attrs.into_iter().find(|it| it.path().is_ident("estree"));
    attr.map(|attr| {
        debug_assert!(attr.path().is_ident("estree"));
        attr.parse_args_with(T::parse).normalize()
    })
}
