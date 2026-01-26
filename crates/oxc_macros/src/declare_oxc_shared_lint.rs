use proc_macro::TokenStream;
use syn::{
    Error, Ident, Result, Token,
    parse::{Parse, ParseStream},
};

use crate::declare_oxc_lint::{DocumentationSource, generate_rule_meta_impl};

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

    let plugin_str = plugin.to_string();

    generate_rule_meta_impl(
        &name,
        is_tsgolint_rule,
        &plugin_str,
        &category,
        &fix,
        DocumentationSource::Path(shared_docs_path),
        used_in_test,
        &config,
    )
}
