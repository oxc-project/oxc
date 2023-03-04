use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::Result;

pub struct AllLintRulesMeta {
    paths: Vec<syn::Path>,
}

impl Parse for AllLintRulesMeta {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let paths = input
            .parse_terminated::<syn::Path, syn::Token![,]>(syn::Path::parse)?
            .into_iter()
            .collect();

        Ok(Self { paths })
    }
}

fn define_rule_mod(path: &syn::Path) -> TokenStream {
    let mut segments = path.segments.iter().rev().peekable();
    let first = &segments.next().unwrap().ident;
    let mut stmts = quote! {mod #first;};
    if segments.peek().is_some() {
        stmts = quote! {pub #stmts};
    }

    while let Some(segment) = segments.next() {
        let ident = &segment.ident;

        stmts = quote! {
            mod #ident {
                #stmts
            }
        };

        if segments.peek().is_some() {
            stmts = quote! {
                pub #stmts
            };
        }
    }

    stmts
}

pub fn declare_all_lint_rules(metadata: AllLintRulesMeta) -> TokenStream {
    let AllLintRulesMeta { paths } = metadata;
    let stmts = paths.iter().map(define_rule_mod).collect::<Vec<_>>();
    quote! { #(#stmts)* }
}
