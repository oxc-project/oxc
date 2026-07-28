use oxc_formatter_graphql::{BracketSpacing, GraphqlFormatOptions};

use super::{super::oxfmtrc::FormatConfig, to_core_options::to_core_options};

/// Convert `FormatConfig` into validated `GraphqlFormatOptions` for `oxc_formatter_graphql`.
///
/// Prettier's `graphql` language consumes only the shared layout options plus `bracketSpacing`.
///
/// # Errors
/// Returns error if any option value is invalid.
pub fn to_oxc_formatter_graphql(config: &FormatConfig) -> Result<GraphqlFormatOptions, String> {
    let core = to_core_options(config)?;

    let mut options = GraphqlFormatOptions {
        indent_style: core.indent_style,
        indent_width: core.indent_width,
        line_width: core.line_width,
        line_ending: core.line_ending,
        ..GraphqlFormatOptions::default()
    };

    // [Prettier] bracketSpacing: boolean
    if let Some(spacing) = config.bracket_spacing {
        options.bracket_spacing = BracketSpacing::from(spacing);
    }

    Ok(options)
}
