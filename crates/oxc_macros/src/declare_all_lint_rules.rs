use convert_case::{Case, Casing};
use itertools::Itertools as _;
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    Result,
};

pub struct LintRuleMeta {
    rule_name: syn::Ident,
    enum_name: syn::Ident,
    path: syn::Path,
}

impl Parse for LintRuleMeta {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let path = input.parse::<syn::Path>()?;

        let segments = &path.segments;
        let combined = segments
            .iter()
            .rev()
            .take(2)
            .rev()
            .map(|seg| seg.ident.to_string().to_case(Case::Pascal))
            .join("");

        let combined = combined.to_case(Case::Pascal);

        let enum_name = syn::parse_str(&combined)?;
        let rule_name = syn::parse_str(
            &path.segments.iter().last().unwrap().ident.to_string().to_case(Case::Pascal),
        )?;
        Ok(Self { rule_name, enum_name, path })
    }
}

pub struct AllLintRulesMeta {
    rules: Vec<LintRuleMeta>,
}

impl Parse for AllLintRulesMeta {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let rules =
            input.parse_terminated(LintRuleMeta::parse, syn::Token![,])?.into_iter().collect();
        Ok(Self { rules })
    }
}

#[allow(clippy::cognitive_complexity, clippy::too_many_lines)]
pub fn declare_all_lint_rules(metadata: AllLintRulesMeta) -> TokenStream {
    let AllLintRulesMeta { rules } = metadata;

    let mut use_stmts = Vec::with_capacity(rules.len());
    let mut struct_names = Vec::with_capacity(rules.len());
    let mut struct_rule_names = Vec::with_capacity(rules.len());
    let mut plugin_names = Vec::with_capacity(rules.len());
    let mut ids = Vec::with_capacity(rules.len());

    for (i, rule) in rules.iter().enumerate() {
        use_stmts.push(&rule.path);
        struct_names.push(&rule.enum_name);
        struct_rule_names.push(&rule.rule_name);
        plugin_names.push(
            rule.path
                .segments
                .iter()
                .take(rule.path.segments.len() - 1)
                .map(|s| format!("{}", s.ident))
                .join("/"),
        );
        ids.push(i);
    }

    let expanded = quote! {
        #(pub use self::#use_stmts::#struct_rule_names as #struct_names;)*

        use crate::{
            context::{ContextHost, LintContext},
            rule::{Rule, RuleCategory, RuleFixMeta, RuleMeta},
            utils::PossibleJestNode,
            AstNode
        };
        use oxc_semantic::SymbolId;

        #[derive(Debug, Clone)]
        #[allow(clippy::enum_variant_names)]
        pub enum RuleEnum {
            #(#struct_names(#struct_names)),*
        }

        impl RuleEnum {
            pub fn id(&self) -> usize {
                match self {
                    #(Self::#struct_names(_) => #ids),*
                }
            }

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

            /// This [`Rule`]'s auto-fix capabilities.
            pub fn fix(&self) -> RuleFixMeta {
                match self {
                    #(Self::#struct_names(_) => #struct_names::FIX),*
                }
            }

            pub fn documentation(&self) -> Option<&'static str> {
                match self {
                    #(Self::#struct_names(_) => #struct_names::documentation()),*
                }
            }

            pub fn plugin_name(&self) -> &'static str {
                match self {
                    #(Self::#struct_names(_) => #plugin_names),*
                }
            }

            pub fn read_json(&self, value: serde_json::Value) -> Self {
                match self {
                    #(Self::#struct_names(_) => Self::#struct_names(
                        #struct_names::from_configuration(value),
                    )),*
                }
            }

            pub(super) fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
                match self {
                    #(Self::#struct_names(rule) => rule.run(node, ctx)),*
                }
            }

            pub(super) fn run_on_symbol<'a>(&self, symbol_id: SymbolId, ctx: &LintContext<'a>) {
                match self {
                    #(Self::#struct_names(rule) => rule.run_on_symbol(symbol_id, ctx)),*
                }
            }

            pub(super) fn run_once<'a>(&self, ctx: &LintContext<'a>) {
                match self {
                    #(Self::#struct_names(rule) => rule.run_once(ctx)),*
                }
            }

            pub(super) fn run_on_jest_node<'a, 'c>(
                &self,
                jest_node: &PossibleJestNode<'a, 'c>,
                ctx: &'c LintContext<'a>,
            ) {
                match self {
                    #(Self::#struct_names(rule) => rule.run_on_jest_node(jest_node, ctx)),*
                }
            }

            pub(super) fn should_run(&self, ctx: &ContextHost) -> bool {
                match self {
                    #(Self::#struct_names(rule) => rule.should_run(ctx)),*
                }
            }
        }

        impl std::hash::Hash for RuleEnum {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                self.id().hash(state);
            }
        }

        impl PartialEq for RuleEnum {
            fn eq(&self, other: &Self) -> bool {
                self.id() == other.id()
            }
        }

        impl Eq for RuleEnum {}

        impl Ord for RuleEnum {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                self.id().cmp(&other.id())
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
    };

    TokenStream::from(expanded)
}
