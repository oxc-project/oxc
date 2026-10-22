use std::path::Path;

use oxc_allocator::Allocator;
use oxc_formatter_htmlish::{HtmlishFormatOptions, HtmlishVariant, format, parse_for_format};
use oxc_formatter_tests::{FixtureFormatter, OptionSet, build_fixture_snapshot};

mod fingerprint;
mod options;
use options::apply_htmlish_options;

/// `.vue` reads as Vue, `.xhtml` / `.xht` as XHTML, `.component.html` as Angular (as the example reads them).
fn variant_of(path: &Path) -> HtmlishVariant {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("vue") => HtmlishVariant::Vue,
        Some("xhtml" | "xht") => HtmlishVariant::Xhtml,
        _ if path.to_string_lossy().ends_with(".component.html") => HtmlishVariant::Angular,
        _ => HtmlishVariant::Html,
    }
}

struct HtmlishHarness;

impl FixtureFormatter for HtmlishHarness {
    type Options = HtmlishFormatOptions;
    type Fingerprint = String;

    fn parse_options(json: &OptionSet) -> Self::Options {
        let mut options = HtmlishFormatOptions::default();
        apply_htmlish_options(&mut options, json);
        options
    }

    fn format(source: &str, path: &Path, options: &Self::Options) -> String {
        let allocator = Allocator::default();
        let options = HtmlishFormatOptions { variant: variant_of(path), ..options.clone() };
        format(&allocator, source, options)
            .expect("format should succeed")
            .print()
            .expect("print should succeed")
            .into_code()
    }

    fn fingerprint(source: &str, path: &Path, options: &Self::Options) -> String {
        let options = HtmlishFormatOptions { variant: variant_of(path), ..options.clone() };
        let allocator = Allocator::default();
        let parsed = parse_for_format(&allocator, source, &options).expect("source should parse");
        fingerprint::fingerprint(
            parsed.source,
            parsed.root,
            options.variant == HtmlishVariant::Html,
        )
    }
}

fn test_file(path: &Path) {
    let snap = build_fixture_snapshot::<HtmlishHarness>(path);
    insta::with_settings!({
        snapshot_path => snap.path,
        prepend_module_to_snapshot => false,
        snapshot_suffix => "",
        omit_expression => true,
    }, {
        insta::assert_snapshot!(snap.name, snap.body);
    });
}

include!(concat!(env!("OUT_DIR"), "/generated_tests.rs"));
