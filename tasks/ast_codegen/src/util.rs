use itertools::Itertools;
use proc_macro2::{Group, TokenStream, TokenTree};
use quote::{quote, ToTokens};
use syn::{GenericArgument, Ident, PathArguments, Type, TypePath};

pub trait TokenStreamExt {
    fn replace_ident(self, needle: &str, replace: &Ident) -> TokenStream;
}

pub trait TypeExt {
    fn get_ident(&self) -> TypeIdentResult;
}

pub trait StrExt: AsRef<str> {
    /// Dead simple, just adds either `s` or `es` based on the last character.
    /// doesn't handle things like `sh`, `x`, `z`, etc. It also creates wrong results when the word
    /// ends with `y` but there is a preceding vowl similar to `toys`,
    /// It WILL output the WRONG result `toies`!
    /// As an edge case would output `children` for the input `child`.
    fn to_plural(self) -> String;
}

#[derive(Debug)]
pub enum TypeIdentResult<'a> {
    Ident(&'a Ident),
    Vec(Box<TypeIdentResult<'a>>),
    Box(Box<TypeIdentResult<'a>>),
    Option(Box<TypeIdentResult<'a>>),
    Reference(Box<TypeIdentResult<'a>>),
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

    fn reference(inner: Self) -> Self {
        Self::Reference(Box::new(inner))
    }

    pub fn inner_ident(&self) -> &'a Ident {
        match self {
            Self::Ident(it) => it,
            Self::Vec(it) | Self::Box(it) | Self::Option(it) | Self::Reference(it) => {
                it.inner_ident()
            }
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
                                        inner
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
