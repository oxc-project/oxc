//! Prettier option-set → `JsonFormatOptions` mapping.
//!
//! Shared by the fixture harness and the conformance target via `#[path]`
//! (one source, no drift; see `oxc_formatter_tests`'s AGENTS.md).

use oxc_formatter_json::{
    ArrayExpand, ArrayLinePattern, BracketSpacing, Expand, JsonFormatOptions, JsonVariant,
    QuoteProps, TrailingCommas,
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
            // NOTE: Not a Prettier option
            "arrayWrap" => {
                if let Some(s) = value.as_str() {
                    options.array_expand = match s {
                        "auto" => ArrayExpand::Auto,
                        "preserve" => ArrayExpand::Preserve,
                        "collapse" => ArrayExpand::Never,
                        _ => options.array_expand,
                    };
                } else if let Some(object) = value.as_object() {
                    // `serde_json` is not a direct dependency, so `Value::as_u64` cannot be named
                    let threshold = match object.get("wrapThreshold") {
                        Some(threshold) => threshold.as_u64(),
                        None => None,
                    };
                    options.array_expand = threshold
                        .and_then(|threshold| u32::try_from(threshold).ok())
                        .map_or(ArrayExpand::Preserve, ArrayExpand::ForceAboveThreshold);
                    options.array_line_pattern = object
                        .get("linePattern")
                        .and_then(|v| v.as_str())
                        .and_then(|pattern| {
                            pattern
                                .split_whitespace()
                                .map(str::parse)
                                .collect::<Result<_, _>>()
                                .ok()
                        })
                        .and_then(ArrayLinePattern::new);
                }
            }
            _ => {}
        }
    }
}
