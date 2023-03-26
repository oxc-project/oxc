use std::collections::HashMap;
use std::iter::{self, Peekable, Rev};

use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Iter;
use syn::{PathSegment, Result};

#[derive(Clone)]
pub struct LintRuleMeta {
    name: syn::Ident,
    path: syn::Path,
}

impl LintRuleMeta {
    pub fn mod_stmt(path_and_modules: (&String, &Vec<LintRuleMeta>)) -> TokenStream {
        let (_, rule_meta_list) = path_and_modules;

        let mut stmts = TokenStream::new();
        let mut segments_opt: Option<Peekable<Rev<syn::punctuated::Iter<'_, PathSegment>>>> = None;

        // Modules are under the same path. List them all first.
        for rule_meta in rule_meta_list {
            let mut segments = rule_meta.path.segments.iter().rev().peekable();
            let first = &segments.next().unwrap().ident;
            let mut stmt = quote! {mod #first;};
            if segments.peek().is_some() {
                stmt = quote! {pub #stmt};
            }

            stmts = quote! {
                #stmts
                #stmt
            };

            segments_opt = Some(segments);
        }

        // Add path module declarations as needed
        match segments_opt {
            Some(mut segments) => {
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
            }
            None => {}
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
        Ok(Self { name, path })
    }
}

pub struct AllLintRulesMeta {
    rules: Vec<LintRuleMeta>,
    path_to_modules: HashMap<String, Vec<LintRuleMeta>>,
}

impl Parse for AllLintRulesMeta {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let rules: Vec<LintRuleMeta> = input
            .parse_terminated::<LintRuleMeta, syn::Token![,]>(LintRuleMeta::parse)?
            .into_iter()
            .collect();

        // Group by first ident of path
        let mut path_to_modules: HashMap<String, Vec<LintRuleMeta>> = HashMap::new();
        for rule in &rules {
            // Should try to upstream a hash for path/segments or implement something, rather than do this hot garbage
            let key = rule
                .path
                .segments
                .iter()
                .rev()
                .skip(1)
                .map(|seg| seg.ident.to_string())
                .intersperse("::".to_string())
                .collect::<String>();

            path_to_modules.entry(key).or_insert_with(Vec::new).push(rule.clone());
        }

        Ok(Self { rules, path_to_modules })
    }
}

#[allow(clippy::cognitive_complexity)]
pub fn declare_all_lint_rules(metadata: AllLintRulesMeta) -> TokenStream {
    let AllLintRulesMeta { rules, path_to_modules } = metadata;

    let mod_stmts = path_to_modules.iter().map(LintRuleMeta::mod_stmt);
    let use_stmts = rules.iter().map(LintRuleMeta::use_stmt);
    let struct_names = rules.iter().map(|rule| &rule.name).collect::<Vec<_>>();

    quote! {
        #(#mod_stmts)*
        #(#use_stmts)*

        use crate::{context::LintContext, rule::{Rule, RuleCategory}, rule::RuleMeta, AstNode};
        use oxc_semantic::Symbol;

        #[derive(Debug, Clone)]
        #[allow(clippy::enum_variant_names)]
        pub enum RuleEnum {
            #(#struct_names(#struct_names)),*
        }

        impl RuleEnum {
            pub fn name(&self) -> &'static str {
                match self {
                    #(Self::#struct_names(_) => #struct_names::NAME),*
                }
            }

            pub fn category(&self) -> RuleCategory {
                match self {
                    #(Self::#struct_names(_) => #struct_names::CATEGORY),*
                }
            }

            pub fn read_json(&self, maybe_value: Option<serde_json::Value>) -> Self {
                match self {
                    #(Self::#struct_names(_) => Self::#struct_names(
                        maybe_value.map(#struct_names::from_configuration).unwrap_or_default(),
                    )),*
                }
            }

            pub fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
                match self {
                    #(Self::#struct_names(rule) => rule.run(node, ctx)),*
                }
            }

            pub fn run_on_symbol<'a>(&self, symbol: &Symbol, ctx: &LintContext<'a>) {
              match self {
                #(Self::#struct_names(rule) => rule.run_on_symbol(symbol, ctx)),*
              }
            }
        }

        lazy_static::lazy_static! {
            pub static ref RULES: Vec<RuleEnum> = vec![
                #(RuleEnum::#struct_names(#struct_names::default())),*
            ];
        }
    }
}
