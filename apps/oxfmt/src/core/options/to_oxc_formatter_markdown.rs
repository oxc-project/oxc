use oxc_formatter_core::{CoreFormatOptions, FormatOptions};
use oxc_formatter_markdown::{MarkdownFormatOptions, MarkdownTableStyle, ProseWrap, SingleQuote};

use super::super::oxfmtrc::{FormatConfig, MarkdownTableStyleConfig, ProseWrapConfig};

/// Convert `FormatConfig` into `MarkdownFormatOptions` for `oxc_formatter_markdown`.
///
/// Prettier's `markdown` language consumes the shared layout options plus
/// `proseWrap` and `singleQuote`.
/// Oxfmt also supports `markdownTableStyle`.
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
    // [Oxfmt] markdownTableStyle: "auto" | "compact"
    if let Some(table_style) = config.markdown_table_style {
        options.table_style = match table_style {
            MarkdownTableStyleConfig::Auto => MarkdownTableStyle::Auto,
            MarkdownTableStyleConfig::Compact => MarkdownTableStyle::Compact,
        };
    }

    options
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::oxfmtrc::Oxfmtrc;

    #[test]
    fn table_style_defaults_to_auto() {
        let options =
            to_oxc_formatter_markdown(&FormatConfig::default(), CoreFormatOptions::default());
        assert_eq!(options.table_style, MarkdownTableStyle::Auto);
    }

    #[test]
    fn table_style_is_deserialized_and_mapped() {
        for (value, expected) in
            [("compact", MarkdownTableStyle::Compact), ("auto", MarkdownTableStyle::Auto)]
        {
            let config: FormatConfig =
                serde_json::from_value(serde_json::json!({ "markdownTableStyle": value })).unwrap();
            let options = to_oxc_formatter_markdown(&config, CoreFormatOptions::default());
            assert_eq!(options.table_style, expected);
            assert_eq!(serde_json::to_value(&config).unwrap()["markdownTableStyle"], value);
        }
    }

    #[test]
    fn table_style_can_be_overridden() {
        let config: Oxfmtrc = serde_json::from_value(serde_json::json!({
            "markdownTableStyle": "compact",
            "overrides": [{
                "files": ["*.md"],
                "options": { "markdownTableStyle": "auto" }
            }]
        }))
        .unwrap();
        let mut resolved = config.format_config;
        // An unrelated override must preserve the inherited style.
        resolved.merge(&FormatConfig {
            prose_wrap: Some(ProseWrapConfig::Always),
            ..Default::default()
        });
        assert_eq!(
            to_oxc_formatter_markdown(&resolved, CoreFormatOptions::default()).table_style,
            MarkdownTableStyle::Compact
        );
        resolved.merge(&config.overrides.unwrap()[0].options);
        assert_eq!(
            to_oxc_formatter_markdown(&resolved, CoreFormatOptions::default()).table_style,
            MarkdownTableStyle::Auto
        );
    }

    #[test]
    fn invalid_table_style_is_rejected() {
        let err = serde_json::from_value::<FormatConfig>(serde_json::json!({
            "markdownTableStyle": "invalid"
        }))
        .unwrap_err();
        assert!(err.to_string().contains("invalid"));

        assert!(
            serde_json::from_value::<Oxfmtrc>(serde_json::json!({
                "overrides": [{
                    "files": ["*.md"],
                    "options": { "markdownTableStyle": "invalid" }
                }]
            }))
            .is_err()
        );
    }
}
