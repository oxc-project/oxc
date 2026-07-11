use std::path::Path;

use oxc_allocator::Allocator;
use oxc_formatter::{
    ArrayExpand, ArrowParentheses, BracketSameLine, BracketSpacing, JsFormatOptions, JsdocOptions,
    QuoteProperties, QuoteStyle, Semicolons, TrailingCommas,
};
use oxc_formatter_core::{
    IndentStyle, IndentWidth, LineEnding, LineWidth,
    test_support::{FixtureFormatter, OptionSet, build_fixture_snapshot},
};
use oxc_span::SourceType;

struct JsHarness;

impl FixtureFormatter for JsHarness {
    type Options = JsFormatOptions;

    fn parse_options(json: &OptionSet) -> Self::Options {
        let mut options = JsFormatOptions::default();

        for (key, value) in json {
            match key.as_str() {
                "semi" => {
                    if let Some(b) = value.as_bool() {
                        options.semicolons =
                            if b { Semicolons::Always } else { Semicolons::AsNeeded };
                    }
                }
                "singleQuote" => {
                    if let Some(b) = value.as_bool() {
                        options.quote_style =
                            if b { QuoteStyle::Single } else { QuoteStyle::Double };
                    }
                }
                "jsxSingleQuote" => {
                    if let Some(b) = value.as_bool() {
                        options.jsx_quote_style =
                            if b { QuoteStyle::Single } else { QuoteStyle::Double };
                    }
                }
                "arrowParens" => {
                    if let Some(s) = value.as_str() {
                        options.arrow_parentheses = match s {
                            "always" => ArrowParentheses::Always,
                            "avoid" => ArrowParentheses::AsNeeded,
                            _ => options.arrow_parentheses,
                        };
                    }
                }
                "trailingComma" => {
                    if let Some(s) = value.as_str() {
                        options.trailing_commas = match s {
                            "none" => TrailingCommas::None,
                            "es5" => TrailingCommas::Es5,
                            "all" => TrailingCommas::All,
                            _ => options.trailing_commas,
                        };
                    }
                }
                "printWidth" => {
                    if let Some(n) = value.as_u64()
                        && let Ok(width) = LineWidth::try_from(u16::try_from(n).unwrap())
                    {
                        options.line_width = width;
                    }
                }
                "tabWidth" => {
                    if let Some(n) = value.as_u64()
                        && let Ok(width) = IndentWidth::try_from(u8::try_from(n).unwrap())
                    {
                        options.indent_width = width;
                    }
                }
                "useTabs" => {
                    if let Some(b) = value.as_bool() {
                        options.indent_style =
                            if b { IndentStyle::Tab } else { IndentStyle::Space };
                    }
                }
                "bracketSpacing" => {
                    if let Some(b) = value.as_bool() {
                        options.bracket_spacing = BracketSpacing::from(b);
                    }
                }
                "bracketSameLine" | "jsxBracketSameLine" => {
                    if let Some(b) = value.as_bool() {
                        options.bracket_same_line = BracketSameLine::from(b);
                    }
                }
                "endOfLine" => {
                    if let Some(s) = value.as_str() {
                        options.line_ending = match s {
                            "lf" => LineEnding::Lf,
                            "crlf" => LineEnding::Crlf,
                            "cr" => LineEnding::Cr,
                            _ => LineEnding::default(),
                        };
                    }
                }
                "quoteProps" => {
                    if let Some(s) = value.as_str() {
                        options.quote_properties = match s {
                            "as-needed" => QuoteProperties::AsNeeded,
                            "preserve" => QuoteProperties::Preserve,
                            "consistent" => QuoteProperties::Consistent,
                            _ => QuoteProperties::default(),
                        };
                    }
                }
                "jsdoc" if value.is_object() => {
                    options.jsdoc = Some(JsdocOptions::default());
                }
                "arrayWrap" => {
                    if let Some(s) = value.as_str() {
                        options.array_expand = match s {
                            "preserve" => ArrayExpand::Preserve,
                            "collapse" => ArrayExpand::Never,
                            _ => options.array_expand,
                        };
                    } else if let Some(object) = value.as_object() {
                        if let Some(threshold) =
                            object.get("minElementsToWrap").and_then(serde_json::Value::as_u64)
                        {
                            options.array_expand =
                                ArrayExpand::ForceAboveThreshold(u32::try_from(threshold).unwrap());
                        } else {
                            options.array_expand = ArrayExpand::Preserve;
                        }
                        if let Some(pattern) =
                            object.get("linePattern").and_then(serde_json::Value::as_str)
                        {
                            options.array_line_pattern = Some(pattern.parse().unwrap());
                        }
                    }
                }
                _ => {}
            }
        }

        options
    }

    fn format(source: &str, path: &Path, options: &Self::Options) -> String {
        let source_type = SourceType::from_path(path).unwrap();
        let allocator = Allocator::default();
        oxc_formatter::format(&allocator, source, source_type, options.clone(), None)
            .unwrap()
            .print()
            .unwrap()
            .into_code()
    }
}

fn test_file(path: &Path) {
    // `insta::assert_snapshot!` is invoked from this file so the snapshot's
    // `source:` header records this path (matching the pre-harness layout).
    let snap = build_fixture_snapshot::<JsHarness>(path);
    insta::with_settings!({
        snapshot_path => snap.path,
        prepend_module_to_snapshot => false,
        snapshot_suffix => "",
        omit_expression => true,
    }, {
        insta::assert_snapshot!(snap.name, snap.body);
    });
}

// Include auto-generated test functions from build.rs
include!(concat!(env!("OUT_DIR"), "/generated_tests.rs"));
