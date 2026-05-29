use oxc_formatter_core::{BracketSpacing, Expand, IndentStyle, IndentWidth, LineEnding, LineWidth};
use oxc_formatter_json::{JsonFormatOptions, JsonVariant};

use super::super::oxfmtrc::{EndOfLineConfig, FormatConfig, ObjectWrapConfig};

/// Convert `FormatConfig` into validated `JsonFormatOptions` for `oxc_formatter_json`.
///
/// JSON-specific output options are intentionally ignored,
/// [`oxc_formatter_json::JsonVariant`] fixes them.
///
/// # Errors
/// Returns error if any option value is invalid.
pub fn to_oxc_formatter_json(
    config: &FormatConfig,
    variant: JsonVariant,
) -> Result<JsonFormatOptions, String> {
    let mut options = JsonFormatOptions { variant, ..JsonFormatOptions::default() };

    // [Prettier] useTabs: boolean
    if let Some(use_tabs) = config.use_tabs {
        options.indent_style = if use_tabs { IndentStyle::Tab } else { IndentStyle::Space };
    }
    // [Prettier] tabWidth: number
    if let Some(width) = config.tab_width {
        options.indent_width =
            IndentWidth::try_from(width).map_err(|e| format!("Invalid tabWidth: {e}"))?;
    }
    // [Prettier] printWidth: number
    if let Some(width) = config.print_width {
        options.line_width =
            LineWidth::try_from(width).map_err(|e| format!("Invalid printWidth: {e}"))?;
    }
    // [Prettier] endOfLine: "lf" | "cr" | "crlf" | "auto"
    // NOTE: "auto" is not supported
    if let Some(ending) = config.end_of_line {
        options.line_ending = match ending {
            EndOfLineConfig::Lf => LineEnding::Lf,
            EndOfLineConfig::Crlf => LineEnding::Crlf,
            EndOfLineConfig::Cr => LineEnding::Cr,
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
