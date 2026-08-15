//! JSDoc formatting tests, driven by local fixture pairs under `tests/jsdoc/fixtures/`
//! (`foo.{js,ts,jsx,tsx}` + expected `foo.output.<ext>`).
//!
//! Plain assertions, not a conformance report: the fixtures are maintained in this
//! repo, so any mismatch is a regression to fix, never a failure to record.
//! On mismatch the per-fixture diff is printed and the test fails listing every
//! mismatched fixture.

// Printing is this test's failure UX (diffs for every mismatched fixture)
#![expect(clippy::print_stdout)]

use std::{
    fs,
    path::{Path, PathBuf},
};

use similar::TextDiff;
use walkdir::WalkDir;

use oxc_allocator::Allocator;
use oxc_formatter::{
    CommentLineStrategy, JsFormatOptions, JsdocOptions, LineWrappingStyle, QuoteStyle,
    format_program, parse_for_format,
};
use oxc_formatter_core::LineWidth;
use oxc_formatter_tests::conformance::print_text_diff;
use oxc_span::SourceType;

fn fixtures_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("jsdoc").join("fixtures")
}

const IGNORED_FIXTURES: &[&str] = &[
    // This case depends on embedded CSS/HTML formatter callbacks provided by the oxfmt app path.
    // This standalone test calls `oxc_formatter` directly and cannot exercise that wiring.
    "descriptions/032-jsx-tsx-css.ts",
];

#[test]
fn jsdoc() {
    let mut failures = Vec::new();

    for (input_path, expected_path) in collect_fixture_pairs() {
        let rel_path = normalized_rel_path(&input_path);

        let source_text = fs::read_to_string(&input_path).unwrap();
        let expected = fs::read_to_string(&expected_path).unwrap();

        let (jsdoc_options, quote_style, line_width) = load_jsdoc_options(&input_path);
        let Some(actual) =
            run_formatter(&source_text, &input_path, &jsdoc_options, quote_style, line_width)
        else {
            println!("PARSE ERROR: {rel_path}");
            failures.push(rel_path);
            continue;
        };

        if actual != expected {
            println!("FAIL: {rel_path}");
            print_text_diff(&TextDiff::from_lines(&expected, &actual));
            println!();
            failures.push(rel_path);
        }
    }

    assert!(
        failures.is_empty(),
        "💥 {} jsdoc fixture(s) mismatched: {failures:#?}",
        failures.len()
    );
}

/// Walk fixtures directory, collect pairs of (input, expected_output).
///
/// A fixture pair is a file `foo.{js,ts,jsx,tsx}` paired with
/// `foo.output.{js,ts,jsx,tsx}` in the same directory.
fn collect_fixture_pairs() -> Vec<(PathBuf, PathBuf)> {
    let mut pairs = Vec::new();

    for entry in WalkDir::new(fixtures_root())
        .min_depth(1)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| !e.file_type().is_dir())
    {
        let path = entry.path();
        let name = path.file_name().unwrap().to_string_lossy();

        // Skip output files and non-source files
        if name.contains(".output.") || name.starts_with('.') {
            continue;
        }

        // Skip non-source files (options.json, etc.)
        let ext = path.extension().map(|e| e.to_string_lossy()).unwrap_or_default();
        if !matches!(ext.as_ref(), "js" | "ts" | "jsx" | "tsx") {
            continue;
        }

        // Check for matching output file
        let stem = path.file_stem().unwrap().to_string_lossy();
        let output_name = format!("{stem}.output.{ext}");
        let output_path = path.with_file_name(&output_name);

        if !output_path.exists() {
            continue;
        }

        let rel_path = normalized_rel_path(path);
        if IGNORED_FIXTURES.iter().any(|ignored| rel_path == *ignored) {
            continue;
        }

        pairs.push((path.to_path_buf(), output_path));
    }

    pairs.sort_unstable();
    pairs
}

/// `/`-separated path relative to the fixtures root, so `IGNORED_FIXTURES`
/// matching and failure listings behave the same on Windows.
fn normalized_rel_path(path: &Path) -> String {
    oxc_tasks_common::normalize_path(path.strip_prefix(fixtures_root()).unwrap())
}

