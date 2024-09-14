use std::mem;

use proc_macro2::{Delimiter, Group, Ident, Literal, Punct, Spacing, TokenStream, TokenTree};
use quote::quote;
use syn::Lit;

/// Similar to `quote!` macro, except evaluates to a `TokenStream` of code to *create* the `TokenStream`
/// which `quote!` evaluates to.
///
/// `code!( foo() )` evaluates to a `TokenStream` containing:
///
/// ```
/// TokenStream::from_iter([
///     TokenTree::Ident(Ident::new("foo", Span::call_site())),
///     TokenTree::Group(Group::new(Delimiter::Parenthesis, TokenStream::new()))
/// ].into_iter())
/// ```
///
/// `@{...}` can be used to inject code into output without conversion.
/// The value in `@{...}` must be a `TokenStream`.
/// e.g. `code!( foo( @{args} ) )` evaluates to a `TokenStream` containing:
///
/// ```
/// TokenStream::from_iter([
///     TokenTree::Ident(Ident::new("foo", Span::call_site())),
///     TokenTree::Group(Group::new(Delimiter::Parenthesis, args))
/// ].into_iter())
/// ```
macro_rules! code {
    ($($tt:tt)*) => {
        $crate::to_code::to_code(quote!($($tt)*))
    }
}
pub(crate) use code;

/// Convert `TokenStream` into Rust code which creates that `TokenStream`
pub fn to_code(stream: TokenStream) -> TokenStream {
    stream.to_code()
}

/// Trait for converting `proc_macro`/`proc_macro2` types into Rust code which creates those types
trait ToCode {
    fn to_code(self) -> TokenStream;
}

impl ToCode for TokenStream {
    fn to_code(self) -> TokenStream {
        let mut stream = TokenStream::new();
        let mut trees = vec![];

        let extend = |stream: &mut TokenStream, extend_stream| {
            if stream.is_empty() {
                *stream = extend_stream;
            } else {
                stream.extend(quote! { .chain(#extend_stream) });
            }
        };

        let mut it = self.into_iter();
        while let Some(tt) = it.next() {
            // Leave contents of `@{...}` unchanged
            if let TokenTree::Punct(punct) = &tt {
                if punct.as_char() == '@' {
                    if let Some(TokenTree::Group(group)) = it.clone().next() {
                        if group.delimiter() == Delimiter::Brace {
                            it.next().unwrap();

                            if !trees.is_empty() {
                                let trees = mem::take(&mut trees);
                                extend(&mut stream, quote! { [ #(#trees),* ].into_iter() });
                            }

                            let extend_stream = group.stream();
                            extend(&mut stream, quote! { #extend_stream.into_iter() });
                            continue;
                        }
                    }
                }
            }

            trees.push(tt.to_code());
        }

        if trees.is_empty() {
            if stream.is_empty() {
                return quote! { TokenStream::new() };
            }
        } else {
            if stream.is_empty() && trees.len() == 1 {
                let tree = trees.into_iter().next().unwrap();
                return quote! { TokenStream::from(#tree) };
            }

            extend(&mut stream, quote! { [ #(#trees),* ].into_iter() });
        }

        quote! { #stream.collect() }
    }
}

impl ToCode for TokenTree {
    fn to_code(self) -> TokenStream {
        match self {
            TokenTree::Ident(ident) => {
                let ident = ident.to_code();
                quote! { TokenTree::Ident(#ident) }
            }
            TokenTree::Punct(punct) => {
                let punct = punct.to_code();
                quote! { TokenTree::Punct(#punct) }
            }
            TokenTree::Literal(literal) => {
                let literal = literal.to_code();
                quote! { TokenTree::Literal(#literal) }
            }
            TokenTree::Group(group) => {
                let group = group.to_code();
                quote! { TokenTree::Group(#group) }
            }
        }
    }
}

impl ToCode for Ident {
    fn to_code(self) -> TokenStream {
        let name = self.to_string();
        quote! { Ident::new(#name, Span::call_site()) }
    }
}

impl ToCode for Punct {
    fn to_code(self) -> TokenStream {
        let ch = self.as_char();
        let spacing = self.spacing().to_code();
        quote! { Punct::new(#ch, #spacing) }
    }
}

#[expect(clippy::todo)]
impl ToCode for Literal {
    fn to_code(self) -> TokenStream {
        let lit: Lit = syn::parse_str(&self.to_string()).unwrap();
        match lit {
            Lit::Str(str) => {
                let str = str.value();
                quote! { Literal::string(#str) }
            }
            Lit::Int(int) => match int.suffix() {
                "u8" => {
                    let n = int.base10_parse::<u8>().unwrap();
                    quote! { Literal::u8_suffixed(#n) }
                }
                // TODO: Other `Int` types
                _ => todo!(),
            },
            Lit::Bool(_) => unreachable!(), // `true` and `false` are `Ident`s
            // TODO: Other `Lit` types
            _ => todo!(),
        }
    }
}

impl ToCode for Group {
    fn to_code(self) -> TokenStream {
        let delimiter = self.delimiter().to_code();
        let stream = self.stream().to_code();
        quote! { Group::new(#delimiter, #stream) }
    }
}

impl ToCode for Delimiter {
    fn to_code(self) -> TokenStream {
        match self {
            Self::Parenthesis => quote!(Delimiter::Parenthesis),
            Self::Brace => quote!(Delimiter::Brace),
            Self::Bracket => quote!(Delimiter::Bracket),
            Self::None => quote!(Delimiter::None),
        }
    }
}

impl ToCode for Spacing {
    fn to_code(self) -> TokenStream {
        match self {
            Self::Joint => quote!(Spacing::Joint),
            Self::Alone => quote!(Spacing::Alone),
        }
    }
}
