//! Prettier conformance for GraphQL.
//!
//! Compares output against the Prettier suite's `tests/format/graphql` snapshots via
//! `oxc_formatter_tests::conformance`; the failure report is pinned with `insta`.
//!
//! Debug a specific test: `PRETTIER_FILTER=<substring> cargo test -p oxc_formatter_graphql --test conformance -- --nocapture`

use std::path::Path;

use oxc_allocator::Allocator;
use oxc_formatter_core::{CoreFormatOptions, FormatOptions as _, LineWidth};
use oxc_formatter_graphql::{GraphqlFormatOptions, format};
use oxc_formatter_tests::{
    OptionSet,
    conformance::{ConformanceConfig, run_conformance},
};

#[path = "fixtures/options.rs"]
mod options;
use options::apply_graphql_options;

const CONFIG: ConformanceConfig = ConformanceConfig {
    language: "graphql",
    fixture_roots: &["graphql"],
    exact_parser: Some("graphql"),
    ignore: &[
        // range formatting, not whole-file formatting
        "graphql/range/",
    ],
    skip_spec: None,
};

fn parse_options(spec: &OptionSet) -> GraphqlFormatOptions {
    let mut options = GraphqlFormatOptions::default();
    // Prettier's default `printWidth` is 80 (oxc defaults to 100); the spec's own
    // `printWidth`/`tabWidth`/`useTabs`/`endOfLine` then override inside `apply_graphql_options`.
    options.apply_core(CoreFormatOptions {
        line_width: LineWidth::try_from(80).unwrap(),
        ..CoreFormatOptions::default()
    });
    apply_graphql_options(&mut options, spec);
    options
}

fn format_graphql(_path: &Path, source_text: &str, spec: &OptionSet) -> Option<String> {
    let options = parse_options(spec);
    let allocator = Allocator::default();
    let formatted = format(&allocator, source_text, options).ok()?;
    Some(formatted.print().ok()?.into_code())
}

#[test]
fn prettier_conformance() {
    let Some(report) = run_conformance(&CONFIG, format_graphql) else { return };
    insta::assert_snapshot!("prettier-graphql", report);
}
