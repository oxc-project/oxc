use convert_case::{Case, Casing};
use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;

use crate::rules::RuleEntry;

/// Generate the RuleEnum and related code that replaces `declare_all_lint_rules!` macro.
pub fn generate_rules_enum(rule_entries: &[RuleEntry<'_>]) -> String {
    let header = quote! {
        // Auto-generated code, DO NOT EDIT DIRECTLY!
        // To regenerate: `cargo run -p oxc_linter_codegen`

        #![expect(
            clippy::default_constructed_unit_structs,  // Many rules are unit structs
            clippy::semicolon_if_nothing_returned,     // Match arms in void-returning methods
            clippy::wrong_self_convention,             // from_configuration takes &self
            clippy::missing_errors_doc,                // Generated code
            clippy::match_same_arms,                   // plugin_name() has many same-body arms
        )]
    };

    let use_statements = generate_use_statements(rule_entries);
    let imports = generate_imports();
    let rule_enum = generate_rule_enum(rule_entries);
    let rule_enum_impl = generate_rule_enum_impl(rule_entries);
    let trait_impls = generate_trait_impls();
    let rules_static = generate_rules_static(rule_entries);

    let tokens = quote! {
        #header

        #use_statements

        #imports

        #rule_enum

        #rule_enum_impl

        #trait_impls

        #rules_static
    };

    tokens.to_string()
}

/// Create an identifier from a rule entry for the enum variant name.
/// e.g., `eslint::no_debugger` -> `EslintNoDebugger`
fn make_enum_ident(rule: &RuleEntry<'_>) -> Ident {
    let name = format!(
        "{}{}",
        rule.plugin_module_name.to_case(Case::Pascal),
        rule.rule_module_name.to_case(Case::Pascal)
    );
    Ident::new(&name, Span::call_site())
}

