use convert_case::{Boundary, Case, Converter};
use itertools::Itertools as _;
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Error, Ident, Result, Token,
    parse::{Parse, ParseStream},
};

/// Metadata for a shared lint rule that references documentation from a shared location
pub struct SharedLintRuleMeta {
    name: Ident,
    // Whether this rule should be exposed to tsgolint integration
    is_tsgolint_rule: bool,
    plugin: Ident,
    category: Ident,
    /// Describes what auto-fixing capabilities the rule has
    fix: Option<Ident>,
    /// Path to the shared documentation module (e.g., `crate::rules::shared::valid_title`)
    shared_docs_path: syn::Path,
    pub used_in_test: bool,
    /// Rule configuration
    /// This is the name of a struct/enum/whatever implementing
    /// schemars::JsonSchema
    config: Option<Ident>,
}

impl Parse for SharedLintRuleMeta {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let struct_name: Ident = input.parse()?;

        // Optional marker `(tsgolint)` directly after the rule struct name
        let mut is_tsgolint_rule = false;
        if input.peek(syn::token::Paren) {
            let content;
            syn::parenthesized!(content in input);
            let marker: Ident = content.parse()?;
            if marker == "tsgolint" {
                if !content.is_empty() {
                    return Err(Error::new_spanned(marker, "unexpected tokens after 'tsgolint'"));
                }
                is_tsgolint_rule = true;
            } else {
                return Err(Error::new_spanned(
                    marker,
                    "unsupported marker (only 'tsgolint' is allowed)",
                ));
            }
        }
        input.parse::<Token!(,)>()?;

        let plugin = input.parse()?;
        input.parse::<Token!(,)>()?;
        let category = input.parse()?;

        // Parse FixMeta if it's specified
        let mut fix: Option<Ident> = None;
        let mut config: Option<Ident> = None;
        let mut shared_docs_path: Option<syn::Path> = None;

        // remaining options are `key = value` pairs
        while input.peek(Token!(,)) {
            input.parse::<Token!(,)>()?;
            if input.is_empty() {
                break;
            }
            let key: Ident = input.parse()?;
            match key.to_string().as_str() {
                "fix" => {
                    if input.peek(Token!(=)) {
                        input.parse::<Token!(=)>()?;
                        fix.replace(input.parse()?);
                    } else {
                        fix.replace(key);
                    }
                }
                "config" => {
                    input.parse::<Token!(=)>()?;
                    config.replace(input.parse()?);
                }
                "shared_docs" => {
                    input.parse::<Token!(=)>()?;
                    shared_docs_path.replace(input.parse()?);
                }
                _ => {
                    if input.peek(Token!(=)) || fix.is_some() {
                        panic!("invalid key: {key}");
                    } else {
                        // fix kind shorthand
                        fix.replace(key);
                    }
                }
            }
        }

        let remaining = input.parse::<proc_macro2::TokenStream>()?;
        if !remaining.is_empty() {
            return Err(Error::new_spanned(
                remaining,
                "unexpected tokens in rule declaration, missing a comma?",
            ));
        }

        let shared_docs_path = shared_docs_path.ok_or_else(|| {
            Error::new(input.span(), "shared_docs path is required for declare_oxc_shared_lint")
        })?;

        Ok(Self {
            name: struct_name,
            is_tsgolint_rule,
            plugin,
            category,
            fix,
            shared_docs_path,
            used_in_test: false,
            config,
        })
    }
}

pub fn rule_name_converter() -> Converter {
    Converter::new().remove_boundary(Boundary::LowerDigit).to_case(Case::Kebab)
}

