//! Prettier option-set → `JsonFormatOptions` mapping.
//!
//! Shared by the fixture harness and the conformance target via `#[path]`
//! (one source, no drift; see `oxc_formatter_tests`'s AGENTS.md).

use oxc_formatter_json::{
    BracketSpacing, Expand, JsonFormatOptions, JsonVariant, QuoteProps, TrailingCommas,
};
use oxc_formatter_tests::{OptionSet, apply_core_options};

/// Applies the four core options plus the JSON-specific keys onto `options`.
/// Parsing is lenient like `apply_core_options`: unknown or invalid values are ignored.
pub fn apply_json_options(options: &mut JsonFormatOptions, json: &OptionSet) {
    apply_core_options(options, json);

    for (key, value) in json {
        match key.as_str() {
            // Fixture-only key: conformance selects the variant via its config
            // (Prettier specs never pass `variant`).
            "variant" => {
                if let Some(s) = value.as_str() {
                    options.variant = match s {
                        "json" => JsonVariant::Json,
                        "jsonc" => JsonVariant::Jsonc,
                        "json5" => JsonVariant::Json5,
                        "json-stringify" => JsonVariant::JsonStringify,
                        _ => options.variant,
                    };
                }
            }
            "trailingComma" => {
                if let Some(s) = value.as_str() {
                    // Translate Prettier's vocabulary into JSON's neutral two states here,
                    // in the harness — the JSON type itself knows no "es5".
                    options.trailing_commas = match s {
                        "all" | "es5" => TrailingCommas::Always,
                        "none" => TrailingCommas::Never,
                        _ => options.trailing_commas,
                    };
                }
            }
            "bracketSpacing" => {
                if let Some(b) = value.as_bool() {
                    options.bracket_spacing = BracketSpacing::from(b);
                }
            }
            "singleQuote" => {
                if let Some(b) = value.as_bool() {
                    options.single_quote = b.into();
                }
            }
            "quoteProps" => {
                if let Some(s) = value.as_str() {
                    options.quote_props = match s {
                        "preserve" => QuoteProps::Preserve,
                        "consistent" => QuoteProps::Consistent,
                        _ => QuoteProps::AsNeeded,
                    };
                }
            }
            "objectWrap" => {
                if let Some(s) = value.as_str() {
                    options.expand = match s {
                        "preserve" => Expand::Auto,
                        "collapse" => Expand::Never,
                        _ => options.expand,
                    };
                }
            }
            _ => {}
        }
    }
}
