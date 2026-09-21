use std::path::Path;

use oxc_allocator::Allocator;
use oxc_formatter_markdown::{MarkdownFormatOptions, format, parse_for_format};
use oxc_formatter_tests::{FixtureFormatter, OptionSet, build_fixture_snapshot};

mod fingerprint;
mod options;
use options::apply_markdown_options;

struct MarkdownHarness;

impl FixtureFormatter for MarkdownHarness {
    type Options = MarkdownFormatOptions;
    /// The semantic fingerprint (see `fingerprint.rs`): `parse(format(x)) ≅ parse(x)`.
    type Fingerprint = String;

    fn parse_options(json: &OptionSet) -> Self::Options {
        let mut options = MarkdownFormatOptions::default();
        apply_markdown_options(&mut options, json);
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

    fn fingerprint(source: &str, _path: &Path, _options: &Self::Options) -> String {
        let allocator = Allocator::default();
        let parsed = parse_for_format(&allocator, source).expect("source should parse");
        fingerprint::fingerprint(parsed.source, parsed.root)
    }
}

fn test_file(path: &Path) {
    // `insta::assert_snapshot!` is invoked from this file so the snapshot's
    // `source:` header records this consumer crate, not the shared harness.
    let snap = build_fixture_snapshot::<MarkdownHarness>(path);
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

/// Prettier prints an empty document as the empty string.
#[test]
fn empty_document() {
    let allocator = Allocator::default();
    for source in ["", "\n", "\n\n  \n"] {
        let formatted = format(&allocator, source, MarkdownFormatOptions::default())
            .expect("empty input should parse")
            .print()
            .expect("print should succeed")
            .into_code();
        assert_eq!(formatted, "", "{source:?}");
    }
}

/// A leading BOM is preserved (Prettier does the same).
#[test]
fn bom_is_preserved() {
    let allocator = Allocator::default();
    let formatted = format(&allocator, "\u{feff}# Title", MarkdownFormatOptions::default())
        .expect("BOM input should parse")
        .print()
        .expect("print should succeed")
        .into_code();
    assert_eq!(formatted, "\u{feff}# Title\n");
}
