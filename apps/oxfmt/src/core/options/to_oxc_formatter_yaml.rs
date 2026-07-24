use oxc_formatter_yaml::{
    BracketSpacing, ProseWrap, SingleQuote, TrailingCommas, YamlFormatOptions,
};

use super::{
    super::oxfmtrc::{FormatConfig, ProseWrapConfig, TrailingCommaConfig},
    to_core_options::to_core_options,
};

/// Convert `FormatConfig` into validated `YamlFormatOptions` for `oxc_formatter_yaml`.
///
/// Prettier's `yaml` language consumes the shared layout options plus
/// `proseWrap`, `singleQuote`, `bracketSpacing`, and `trailingComma`.
///
/// # Errors
/// Returns error if any option value is invalid.
pub fn to_oxc_formatter_yaml(config: &FormatConfig) -> Result<YamlFormatOptions, String> {
    let core = to_core_options(config)?;

    let mut options = YamlFormatOptions {
        indent_style: core.indent_style,
        indent_width: core.indent_width,
        line_width: core.line_width,
        line_ending: core.line_ending,
        ..YamlFormatOptions::default()
    };

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

    Ok(options)
}
