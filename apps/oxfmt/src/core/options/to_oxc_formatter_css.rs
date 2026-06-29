use oxc_formatter_css::{CssFormatOptions, CssVariant, SingleQuote, TrailingCommas};

use super::{
    super::oxfmtrc::{FormatConfig, TrailingCommaConfig},
    to_core_options::to_core_options,
};

/// Convert `FormatConfig` into validated `CssFormatOptions` for `oxc_formatter_css`.
///
/// Prettier's CSS languages consume the shared layout options plus
/// `singleQuote` and `trailingComma` (SCSS maps only).
///
/// # Errors
/// Returns error if any option value is invalid.
pub fn to_oxc_formatter_css(
    config: &FormatConfig,
    variant: CssVariant,
) -> Result<CssFormatOptions, String> {
    let core = to_core_options(config)?;

    let mut options = CssFormatOptions {
        indent_style: core.indent_style,
        indent_width: core.indent_width,
        line_width: core.line_width,
        line_ending: core.line_ending,
        variant,
        ..CssFormatOptions::default()
    };

    // [Prettier] singleQuote: boolean
    if let Some(single_quote) = config.single_quote {
        options.single_quote = SingleQuote::from(single_quote);
    }
    // [Prettier] trailingComma: "all" | "es5" | "none"
    // `all`/`es5` are indistinguishable for CSS (SCSS maps only check "not none")
    if let Some(trailing_comma) = config.trailing_comma {
        options.trailing_commas = match trailing_comma {
            TrailingCommaConfig::All | TrailingCommaConfig::Es5 => TrailingCommas::Always,
            TrailingCommaConfig::None => TrailingCommas::Never,
        };
    }
    // [Oxfmt] sortTailwindcss: collect `@apply` classes for batch sorting.
    // The sorter itself is JS-side, so this stays off in the pure Rust build
    // (classes would print as-is anyway, but skipping collection is cheaper).
    #[cfg(feature = "napi")]
    {
        options.sort_tailwindcss = config.is_tailwind_enabled();
    }

    Ok(options)
}
