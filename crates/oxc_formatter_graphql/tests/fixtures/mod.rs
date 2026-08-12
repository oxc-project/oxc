use std::path::Path;

use oxc_allocator::Allocator;
use oxc_formatter_graphql::{GraphqlFormatOptions, format};
use oxc_formatter_tests::{FixtureFormatter, OptionSet, build_fixture_snapshot};

mod options;
use options::apply_graphql_options;

struct GraphqlHarness;

impl FixtureFormatter for GraphqlHarness {
    type Options = GraphqlFormatOptions;

    fn parse_options(json: &OptionSet) -> Self::Options {
        let mut options = GraphqlFormatOptions::default();
        apply_graphql_options(&mut options, json);
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
    for source in ["query {{{", "", "# comments-only"] {
        assert!(
            format(&allocator, source, GraphqlFormatOptions::default()).is_err(),
            "{source:?} should fail to format"
        );
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
