//! Prettier conformance for YAML.
//!
//! Compares output against the Prettier suite's `tests/format/yaml` snapshots via `oxc_formatter_tests::conformance`;
//! the failure report is pinned with `insta`.
//!
//! Debug a specific test: `PRETTIER_FILTER=<substring> cargo test -p oxc_formatter_yaml --test conformance -- --nocapture`

use std::path::Path;

use oxc_allocator::Allocator;
use oxc_formatter_core::{CoreFormatOptions, FormatOptions as _, LineWidth};
use oxc_formatter_tests::{
    OptionSet,
    conformance::{ConformanceConfig, run_conformance},
};
use oxc_formatter_yaml::{YamlFormatOptions, format};

#[path = "fixtures/options.rs"]
mod options;
use options::apply_yaml_options;

const CONFIG: ConformanceConfig = ConformanceConfig {
    language: "yaml",
    fixture_roots: &["yaml"],
    exact_parser: Some("yaml"),
    ignore: &[
        // Prettier's yaml parser rejects these (https://github.com/eemeli/yaml/issues/646),
        // so no snapshot exists (`3-style.yml` is even marked `errors` in its format.test.js).
        // oxc-yaml-parser parses them fine, but there is nothing to compare against.
        "yaml/mapping/3-style.yml",
        "yaml/spec/spec-example-2-11-mapping-between-sequences.yml",
        // Pragma support (`@format` insertion / require)
        "yaml/insert-pragma/",
        "yaml/require-pragma/",
    ],
    skip_spec: None,
};

fn parse_options(spec: &OptionSet) -> YamlFormatOptions {
    let mut options = YamlFormatOptions::default();
    // Prettier's default `printWidth` is 80 (oxc defaults to 100); the spec's own
    // `printWidth`/`tabWidth`/`useTabs`/`endOfLine` then override inside `apply_yaml_options`.
    options.apply_core(CoreFormatOptions {
        line_width: LineWidth::try_from(80).unwrap(),
        ..CoreFormatOptions::default()
    });
    apply_yaml_options(&mut options, spec);
    options
}

fn format_yaml(_path: &Path, source_text: &str, spec: &OptionSet) -> Option<String> {
    let options = parse_options(spec);
    let allocator = Allocator::default();
    let formatted = format(&allocator, source_text, options).ok()?;
    Some(formatted.print().ok()?.into_code())
}

#[test]
fn prettier_conformance() {
    let Some(report) = run_conformance(&CONFIG, format_yaml) else { return };
    insta::assert_snapshot!("prettier-yaml", report);
}
