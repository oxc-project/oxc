use std::path::Path;

use oxc_allocator::Allocator;
use oxc_formatter::JsFormatOptions;
use oxc_formatter_tests::{FixtureFormatter, OptionSet, build_fixture_snapshot};
use oxc_span::SourceType;

mod options;
use options::apply_js_options;

struct JsHarness;

impl FixtureFormatter for JsHarness {
    type Options = JsFormatOptions;

    fn parse_options(json: &OptionSet) -> Self::Options {
        let mut options = JsFormatOptions::default();
        apply_js_options(&mut options, json);
        options
    }

    fn format(source: &str, path: &Path, options: &Self::Options) -> String {
        let source_type = SourceType::from_path(path).unwrap();
        let allocator = Allocator::default();
        oxc_formatter::format(&allocator, source, source_type, options.clone())
            .unwrap()
            .print()
            .unwrap()
            .into_code()
    }
}

fn test_file(path: &Path) {
    // `insta::assert_snapshot!` is invoked from this file so the snapshot's
    // `source:` header records this consumer crate, not the shared harness.
    let snap = build_fixture_snapshot::<JsHarness>(path);
    insta::with_settings!({
        snapshot_path => snap.path,
        prepend_module_to_snapshot => false,
        snapshot_suffix => "",
        omit_expression => true,
    }, {
        insta::assert_snapshot!(snap.name, snap.body);
    });
}

/// A leading BOM is preserved (Prettier does the same);
/// `oxc_parser` keeps it in the source and the root re-emits it at byte 0.
#[test]
fn bom_is_preserved() {
    let allocator = Allocator::default();
    let formatted = oxc_formatter::format(
        &allocator,
        "\u{feff}let a = 1",
        SourceType::mjs(),
        JsFormatOptions::default(),
    )
    .expect("BOM input should parse")
    .print()
    .expect("print should succeed")
    .into_code();
    assert_eq!(formatted, "\u{feff}let a = 1;\n");
}

// Include auto-generated test functions from build.rs
include!(concat!(env!("OUT_DIR"), "/generated_tests.rs"));
