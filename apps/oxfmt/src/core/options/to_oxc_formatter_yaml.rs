use oxc_formatter_core::{CoreFormatOptions, FormatOptions};
use oxc_formatter_yaml::{
    BracketSpacing, ProseWrap, SingleQuote, TrailingCommas, YamlFormatOptions,
};

use super::super::oxfmtrc::{FormatConfig, ProseWrapConfig, TrailingCommaConfig};

/// Convert `FormatConfig` into `YamlFormatOptions` for `oxc_formatter_yaml`.
///
/// Prettier's `yaml` language consumes the shared layout options plus
/// `proseWrap`, `singleQuote`, `bracketSpacing`, and `trailingComma`.
///
/// NOTE: Pure field translation:
/// `core` comes pre-validated from the config-resolution gate (`validate()`), so this cannot fail.
pub fn to_oxc_formatter_yaml(
    config: &FormatConfig,
    core_options: CoreFormatOptions,
) -> YamlFormatOptions {
    let mut options = YamlFormatOptions::default();
    options.apply_core(core_options);

    // [Prettier] proseWrap: "preserve" | "always" | "never"
    if let Some(prose_wrap) = config.prose_wrap {
        options.prose_wrap = match prose_wrap {
            ProseWrapConfig::Preserve => ProseWrap::Preserve,
            ProseWrapConfig::Always => ProseWrap::Always,
            ProseWrapConfig::Never => ProseWrap::Never,
        };
    }
    // [Prettier] singleQuote: boolean
    if let Some(single_quote) = config.single_quote {
        options.single_quote = SingleQuote::from(single_quote);
    }
    // [Prettier] bracketSpacing: boolean
    if let Some(spacing) = config.bracket_spacing {
        options.bracket_spacing = BracketSpacing::from(spacing);
    }
    // [Prettier] trailingComma: "all" | "es5" | "none"
    // `all`/`es5` are indistinguishable for YAML (flow collections only check "not none")
    if let Some(trailing_comma) = config.trailing_comma {
        options.trailing_commas = match trailing_comma {
            TrailingCommaConfig::All | TrailingCommaConfig::Es5 => TrailingCommas::Always,
            TrailingCommaConfig::None => TrailingCommas::Never,
        };
    }

    options
}
