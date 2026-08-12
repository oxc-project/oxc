//! Prettier option-set → `CssFormatOptions` mapping.
//!
//! Shared by the fixture harness and the conformance target via `#[path]`
//! (one source, no drift; see `oxc_formatter_tests`'s AGENTS.md).
//! The dialect (`variant`) is NOT set here: the fixture harness derives it from the file extension,
//! conformance from its config.

use oxc_formatter_css::{CssFormatOptions, TrailingCommas};
use oxc_formatter_tests::{OptionSet, apply_core_options};

/// Applies the four core options plus the CSS-specific keys onto `options`.
/// Parsing is lenient like `apply_core_options`: unknown or invalid values are ignored.
pub fn apply_css_options(options: &mut CssFormatOptions, json: &OptionSet) {
    apply_core_options(options, json);

    for (key, value) in json {
        match key.as_str() {
            "singleQuote" => {
                if let Some(b) = value.as_bool() {
                    options.single_quote = b.into();
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
