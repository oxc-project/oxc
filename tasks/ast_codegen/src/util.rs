use itertools::Itertools;
use proc_macro2::{Group, TokenStream, TokenTree};
use quote::{format_ident, ToTokens};
use serde::Serialize;
use syn::{spanned::Spanned, GenericArgument, Ident, ItemMacro, PathArguments, Type, TypePath};

use crate::{EarlyCtx, TypeId};

pub trait NormalizeError<T> {
    fn normalize(self) -> crate::Result<T>;
    fn normalize_with<E>(self, err: E) -> crate::Result<T>
    where
        E: ToString;
}

impl<T, E> NormalizeError<T> for Result<T, E>
where
    E: ToString,
{
    fn normalize(self) -> crate::Result<T> {
        self.map_err(|e| e.to_string())
    }

    fn normalize_with<U>(self, err: U) -> crate::Result<T>
    where
        U: ToString,
    {
        self.map_err(|_| err.to_string())
    }
}

impl<T> NormalizeError<T> for Option<T> {
    fn normalize(self) -> crate::Result<T> {
        self.normalize_with(String::default())
    }

    fn normalize_with<E>(self, err: E) -> crate::Result<T>
    where
        E: ToString,
    {
        self.map_or_else(|| Err(err.to_string()), |r| Ok(r))
    }
}

pub trait TokenStreamExt {
    fn replace_ident(self, needle: &str, replace: &Ident) -> TokenStream;
}

pub trait TypeExt {
    fn get_ident(&self) -> TypeIdentResult;
    fn analyze(&self, ctx: &EarlyCtx) -> TypeAnalysis;
}

pub trait StrExt: AsRef<str> {
    /// Dead simple, just adds either `s` or `es` based on the last character.
    /// doesn't handle things like `sh`, `x`, `z`, etc. It also creates wrong results when the word
    /// ends with `y` but there is a preceding vowl similar to `toys`,
    /// It WILL output the WRONG result `toies`!
    /// As an edge case would output `children` for the input `child`.
    fn to_plural(self) -> String;
}

pub trait ToIdent {
    fn to_ident(&self) -> Ident;
}

