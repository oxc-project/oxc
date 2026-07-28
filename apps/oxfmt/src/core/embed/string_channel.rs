//! String-in/string-out embedded channel.
//!
//! Two consumers reach this channel through the same `EmbeddedFormatterCallback`
//! contract on `ExternalCallbacks`:
//! - JSDoc fenced code blocks (` ```css `, ` ```graphql `, …)
//! - html-in-js fallback (`format_js_in_html_as_fallback`): the IR channel's
//!   HTML route returned Prettier Doc that the IR converter can't represent,
//!   so the parent re-requests the same HTML via this string channel and
//!   substitutes placeholders back to `${expr}`.
//!
//! Unlike the [`super::ir_channel`], errors here keep the input verbatim (no
//! Prettier fallback, no diagnostics for JSDoc since it's inside a comment).

use std::sync::Arc;

use serde_json::Value;
use tracing::{debug, debug_span};

use oxc_allocator::Allocator;
use oxc_formatter::EmbeddedFormatterCallback;
use oxc_formatter_core::{FormatContext, Formatted};
use oxc_formatter_css::{CssFormatOptions, CssVariant};
use oxc_formatter_graphql::GraphqlFormatOptions;

use crate::core::{
    embed::{
        FormatEmbeddedWithConfigCallback, TailwindWithConfigCallback, language_to_prettier_parser,
    },
    options::inject_parser,
};

/// Build the `embedded_formatter` callback installed on `ExternalCallbacks`.
///
/// Dispatches by language identifier: a Rust formatter when available
/// (graphql/gql, css/scss/less), otherwise Prettier via `format_embedded`.
/// The JSDoc fenced consumer reaches every language; the html-in-js fallback
/// only ever passes `"html"` and therefore always lands on the Prettier branch.
///
/// `options` already carries the Tailwind plugin payload (`tailwindConfig`,
/// `tailwindStylesheet`, …) so the JS-side sorter can resolve class order
/// when the CSS branch passes its collected `@apply` classes.
pub fn build_embedded_callback(
    format_embedded: FormatEmbeddedWithConfigCallback,
    sort_tailwind: Option<TailwindWithConfigCallback>,
    options: Value,
    graphql_options: GraphqlFormatOptions,
    css_options: CssFormatOptions,
) -> EmbeddedFormatterCallback {
    Arc::new(move |language: &str, code: &str| {
        // Rust implementations first (JSDoc fenced code blocks).
        match language {
            "graphql" | "gql" => {
                return format_graphql_embedded(code, graphql_options);
            }
            "css" | "scss" | "less" => {
                return match &sort_tailwind {
                    Some(sort) => {
                        let sorter = |classes: Vec<String>| sort(&options, classes);
                        format_css_embedded(code, language, css_options, Some(&sorter))
                    }
                    None => format_css_embedded(code, language, css_options, None),
                };
            }
            _ => {}
        }
        let Some(parser_name) = language_to_prettier_parser(language) else {
            // NOTE: Do not return `Ok(original)` here.
            // We need to keep unsupported content as-is.
            return Err(format!("Unsupported language: {language}"));
        };
        debug_span!("oxfmt::external::format_embedded", parser = parser_name).in_scope(|| {
            // `clone()` is unavoidable here,
            // because there may be multiple embedded sections in one JS/TS file.
            let mut options = options.clone();
            inject_parser(&mut options, parser_name);
            (format_embedded)(options, code)
                .map(|mut code| {
                    // Remove trailing newline added by Prettier without allocation.
                    // For embedded code, we never want trailing newlines, regardless of options.
                    let trimmed_len = code.trim_end().len();
                    code.truncate(trimmed_len);
                    code
                })
                .inspect_err(|err| {
                    debug!("Failed to format embedded code for parser '{parser_name}': {err}");
                })
        })
    })
}

/// Format a JSDoc fenced code block as a standalone GraphQL document
/// (string-in/string-out, unlike the [`super::ir_channel`] contract).
fn format_graphql_embedded(code: &str, options: GraphqlFormatOptions) -> Result<String, String> {
    debug_span!("oxfmt::external::format_graphql_embedded").in_scope(|| {
        let allocator = Allocator::default();
        let formatted = oxc_formatter_graphql::format(&allocator, code, options)
            .map_err(|err| err.to_string())?;
        print_embedded_block(formatted)
    })
}

/// Format a JSDoc fenced code block as a standalone stylesheet
/// (string-in/string-out, unlike the [`super::ir_channel`] contract).
///
/// The `options` carry the IR channel's css-in-js variant (Scss),
/// so the variant is re-derived from the fence language here.
fn format_css_embedded(
    code: &str,
    language: &str,
    options: CssFormatOptions,
    sorter: Option<oxc_formatter_css::TailwindSorter<'_>>,
) -> Result<String, String> {
    debug_span!("oxfmt::external::format_css_embedded", language = language).in_scope(|| {
        let variant = match language {
            "scss" => CssVariant::Scss,
            "less" => CssVariant::Less,
            _ => CssVariant::Css,
        };
        let options = CssFormatOptions { variant, ..options };
        let allocator = Allocator::default();
        let formatted = oxc_formatter_css::format(&allocator, code, options, sorter)
            .map_err(|err| err.to_string())?;
        print_embedded_block(formatted)
    })
}

/// Print a formatted JSDoc fenced code block to a string.
/// The trailing newline is trimmed because the block is re-embedded
/// line-by-line into the comment.
fn print_embedded_block<C: FormatContext>(formatted: Formatted<'_, C>) -> Result<String, String> {
    let printed = formatted.print().map_err(|err| err.to_string())?;
    let mut code = printed.into_code();
    code.truncate(code.trim_end().len());
    Ok(code)
}