fn generate_use_statements(rule_entries: &[RuleEntry<'_>]) -> TokenStream {
    let statements: Vec<TokenStream> = rule_entries
        .iter()
        .map(|rule| {
            let plugin_module = Ident::new(rule.plugin_module_name, Span::call_site());
            let rule_module = Ident::new(rule.rule_module_name, Span::call_site());
            let rule_struct = Ident::new(&rule.rule_struct_name(), Span::call_site());
            let enum_name = make_enum_ident(rule);

            quote! {
                pub use crate::rules::#plugin_module::#rule_module::#rule_struct as #enum_name;
            }
        })
        .collect();

    quote! { #(#statements)* }
}

fn generate_imports() -> TokenStream {
    quote! {
        use crate::{
            context::{ContextHost, LintContext},
            rule::{Rule, RuleCategory, RuleFixMeta, RuleMeta, RuleRunner, RuleRunFunctionsImplemented},
            utils::PossibleJestNode,
            AstNode
        };
        use oxc_semantic::AstTypesBitset;
    }
}

fn generate_rule_enum(rule_entries: &[RuleEntry<'_>]) -> TokenStream {
    let variants: Vec<TokenStream> = rule_entries
        .iter()
        .map(|rule| {
            let enum_name = make_enum_ident(rule);
            quote! { #enum_name(#enum_name) }
        })
        .collect();

    quote! {
        #[derive(Debug, Clone)]
        pub enum RuleEnum {
            #(#variants),*
        }
    }
}

fn generate_rule_enum_impl(rule_entries: &[RuleEntry<'_>]) -> TokenStream {
    let id_arms: Vec<TokenStream> = rule_entries
        .iter()
        .enumerate()
        .map(|(idx, rule)| {
            let enum_name = make_enum_ident(rule);
            quote! { Self::#enum_name(_) => #idx }
        })
        .collect();

    let name_arms: Vec<TokenStream> = rule_entries
        .iter()
        .map(|rule| {
            let enum_name = make_enum_ident(rule);
            quote! { Self::#enum_name(_) => #enum_name::NAME }
        })
        .collect();

    let category_arms: Vec<TokenStream> = rule_entries
        .iter()
        .map(|rule| {
            let enum_name = make_enum_ident(rule);
            quote! { Self::#enum_name(_) => #enum_name::CATEGORY }
        })
        .collect();

    let fix_arms: Vec<TokenStream> = rule_entries
        .iter()
        .map(|rule| {
            let enum_name = make_enum_ident(rule);
            quote! { Self::#enum_name(_) => #enum_name::FIX }
        })
        .collect();

    let documentation_arms: Vec<TokenStream> = rule_entries
        .iter()
        .map(|rule| {
            let enum_name = make_enum_ident(rule);
            quote! { Self::#enum_name(_) => #enum_name::documentation() }
        })
        .collect();

    let schema_arms: Vec<TokenStream> = rule_entries
        .iter()
        .map(|rule| {
            let enum_name = make_enum_ident(rule);
            quote! { Self::#enum_name(_) => #enum_name::config_schema(generator).or_else(|| #enum_name::schema(generator)) }
        })
        .collect();

    let plugin_name_arms: Vec<TokenStream> = rule_entries
        .iter()
        .map(|rule| {
            let enum_name = make_enum_ident(rule);
            let plugin_name = rule.plugin_module_name;
            quote! { Self::#enum_name(_) => #plugin_name }
        })
        .collect();

    let from_configuration_arms: Vec<TokenStream> = rule_entries
        .iter()
        .map(|rule| {
            let enum_name = make_enum_ident(rule);
            quote! { Self::#enum_name(_) => Ok(Self::#enum_name(#enum_name::from_configuration(value)?)) }
        })
        .collect();

    let to_configuration_arms: Vec<TokenStream> = rule_entries
        .iter()
        .map(|rule| {
            let enum_name = make_enum_ident(rule);
            quote! { Self::#enum_name(rule) => rule.to_configuration() }
        })
        .collect();

    let run_arms: Vec<TokenStream> = rule_entries
        .iter()
        .map(|rule| {
            let enum_name = make_enum_ident(rule);
            quote! { Self::#enum_name(rule) => rule.run(node, ctx) }
        })
        .collect();

    let run_once_arms: Vec<TokenStream> = rule_entries
        .iter()
        .map(|rule| {
            let enum_name = make_enum_ident(rule);
            quote! { Self::#enum_name(rule) => rule.run_once(ctx) }
        })
        .collect();

    let run_on_jest_node_arms: Vec<TokenStream> = rule_entries
        .iter()
        .map(|rule| {
            let enum_name = make_enum_ident(rule);
            quote! { Self::#enum_name(rule) => rule.run_on_jest_node(jest_node, ctx) }
        })
        .collect();

    let should_run_arms: Vec<TokenStream> = rule_entries
        .iter()
        .map(|rule| {
            let enum_name = make_enum_ident(rule);
            quote! { Self::#enum_name(rule) => rule.should_run(ctx) }
        })
        .collect();

    let is_tsgolint_rule_arms: Vec<TokenStream> = rule_entries
        .iter()
        .map(|rule| {
            let enum_name = make_enum_ident(rule);
            quote! { Self::#enum_name(_) => #enum_name::IS_TSGOLINT_RULE }
        })
        .collect();

    let types_info_arms: Vec<TokenStream> = rule_entries
        .iter()
        .map(|rule| {
            let enum_name = make_enum_ident(rule);
            quote! { Self::#enum_name(rule) => rule.types_info() }
        })
        .collect();

    let run_info_arms: Vec<TokenStream> = rule_entries
        .iter()
        .map(|rule| {
            let enum_name = make_enum_ident(rule);
            quote! { Self::#enum_name(rule) => rule.run_info() }
        })
        .collect();

    quote! {
        impl RuleEnum {
            pub fn id(&self) -> usize {
                match self {
                    #(#id_arms),*
                }
            }

            pub fn name(&self) -> &'static str {
                match self {
                    #(#name_arms),*
                }
            }

            pub fn category(&self) -> RuleCategory {
                match self {
                    #(#category_arms),*
                }
            }

            /// This [`Rule`]'s auto-fix capabilities.
            pub fn fix(&self) -> RuleFixMeta {
                match self {
                    #(#fix_arms),*
                }
            }

            #[cfg(feature = "ruledocs")]
            pub fn documentation(&self) -> Option<&'static str> {
                match self {
                    #(#documentation_arms),*
                }
            }

            #[cfg(feature = "ruledocs")]
            pub fn schema(&self, generator: &mut schemars::SchemaGenerator) -> Option<schemars::schema::Schema> {
                match self {
                    #(#schema_arms),*
                }
            }

            pub fn plugin_name(&self) -> &'static str {
                match self {
                    #(#plugin_name_arms),*
                }
            }

            pub fn from_configuration(&self, value: serde_json::Value) -> Result<Self, serde_json::error::Error> {
                match self {
                    #(#from_configuration_arms),*
                }
            }

            pub fn to_configuration(&self) -> Option<Result<serde_json::Value, serde_json::Error>> {
                match self {
                    #(#to_configuration_arms),*
                }
            }

            pub(crate) fn run<'a>(&self, node: &AstNode<'a>, ctx: &LintContext<'a>) {
                match self {
                    #(#run_arms),*
                }
            }

            pub(crate) fn run_once(&self, ctx: &LintContext<'_>) {
                match self {
                    #(#run_once_arms),*
                }
            }

            pub(crate) fn run_on_jest_node<'a, 'c>(
                &self,
                jest_node: &PossibleJestNode<'a, 'c>,
                ctx: &'c LintContext<'a>,
            ) {
                match self {
                    #(#run_on_jest_node_arms),*
                }
            }

            pub(crate) fn should_run(&self, ctx: &ContextHost) -> bool {
                match self {
                    #(#should_run_arms),*
                }
            }

            pub fn is_tsgolint_rule(&self) -> bool {
                match self {
                    #(#is_tsgolint_rule_arms),*
                }
            }

            pub fn types_info(&self) -> Option<&'static AstTypesBitset> {
                match self {
                    #(#types_info_arms),*
                }
            }

            pub fn run_info(&self) -> RuleRunFunctionsImplemented {
                match self {
                    #(#run_info_arms),*
                }
            }
        }
    }
}

fn generate_trait_impls() -> TokenStream {
    quote! {
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
                Some(self.cmp(other))
            }
        }
    }
}

fn generate_rules_static(rule_entries: &[RuleEntry<'_>]) -> TokenStream {
    let entries: Vec<TokenStream> = rule_entries
        .iter()
        .map(|rule| {
            let enum_name = make_enum_ident(rule);
            quote! { RuleEnum::#enum_name(#enum_name::default()) }
        })
        .collect();

    quote! {
        pub static RULES: std::sync::LazyLock<Vec<RuleEnum>> = std::sync::LazyLock::new(|| vec![
            #(#entries),*
        ]);
    }
}
