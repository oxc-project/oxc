use oxc_formatter_core::{CoreFormatOptions, FormatOptions};
use oxc_formatter_htmlish::{HtmlishFormatOptions, WhitespaceSensitivity};

use super::super::oxfmtrc::{FormatConfig, HtmlWhitespaceSensitivityConfig};

/// Convert `FormatConfig` into `HtmlishFormatOptions` for `oxc_formatter_htmlish`.
///
/// Prettier's `html` language consumes the shared layout options plus
/// `htmlWhitespaceSensitivity`, `bracketSameLine` and `singleAttributePerLine`.
///
/// NOTE: Pure field translation:
/// `core` comes pre-validated from the config-resolution gate (`validate()`), so this cannot fail.
pub fn to_oxc_formatter_htmlish(
    config: &FormatConfig,
    core_options: CoreFormatOptions,
) -> HtmlishFormatOptions {
    let mut options = HtmlishFormatOptions::default();
    options.apply_core(core_options);

    // [Prettier] htmlWhitespaceSensitivity: "css" | "strict" | "ignore"
    if let Some(sensitivity) = config.html_whitespace_sensitivity {
        options.whitespace_sensitivity = match sensitivity {
            HtmlWhitespaceSensitivityConfig::Css => WhitespaceSensitivity::Css,
            HtmlWhitespaceSensitivityConfig::Strict => WhitespaceSensitivity::Strict,
            HtmlWhitespaceSensitivityConfig::Ignore => WhitespaceSensitivity::Ignore,
        };
    }
    // [Prettier] bracketSameLine: boolean
    if let Some(bracket_same_line) = config.bracket_same_line {
        options.bracket_same_line = bracket_same_line;
    }
    // [Prettier] singleAttributePerLine: boolean
    if let Some(single_attribute_per_line) = config.single_attribute_per_line {
        options.single_attribute_per_line = single_attribute_per_line;
    }

    // [Oxfmt] sortTailwindcss: collect `class` (and `attributes`) classes for batch sorting.
    // napi only, like the JS / CSS mappers: the sorter itself is JS-side.
    #[cfg(feature = "napi")]
    {
        options.sort_tailwindcss =
            config.tailwind_config().map(|tw_config| tw_config.attributes.unwrap_or_default());
    }

    options
}
