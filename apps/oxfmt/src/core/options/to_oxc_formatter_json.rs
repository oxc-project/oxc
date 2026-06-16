use oxc_formatter_json::{
    BracketSpacing, Expand, JsonFormatOptions, JsonVariant, QuoteProps, TrailingCommas,
};

use super::{
    super::oxfmtrc::{
        FormatConfig, ObjectWrapConfig, QuotePropsConfig, SortPackageJsonConfig,
        SortPackageJsonUserConfig, TrailingCommaConfig,
    },
    to_core_options::to_core_options,
};

/// Convert `FormatConfig` into validated `JsonFormatOptions` for `oxc_formatter_json`.
///
/// Most JSON-specific output options are fixed by [`oxc_formatter_json::JsonVariant`].
///
/// # Errors
/// Returns error if any option value is invalid.
pub fn to_oxc_formatter_json(
    config: &FormatConfig,
    variant: JsonVariant,
) -> Result<JsonFormatOptions, String> {
    let core = to_core_options(config)?;

    let mut options = JsonFormatOptions {
        variant,
        indent_style: core.indent_style,
        indent_width: core.indent_width,
        line_width: core.line_width,
        line_ending: core.line_ending,
        ..JsonFormatOptions::default()
    };

    // [Prettier] trailingComma: "all" | "es5" | "none"
    if let Some(commas) = config.trailing_comma {
        options.trailing_commas = match commas {
            // "all" and "es5" are indistinguishable for JSON
            TrailingCommaConfig::All | TrailingCommaConfig::Es5 => TrailingCommas::Always,
            TrailingCommaConfig::None => TrailingCommas::Never,
        };
    }
    // [Prettier] bracketSpacing: boolean
    if let Some(spacing) = config.bracket_spacing {
        options.bracket_spacing = BracketSpacing::from(spacing);
    }
    // [Prettier] objectWrap: "preserve" | "collapse"
    if let Some(wrap) = config.object_wrap {
        options.expand = match wrap {
            ObjectWrapConfig::Preserve => Expand::Auto,
            ObjectWrapConfig::Collapse => Expand::Never,
        };
    }
    // [Prettier] singleQuote: boolean
    if let Some(single_quote) = config.single_quote {
        options.single_quote = single_quote.into();
    }
    // [Prettier] quoteProps: "as-needed" | "consistent" | "preserve"
    if let Some(quote_props) = config.quote_props {
        options.quote_props = match quote_props {
            QuotePropsConfig::AsNeeded => QuoteProps::AsNeeded,
            QuotePropsConfig::Consistent => QuoteProps::Consistent,
            QuotePropsConfig::Preserve => QuoteProps::Preserve,
        };
    }

    Ok(options)
}

/// Convert `FormatConfig` into `sort_package_json::SortOptions`,
/// the companion of [`to_oxc_formatter_json`] for the `package.json` pipeline
/// (sorted by `sort-package-json`, then formatted with the `json-stringify` variant).
///
/// `package.json` sorting is opt-out: when `sort_package_json` is unset,
/// it defaults to enabled with default options.
/// Returns `None` only when the user explicitly sets `sort_package_json: false`.
pub fn to_sort_package_json(config: &FormatConfig) -> Option<sort_package_json::SortOptions> {
    let sort_config = config.sort_package_json.clone().map_or_else(
        || Some(SortPackageJsonConfig::default()),
        SortPackageJsonUserConfig::into_config,
    )?;
    Some(sort_package_json::SortOptions {
        sort_scripts: sort_config.sort_scripts.unwrap_or(false),
        // Small optimization: `oxc_formatter_json` will reformat anyway
        pretty: false,
    })
}
