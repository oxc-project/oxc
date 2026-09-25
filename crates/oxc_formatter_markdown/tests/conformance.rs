//! Prettier conformance for Markdown.
//!
//! Compares output against the Prettier suite's `tests/format/markdown` snapshots via `oxc_formatter_tests::conformance`;
//! the failure report is pinned with `insta`.
//!
//! Debug a specific test: `PRETTIER_FILTER=<substring> cargo test -p oxc_formatter_markdown --test conformance -- --nocapture`

use std::path::Path;

use oxc_allocator::Allocator;
use oxc_formatter_core::LineWidth;
use oxc_formatter_markdown::{MarkdownFormatOptions, format};
use oxc_formatter_tests::{
    OptionSet,
    conformance::{ConformanceConfig, run_conformance},
};

#[path = "fixtures/options.rs"]
mod options;
use options::apply_markdown_options;

const CONFIG: ConformanceConfig = ConformanceConfig {
    language: "markdown",
    fixture_roots: &["markdown"],
    exact_parser: Some("markdown"),
    ignore: &[
        // Cursor tracking is an editor feature, not formatting.
        "markdown/cursor/",
        // Exercises Prettier's plugin loader.
        "markdown/broken-plugins/",
        // Embedded languages need the dispatcher; covered by oxfmt's E2E conformance.
        "markdown/multiparser-css/",
        "markdown/multiparser-js/",
        "markdown/multiparser-json/",
        "markdown/jsx-semi/",
        // Test the front matter's body, which the dispatcher formats (`front-matter/` covers the envelope itself).
        "markdown/yaml/",
        "markdown/toml/",
        // Mostly fences Prettier formats through its embed; the fence itself is covered by the commonmark suite.
        "markdown/code/",
        // Fenced blocks whose only difference is Prettier formatting the embedded language (js / ts / css / json / html / markdown).
        "markdown/blockquote/code.md",
        // Also the `always`-only trailing `>` line: DIVERGENCES.md#ignored-block-trailing-quote-line.
        "markdown/blockquote/ignore-code.md",
        "markdown/list/codeblock.md",
        "markdown/list/parser-regression/issue-11202.md",
        "markdown/markdown/real-world-case.md",
        "markdown/markdown/test-case.md",
    ],
    skip_spec: None,
};

fn parse_options(spec: &OptionSet) -> MarkdownFormatOptions {
    // Prettier's default `printWidth` is 80 (oxc defaults to 100); the spec's own
    // `printWidth`/`tabWidth`/`useTabs`/`endOfLine` then override inside `apply_markdown_options`.
    let mut options = MarkdownFormatOptions {
        line_width: LineWidth::try_from(80).unwrap(),
        ..MarkdownFormatOptions::default()
    };
    apply_markdown_options(&mut options, spec);
    options
}

fn format_markdown(_path: &Path, source_text: &str, spec: &OptionSet) -> Option<String> {
    let options = parse_options(spec);
    let allocator = Allocator::default();
    let formatted = format(&allocator, source_text, options).ok()?;
    Some(formatted.print().ok()?.into_code())
}

#[test]
fn prettier_conformance() {
    let Some(report) = run_conformance(&CONFIG, format_markdown) else { return };
    insta::assert_snapshot!("prettier-markdown", report);
}
