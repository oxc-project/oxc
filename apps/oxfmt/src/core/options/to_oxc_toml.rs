use oxc_formatter_core::CoreFormatOptions;
use oxc_toml::Options as TomlFormatterOptions;

use super::super::oxfmtrc::{FormatConfig, TrailingCommaConfig};

/// Convert `FormatConfig` into `TomlFormatterOptions` for `oxc_toml`.
///
/// Currently, `oxfmtrc` does not have dedicated TOML-specific options.
/// So it mirrors Prettier's `prettier-plugin-toml` conventions.
/// <https://github.com/un-ts/prettier/blob/7a4346d5dbf6b63987c0f81228fc46bb12f8692f/packages/toml/src/index.ts#L27-L31>
///
/// NOTE: Pure field translation:
/// `core` comes pre-validated from the config-resolution gate (`validate()`), so this cannot fail.
pub fn to_oxc_toml(config: &FormatConfig, core_options: CoreFormatOptions) -> TomlFormatterOptions {
    TomlFormatterOptions {
        column_width: core_options.line_width.value() as usize,
        indent_string: if core_options.indent_style.is_tab() {
            "\t".to_string()
        } else {
            " ".repeat(core_options.indent_width.value() as usize)
        },
        crlf: core_options.line_ending.is_carriage_return_line_feed(),
        // [Prettier] trailingComma: "all" | "es5" | "none"
        array_trailing_comma: !matches!(config.trailing_comma, Some(TrailingCommaConfig::None)),
        // NOTE: This is needed to `oxfmtrc.insertFinalNewline` work
        trailing_newline: true,
        ..Default::default()
    }
}
