use std::path::Path;

use oxc_allocator::Allocator;
use oxc_formatter_graphql::{BracketSpacing, GraphqlFormatOptions, format, parse_for_format};
use oxc_formatter_tests::{FixtureFormatter, OptionValue, build_fixture_snapshot};

struct GraphqlHarness;

/// What formatting must leave unchanged, see `FixtureFormatter::Fingerprint`.
#[derive(Debug, PartialEq)]
struct Fingerprint {
    comments: usize,
}

impl FixtureFormatter for GraphqlHarness {
    type Options = GraphqlFormatOptions;
    type Fingerprint = Fingerprint;

    fn apply_option(options: &mut Self::Options, key: &str, value: &OptionValue) -> bool {
        match key {
            "bracketSpacing" => {
                if let Some(b) = value.as_bool() {
                    options.bracket_spacing = BracketSpacing::from(b);
                }
            }
            _ => return false,
        }
        true
    }

    fn format(
        source: &str,
        _path: &Path,
        options: &Self::Options,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let allocator = Allocator::default();
        Ok(format(&allocator, source, *options)?.print()?.into_code())
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
    let snap = build_fixture_snapshot::<GraphqlHarness>(path);
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

/// Any parse error must surface as `Err` (the oxfmt fallback trigger),
/// even though oxc-graphql-parser itself is error-tolerant.
#[test]
fn parse_error_is_err() {
    let allocator = Allocator::default();
    for source in ["query {{{", "# comments-only"] {
        assert!(
            format(&allocator, source, GraphqlFormatOptions::default()).is_err(),
            "{source:?} should fail to format"
        );
    }
}

/// An empty document is a parse error, but a whitespace-only file formats to empty (Prettier does the same).
#[test]
fn whitespace_only_is_empty() {
    let allocator = Allocator::default();
    for (source, expected) in
        [("", ""), (" \n", ""), ("\u{feff}", "\u{feff}"), ("\u{feff} \n", "\u{feff}")]
    {
        let formatted = format(&allocator, source, GraphqlFormatOptions::default())
            .expect("whitespace-only input should format")
            .print()
            .expect("print should succeed")
            .into_code();
        assert_eq!(formatted, expected, "{source:?}");
    }
}

/// A leading BOM is preserved (Prettier does the same).
#[test]
fn bom_is_preserved() {
    let allocator = Allocator::default();
    let formatted = format(&allocator, "\u{feff}{ a }", GraphqlFormatOptions::default())
        .expect("BOM input should parse")
        .print()
        .expect("print should succeed")
        .into_code();
    assert_eq!(formatted, "\u{feff}{\n  a\n}\n");
}
