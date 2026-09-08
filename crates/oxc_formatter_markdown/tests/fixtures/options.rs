//! Prettier option-set → `MarkdownFormatOptions` mapping.
//!
//! Shared by the fixture harness and the conformance target via `#[path]`
//! (one source, no drift; see `oxc_formatter_tests`'s AGENTS.md).

use oxc_formatter_core::IndentWidth;
use oxc_formatter_markdown::{MarkdownFormatOptions, ProseWrap};
use oxc_formatter_tests::{OptionSet, apply_core_options};

/// Applies the four core options plus the Markdown-specific keys onto `options`.
/// Parsing is lenient like `apply_core_options`: unknown or invalid values are ignored.
pub fn apply_markdown_options(options: &mut MarkdownFormatOptions, json: &OptionSet) {
    apply_core_options(options, json);
    // Prettier accepts any `tabWidth`; `IndentWidth` caps at 24.
    // Every markdown use of it clamps or takes a remainder,
    // so the cap prints the same (the suite's `tabWidth: 999` cases).
    if let Some(width) = json.get("tabWidth").and_then(serde_json::Value::as_u64)
        && width > u64::from(IndentWidth::MAX)
    {
        options.indent_width = IndentWidth::try_from(IndentWidth::MAX).unwrap();
    }

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
            _ => {}
        }
    }
}