/// Load per-fixture JsdocOptions and format overrides.
/// Checks for a per-file sidecar `{stem}.options.json` first, then
/// directory-level `options.json`. Falls back to default options.
fn load_jsdoc_options(input_path: &Path) -> (JsdocOptions, QuoteStyle, Option<u16>) {
    let dir = input_path.parent().unwrap();

    // Per-file options: e.g. 033-not-capitalizing-false.options.json
    let stem = input_path.file_stem().unwrap().to_string_lossy();
    let per_file_path = dir.join(format!("{stem}.options.json"));
    if per_file_path.exists() {
        return parse_jsdoc_options(&per_file_path);
    }

    // Per-directory options: options.json
    let dir_options_path = dir.join("options.json");
    if dir_options_path.exists() {
        return parse_jsdoc_options(&dir_options_path);
    }

    (JsdocOptions::default(), QuoteStyle::default(), None)
}

fn parse_jsdoc_options(path: &Path) -> (JsdocOptions, QuoteStyle, Option<u16>) {
    let content = fs::read_to_string(path).unwrap();
    let json: serde_json::Value = serde_json::from_str(&content)
        .unwrap_or_else(|e| panic!("Failed to parse {}: {e}", path.display()));

    let mut options = JsdocOptions::default();
    let mut quote_style = QuoteStyle::default();
    let mut line_width: Option<u16> = None;

    if json.get("capitalize_descriptions").and_then(serde_json::Value::as_bool) == Some(false) {
        options.capitalize_descriptions = false;
    }
    if json.get("separate_tag_groups").and_then(serde_json::Value::as_bool) == Some(true) {
        options.separate_tag_groups = true;
    }
    if json.get("separate_returns_from_param").and_then(serde_json::Value::as_bool) == Some(true) {
        options.separate_returns_from_param = true;
    }
    if json.get("bracket_spacing").and_then(serde_json::Value::as_bool) == Some(true) {
        options.bracket_spacing = true;
    }
    if json.get("single_line_when_possible").and_then(serde_json::Value::as_bool) == Some(false) {
        options.comment_line_strategy = CommentLineStrategy::Multiline;
    }
    if let Some(strategy) = json.get("comment_line_strategy").and_then(serde_json::Value::as_str) {
        options.comment_line_strategy = match strategy {
            "multiline" => CommentLineStrategy::Multiline,
            "keep" => CommentLineStrategy::Keep,
            _ => CommentLineStrategy::SingleLine,
        };
    }
    if json.get("description_with_dot").and_then(serde_json::Value::as_bool) == Some(true) {
        options.description_with_dot = true;
    }
    if json.get("add_default_to_description").and_then(serde_json::Value::as_bool) == Some(false) {
        options.add_default_to_description = false;
    }
    if json.get("prefer_code_fences").and_then(serde_json::Value::as_bool) == Some(true) {
        options.prefer_code_fences = true;
    }
    if let Some(style) = json.get("line_wrapping_style").and_then(serde_json::Value::as_str) {
        options.line_wrapping_style = match style {
            "balance" => LineWrappingStyle::Balance,
            _ => LineWrappingStyle::Greedy,
        };
    }
    if json.get("description_tag").and_then(serde_json::Value::as_bool) == Some(true) {
        options.description_tag = true;
    }
    if json.get("keep_unparsable_example_indent").and_then(serde_json::Value::as_bool) == Some(true)
    {
        options.keep_unparsable_example_indent = true;
    }
    if json.get("single_quote").and_then(serde_json::Value::as_bool) == Some(true) {
        quote_style = QuoteStyle::Single;
    }
    if let Some(w) = json.get("print_width").and_then(serde_json::Value::as_u64) {
        line_width = u16::try_from(w).ok();
    }

    (options, quote_style, line_width)
}

fn run_formatter(
    source_text: &str,
    path: &Path,
    jsdoc_options: &JsdocOptions,
    quote_style: QuoteStyle,
    line_width_override: Option<u16>,
) -> Option<String> {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path(path).unwrap_or_default();

    // NOTE: Parse here (not via `oxc_formatter::format`) so recoverable parse errors are tolerated,
    // some jsdoc fixtures (e.g. duplicate declarations) still format correctly.
    // Only a hard parser panic aborts.
    // This deliberately DIVERGES from production oxfmt,
    // whose fail-loud `format()` would report a diagnostic for these fixtures instead of formatting.
    let ret = parse_for_format(&allocator, source_text, source_type);
    if ret.panicked {
        return None;
    }

    // Prettier's default `printWidth` is 80 (oxc defaults to 100)
    let width = line_width_override.unwrap_or(80);
    let options = JsFormatOptions {
        line_width: LineWidth::try_from(width).unwrap(),
        quote_style,
        jsdoc: Some(jsdoc_options.clone()),
        ..JsFormatOptions::default()
    };
    Some(format_program(&allocator, &ret.program, options).print().ok()?.into_code())
}
