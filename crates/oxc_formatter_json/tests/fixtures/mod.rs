use std::path::Path;

use oxc_allocator::Allocator;
use oxc_formatter_json::{
    BracketSpacing, Expand, JsonFormatOptions, JsonVariant, QuoteProps, TrailingCommas, format,
    parse_for_format,
};
use oxc_formatter_tests::{FixtureFormatter, OptionValue, build_fixture_snapshot};

struct JsonHarness;

/// What formatting must leave unchanged, see `FixtureFormatter::Fingerprint`.
#[derive(Debug, PartialEq)]
struct Fingerprint {
    comments: usize,
}

impl FixtureFormatter for JsonHarness {
    type Options = JsonFormatOptions;
    type Fingerprint = Fingerprint;

    fn apply_option(options: &mut Self::Options, key: &str, value: &OptionValue) -> bool {
        match key {
            // Fixture-only key, Prettier has no `variant` option
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
                    // in the harness, the JSON type itself knows no "es5".
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
            _ => return false,
        }
        true
    }

    fn format(source: &str, _path: &Path, options: &Self::Options) -> String {
        let allocator = Allocator::default();
        format(&allocator, source, *options)
            .expect("format should succeed")
            .print()
            .expect("print should succeed")
            .into_code()
    }

    fn fingerprint(source: &str, _path: &Path, options: &Self::Options) -> Fingerprint {
        let allocator = Allocator::default();
        let parsed = parse_for_format(&allocator, source, *options).expect("source should parse");
        Fingerprint { comments: parsed.comments.len() }
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

// ---

/// JSON-invalid syntax the lenient JS parse accepts must surface as `Err` (the oxfmt fallback trigger).
#[test]
fn invalid_json_is_err() {
    // `json` / `jsonc` / `json5` share one printer, `json-stringify` has its own
    for variant in [JsonVariant::Json, JsonVariant::JsonStringify] {
        let options = JsonFormatOptions { variant, ..Default::default() };
        // Unary other than `+` / `-` before a number, `Infinity` or `NaN`
        for source in ["!true", "typeof Infinity", "-true", "-undefined", "-(-1)", "+ +1"] {
            let allocator = Allocator::default();
            assert!(format(&allocator, source, options).is_err(), "{variant:?}: {source:?}");
        }
    }
}
