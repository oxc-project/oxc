use std::path::Path;

use oxc_allocator::Allocator;
use oxc_formatter_json::{JsonFormatOptions, format};
use oxc_formatter_tests::{FixtureFormatter, OptionSet, build_fixture_snapshot};

mod options;
use options::apply_json_options;

struct JsonHarness;

impl FixtureFormatter for JsonHarness {
    type Options = JsonFormatOptions;

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
