//! Prettier option-set → `GraphqlFormatOptions` mapping.
//!
//! Shared by the fixture harness and the conformance target via `#[path]`
//! (one source, no drift; see `oxc_formatter_tests`'s AGENTS.md).

use oxc_formatter_graphql::GraphqlFormatOptions;
use oxc_formatter_tests::{OptionSet, apply_core_options};

/// Applies the four core options plus the GraphQL-specific keys onto `options`.
/// Parsing is lenient like `apply_core_options`: unknown or invalid values are ignored.
pub fn apply_graphql_options(options: &mut GraphqlFormatOptions, json: &OptionSet) {
    apply_core_options(options, json);

    for (key, value) in json {
        if key.as_str() == "bracketSpacing"
            && let Some(b) = value.as_bool()
        {
            options.bracket_spacing = b.into();
        }
    }
}