pub fn declare_oxc_shared_lint(metadata: SharedLintRuleMeta) -> TokenStream {
    let SharedLintRuleMeta {
        name,
        is_tsgolint_rule,
        plugin,
        category,
        fix,
        shared_docs_path,
        used_in_test,
        config,
    } = metadata;

    let canonical_name = rule_name_converter().convert(name.to_string());
    let plugin_str = plugin.to_string();

    let category = match category.to_string().as_str() {
        "correctness" => quote! { RuleCategory::Correctness },
        "suspicious" => quote! { RuleCategory::Suspicious },
        "pedantic" => quote! { RuleCategory::Pedantic },
        "perf" => quote! { RuleCategory::Perf },
        "style" => quote! { RuleCategory::Style },
        "restriction" => quote! { RuleCategory::Restriction },
        "nursery" => quote! { RuleCategory::Nursery },
        _ => panic!("invalid rule category"),
    };

    let fix = fix.as_ref().map(Ident::to_string).map(|fix| {
        let fix = parse_fix(&fix);
        quote! {
            const FIX: RuleFixMeta = #fix;
        }
    });

    let import_statement = if used_in_test {
        None
    } else {
        Some(quote! {
            use crate::{rule::{RuleCategory, RuleMeta, RuleFixMeta, RuleRunner}, fixer::FixKind};
            use oxc_semantic::AstTypesBitset;
        })
    };

    #[cfg(not(feature = "ruledocs"))]
    let docs: Option<proc_macro2::TokenStream> = {
        let _ = shared_docs_path;
        None
    };

    #[cfg(feature = "ruledocs")]
    let docs = Some(quote! {
        fn documentation() -> Option<&'static str> {
            #shared_docs_path::DOCUMENTATION
        }
    });

    #[cfg(not(feature = "ruledocs"))]
    let config_schema: Option<proc_macro2::TokenStream> = {
        let _ = config;
        None
    };
    #[cfg(feature = "ruledocs")]
    let config_schema = match config {
        Some(config) => Some(quote! {
            fn config_schema(generator: &mut schemars::SchemaGenerator) -> Option<schemars::schema::Schema> {
                Some(generator.subschema_for::<#config>())
            }
        }),
        None => Some(quote! {
            fn config_schema(_generator: &mut schemars::SchemaGenerator) -> Option<schemars::schema::Schema> {
                None
            }
        }),
    };

    let output = quote! {
        #import_statement

        impl RuleMeta for #name {
            const NAME: &'static str = #canonical_name;

            const PLUGIN: &'static str = #plugin_str;

            const CATEGORY: RuleCategory = #category;

            const IS_TSGOLINT_RULE: bool = #is_tsgolint_rule;

            #fix

            #docs

            #config_schema
        }
    };

    TokenStream::from(output)
}

fn parse_fix(s: &str) -> proc_macro2::TokenStream {
    const SEP: char = '_';

    match s {
        "none" => {
            return quote! { RuleFixMeta::None };
        }
        "pending" => {
            return quote! { RuleFixMeta::FixPending };
        }
        "fix" => return quote! { RuleFixMeta::Fixable(FixKind::SafeFix) },
        "suggestion" => return quote! { RuleFixMeta::Fixable(FixKind::Suggestion) },
        "conditional" => {
            panic!("Invalid fix capabilities: missing a fix kind. Did you mean 'fix-conditional'?")
        }
        "None" => panic!("Invalid fix capabilities. Did you mean 'none'?"),
        "Pending" => panic!("Invalid fix capabilities. Did you mean 'pending'?"),
        "Fix" => panic!("Invalid fix capabilities. Did you mean 'fix'?"),
        "Suggestion" => panic!("Invalid fix capabilities. Did you mean 'suggestion'?"),
        invalid if !invalid.contains(SEP) => panic!(
            "invalid fix capabilities: {invalid}. Valid capabilities are none, pending, fix, suggestion, or [fix|suggestion]_[conditional?]_[dangerous?]."
        ),
        _ => {}
    }

    assert!(s.contains(SEP));

    let mut is_conditional = false;
    let fix_kinds = s
        .split(SEP)
        .filter(|seg| match *seg {
            "conditional" => {
                is_conditional = true;
                false
            }
            // e.g. "safe_fix". safe is implied
            "safe"
            // e.g. fix_or_suggestion
            | "and" | "or"
            => false,
            _ => true,
        })
        .unique()
        .map(parse_fix_kind)
        .reduce(|acc, kind| quote! { #acc.union(#kind) })
        .expect("No fix kinds were found during parsing, but at least one is required.");

    if is_conditional {
        quote! { RuleFixMeta::Conditional(#fix_kinds) }
    } else {
        quote! { RuleFixMeta::Fixable(#fix_kinds) }
    }
}

fn parse_fix_kind(s: &str) -> proc_macro2::TokenStream {
    match s {
        "fix" | "fixes" => quote! { FixKind::Fix },
        "suggestion" | "suggestions" => quote! { FixKind::Suggestion },
        "dangerous" => quote! { FixKind::Dangerous },
        _ => panic!("invalid fix kind: {s}. Valid fix kinds are fix, suggestion, or dangerous."),
    }
}
