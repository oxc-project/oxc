use oxc_formatter_core::{CoreFormatOptions, FormatOptions};
use oxc_formatter_toml::{TomlFormatOptions, TrailingCommas};

use super::super::oxfmtrc::{FormatConfig, TrailingCommaConfig};

/// Convert `FormatConfig` into `TomlFormatOptions` for `oxc_formatter_toml`.
///
/// Currently, `oxfmtrc` does not have dedicated TOML-specific options.
/// So it mirrors Prettier's `prettier-plugin-toml` conventions.
///
/// NOTE: Pure field translation:
/// `core` comes pre-validated from the config-resolution gate (`validate()`), so this cannot fail.
pub fn to_oxc_formatter_toml(
    config: &FormatConfig,
    core_options: CoreFormatOptions,
) -> TomlFormatOptions {
    let mut options = TomlFormatOptions::default();
    options.apply_core(core_options);

    // [Prettier] trailingComma: "all" | "es5" | "none"
    // `all`/`es5` are indistinguishable for TOML (arrays only check "not none")
    if let Some(trailing_comma) = config.trailing_comma {
        options.trailing_commas = match trailing_comma {
            TrailingCommaConfig::All | TrailingCommaConfig::Es5 => TrailingCommas::Always,
            TrailingCommaConfig::None => TrailingCommas::Never,
        };
    }

    options
}