#[derive(Debug)]
pub enum TypeIdentResult<'a> {
    Ident(&'a Ident),
    Vec(Box<TypeIdentResult<'a>>),
    Box(Box<TypeIdentResult<'a>>),
    Option(Box<TypeIdentResult<'a>>),
    Reference(Box<TypeIdentResult<'a>>),
    /// We bailed on detecting wrapper
    Complex(Box<TypeIdentResult<'a>>),
}

impl<'a> TypeIdentResult<'a> {
    fn boxed(inner: Self) -> Self {
        Self::Box(Box::new(inner))
    }

    fn vec(inner: Self) -> Self {
        Self::Vec(Box::new(inner))
    }

    fn option(inner: Self) -> Self {
        Self::Option(Box::new(inner))
    }

    fn complex(inner: Self) -> Self {
        Self::Complex(Box::new(inner))
    }

    fn reference(inner: Self) -> Self {
        Self::Reference(Box::new(inner))
    }

    pub fn inner_ident(&self) -> &'a Ident {
        match self {
            Self::Ident(it) => it,
            Self::Complex(it)
            | Self::Vec(it)
            | Self::Box(it)
            | Self::Option(it)
            | Self::Reference(it) => it.inner_ident(),
        }
    }

    pub fn as_ident(&self) -> Option<&'a Ident> {
        if let Self::Ident(it) = self {
            Some(it)
        } else {
            None
        }
    }
}

// TODO: remove me
#[allow(dead_code)]
#[derive(Debug, PartialEq, Clone, Serialize)]
pub enum TypeWrapper {
    None,
    Box,
    Vec,
    Opt,
    VecBox,
    VecOpt,
    OptBox,
    OptVec,
    Ref,
    /// We bailed on detecting the type wrapper
    Complex,
}

#[derive(Debug, Clone, Serialize)]
pub struct TypeAnalysis {
    pub type_id: Option<TypeId>,
    pub wrapper: TypeWrapper,
    // pub name: String,
    #[serde(skip)]
    pub typ: Type,
}

impl TypeExt for Type {
    fn get_ident(&self) -> TypeIdentResult {
        match self {
            Type::Path(TypePath { path, .. }) => {
                let seg1 = path.segments.first().unwrap();
                match &seg1.arguments {
                    PathArguments::None => TypeIdentResult::Ident(&seg1.ident),
                    PathArguments::AngleBracketed(it) => {
                        let args = &it.args.iter().collect_vec();
                        assert!(args.len() < 3, "Max path arguments here is 2, eg `Box<'a, Adt>`");
                        if let Some(second) = args.get(1) {
                            let GenericArgument::Type(second) = second else { panic!() };
                            let inner = second.get_ident();
                            if seg1.ident == "Box" {
                                TypeIdentResult::boxed(inner)
                            } else if seg1.ident == "Vec" {
                                TypeIdentResult::vec(inner)
                            } else {
                                panic!();
                            }
                        } else {
                            match args.first() {
                                Some(GenericArgument::Type(it)) => {
                                    let inner = it.get_ident();
                                    if seg1.ident == "Option" {
                                        TypeIdentResult::option(inner)
                                    } else {
                                        TypeIdentResult::complex(inner)
                                    }
                                }
                                Some(GenericArgument::Lifetime(_)) => {
                                    TypeIdentResult::Ident(&seg1.ident)
                                }
                                _ => panic!("unsupported type!"),
                            }
                        }
                    }
                    PathArguments::Parenthesized(_) => {
                        panic!("Parenthesized path arguments aren't supported!")
                    }
                }
            }
            Type::Reference(typ) => TypeIdentResult::reference(typ.elem.get_ident()),
            _ => panic!("Unsupported type."),
        }
    }

    fn analyze(&self, ctx: &EarlyCtx) -> TypeAnalysis {
        fn analyze<'a>(res: &'a TypeIdentResult) -> Option<(&'a Ident, TypeWrapper)> {
            let mut wrapper = TypeWrapper::None;
            let ident = match res {
                TypeIdentResult::Ident(inner) => inner,
                TypeIdentResult::Complex(inner) => {
                    wrapper = TypeWrapper::Complex;
                    let (inner, _) = analyze(inner)?;
                    inner
                }
                TypeIdentResult::Box(inner) => {
                    wrapper = TypeWrapper::Box;
                    let (inner, inner_kind) = analyze(inner)?;
                    assert!(inner_kind == TypeWrapper::None,);
                    inner
                }
                TypeIdentResult::Vec(inner) => {
                    wrapper = TypeWrapper::Vec;
                    let (inner, inner_kind) = analyze(inner)?;
                    if inner_kind == TypeWrapper::Opt {
                        wrapper = TypeWrapper::VecOpt;
                    } else if inner_kind != TypeWrapper::None {
                        panic!();
                    }
                    inner
                }
                TypeIdentResult::Option(inner) => {
                    wrapper = TypeWrapper::Opt;
                    let (inner, inner_kind) = analyze(inner)?;
                    if inner_kind == TypeWrapper::Vec {
                        wrapper = TypeWrapper::OptVec;
                    } else if inner_kind == TypeWrapper::Box {
                        wrapper = TypeWrapper::OptBox;
                    } else if inner_kind != TypeWrapper::None {
                        panic!();
                    }
                    inner
                }
                TypeIdentResult::Reference(_) => return None,
            };
            Some((ident, wrapper))
        }
        let type_ident = self.get_ident();
        let Some((type_ident, wrapper)) = analyze(&type_ident) else {
            return TypeAnalysis { type_id: None, wrapper: TypeWrapper::Ref, typ: self.clone() };
        };

        let type_id = ctx.type_id(&type_ident.to_string());
        TypeAnalysis { type_id, wrapper, typ: self.clone() }
    }
}

impl<T: AsRef<str>> StrExt for T {
    fn to_plural(self) -> String {
        let txt = self.as_ref();
        if txt.is_empty() {
            return String::default();
        }

        let mut txt = txt.to_string();
        if txt.ends_with("child") {
            txt.push_str("ren");
        } else {
            match txt.chars().last() {
                Some('s') => {
                    txt.push_str("es");
                }
                Some('y') => {
                    txt.pop();
                    txt.push_str("ies");
                }
                _ => txt.push('s'),
            }
        }
        txt
    }
}

impl TokenStreamExt for TokenStream {
    fn replace_ident(self, needle: &str, replace: &Ident) -> TokenStream {
        self.into_iter()
            .map(|it| match it {
                TokenTree::Ident(ident) if ident == needle => replace.to_token_stream(),
                TokenTree::Group(group) => {
                    Group::new(group.delimiter(), group.stream().replace_ident(needle, replace))
                        .to_token_stream()
                }
                _ => it.to_token_stream(),
            })
            .collect()
    }
}

impl<S> ToIdent for S
where
    S: AsRef<str>,
{
    fn to_ident(&self) -> Ident {
        format_ident!("{}", self.as_ref())
    }
}

pub fn write_all_to<S: AsRef<std::path::Path>>(data: &[u8], path: S) -> std::io::Result<()> {
    use std::{fs, io::Write};
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = fs::File::create(path)?;
    file.write_all(data)?;
    Ok(())
}

pub fn unexpanded_macro_err(mac: &ItemMacro) -> String {
    format!("Unexpanded macro: {:?}:{:?}", mac.ident, mac.span())
}
