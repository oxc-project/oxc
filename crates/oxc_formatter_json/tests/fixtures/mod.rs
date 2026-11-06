use std::path::Path;

use oxc_allocator::Allocator;
use oxc_formatter_json::{
    ArrayExpand, ArrayLinePattern, BracketSpacing, Expand, JsonFormatOptions, JsonVariant,
    QuoteProps, TrailingCommas, format, parse_for_format,
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
            // NOTE: Not a Prettier option
            "arrayWrap" => {
                if let Some(s) = value.as_str() {
                    options.array_expand = match s {
                        "auto" => ArrayExpand::Auto,
                        "preserve" => ArrayExpand::Preserve,
                        "collapse" => ArrayExpand::Never,
                        _ => options.array_expand,
                    };
                } else if let Some(object) = value.as_object() {
                    options.array_expand = object
                        .get("wrapThreshold")
                        .and_then(OptionValue::as_u64)
                        .and_then(|threshold| u32::try_from(threshold).ok())
                        .map_or(ArrayExpand::Preserve, ArrayExpand::ForceAboveThreshold);
                    options.array_line_pattern = object
                        .get("linePattern")
                        .and_then(OptionValue::as_str)
                        .and_then(|pattern| {
                            pattern
                                .split_whitespace()
                                .map(str::parse)
                                .collect::<Result<_, _>>()
                                .ok()
                        })
                        .and_then(ArrayLinePattern::new);
                }
            }
            _ => return false,
        }
        true
    }

    fn format(source: &str, _path: &Path, options: &Self::Options) -> String {
        let allocator = Allocator::default();
        format(&allocator, source, options.clone())
            .expect("format should succeed")
            .print()
            .expect("print should succeed")
            .into_code()
    }

    fn fingerprint(source: &str, _path: &Path, options: &Self::Options) -> Fingerprint {
        let allocator = Allocator::default();
        let parsed = parse_for_format(&allocator, source, options).expect("source should parse");
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
            assert!(
                format(&allocator, source, options.clone()).is_err(),
                "{variant:?}: {source:?}"
            );
        }
    }
}
