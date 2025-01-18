use convert_case::{Boundary, Case, Converter};
use itertools::Itertools as _;
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    Attribute, Error, Expr, Ident, Lit, LitStr, Meta, Result, Token,
};

pub struct LintRuleMeta {
    name: Ident,
    plugin: Ident,
    category: Ident,
    /// Describes what auto-fixing capabilities the rule has
    fix: Option<Ident>,
    documentation: String,
    pub used_in_test: bool,
}

impl Parse for LintRuleMeta {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut documentation = String::new();
        for attr in input.call(Attribute::parse_outer)? {
            if let Some(lit) = parse_attr(["doc"], &attr) {
                let value = lit.value();
                let line = value.strip_prefix(' ').unwrap_or(&value);

                documentation.push_str(line);
                documentation.push('\n');
            } else {
                return Err(Error::new_spanned(attr, "unexpected attribute"));
            }
        }

        let struct_name = input.parse()?;
        input.parse::<Token!(,)>()?;
        let plugin = input.parse()?;
        input.parse::<Token!(,)>()?;
        let category = input.parse()?;

        // Parse FixMeta if it's specified. It will otherwise be excluded from
        // the RuleMeta impl, falling back on default set by RuleMeta itself.
        // Do not provide a default value here so that it can be set there instead.
        let fix: Option<Ident> = if input.peek(Token!(,)) {
            input.parse::<Token!(,)>()?;
            input.parse()?
        } else {
            None
        };

        // Ignore the rest
        input.parse::<proc_macro2::TokenStream>()?;

        Ok(Self { name: struct_name, plugin, category, fix, documentation, used_in_test: false })
    }
}

pub(crate) fn rule_name_converter() -> Converter {
    Converter::new().remove_boundary(Boundary::LOWER_DIGIT).to_case(Case::Kebab)
}

pub fn declare_oxc_lint(metadata: LintRuleMeta) -> TokenStream {
    let LintRuleMeta { name, plugin, category, fix, documentation, used_in_test } = metadata;

    let canonical_name = rule_name_converter().convert(name.to_string());
    let plugin = plugin.to_string(); // ToDo: validate plugin name

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
        Some(quote! { use crate::{rule::{RuleCategory, RuleMeta, RuleFixMeta}, fixer::FixKind}; })
    };

    let output = quote! {
        #import_statement

        impl RuleMeta for #name {
            const NAME: &'static str = #canonical_name;

            const PLUGIN: &'static str = #plugin;

            const CATEGORY: RuleCategory = #category;

            #fix

            fn documentation() -> Option<&'static str> {
                Some(#documentation)
            }
        }
    };

    TokenStream::from(output)
}

fn parse_attr<'a, const LEN: usize>(
    path: [&'static str; LEN],
    attr: &'a Attribute,
) -> Option<&'a LitStr> {
    if let Meta::NameValue(name_value) = &attr.meta {
        let path_idents = name_value.path.segments.iter().map(|segment| &segment.ident);
        if itertools::equal(path_idents, path) {
            if let Expr::Lit(expr_lit) = &name_value.value {
                if let Lit::Str(s) = &expr_lit.lit {
                    return Some(s);
                }
            }
        }
    }
    None
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
        // "fix-dangerous" => quote! { RuleFixMeta::Fixable(FixKind::Fix.union(FixKind::Dangerous)) },
        // "suggestion" => quote! { RuleFixMeta::Fixable(FixKind::Suggestion) },
        // "suggestion-dangerous" => quote! { RuleFixMeta::Fixable(FixKind::Suggestion.union(FixKind::Dangerous)) },
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
        .filter(|seg| {
            let conditional = *seg == "conditional";
            is_conditional = is_conditional || conditional;
            !conditional
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
        "fix" => quote! { FixKind::Fix },
        "suggestion" => quote! { FixKind::Suggestion },
        "dangerous" => quote! { FixKind::Dangerous },
        _ => panic!("invalid fix kind: {s}. Valid fix kinds are fix, suggestion, or dangerous."),
    }
}
