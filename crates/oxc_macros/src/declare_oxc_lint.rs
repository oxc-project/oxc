use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Attribute, Error, Ident, Lit, LitStr, Meta, Result, Token};

fn parse_attr<const LEN: usize>(path: [&'static str; LEN], attr: &Attribute) -> Option<LitStr> {
    if let Meta::NameValue(name_value) = attr.parse_meta().ok()? {
        let path_idents = name_value.path.segments.iter().map(|segment| &segment.ident);

        if itertools::equal(path_idents, path) {
            if let Lit::Str(lit) = name_value.lit {
                return Some(lit);
            }
        }
    }

    None
}

pub struct LintRuleMeta {
    name: Ident,
    category: Ident,
    documentation: String,
    pub used_in_test: bool,
}

impl Parse for LintRuleMeta {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;

        let mut documentation = String::new();
        for attr in &attrs {
            if let Some(lit) = parse_attr(["doc"], attr) {
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

        // Ignore the rest
        input.parse::<TokenStream>()?;

        Ok(Self { name: struct_name, category, documentation, used_in_test: false })
    }
}

pub fn declare_oxc_lint(metadata: LintRuleMeta) -> TokenStream {
    let LintRuleMeta { name, category, documentation, used_in_test } = metadata;
    let canonical_name = name.to_string().to_case(Case::Kebab);
    let category = category.to_string().to_case(Case::Lower);

    let import_statement =
        if used_in_test { None } else { Some(quote! { use crate::rule::RuleMeta; }) };

    let output = quote! {
        #import_statement

        impl RuleMeta for #name {
            const NAME: &'static str = #canonical_name;
            const CATEGORY: &'static str = #category;

            fn documentation() -> Option<&'static str> {
                Some(#documentation)
            }
        }
    };

    output
}
