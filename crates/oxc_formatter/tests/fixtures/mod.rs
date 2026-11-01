use std::{path::Path, str::FromStr};

use oxc_allocator::Allocator;
use oxc_formatter::{
    BracketSpacing, JsFormatOptions, JsdocOptions, OperatorPosition, QuoteProperties, QuoteStyle,
    Semicolons, TrailingCommas,
};
use oxc_formatter_tests::{FixtureFormatter, OptionValue, build_fixture_snapshot};
use oxc_span::SourceType;

struct JsHarness;

/// What formatting must leave unchanged, see `FixtureFormatter::Fingerprint`.
#[derive(Debug, PartialEq)]
struct Fingerprint {
    comments: usize,
}

impl FixtureFormatter for JsHarness {
    type Options = JsFormatOptions;
    type Fingerprint = Fingerprint;

    fn apply_option(options: &mut Self::Options, key: &str, value: &OptionValue) -> bool {
        match key {
            "semi" => {
                if let Some(b) = value.as_bool() {
                    options.semicolons = if b { Semicolons::Always } else { Semicolons::AsNeeded };
                }
            }
            "bracketSpacing" => {
                if let Some(b) = value.as_bool() {
                    options.bracket_spacing = BracketSpacing::from(b);
                }
            }
            "singleQuote" => {
                if let Some(b) = value.as_bool() {
                    options.quote_style = if b { QuoteStyle::Single } else { QuoteStyle::Double };
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
            "quoteProps" => {
                if let Some(s) = value.as_str() {
                    options.quote_properties = QuoteProperties::from_str(s).unwrap_or_default();
                }
            }
            "experimentalOperatorPosition" => {
                if let Some(s) = value.as_str() {
                    options.operator_position = OperatorPosition::from_str(s).unwrap_or_default();
                }
            }
            // NOTE: Not a Prettier option
            // fixture test only toggle enabling JSDoc formatting for the fixtures under `js/jsdoc/`.
            // Deliberately a bool, an object form not yet supported.
            "jsdoc" => {
                if value.as_bool() == Some(true) {
                    options.jsdoc = Some(JsdocOptions::default());
                }
            }
            _ => return false,
        }
        true
    }

    fn format(source: &str, path: &Path, options: &Self::Options) -> String {
        let source_type = SourceType::from_path(path).unwrap();
        let allocator = Allocator::default();
        oxc_formatter::format(&allocator, source, source_type, options.clone())
            .unwrap()
            .print()
            .unwrap()
            .into_code()
    }

    fn fingerprint(source: &str, path: &Path, options: &Self::Options) -> Fingerprint {
        let source_type = SourceType::from_path(path).unwrap();
        let allocator = Allocator::default();
        let ret = oxc_formatter::parse_for_format(&allocator, source, source_type);
        // The `jsdoc` option owns JSDoc blocks: it rewrites them and drops empty ones,
        // so only the other comments are under the lossless contract there.
        let jsdoc_rewritten = options.jsdoc.is_some();
        Fingerprint {
            comments: ret
                .program
                .comments
                .iter()
                .filter(|c| !(jsdoc_rewritten && c.is_jsdoc()))
                .count(),
        }
    }
}

fn test_file(path: &Path) {
    // `insta::assert_snapshot!` is invoked from this file so the snapshot's
    // `source:` header records this consumer crate, not the shared harness.
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

/// A leading BOM is preserved (Prettier does the same);
/// `oxc_parser` keeps it in the source and the root re-emits it at byte 0.
#[test]
fn bom_is_preserved() {
    let allocator = Allocator::default();
    let formatted = oxc_formatter::format(
        &allocator,
        "\u{feff}let a = 1",
        SourceType::mjs(),
        JsFormatOptions::default(),
    )
    .expect("BOM input should parse")
    .print()
    .expect("print should succeed")
    .into_code();
    assert_eq!(formatted, "\u{feff}let a = 1;\n");
}

// Include auto-generated test functions from build.rs
include!(concat!(env!("OUT_DIR"), "/generated_tests.rs"));
