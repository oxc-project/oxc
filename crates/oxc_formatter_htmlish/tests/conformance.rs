//! Prettier conformance for HTML.
//!
//! Compares output against the Prettier suite's `tests/format/html` snapshots via `oxc_formatter_tests::conformance`;
//! the failure report is pinned with `insta`.
//!
//! Debug a specific test: `PRETTIER_FILTER=<substring> cargo test -p oxc_formatter_htmlish --test conformance -- --nocapture`

use std::path::Path;

use oxc_allocator::Allocator;
use oxc_formatter_core::LineWidth;
use oxc_formatter_htmlish::{HtmlishFormatOptions, format};
use oxc_formatter_tests::{
    OptionSet,
    conformance::{ConformanceConfig, run_conformance},
};

#[path = "fixtures/options.rs"]
mod options;
use options::apply_htmlish_options;

const CONFIG: ConformanceConfig = ConformanceConfig {
    language: "html",
    fixture_roots: &["html"],
    exact_parser: Some("html"),
    ignore: &[
        // Cursor tracking is an editor feature, not formatting.
        "html/cursor/",
        // Embedded languages need the dispatcher; covered by oxfmt's E2E conformance.
        "html/css/",
        "html/js/",
        "html/multiparser/",
        "html/script/",
        "html/svg/embeded/",
        "html/bracket-same-line/embed.html",
        "html/prettier_ignore/issue-15738.html",
        "html/basics/html5-boilerplate.html",
        "html/case/case.html",
        // `<script type="text/x-handlebars-template">` is formatted as Glimmer.
        "html/handlebars-venerable/",
        // The front matter's YAML body.
        "html/yaml/yaml.html",
        // `style="…"` is formatted as CSS, `on*="…"` as JS.
        "html/attributes/style.html",
        "html/attributes/event-attributes.html",
        "html/attributes/no-semi/",
        "html/doctype_declarations/xhtml1.1.html",
        "html/tags/marquee.html",
        "html/tags/menu.html",
        "html/whitespace/fill.html",
        "html/whitespace/nested-inline-without-whitespace.html",
        // Pragma support (`@format` insertion / require).
        "html/pragma/",
    ],
    skip_spec: None,
};

fn format_html(_path: &Path, source_text: &str, spec: &OptionSet) -> Option<String> {
    // Prettier's default `printWidth` is 80 (oxc defaults to 100).
    let mut options = HtmlishFormatOptions {
        line_width: LineWidth::try_from(80).unwrap(),
        ..HtmlishFormatOptions::default()
    };
    apply_htmlish_options(&mut options, spec);
    let allocator = Allocator::default();
    let formatted = format(&allocator, source_text, options).ok()?;
    Some(formatted.print().ok()?.into_code())
}

#[test]
fn prettier_conformance() {
    let Some(report) = run_conformance(&CONFIG, format_html) else { return };
    insta::assert_snapshot!("prettier-html", report);
}
