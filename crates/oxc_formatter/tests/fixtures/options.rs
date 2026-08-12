//! Prettier option-set → `JsFormatOptions` mapping.
//!
//! Shared by the fixture harness (`fixtures/mod.rs`) and the conformance target
//! (`conformance.rs`) via `#[path]` — integration-test targets are separate
//! crates, so each compiles this file; the SOURCE is the single copy that keeps
//! the two parsers from drifting. The `jsdoc` key only appears in fixture
//! `options.json` files (Prettier specs never pass it); the `objectWrap` /
//! `singleAttributePerLine` / `experimental*` keys only appear in specs.

use std::str::FromStr;

use oxc_formatter::{
    ArrowParentheses, AttributePosition, BracketSameLine, BracketSpacing, Expand, JsFormatOptions,
    JsdocOptions, OperatorPosition, QuoteProperties, QuoteStyle, Semicolons, TrailingCommas,
};
use oxc_formatter_tests::{OptionSet, apply_core_options};

/// Applies the four core options plus the JS-specific keys onto `options`.
/// Parsing is lenient like `apply_core_options`: unknown or invalid values are ignored.
pub fn apply_js_options(options: &mut JsFormatOptions, json: &OptionSet) {
    apply_core_options(options, json);

    for (key, value) in json {
        match key.as_str() {
            "semi" => {
                if let Some(b) = value.as_bool() {
                    options.semicolons = if b { Semicolons::Always } else { Semicolons::AsNeeded };
                }
            }
            "bracketSpacing" => {
                if let Some(b) = value.as_bool() {
                    options.bracket_spacing = BracketSpacing::from(b);
                }
            }
            // Deprecated alias pair: either being `true` wins, `false` never overwrites
            // (Prettier's OR semantics; also, `OptionSet` iterates in KEY order, not
            // source order, so last-write precedence cannot be replicated here).
            "bracketSameLine" | "jsxBracketSameLine" => {
                if value.as_bool() == Some(true) {
                    options.bracket_same_line = BracketSameLine::from(true);
                }
            }
            "singleQuote" => {
                if let Some(b) = value.as_bool() {
                    options.quote_style = if b { QuoteStyle::Single } else { QuoteStyle::Double };
                }
            }
            "jsxSingleQuote" => {
                if let Some(b) = value.as_bool() {
                    options.jsx_quote_style =
                        if b { QuoteStyle::Single } else { QuoteStyle::Double };
                }
            }
            "experimentalTernaries" => {
                if let Some(b) = value.as_bool() {
                    options.experimental_ternaries = b;
                }
            }
            "singleAttributePerLine" => {
                if let Some(b) = value.as_bool() {
                    options.attribute_position =
                        if b { AttributePosition::Multiline } else { AttributePosition::Auto };
                }
            }
            "trailingComma" => {
                if let Some(s) = value.as_str() {
                    options.trailing_commas = match s {
                        "none" => TrailingCommas::None,
                        "es5" => TrailingCommas::Es5,
                        "all" => TrailingCommas::All,
                        _ => options.trailing_commas,
                    };
                }
            }
            "quoteProps" => {
                if let Some(s) = value.as_str() {
                    options.quote_properties = QuoteProperties::from_str(s).unwrap_or_default();
                }
            }
            "objectWrap" => {
                if let Some(s) = value.as_str() {
                    // Prettier uses "preserve"/"collapse", but we use "auto"/"never"
                    options.expand = Expand::from_str(match s {
                        "preserve" => "auto",
                        "collapse" => "never",
                        _ => s,
                    })
                    .unwrap_or_default();
                }
            }
            "arrowParens" => {
                if let Some(s) = value.as_str() {
                    // Prettier uses "avoid", but we use "as-needed"
                    options.arrow_parentheses =
                        ArrowParentheses::from_str(if s == "avoid" { "as-needed" } else { s })
                            .unwrap_or_default();
                }
            }
            "experimentalOperatorPosition" => {
                if let Some(s) = value.as_str() {
                    options.experimental_operator_position =
                        OperatorPosition::from_str(s).unwrap_or_default();
                }
            }
            "jsdoc" if value.is_object() => {
                options.jsdoc = Some(JsdocOptions::default());
            }
            _ => {}
        }
    }
}
