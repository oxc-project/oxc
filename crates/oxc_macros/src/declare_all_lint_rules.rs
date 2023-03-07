use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::Result;

pub struct LintRuleMeta {
    name: syn::Ident,
    path: syn::Path,
}

impl LintRuleMeta {
    pub fn mod_stmt(&self) -> TokenStream {
        let mut segments = self.path.segments.iter().rev().peekable();
        let first = &segments.next().unwrap().ident;
        let mut stmts = quote! {mod #first;};
        if segments.peek().is_some() {
            stmts = quote! {pub #stmts};
        }

        while let Some(segment) = segments.next() {
            let ident = &segment.ident;

            stmts = quote! {
                mod #ident { #stmts }
            };

            if segments.peek().is_some() {
                stmts = quote! {
                    pub #stmts
                };
            }
        }

        stmts
    }

    pub fn use_stmt(&self) -> TokenStream {
        let mut path = self.path.clone();
        path.segments.push(self.name.clone().into());

        quote! {
            pub use #path;
        }
    }
}

impl Parse for LintRuleMeta {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let path = input.parse::<syn::Path>()?;
        let name = syn::parse_str(
            &path.segments.iter().last().unwrap().ident.to_string().to_case(Case::Pascal),
        )
        .unwrap();
        Ok(Self { path, name })
    }
}

pub struct AllLintRulesMeta {
    rules: Vec<LintRuleMeta>,
}

impl Parse for AllLintRulesMeta {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let rules = input
            .parse_terminated::<LintRuleMeta, syn::Token![,]>(LintRuleMeta::parse)?
            .into_iter()
            .collect();

        Ok(Self { rules })
    }
}

pub fn declare_all_lint_rules(metadata: AllLintRulesMeta) -> TokenStream {
    let AllLintRulesMeta { rules } = metadata;

    let mod_stmts = rules.iter().map(|rule| rule.mod_stmt());
    let use_stmts = rules.iter().map(|rule| rule.use_stmt());
    let names = rules.iter().map(|rule| &rule.name).collect::<Vec<_>>();

    quote! {
        #(#mod_stmts)*
        #(#use_stmts)*

        use crate::{context::LintContext, rule::Rule, rule::RuleMeta, AstNode};

        #[derive(Debug, Clone)]
        #[allow(clippy::enum_variant_names)]
        pub enum RuleEnum {
            #(#names(#names)),*
        }

        impl RuleEnum {
            pub const fn name(&self) -> &'static str {
                match self {
                    #(Self::#names(_) => #names::NAME),*
                }
            }

            pub fn read_json(&self, maybe_value: Option<serde_json::Value>) -> Self {
                match self {
                    #(Self::#names(_) => Self::#names(
                        maybe_value.map(#names::from_configuration).unwrap_or_default(),
                    )),*
                }
            }

            pub fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
                match self {
                    #(Self::#names(rule) => rule.run(node, ctx)),*
                }
            }
        }

        lazy_static::lazy_static! {
            pub static ref RULES: Vec<RuleEnum> = vec![
                #(RuleEnum::#names(#names::default())),*
            ];
        }
    }
}
