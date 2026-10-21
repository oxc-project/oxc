use std::path::Path;

use oxc_allocator::Allocator;
use oxc_formatter_json::{JsonFormatOptions, JsonVariant, format, parse_for_format};
use oxc_formatter_tests::{FixtureFormatter, OptionSet, build_fixture_snapshot};

mod options;
use options::apply_json_options;

struct JsonHarness;

/// What formatting must leave unchanged, see `FixtureFormatter::Fingerprint`.
#[derive(Debug, PartialEq)]
struct Fingerprint {
    comments: usize,
}

impl FixtureFormatter for JsonHarness {
    type Options = JsonFormatOptions;
    type Fingerprint = Fingerprint;

    fn parse_options(json: &OptionSet) -> Self::Options {
        let mut options = JsonFormatOptions::default();
        apply_json_options(&mut options, json);
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
