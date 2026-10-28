//! Prettier option-set → `YamlFormatOptions` mapping.
//!
//! Shared by the fixture harness and the conformance target via `#[path]`
//! (one source, no drift; see `oxc_formatter_tests`'s AGENTS.md).

use oxc_formatter_tests::{OptionSet, apply_core_options};
use oxc_formatter_yaml::{ProseWrap, TrailingCommas, YamlFormatOptions};

/// Applies the four core options plus the YAML-specific keys onto `options`.
/// Parsing is lenient like `apply_core_options`: unknown or invalid values are ignored.
pub fn apply_yaml_options(options: &mut YamlFormatOptions, json: &OptionSet) {
    apply_core_options(options, json);

    for (key, value) in json {
        match key.as_str() {
            "proseWrap" => {
                if let Some(s) = value.as_str() {
                    options.prose_wrap = match s {
                        "always" => ProseWrap::Always,
                        "never" => ProseWrap::Never,
                        _ => ProseWrap::Preserve,
                    };
                }
            }
            "singleQuote" => {
                if let Some(b) = value.as_bool() {
                    options.single_quote = b.into();
                }
            }
            "bracketSpacing" => {
                if let Some(b) = value.as_bool() {
                    options.bracket_spacing = b.into();
                }
            }
            "trailingComma" => {
                if let Some(s) = value.as_str() {
                    options.trailing_commas =
                        if s == "none" { TrailingCommas::Never } else { TrailingCommas::Always };
                }
            }
            _ => {}
        }
    }
}
