use std::path::Path;

use oxc_allocator::Allocator;
use oxc_formatter_core::{
    IndentStyle, IndentWidth, LineEnding, LineWidth,
    test_support::{FixtureFormatter, OptionSet, build_fixture_snapshot},
};
use oxc_formatter_json::{
    BracketSpacing, Expand, JsonFormatOptions, JsonVariant, QuoteProps, TrailingCommas, format,
};

struct JsonHarness;

impl FixtureFormatter for JsonHarness {
    type Options = JsonFormatOptions;

    fn parse_options(json: &OptionSet) -> Self::Options {
        let mut options = JsonFormatOptions::default();

        for (key, value) in json {
            match key.as_str() {
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
                "variant" => {
                    if let Some(s) = value.as_str() {
                        options.variant = match s {
                            "json" => JsonVariant::Json,
                            "jsonc" => JsonVariant::Jsonc,
                            "json5" => JsonVariant::Json5,
                            "json-stringify" => JsonVariant::JsonStringify,
                            _ => options.variant,
                        };
                    }
                }
                "trailingComma" => {
                    if let Some(s) = value.as_str() {
                        // Translate Prettier's vocabulary into JSON's neutral two states here,
                        // in the harness — the JSON type itself knows no "es5".
                        options.trailing_commas = match s {
                            "all" | "es5" => TrailingCommas::Always,
                            "none" => TrailingCommas::Never,
                            _ => options.trailing_commas,
                        };
                    }
                }
                "bracketSpacing" => {
                    if let Some(b) = value.as_bool() {
                        options.bracket_spacing = BracketSpacing::from(b);
                    }
                }
                "singleQuote" => {
                    if let Some(b) = value.as_bool() {
                        options.single_quote = b.into();
                    }
                }
                "quoteProps" => {
                    if let Some(s) = value.as_str() {
                        options.quote_props = match s {
                            "preserve" => QuoteProps::Preserve,
                            "consistent" => QuoteProps::Consistent,
                            _ => QuoteProps::AsNeeded,
                        };
                    }
                }
                "objectWrap" => {
                    if let Some(s) = value.as_str() {
                        options.expand = match s {
                            "preserve" => Expand::Auto,
                            "collapse" => Expand::Never,
                            _ => options.expand,
                        };
                    }
                }
                _ => {}
            }
        }

        options
    }

    fn format(source: &str, _path: &Path, options: &Self::Options) -> String {
        let allocator = Allocator::default();
        format(&allocator, source, *options)
            .expect("format should succeed")
            .print()
            .expect("print should succeed")
            .into_code()
    }
}

fn test_file(path: &Path) {
    // `insta::assert_snapshot!` is invoked from this file so the snapshot's
    // `source:` header records this consumer crate, not the shared harness.
    let snap = build_fixture_snapshot::<JsonHarness>(path);
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
