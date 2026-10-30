//! Prettier option-set → `HtmlishFormatOptions` mapping.
//!
//! Shared by the fixture harness and the conformance target via `#[path]`.

use oxc_formatter_htmlish::{HtmlishFormatOptions, WhitespaceSensitivity};
use oxc_formatter_tests::{OptionSet, apply_core_options};

/// Applies the four core options plus the HTML-specific keys onto `options`.
/// Parsing is lenient like `apply_core_options`: unknown or invalid values are ignored.
pub fn apply_htmlish_options(options: &mut HtmlishFormatOptions, json: &OptionSet) {
    apply_core_options(options, json);
    for (key, value) in json {
        match key.as_str() {
            "htmlWhitespaceSensitivity" => {
                if let Some(s) = value.as_str() {
                    options.whitespace_sensitivity = match s {
                        "strict" => WhitespaceSensitivity::Strict,
                        "ignore" => WhitespaceSensitivity::Ignore,
                        _ => WhitespaceSensitivity::Css,
                    };
                }
            }
            "bracketSameLine" => {
                if let Some(b) = value.as_bool() {
                    options.bracket_same_line = b;
                }
            }
            "singleAttributePerLine" => {
                if let Some(b) = value.as_bool() {
                    options.single_attribute_per_line = b;
                }
            }
            _ => {}
        }
    }
}
