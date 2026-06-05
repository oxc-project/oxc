use oxc_formatter_core::{IndentStyle, IndentWidth, LineEnding, LineWidth};

use super::super::oxfmtrc::{EndOfLineConfig, FormatConfig};

/// Language-neutral core formatting options shared by every formatter.
#[derive(Debug, Clone, Copy, Default)]
pub struct CoreOptions {
    pub indent_style: IndentStyle,
    pub indent_width: IndentWidth,
    pub line_width: LineWidth,
    pub line_ending: LineEnding,
}

/// Convert the language-neutral core options shared by every formatter into a
/// validated [`CoreOptions`].
///
/// This is the single source of truth for core-option parsing, validation, and default fallbacks.
/// Each downstream converter ([`super::to_oxc_formatter()`], [`super::to_oxc_toml()`], etc) layers
/// its language-specific options on top of the result,
/// so none of them depends on another formatter crate just to resolve these shared fields.
///
/// # Errors
/// Returns an error if `tabWidth` or `printWidth` is out of range.
pub fn to_core_options(config: &FormatConfig) -> Result<CoreOptions, String> {
    let mut options = CoreOptions::default();

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

    Ok(options)
}
