use oxc_formatter_core::{CoreFormatOptions, FormatOptions};
use oxc_formatter_markdown::{MarkdownFormatOptions, ProseWrap, SingleQuote};

use super::super::oxfmtrc::{FormatConfig, ProseWrapConfig};

/// Convert `FormatConfig` into `MarkdownFormatOptions` for `oxc_formatter_markdown`.
///
/// Prettier's `markdown` language consumes the shared layout options plus
/// `proseWrap` and `singleQuote`.
///
/// NOTE: Pure field translation:
/// `core` comes pre-validated from the config-resolution gate (`validate()`), so this cannot fail.
pub fn to_oxc_formatter_markdown(
    config: &FormatConfig,
    core_options: CoreFormatOptions,
) -> MarkdownFormatOptions {
    let mut options = MarkdownFormatOptions::default();
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

    options
}
