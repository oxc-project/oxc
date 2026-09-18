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
        // Front matter and embedded languages need the dispatcher; covered by oxfmt's E2E conformance.
        "markdown/front-matter/",
        "markdown/yaml/",
        "markdown/toml/",
        "markdown/multiparser-css/",
        "markdown/multiparser-js/",
        "markdown/multiparser-json/",
        "markdown/jsx-semi/",
        // Prettier reads the leading `---` block as (empty) front matter and keeps it verbatim;
        // CommonMark alone sees a thematic break + setext heading. Recovered with the front matter pre-pass.
        "markdown/commonmark-test-suite/snippet: example-96.md",
        "markdown/commonmark-test-suite/snippet: example-98.md",
        // Fenced blocks whose only difference is Prettier formatting the embedded language (js / ts / css / json / html / markdown).
        "markdown/blockquote/code.md",
        // Also the `always`-only trailing `>` line: DIVERGENCES.md `ignored-block-trailing-quote-line`.
        "markdown/blockquote/ignore-code.md",
        "markdown/list/codeblock.md",
        "markdown/list/parser-regression/issue-11202.md",
        "markdown/markdown/real-world-case.md",
        "markdown/markdown/test-case.md",
        "markdown/code/0-indent-js.md",
        "markdown/code/format.md",
        "markdown/code/mdn-auth-api.md",
        "markdown/code/mdn-background-1.md",
        "markdown/code/mdn-background-2.md",
        "markdown/code/mdn-background-3.md",
        "markdown/code/mdn-background-4.md",
        "markdown/code/mdn-background-5.md",
        "markdown/code/mdn-background-6.md",
        "markdown/code/mdn-background-7.md",
        "markdown/code/mdn-background-8.md",
        "markdown/code/mdn-background-9.md",
        "markdown/code/mdn-filter-1.md",
        "markdown/code/mdn-filter-2.md",
        "markdown/code/mdn-font-face-1.md",
        "markdown/code/mdn-font-face-2.md",
        "markdown/code/mdn-grid-auto-columns.md",
        "markdown/code/mdn-import.md",
        "markdown/code/mdn-mask-image.md",
        "markdown/code/mdn-padding-1.md",
        "markdown/code/mdn-padding-2.md",
        "markdown/code/mdn-transform.md",
        "markdown/code/mdn-unicode-range.md",
        "markdown/code/ts-trailing-comma.md",
        "markdown/code/angular/",
        "markdown/code/lwc/",
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
