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
    category: Ident,
    /// Struct implementing [`JsonSchema`] that describes the rule's config.
    ///
    /// Note: Intentionally does not allow `Self`
    schema: Option<Ident>,
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
        let category = input.parse()?;

        // Parse FixMeta if it's specified. It will otherwise be excluded from
        // the RuleMeta impl, falling back on default set by RuleMeta itself.
        // Do not provide a default value here so that it can be set there
        // instead. We distinguish between a specified fix kind and a schema
        // based on case - fixes use snake_case, while schemas use PascaleCase.
        let mut fix: Option<Ident> = None;
        let mut schema: Option<Ident> = None;
        while input.peek(Token!(,)) {
            input.parse::<Token!(,)>()?;
            // allow for trailing commas
            let Ok(ident) = input.parse::<Ident>() else {
                break;
            };
            let s = format!("{ident}");
            if s.chars().next().is_some_and(|c| c.is_uppercase()) {
                schema = Some(ident);
            } else if s.chars().all(|c| c.is_alphabetic() || c == '_') {
                fix = Some(ident);
            }
        }

        // Ignore the rest
        input.parse::<proc_macro2::TokenStream>()?;

        Ok(Self { name: struct_name, category, fix, schema, documentation, used_in_test: false })
    }
}

fn rule_name_converter() -> Converter {
    Converter::new().remove_boundary(Boundary::LowerDigit).to_case(Case::Kebab)
}

pub fn declare_oxc_lint(metadata: LintRuleMeta) -> TokenStream {
    let LintRuleMeta { name, category, fix, schema, documentation, used_in_test } = metadata;

    let canonical_name = rule_name_converter().convert(name.to_string());
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
        let fix = parse_fix(&fix).unwrap();
        quote! {
            const FIX: RuleFixMeta = #fix;
        }
    });

    let import_statement = if used_in_test {
        None
    } else {
        Some(quote! { use crate::{rule::{RuleCategory, RuleMeta, RuleFixMeta}, fixer::FixKind}; })
    };

    let schema_impl = if let Some(schema) = schema {
        quote! {
            gen.subschema_for::<#schema>()
        }
    } else {
        quote! {
            {
                let mut obj = SchemaObject::default();
                obj.object().additional_properties = Some(Box::new(Schema::Bool(true)));
                obj.into()
            };
        }
    };

    let output = quote! {
        #import_statement

        impl RuleMeta for #name {
            const NAME: &'static str = #canonical_name;

            const CATEGORY: RuleCategory = #category;

            #fix

            fn documentation() -> Option<&'static str> {
                Some(#documentation)
            }
        }

        impl schemars::JsonSchema for #name {
            #[inline]
            fn schema_name() -> String {
                Self::NAME.to_string()
            }

            #[inline]
            fn schema_id() -> std::borrow::Cow<'static, str> {
                std::borrow::Cow::Borrowed(#canonical_name)
            }

            fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
                use schemars::schema::{Schema, SchemaObject};

                let mut schema: Schema = #schema_impl;

                let schema = match schema {
                    Schema::Object(mut obj) => {
                        let meta = obj.metadata();
                        meta.title = Some("Config for ".to_string() + Self::NAME);
                        Schema::Object(obj)
                    },
                    s => s
                };

                schema
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

fn parse_fix(s: &str) -> core::result::Result<proc_macro2::TokenStream, &'static str> {
    const SEP: char = '_';

    match s {
        "none" => {
            return Ok(quote! { RuleFixMeta::None });
        }
        "pending" => { return Ok(quote! { RuleFixMeta::FixPending }); }
        "fix" => {
            return Ok(quote! { RuleFixMeta::Fixable(FixKind::SafeFix) })
        },
        "suggestion" => {
            return Ok( quote! { RuleFixMeta::Fixable(FixKind::Suggestion) } )
        },
        // "fix-dangerous" => quote! { RuleFixMeta::Fixable(FixKind::Fix.union(FixKind::Dangerous)) },
        // "suggestion" => quote! { RuleFixMeta::Fixable(FixKind::Suggestion) },
        // "suggestion-dangerous" => quote! { RuleFixMeta::Fixable(FixKind::Suggestion.union(FixKind::Dangerous)) },
        "conditional" => return Err("Invalid fix capabilities: missing a fix kind. Did you mean 'fix-conditional'?"),
        "None" => return Err("Invalid fix capabilities. Did you mean 'none'?"),
        "Pending" => return Err("Invalid fix capabilities. Did you mean 'pending'?"),
        "Fix" => return Err("Invalid fix capabilities. Did you mean 'fix'?"),
        "Suggestion" => return Err("Invalid fix capabilities. Did you mean 'suggestion'?"),
        invalid if !invalid.contains(SEP) => return Err("invalid fix capabilities: {invalid}. Valid capabilities are none, pending, fix, suggestion, or [fix|suggestion]_[conditional?]_[dangerous?]."),
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
        Ok(quote! { RuleFixMeta::Conditional(#fix_kinds) })
    } else {
        Ok(quote! { RuleFixMeta::Fixable(#fix_kinds) })
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
