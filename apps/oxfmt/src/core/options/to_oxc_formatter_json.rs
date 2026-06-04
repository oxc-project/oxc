use oxc_formatter_core::{BracketSpacing, Expand, TrailingCommas};
use oxc_formatter_json::{JsonFormatOptions, JsonVariant};

use super::{
    super::oxfmtrc::{FormatConfig, ObjectWrapConfig, TrailingCommaConfig},
    to_core_options::to_core_options,
};

/// Convert `FormatConfig` into validated `JsonFormatOptions` for `oxc_formatter_json`.
///
/// Most JSON-specific output options are fixed by [`oxc_formatter_json::JsonVariant`].
/// The exception is `trailingComma`, which the `jsonc`/`json5` variants honor
/// (the `json` variant ignores it, matching Prettier forcing `"none"`).
///
/// # Errors
/// Returns error if any option value is invalid.
pub fn to_oxc_formatter_json(
    config: &FormatConfig,
    variant: JsonVariant,
) -> Result<JsonFormatOptions, String> {
    let core = to_core_options(config)?;

    let mut options = JsonFormatOptions {
        variant,
        indent_style: core.indent_style,
        indent_width: core.indent_width,
        line_width: core.line_width,
        line_ending: core.line_ending,
        ..JsonFormatOptions::default()
    };

    // [Prettier] trailingComma: "all" | "es5" | "none" (default "all").
    // Only honored by `jsonc`/`json5`, `json`/`json-stringify` ignore it.
    // And "all" and "es5" are the same for JSON.
    if let Some(commas) = config.trailing_comma {
        options.trailing_commas = match commas {
            TrailingCommaConfig::All => TrailingCommas::All,
            TrailingCommaConfig::Es5 => TrailingCommas::Es5,
            TrailingCommaConfig::None => TrailingCommas::None,
        };
    }
    // [Prettier] bracketSpacing: boolean
    if let Some(spacing) = config.bracket_spacing {
        options.bracket_spacing = BracketSpacing::from(spacing);
    }
    // [Prettier] objectWrap: "preserve" | "collapse"
    if let Some(wrap) = config.object_wrap {
        options.expand = match wrap {
            ObjectWrapConfig::Preserve => Expand::Auto,
            ObjectWrapConfig::Collapse => Expand::Never,
        };
    }

    Ok(options)
}
