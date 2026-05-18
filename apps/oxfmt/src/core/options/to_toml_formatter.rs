use oxc_toml::Options as TomlFormatterOptions;

use super::super::oxfmtrc::FormatConfig;
use super::to_oxc_formatter::to_oxc_formatter;

/// Convert `FormatConfig` into validated `TomlFormatterOptions` for `oxc_toml`.
///
/// Routes through [`to_oxc_formatter`] so validation (e.g., `printWidth` bounds)
/// and oxfmt's default fallbacks are applied consistently with the JS path.
/// Mirrors `prettier-plugin-toml` semantics:
/// <https://github.com/un-ts/prettier/blob/7a4346d5dbf6b63987c0f81228fc46bb12f8692f/packages/toml/src/index.ts#L27-L31>
///
/// TODO: After `oxc_formatter_core` separation lands, derive the core fields (`column_width` / `indent_string` / `crlf`)
/// from a neutral resolved form instead of `oxc_formatter::FormatOptions`.
/// `trailing_comma` is JS-specific (not a core option):
/// - either keep mirroring Prettier's convention by reading it from `FormatConfig` directly,
/// - or introduce a dedicated `toml: {}` namespace options in oxfmtrc for explicit TOML-side config
///
/// # Errors
/// Returns error if any option value is invalid (delegates validation to `to_oxc_formatter`).
pub fn to_toml_formatter(config: &FormatConfig) -> Result<TomlFormatterOptions, String> {
    let format_options = to_oxc_formatter(config)?;
    Ok(TomlFormatterOptions {
        column_width: format_options.line_width.value() as usize,
        indent_string: if format_options.indent_style.is_tab() {
            "\t".to_string()
        } else {
            " ".repeat(format_options.indent_width.value() as usize)
        },
        array_trailing_comma: !format_options.trailing_commas.is_none(),
        crlf: format_options.line_ending.is_carriage_return_line_feed(),
        // Align with `oxc_formatter` and Prettier so `insertFinalNewline` works.
        trailing_newline: true,
        ..Default::default()
    })
}
