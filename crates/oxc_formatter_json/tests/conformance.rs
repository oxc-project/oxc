//! Prettier conformance for the JSON family (json / jsonc / json5 / json-stringify).
//!
//! Compares output against the Prettier suite's `tests/format/json` snapshots via
//! `oxc_formatter_tests::conformance`; each variant pins its failure report with `insta`.
//!
//! The `json/json` and `json/with-comment` dirs are shared between variants: each
//! `format.test.js` call lists its own parser, and `exact_parser` keeps only the
//! matching calls. Out-of-scope siblings (all variants):
//! - `json-superset/`: inline `snippets`, not parseable by the spec parser
//! - `range/`: range-formatting, not whole-file
//!
//! Debug a specific test: `PRETTIER_FILTER=<substring> cargo test -p oxc_formatter_json --test conformance -- --nocapture`

use std::path::Path;

use oxc_allocator::Allocator;
use oxc_formatter_core::{CoreFormatOptions, FormatOptions as _, LineWidth};
use oxc_formatter_json::{JsonFormatOptions, JsonVariant, format};
use oxc_formatter_tests::{
    OptionSet,
    conformance::{ConformanceConfig, run_conformance},
};

#[path = "fixtures/options.rs"]
mod options;
use options::apply_json_options;

const JSON: ConformanceConfig = ConformanceConfig {
    language: "json",
    fixture_roots: &["json/json", "json/with-comment"],
    exact_parser: Some("json"),
    ignore: &[],
    skip_spec: None,
};

const JSONC: ConformanceConfig = ConformanceConfig {
    language: "jsonc",
    fixture_roots: &["json/jsonc", "json/with-comment"],
    exact_parser: Some("jsonc"),
    ignore: &[],
    skip_spec: None,
};

const JSON5: ConformanceConfig = ConformanceConfig {
    language: "json5",
    fixture_roots: &["json/json", "json/with-comment", "json/json5-as-json-with-trailing-commas"],
    exact_parser: Some("json5"),
    ignore: &[],
    skip_spec: None,
};

const JSON_STRINGIFY: ConformanceConfig = ConformanceConfig {
    language: "json-stringify",
    fixture_roots: &["json/json"],
    exact_parser: Some("json-stringify"),
    ignore: &[],
    skip_spec: None,
};

fn parse_options(variant: JsonVariant, spec: &OptionSet) -> JsonFormatOptions {
    let mut options = JsonFormatOptions { variant, ..JsonFormatOptions::default() };
    // Prettier's default `printWidth` is 80 (oxc defaults to 100); the spec's own
    // `printWidth`/`tabWidth`/`useTabs`/`endOfLine` then override inside `apply_json_options`.
    options.apply_core(CoreFormatOptions {
        line_width: LineWidth::try_from(80).unwrap(),
        ..CoreFormatOptions::default()
    });
    apply_json_options(&mut options, spec);
    options
}

fn format_json(variant: JsonVariant, source_text: &str, spec: &OptionSet) -> Option<String> {
    let options = parse_options(variant, spec);
    let allocator = Allocator::default();
    let formatted = format(&allocator, source_text, options).ok()?;
    Some(formatted.print().ok()?.into_code())
}

#[test]
fn prettier_conformance_json() {
    let format =
        |_: &Path, source: &str, spec: &OptionSet| format_json(JsonVariant::Json, source, spec);
    let Some(report) = run_conformance(&JSON, format) else { return };
    insta::assert_snapshot!("prettier-json", report);
}

#[test]
fn prettier_conformance_jsonc() {
    let format =
        |_: &Path, source: &str, spec: &OptionSet| format_json(JsonVariant::Jsonc, source, spec);
    let Some(report) = run_conformance(&JSONC, format) else { return };
    insta::assert_snapshot!("prettier-jsonc", report);
}

#[test]
fn prettier_conformance_json5() {
    let format =
        |_: &Path, source: &str, spec: &OptionSet| format_json(JsonVariant::Json5, source, spec);
    let Some(report) = run_conformance(&JSON5, format) else { return };
    insta::assert_snapshot!("prettier-json5", report);
}

#[test]
fn prettier_conformance_json_stringify() {
    let format = |_: &Path, source: &str, spec: &OptionSet| {
        format_json(JsonVariant::JsonStringify, source, spec)
    };
    let Some(report) = run_conformance(&JSON_STRINGIFY, format) else { return };
    insta::assert_snapshot!("prettier-json-stringify", report);
}
