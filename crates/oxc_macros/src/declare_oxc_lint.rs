use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    Attribute, Expr, Ident, Lit, Meta, Result, Token,
};

pub struct LintRuleMeta {
    name: Ident,
    category: Ident,
    documentation: String,
    use_cfg: bool,
    pub used_in_test: bool,
}

impl Parse for LintRuleMeta {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut documentation = String::new();
        let use_cfg = 'use_cfg: {
            for attr in input.call(Attribute::parse_outer)? {
                if let Some(value) = parse_attr(["doc"], &attr) {
                    let line = value.strip_prefix(' ').unwrap_or(&value);

                    documentation.push_str(line);
                    documentation.push('\n');
                } else {
                    break 'use_cfg parse_attr(["use_cfg"], &attr).is_some();
                }
            }
            false
        };

        let struct_name = input.parse()?;
        input.parse::<Token![,]>()?;
        let category = input.parse()?;

        // Ignore the rest
        input.parse::<proc_macro2::TokenStream>()?;

        Ok(Self { name: struct_name, category, documentation, use_cfg, used_in_test: false })
    }
}

pub fn declare_oxc_lint(metadata: LintRuleMeta) -> TokenStream {
    let LintRuleMeta { name, category, documentation, use_cfg, used_in_test } = metadata;
    let canonical_name = name.to_string().to_case(Case::Kebab);
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

    let import_statement = if used_in_test {
        None
    } else {
        Some(quote! { use crate::{context::LintCtx, rule::{RuleCategory, RuleMeta, RuleContext}}; })
    };

    let context = if use_cfg {
        quote! { CFGLintContext<'a> }
    } else {
        quote! { LintContext<'a> }
    };

    let output = quote! {
        #import_statement

        impl RuleMeta for #name {
            const NAME: &'static str = #canonical_name;

            const CATEGORY: RuleCategory = #category;

            const USE_CFG: bool = #use_cfg;

            fn documentation() -> Option<&'static str> {
                Some(#documentation)
            }
        }

        impl RuleContext for #name {
            type Context<'a> = #context;
        }
    };

    TokenStream::from(output)
}

fn parse_attr<const LEN: usize>(path: [&'static str; LEN], attr: &Attribute) -> Option<String> {
    match &attr.meta {
        Meta::NameValue(name_value) => {
            let path_idents = name_value.path.segments.iter().map(|segment| &segment.ident);
            if itertools::equal(path_idents, path) {
                if let Expr::Lit(expr_lit) = &name_value.value {
                    if let Lit::Str(s) = &expr_lit.lit {
                        return Some(s.value());
                    }
                }
            }
            None
        }
        Meta::Path(p) if p.is_ident(path[0]) => Some(path[0].to_string()),
        _ => None,
    }
}
