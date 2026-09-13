//! Prettier conformance for CSS / SCSS / Less.
//!
//! Compares output against the Prettier suite's `tests/format/{css,scss,less}` snapshots
//! via `oxc_formatter_tests::conformance`; each dialect pins its failure report with `insta`.
//!
//! Debug a specific test: `PRETTIER_FILTER=<substring> cargo test -p oxc_formatter_css --test conformance -- --nocapture`

use std::path::Path;

use oxc_allocator::Allocator;
use oxc_formatter_core::{CoreFormatOptions, FormatOptions as _, LineWidth};
use oxc_formatter_css::{CssFormatOptions, CssVariant, format};
use oxc_formatter_tests::{
    OptionSet,
    conformance::{ConformanceConfig, run_conformance},
};

#[path = "fixtures/options.rs"]
mod options;
use options::apply_css_options;

const CSS: ConformanceConfig = ConformanceConfig {
    language: "css",
    fixture_roots: &["css"],
    exact_parser: Some("css"),
    ignore: &[
        // postcss-conditionals (archived: https://github.com/andyjansson/postcss-conditionals).
        "css/atrule/if-else.css",
        // YAML frontmatter
        "css/yaml/dirty.css",
        // range formatting / IDE cursor, not whole-file formatting
        "css/range/",
        "css/cursor/",
    ],
    skip_spec: None,
};

const SCSS: ConformanceConfig = ConformanceConfig {
    language: "scss",
    fixture_roots: &["scss"],
    exact_parser: Some("scss"),
    ignore: &[],
    skip_spec: None,
};

const LESS: ConformanceConfig = ConformanceConfig {
    language: "less",
    fixture_roots: &["less"],
    exact_parser: Some("less"),
    ignore: &[],
    skip_spec: None,
};

fn parse_options(variant: CssVariant, spec: &OptionSet) -> CssFormatOptions {
    let mut options = CssFormatOptions { variant, ..CssFormatOptions::default() };
    // Prettier's default `printWidth` is 80 (oxc defaults to 100); the spec's own
    // `printWidth`/`tabWidth`/`useTabs`/`endOfLine` then override inside `apply_css_options`.
    options.apply_core(CoreFormatOptions {
        line_width: LineWidth::try_from(80).unwrap(),
        ..CoreFormatOptions::default()
    });
    apply_css_options(&mut options, spec);
    options
}

fn format_css(variant: CssVariant, source_text: &str, spec: &OptionSet) -> Option<String> {
    let options = parse_options(variant, spec);
    let allocator = Allocator::default();
    let formatted = format(&allocator, source_text, options).ok()?;
    Some(formatted.print().ok()?.into_code())
}

#[test]
fn prettier_conformance_css() {
    let format =
        |_: &Path, source: &str, spec: &OptionSet| format_css(CssVariant::Css, source, spec);
    let Some(report) = run_conformance(&CSS, format) else { return };
    insta::assert_snapshot!("prettier-css", report);
}

#[test]
fn prettier_conformance_scss() {
    let format =
        |_: &Path, source: &str, spec: &OptionSet| format_css(CssVariant::Scss, source, spec);
    let Some(report) = run_conformance(&SCSS, format) else { return };
    insta::assert_snapshot!("prettier-scss", report);
}

#[test]
fn prettier_conformance_less() {
    let format =
        |_: &Path, source: &str, spec: &OptionSet| format_css(CssVariant::Less, source, spec);
    let Some(report) = run_conformance(&LESS, format) else { return };
    insta::assert_snapshot!("prettier-less", report);
}
