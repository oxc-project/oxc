use std::path::Path;

use oxc_allocator::Allocator;
use oxc_formatter_tests::{FixtureFormatter, OptionValue, build_fixture_snapshot};
use oxc_formatter_yaml::{ProseWrap, YamlFormatOptions, format, parse_for_format};

struct YamlHarness;

/// What formatting must leave unchanged, see `FixtureFormatter::Fingerprint`.
#[derive(Debug, PartialEq)]
struct Fingerprint {
    comments: usize,
}

impl FixtureFormatter for YamlHarness {
    type Options = YamlFormatOptions;
    type Fingerprint = Fingerprint;

    fn apply_option(options: &mut Self::Options, key: &str, value: &OptionValue) -> bool {
        match key {
            "proseWrap" => {
                if let Some(s) = value.as_str() {
                    options.prose_wrap = match s {
                        "always" => ProseWrap::Always,
                        "never" => ProseWrap::Never,
                        _ => ProseWrap::Preserve,
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

    fn fingerprint(source: &str, _path: &Path, _options: &Self::Options) -> Fingerprint {
        let allocator = Allocator::default();
        let parsed = parse_for_format(&allocator, source).expect("source should parse");
        Fingerprint { comments: parsed.comments.len() }
    }
}

fn test_file(path: &Path) {
    // `insta::assert_snapshot!` is invoked from this file so the snapshot's
    // `source:` header records this consumer crate, not the shared harness.
    let snap = build_fixture_snapshot::<YamlHarness>(path);
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

/// Any parse error must surface as `Err` (standalone callers report it as a
/// diagnostic; embedded callers fall back to verbatim).
#[test]
fn parse_error_is_err() {
    let allocator = Allocator::default();
    for source in ["key: [a, b", "key: \"unterminated"] {
        assert!(
            format(&allocator, source, YamlFormatOptions::default()).is_err(),
            "{source:?} should fail to format"
        );
    }
}

/// A leading BOM is preserved (Prettier does the same).
#[test]
fn bom_is_preserved() {
    let allocator = Allocator::default();
    let formatted = format(&allocator, "\u{feff}key: value", YamlFormatOptions::default())
        .expect("BOM input should parse")
        .print()
        .expect("print should succeed")
        .into_code();
    assert_eq!(formatted, "\u{feff}key: value\n");
}
