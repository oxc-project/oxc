use std::path::Path;

use oxc_allocator::Allocator;
use oxc_formatter_core::{
    IndentStyle, IndentWidth, LineEnding, LineWidth,
    test_support::{FixtureFormatter, OptionSet, build_fixture_snapshot},
};
use oxc_formatter_graphql::{GraphqlFormatOptions, format};

struct GraphqlHarness;

impl FixtureFormatter for GraphqlHarness {
    type Options = GraphqlFormatOptions;

    fn parse_options(json: &OptionSet) -> Self::Options {
        let mut options = GraphqlFormatOptions::default();

        for (key, value) in json {
            match key.as_str() {
                "printWidth" => {
                    if let Some(n) = value.as_u64()
                        && let Ok(width) = LineWidth::try_from(u16::try_from(n).unwrap())
                    {
                        options.line_width = width;
                    }
                }
                "tabWidth" => {
                    if let Some(n) = value.as_u64()
                        && let Ok(width) = IndentWidth::try_from(u8::try_from(n).unwrap())
                    {
                        options.indent_width = width;
                    }
                }
                "useTabs" => {
                    if let Some(b) = value.as_bool() {
                        options.indent_style =
                            if b { IndentStyle::Tab } else { IndentStyle::Space };
                    }
                }
                "endOfLine" => {
                    if let Some(s) = value.as_str() {
                        options.line_ending = match s {
                            "lf" => LineEnding::Lf,
                            "crlf" => LineEnding::Crlf,
                            "cr" => LineEnding::Cr,
                            _ => LineEnding::default(),
                        };
                    }
                }
                "bracketSpacing" => {
                    if let Some(b) = value.as_bool() {
                        options.bracket_spacing = b.into();
                    }
                }
                _ => {}
            }
        }

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
/// even though apollo-parser itself is error-tolerant.
#[test]
fn parse_error_is_err() {
    let allocator = Allocator::default();
    for source in [
        "query {{{",
        "",
        "# comments-only",
        // Draft-spec syntax that Prettier (graphql-js) accepts but apollo-parser does not.
        "fragment F($x: Int) on T { f }",
    ] {
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
