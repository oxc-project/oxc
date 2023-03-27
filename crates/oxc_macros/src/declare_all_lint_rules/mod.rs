mod trie;

use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::Result;
use trie::RulePathTrieBuilder;

pub struct LintRuleMeta {
    name: syn::Ident,
    path: syn::Path,
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

#[allow(clippy::cognitive_complexity)]
pub fn declare_all_lint_rules(metadata: AllLintRulesMeta) -> TokenStream {
    let AllLintRulesMeta { rules } = metadata;
    // all the top-level module trees
    let module_tries = {
        let mut builder = RulePathTrieBuilder::new();
        for rule in &rules {
            builder.push(rule);
        }
        builder.finish()
    };
    let mod_stmts = module_tries.iter().map(|node| node.mod_stmt(true));
    let use_stmts = module_tries.iter().map(|node| node.use_stmt(true));
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

        impl std::hash::Hash for RuleEnum {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                self.name().hash(state);
            }
        }

        impl PartialEq for RuleEnum {
            fn eq(&self, other: &Self) -> bool {
                self.name() == other.name()
            }
        }

        impl Eq for RuleEnum {}

        impl Ord for RuleEnum {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                self.name().cmp(&other.name())
            }
        }

        impl PartialOrd for RuleEnum {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(&other))
            }
        }

        lazy_static::lazy_static! {
            pub static ref RULES: Vec<RuleEnum> = vec![
                #(RuleEnum::#struct_names(#struct_names::default())),*
            ];
        }
    }
}
