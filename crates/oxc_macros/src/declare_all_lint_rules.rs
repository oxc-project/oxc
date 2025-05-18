use convert_case::{Case, Casing};
use itertools::Itertools as _;
use proc_macro::TokenStream;
use proc_macro::{Ident, Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    Result,
    parse::{Parse, ParseStream},
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
            &path.segments.iter().next_back().unwrap().ident.to_string().to_case(Case::Pascal),
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

//use heck::ToCase;
//#[expect(clippy::cognitive_complexity, clippy::too_many_lines)]
//use proc_macro::TokenStream;
//use quote::{format_ident, quote};

pub fn declare_all_lint_rules(metadata: AllLintRulesMeta) -> TokenStream {
    let AllLintRulesMeta { rules } = metadata;

    let mut use_stmts = Vec::new();
    let mut rules_map_fields = Vec::new();
    let mut enum_variants = Vec::new();
    let mut enum_id_matches = Vec::new();
    let mut enum_name_matches = Vec::new();
    let mut enum_category_matches = Vec::new();
    let mut enum_fix_matches = Vec::new();
    let mut enum_doc_matches = Vec::new();
    let mut enum_schema_matches = Vec::new();
    let mut enum_plugin_matches = Vec::new();
    let mut enum_read_json_matches = Vec::new();
    let mut enum_run_matches = Vec::new();
    let mut enum_run_symbol_matches = Vec::new();
    let mut enum_run_once_matches = Vec::new();
    let mut enum_run_jest_matches = Vec::new();
    let mut enum_should_run_matches = Vec::new();
    let mut static_rules = Vec::new();

    let mut id_counter: usize = 0;

    for rule in &rules {
        let plugin_name = rule
            .path
            .segments
            .iter()
            .take(rule.path.segments.len() - 1)
            .map(|s| format!("{}", s.ident))
            .join("/");

        if plugin_name == "test" {
            // Handle test rules with const generics
            let rule_path = &rule.path;
            let rule_rule_name = &rule.rule_name;
            let rule_enum_name = &rule.enum_name;

            // Add use statement for the base rule
            let struct_name = &rule.enum_name;
            use_stmts.push(quote! {
                pub use self::#rule_path::#rule_rule_name as #struct_name;
            });

            // Create variants for both Jest and Vitest
            for (framework, framework_str) in [("Jest", "jest"), ("Vitest", "vitest")] {
                let variant_name = format_ident!("{}{}", framework, rule_enum_name);
                let framework_type = format_ident!("TestFramework{}", framework); // "Jest" -> Jest
                let struct_type = quote! { #rule_enum_name<#framework_type> };
                //let struct_type = quote! { #rule_enum_name<#framework_str> };
                let id = id_counter;
                id_counter += 1;

                let rename_str = format!(
                    "{}/{}",
                    framework_str,
                    rule.rule_name.to_string().to_case(Case::Kebab)
                );

                rules_map_fields.push(quote! {
                    #[serde(rename = #rename_str)]
                    #variant_name: Option<RuleConfig>,
                });

                // Add enum variant
                enum_variants.push(quote! {
                    #variant_name(#struct_type)
                });

                // Add match arms for all methods
                enum_id_matches.push(quote! {
                    Self::#variant_name(_) => #id
                });

                enum_name_matches.push(quote! {
                    Self::#variant_name(_) => <#struct_type>::NAME
                });

                enum_category_matches.push(quote! {
                    Self::#variant_name(_) => <#struct_type>::CATEGORY
                });

                enum_fix_matches.push(quote! {
                    Self::#variant_name(_) => <#struct_type>::FIX
                });

                enum_doc_matches.push(quote! {
                    Self::#variant_name(_) => <#struct_type>::documentation()
                });

                enum_schema_matches.push(quote! {
                    Self::#variant_name(_) => <#struct_type>::config_schema(generator).or_else(|| <#struct_type>::schema(generator))
                });

                enum_plugin_matches.push(quote! {
                    Self::#variant_name(_) => #framework_str
                });

                enum_read_json_matches.push(quote! {
                    Self::#variant_name(_) => Self::#variant_name(<#struct_type>::from_configuration(value))
                });

                enum_run_matches.push(quote! {
                    Self::#variant_name(rule) => rule.run(node, ctx)
                });

                enum_run_symbol_matches.push(quote! {
                    Self::#variant_name(rule) => rule.run_on_symbol(symbol_id, ctx)
                });

                enum_run_once_matches.push(quote! {
                    Self::#variant_name(rule) => rule.run_once(ctx)
                });

                enum_run_jest_matches.push(quote! {
                    Self::#variant_name(rule) => rule.run_on_jest_node(jest_node, ctx)
                });

                enum_should_run_matches.push(quote! {
                    Self::#variant_name(rule) => rule.should_run(ctx)
                });

                // Add to static RULES
                static_rules.push(quote! {
                    RuleEnum::#variant_name(<#struct_type>::default())
                });
            }
        } else {
            // Handle regular rules
            let rule_path = &rule.path;
            let rule_rule_name = &rule.rule_name;
            let struct_name = &rule.enum_name;
            let id = id_counter;
            id_counter += 1;

            use_stmts.push(quote! {
                pub use self::#rule_path::#rule_rule_name as #struct_name;
            });

            let rename_str =
                format!("{}/{}", &plugin_name, rule.rule_name.to_string().to_case(Case::Kebab));

            if plugin_name == "eslint" {
                let alias_str = rule.rule_name.to_string().to_case(Case::Kebab);
                rules_map_fields.push(quote! {
                    #[serde(rename = #rename_str, alias = #alias_str)]
                    #struct_name: Option<RuleConfig>,
                });
            } else {
                rules_map_fields.push(quote! {
                    #[serde(rename = #rename_str)]
                    #struct_name: Option<RuleConfig>,
                });
            }

            // Add enum variant
            enum_variants.push(quote! {
                #struct_name(#struct_name)
            });

            // Add match arms
            enum_id_matches.push(quote! {
                Self::#struct_name(_) => #id
            });

            enum_name_matches.push(quote! {
                Self::#struct_name(_) => #struct_name::NAME
            });

            enum_category_matches.push(quote! {
                Self::#struct_name(_) => #struct_name::CATEGORY
            });

            enum_fix_matches.push(quote! {
                Self::#struct_name(_) => #struct_name::FIX
            });

            enum_doc_matches.push(quote! {
                Self::#struct_name(_) => #struct_name::documentation()
            });

            enum_schema_matches.push(quote! {
                Self::#struct_name(_) => #struct_name::config_schema(generator).or_else(|| #struct_name::schema(generator))
            });

            enum_plugin_matches.push(quote! {
                Self::#struct_name(_) => #plugin_name
            });

            enum_read_json_matches.push(quote! {
                Self::#struct_name(_) => Self::#struct_name(#struct_name::from_configuration(value))
            });

            enum_run_matches.push(quote! {
                Self::#struct_name(rule) => rule.run(node, ctx)
            });

            enum_run_symbol_matches.push(quote! {
                Self::#struct_name(rule) => rule.run_on_symbol(symbol_id, ctx)
            });

            enum_run_once_matches.push(quote! {
                Self::#struct_name(rule) => rule.run_once(ctx)
            });

            enum_run_jest_matches.push(quote! {
                Self::#struct_name(rule) => rule.run_on_jest_node(jest_node, ctx)
            });

            enum_should_run_matches.push(quote! {
                Self::#struct_name(rule) => rule.should_run(ctx)
            });

            // Add to static RULES
            static_rules.push(quote! {
                RuleEnum::#struct_name(#struct_name::default())
            });
        }
    }

    let expanded = quote! {
        #(#use_stmts)*

        use crate::{
            context::{ContextHost, LintContext},
            rule::{Rule, RuleCategory, RuleFixMeta, RuleMeta, RuleConfig},
            utils::PossibleJestNode,
            AstNode
        };
        use oxc_semantic::SymbolId;

        #[derive(Debug, Clone)]
        #[expect(clippy::enum_variant_names)]
        pub enum RuleEnum {
            #(#enum_variants),*
        }

        impl RuleEnum {
            pub fn id(&self) -> usize {
                match self {
                    #(#enum_id_matches),*
                }
            }

            pub fn name(&self) -> &'static str {
                match self {
                    #(#enum_name_matches),*
                }
            }

            pub fn category(&self) -> RuleCategory {
                match self {
                    #(#enum_category_matches),*
                }
            }

            /// This [`Rule`]'s auto-fix capabilities.
            pub fn fix(&self) -> RuleFixMeta {
                match self {
                    #(#enum_fix_matches),*
                }
            }

            #[cfg(feature = "ruledocs")]
            pub fn documentation(&self) -> Option<&'static str> {
                match self {
                    #(#enum_doc_matches),*
                }
            }

            #[cfg(feature = "ruledocs")]
            pub fn schema(&self, generator: &mut schemars::SchemaGenerator) -> Option<schemars::schema::Schema> {
                match self {
                    #(#enum_schema_matches),*
                }
            }

            pub fn plugin_name(&self) -> &'static str {
                match self {
                    #(#enum_plugin_matches),*
                }
            }

            pub fn read_json(&self, value: serde_json::Value) -> Self {
                match self {
                    #(#enum_read_json_matches),*
                }
            }

            pub(super) fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
                match self {
                    #(#enum_run_matches),*
                }
            }

            pub(super) fn run_on_symbol<'a>(&self, symbol_id: SymbolId, ctx: &LintContext<'a>) {
                match self {
                    #(#enum_run_symbol_matches),*
                }
            }

            pub(super) fn run_once<'a>(&self, ctx: &LintContext<'a>) {
                match self {
                    #(#enum_run_once_matches),*
                }
            }

            pub(super) fn run_on_jest_node<'a, 'c>(
                &self,
                jest_node: &PossibleJestNode<'a, 'c>,
                ctx: &'c LintContext<'a>,
            ) {
                match self {
                    #(#enum_run_jest_matches),*
                }
            }

            pub(super) fn should_run(&self, ctx: &ContextHost) -> bool {
                match self {
                    #(#enum_should_run_matches),*
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
                #(#static_rules),*
            ];
        }

        #[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
        /// See [Oxlint Rules](https://oxc.rs/docs/guide/usage/linter/rules.html)
        pub struct RulesMap {
            #(#rules_map_fields)*
        }
    };

    TokenStream::from(expanded)
}
