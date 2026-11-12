use std::path::Path;

use cow_utils::CowUtils;
use oxc_allocator::Allocator;
use oxc_formatter_markdown::{
    MarkdownFormatOptions, MarkdownTableStyle, ProseWrap, SingleQuote, format, parse_for_format,
};
use oxc_formatter_tests::{FixtureFormatter, OptionValue, build_fixture_snapshot};

mod fingerprint;

struct MarkdownHarness;

impl FixtureFormatter for MarkdownHarness {
    type Options = MarkdownFormatOptions;
    /// The semantic fingerprint (see `fingerprint.rs`): `parse(format(x)) ≅ parse(x)`.
    type Fingerprint = String;

    fn apply_option(options: &mut Self::Options, key: &str, value: &OptionValue) -> bool {
        match key {
            "markdownTableStyle" => {
                if let Some(s) = value.as_str() {
                    options.table_style = match s {
                        "compact" => MarkdownTableStyle::Compact,
                        _ => MarkdownTableStyle::Auto,
                    };
                }
            }
            "proseWrap" => {
                if let Some(s) = value.as_str() {
                    options.prose_wrap = match s {
                        "always" => ProseWrap::Always,
                        "never" => ProseWrap::Never,
                        _ => ProseWrap::Preserve,
                    };
                }
            }
            "singleQuote" => {
                if let Some(b) = value.as_bool() {
                    options.single_quote = SingleQuote::from(b);
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

/// Changing one cell in a compact table must not re-pad any other row.
#[test]
fn compact_table_edit_only_changes_its_row() {
    let source = include_str!("markdown/table-style/compact/tables.md");
    let edited =
        source.cow_replace("short value", "a value longer than every other cell in this column");
    for prose_wrap in [ProseWrap::Preserve, ProseWrap::Always, ProseWrap::Never] {
        let options = MarkdownFormatOptions {
            table_style: MarkdownTableStyle::Compact,
            prose_wrap,
            ..Default::default()
        };
        let format_source = |source: &str| {
            let allocator = Allocator::default();
            format(&allocator, source, options).unwrap().print().unwrap().into_code()
        };
        let before = format_source(source);
        let after = format_source(&edited);
        assert_eq!(before.lines().count(), after.lines().count());
        let changed_rows: Vec<_> = before
            .lines()
            .zip(after.lines())
            .enumerate()
            .filter_map(|(i, (before, after))| (before != after).then_some(i))
            .collect();
        assert_eq!(changed_rows, [2], "{prose_wrap:?}");
        assert_eq!(format_source(&after), after, "{prose_wrap:?}");
    }
}
