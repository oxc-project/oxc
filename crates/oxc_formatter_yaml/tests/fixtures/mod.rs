use std::path::Path;

use oxc_allocator::Allocator;
use oxc_formatter_core::LineEnding;
use oxc_formatter_tests::{FixtureFormatter, OptionSet, build_fixture_snapshot};
use oxc_formatter_yaml::{YamlFormatOptions, format};

mod options;
use options::apply_yaml_options;

struct YamlHarness;

impl FixtureFormatter for YamlHarness {
    type Options = YamlFormatOptions;

    fn parse_options(json: &OptionSet) -> Self::Options {
        let mut options = YamlFormatOptions::default();
        apply_yaml_options(&mut options, json);
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

/// The configured `end_of_line` is applied to EVERY output line break,
/// including the blank-line runs emitted as raw `"\n"` text (block scalars,
/// flow scalars) — the snapshot harness cannot pin this (insta normalizes
/// line endings), so assert it directly.
#[test]
fn line_ending_is_applied() {
    let allocator = Allocator::default();
    let options =
        YamlFormatOptions { line_ending: LineEnding::Crlf, ..YamlFormatOptions::default() };
    let formatted = format(&allocator, "a: 1\nblock: |\n  x\n\n  y\nb: 2\n", options)
        .expect("input should parse")
        .print()
        .expect("print should succeed")
        .into_code();
    assert_eq!(formatted, "a: 1\r\nblock: |\r\n  x\r\n\r\n  y\r\nb: 2\r\n");
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
