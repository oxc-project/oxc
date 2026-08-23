//! Prettier-conformance machinery.
//!
//! Runs a language formatter over the Prettier repository's own test suite (`tests/format/<dir>`)
//! and compares the output byte-for-byte against Prettier's committed jest snapshots.
//! Every output is also re-formatted; files whose second pass differs are tracked in the report's `# Not idempotent` section.
//! Language specifics arrive through [`ConformanceConfig`] (which fixture dirs, which parser names, what to ignore)
//! and the `format` callback (spec options → typed options → formatted output),
//! so this module never depends on a language crate.
//!
//! A spec dir looks like:
//!
//! ```text
//! yaml/flow-mapping
//! ├── __snapshots__/format.test.js.snap   <- Prettier's expected outputs
//! ├── format.test.js                      <- `runFormatTest(import.meta, ["yaml"], {...})` calls
//! └── *.yml                               <- inputs, one snapshot section per file × option set
//! ```
//!
//! Debugging: `PRETTIER_FILTER=<substring>` formats only matching files
//! and prints per-option-set diffs instead of producing a report.

// Printing is this module's debug/summary UX (run under `cargo test -- --nocapture` or `PRETTIER_FILTER`)
#![expect(clippy::print_stdout)]

use std::{
    env,
    fmt::Write as _,
    fs,
    path::{Path, PathBuf},
};

use cow_utils::CowUtils;
use rustc_hash::FxHashSet;
use similar::{ChangeTag, TextDiff};
use walkdir::WalkDir;

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Argument, ArrayExpressionElement, CallExpression, Expression, ObjectPropertyKind,
    VariableDeclarator,
};
use oxc_ast_visit::VisitMut;
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};

use crate::{OptionSet, ensure_prettier_suite};

const FORMAT_TEST_SPEC_NAME: &str = "format.test.js";
const SNAPSHOT_DIR_NAME: &str = "__snapshots__";
const SNAPSHOT_FILE_NAME: &str = "format.test.js.snap";

/// Spec dirs every language skips: parser error message snapshots
/// (added in Prettier v3.9.1) are not a formatter concern.
/// Matched against the suite-relative path, like every ignore entry.
const UNIVERSAL_IGNORE: &str = "/_errors_/";

/// Language-specific wiring for [`run_conformance`].
pub struct ConformanceConfig<'a> {
    /// Display name used in the report summary (e.g. `"yaml"`).
    pub language: &'a str,
    /// Fixture roots relative to the suite's `tests/format/` (e.g. `&["yaml"]`).
    pub fixture_roots: &'a [&'a str],
    /// When set, only `runFormatTest` calls whose parser list contains this name are exercised
    /// (a shared `format.test.js` may target several parsers).
    /// `None` accepts every call (JS/TS).
    pub exact_parser: Option<&'a str>,
    /// Path substrings to skip (unsupported constructs, no snapshot to compare, ...).
    pub ignore: &'a [&'a str],
    /// When set, spec calls for which this returns `true` are dropped entirely
    /// (e.g. option combinations the formatter does not support yet).
    pub skip_spec: Option<fn(&OptionSet) -> bool>,
}

/// Runs the conformance comparison and returns the report to snapshot,
/// or `None` when `PRETTIER_FILTER` is set (debug mode prints diffs instead).
///
/// Provisioning needs network + curl/tar.
/// Environments without them (CI's cross-compiled s390x/armv7 jobs) opt out in `ci.yml` via `-- --skip prettier_conformance`.
/// Which is why every consumer's test fn name starts with `prettier_conformance`.
///
/// # Panics
/// Panics when the suite cannot be provisioned (network/curl/tar failure)
/// and on malformed suite content (unreadable spec/snapshot files);
/// neither is a formatter bug, and a silent skip would let conformance rot green.
pub fn run_conformance<F>(config: &ConformanceConfig, mut format: F) -> Option<String>
where
    F: FnMut(&Path, &str, &OptionSet) -> Option<String>,
{
    let suite_root = ensure_prettier_suite()
        .unwrap_or_else(|err| panic!("failed to provision the Prettier suite: {err}"));
    let format_root = suite_root.join("tests").join("format");

    let fixture_roots =
        config.fixture_roots.iter().map(|dir| format_root.join(dir)).collect::<Vec<_>>();
    let test_dirs = collect_test_dirs(&fixture_roots);

    let filter = env::var("PRETTIER_FILTER").ok();
    if let Some(filter) = filter.as_deref() {
        for dir in &test_dirs {
            let inputs = collect_test_files(dir, &format_root, config.ignore, Some(filter));
            if !inputs.is_empty() {
                test_snapshots(config, &mut format, dir, &inputs, true);
            }
        }
        return None;
    }

    let mut total_tested_file_count = 0;
    let mut total_failed_file_count = 0;
    let mut total_skipped_files = vec![];
    let mut total_non_idempotent_files = vec![];
    let mut failed_reports = String::new();
    failed_reports.push_str("# Failed\n");
    failed_reports.push('\n');
    failed_reports.push_str("| Spec path | Failed or Passed | Match ratio |\n");
    failed_reports.push_str("| :-------- | :--------------: | :---------: |\n");
    for dir in &test_dirs {
        let inputs = collect_test_files(dir, &format_root, config.ignore, None);
        // `None`: no spec call targets this language config (shared dir, or every combination skipped).
        // The files were never exercised, keep them out of the totals instead of counting them as passed.
        let Some(results) = test_snapshots(config, &mut format, dir, &inputs, false) else {
            continue;
        };

        total_tested_file_count += inputs.len();
        total_failed_file_count += results.failed.len();
        total_skipped_files.extend(results.skipped);
        total_non_idempotent_files.extend(results.non_idempotent);

        for (path, (failed, passed, ratio)) in results.failed {
            writeln!(
                failed_reports,
                "| {} | {}{} | {:.2}% |",
                report_path(&path, &format_root),
                "💥".repeat(failed),
                "✨".repeat(passed),
                ratio * 100.0
            )
            .unwrap();
        }
    }

    let passed = total_tested_file_count - total_failed_file_count;
    #[expect(clippy::cast_precision_loss)]
    let percentage = (passed as f64 / total_tested_file_count as f64) * 100.0;
    // Files the formatter cannot parse are counted as passed above
    // (they have no diff to report); surface them so the gap is visible.
    let summary = format!(
        "{} compatibility: {passed}/{total_tested_file_count} ({percentage:.2}%), {} files skipped",
        config.language,
        total_skipped_files.len()
    );
    println!("{summary}");

    let mut report = format!("{summary}\n\n{failed_reports}");
    if !total_skipped_files.is_empty() {
        report.push_str("\n# Skipped (parse error, TODO: should be ignored or supported)\n\n");
        for path in &total_skipped_files {
            writeln!(report, "- {}", report_path(path, &format_root)).unwrap();
        }
    }
    if !total_non_idempotent_files.is_empty() {
        report.push_str("\n# Not idempotent\n\n");
        for (path, parse_failed) in &total_non_idempotent_files {
            let note = if *parse_failed { " (second pass failed to parse)" } else { "" };
            writeln!(report, "- {}{note}", report_path(path, &format_root)).unwrap();
        }
    }

    Some(report)
}

/// Suite-relative path with `/` separators, so reports are identical on Windows.
fn report_path(path: &Path, format_root: &Path) -> String {
    path.strip_prefix(format_root).unwrap().to_string_lossy().cow_replace('\\', "/").into_owned()
}

/// Read the first level of directories that contain `__snapshots__` and `format.test.js`
/// ```text
/// js/arrows <------------------------------- THIS
/// ├── __snapshots__
/// ├── arrow-chain-with-trailing-comments.js
/// ├── format.test.js
/// ├── semi <-------------------------------- AND THIS
/// │   ├── __snapshots__
/// │   ├── format.test.js
/// │   └── semi.js
/// └── tuple-and-record.js
/// ```
fn collect_test_dirs(fixture_roots: &[PathBuf]) -> Vec<PathBuf> {
    let mut test_dirs = FxHashSet::default();

    for fixture_root in fixture_roots {
        let dirs = WalkDir::new(fixture_root)
            .min_depth(1)
            .into_iter()
            .filter_map(Result::ok)
            .map(|e| {
                let mut path = e.into_path();
                if path.is_file()
                    && let Some(parent_path) = path.parent()
                {
                    path = parent_path.into();
                }
                path
            })
            .filter(|path| {
                path.join(SNAPSHOT_DIR_NAME).exists() && path.join(FORMAT_TEST_SPEC_NAME).exists()
            })
            .collect::<Vec<_>>();

        test_dirs.extend(dirs);
    }

    let mut test_dirs = test_dirs.into_iter().collect::<Vec<_>>();
    test_dirs.sort_unstable();

    test_dirs
}

/// Read all test files in the directory with applying ignore + filter.
///
/// Ignore/filter substrings match against the SUITE-RELATIVE path (`/`-separated):
/// matching the absolute path would let the checkout location leak in (a repo
/// under e.g. `.../cursor-work/` would silently ignore everything via `"cursor"`).
fn collect_test_files(
    dir: &Path,
    format_root: &Path,
    ignore: &[&str],
    filter: Option<&str>,
) -> Vec<PathBuf> {
    let mut test_files: Vec<PathBuf> = WalkDir::new(dir)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| !e.file_type().is_dir())
        .filter(|e| e.path().file_name().is_none_or(|name| name != FORMAT_TEST_SPEC_NAME))
        .filter(|e| {
            let path = report_path(e.path(), format_root);
            !path.contains(UNIVERSAL_IGNORE)
                && !ignore.iter().any(|s| path.contains(s))
                && filter.is_none_or(|name| path.contains(name))
        })
        .map(|e| e.path().to_path_buf())
        .collect();
    test_files.sort_unstable();

    test_files
}

#[derive(Default)]
struct SnapshotResults {
    /// `(path, (failed_count, passed_count, diff_ratio))` per file with at least one mismatch
    failed: Vec<(PathBuf, (usize, usize, f32))>,
    /// Files the formatter failed to parse for at least one options combination
    skipped: Vec<PathBuf>,
    /// `(path, second_pass_parse_failed)` per file
    /// whose re-formatted output differs from the first pass for at least one options combination
    non_idempotent: Vec<(PathBuf, bool)>,
}

/// Run the formatter and compare the output with the Prettier's snapshot.
///
/// Returns `None` when no spec call applies to this language config.
fn test_snapshots<F>(
    config: &ConformanceConfig,
    format: &mut F,
    dir: &Path,
    test_files: &[PathBuf],
    debug: bool,
) -> Option<SnapshotResults>
where
    F: FnMut(&Path, &str, &OptionSet) -> Option<String>,
{
    // Parse all `runFormatTest()` calls and collect format options
    let spec_path = &dir.join(FORMAT_TEST_SPEC_NAME);
    let (mut spec_calls, saw_run_format_test) = parse_spec(spec_path, config.exact_parser);
    debug_assert!(
        saw_run_format_test,
        "There is no `runFormatTest()` in {}, please check if it is correct?",
        spec_path.to_string_lossy()
    );
    if let Some(skip_spec) = config.skip_spec {
        spec_calls.retain(|call| !skip_spec(&call.options));
    }
    if spec_calls.is_empty() {
        return None;
    }

    let snapshots =
        fs::read_to_string(dir.join(SNAPSHOT_DIR_NAME).join(SNAPSHOT_FILE_NAME)).unwrap();

    let mut results = SnapshotResults::default();
    for path in test_files {
        if debug {
            println!("Test: {}", path.to_string_lossy());
        }
        // Single source text is used for multiple options
        let source_text = fs::read_to_string(path).unwrap();

        let mut failed_count = 0;
        let mut skipped_count = 0;
        let mut non_idempotent_count = 0;
        let mut second_pass_parse_failed = false;
        let mut total_diff_ratio = 0.0;
        // Check every combination of options!
        for call in &spec_calls {
            // Single snapshot file contains multiple test cases, so need to find the right one
            let expected = find_output_from_snapshots(
                &snapshots,
                path.file_name().unwrap().to_string_lossy().as_ref(),
                &call.snapshot_options,
            )
            .unwrap();

            let Some(actual) = format(path, &source_text, &call.options) else {
                // Skip the test if parsing failed
                skipped_count += 1;
                if debug {
                    println!("  => Skipped (parsing failed)");
                }
                continue;
            };

            // Idempotency: re-formatting the raw output must reproduce it.
            // Checked before snapshot escaping/EOL visualization.
            // A second pass that fails to parse is the stronger violation
            // (the output's own parser rejects it) and is annotated separately in the report.
            let reformatted = format(path, &actual, &call.options);
            let idempotent = reformatted.as_deref() == Some(actual.as_str());
            if !idempotent {
                non_idempotent_count += 1;
                second_pass_parse_failed |= reformatted.is_none();
            }

            let escaped = replace_escape_and_eol(
                &actual,
                expected.contains("LF>") || expected.contains("<CR"),
            );

            let result = expected == escaped;
            if !result {
                failed_count += 1;
                total_diff_ratio += TextDiff::from_lines(&expected, &escaped).ratio();
            }

            if debug {
                println!(
                    "Options: {{ {} }}",
                    call.snapshot_options
                        .iter()
                        .filter(|(k, _)| k != "parsers")
                        .map(|(k, v)| format!("{k}: {v}"))
                        .collect::<Vec<_>>()
                        .join(", ")
                );

                if result {
                    println!("Passed ✅");
                } else {
                    println!("Failed ❌");
                    print_text_diff(&TextDiff::from_lines(&expected, &escaped));
                }
                if !idempotent {
                    match &reformatted {
                        Some(reformatted) => {
                            println!("Not idempotent ⚠️ (second pass differs)");
                            print_text_diff(&TextDiff::from_lines(&actual, reformatted));
                        }
                        None => println!("Not idempotent 💥 (second pass failed to parse)"),
                    }
                }
                println!();
            }
        }

        if failed_count != 0 {
            let total_count = spec_calls.len();
            let passed_count = total_count - failed_count;
            #[expect(clippy::cast_precision_loss)]
            let max_diff_ratio = total_count as f32;
            results.failed.push((
                path.clone(),
                (failed_count, passed_count, total_diff_ratio / max_diff_ratio),
            ));
        }
        if skipped_count != 0 {
            results.skipped.push(path.clone());
        }
        if non_idempotent_count != 0 {
            results.non_idempotent.push((path.clone(), second_pass_parse_failed));
        }
    }

    Some(results)
}

/// Prints a line diff with `-`/`+`/` ` gutters (debug output for conformance-style tests).
pub fn print_text_diff(diff: &TextDiff<'_, '_, str>) {
    for change in diff.iter_all_changes() {
        let sign = match change.tag() {
            ChangeTag::Delete => "-",
            ChangeTag::Insert => "+",
            ChangeTag::Equal => " ",
        };
        print!("{sign}{change}");
    }
}

/// Extract single output section from snapshot file which contains multiple test cases.
///
/// Format is like below:
/// ```text
/// filename1
/// ===optionsA===
/// ====input1====
/// ===output1A===
/// ==============
/// filename1
/// ===optionsB===
/// ====input1====
/// ===output1B===
/// ==============
///
/// filename2
/// ===optionsA===
/// ====input2====
/// ===output2A===
/// ==============
/// ```
///
/// There are also options-like strings after the filename, but it seems that format is not guaranteed...
/// Thus, we need to find the right section by filename and options for sure.
fn find_output_from_snapshots(
    snap_content: &str,
    file_name: &str,
    snapshot_options: &[(String, String)],
) -> Option<String> {
    let filename_started = snap_content.find(&format!("exports[`{file_name} "))?;
    let after_filename = &snap_content[filename_started..];

    // Anchor on the options block (header + key:value lines). The line that
    // follows is a `printWidth` visualization whose exact rendering varies by
    // Prettier version, so we skip it by jumping to the following
    // `=====input=====`. To disambiguate between blocks where one options
    // list is a prefix of another (e.g. `parsers: [...]` vs
    // `parsers: [...] + singleQuote: true`), we require the gap between the
    // matched options and the input marker to be exactly one line (the
    // visualization), retrying past the false match otherwise.
    let options_pattern = format!(
        "====================================options=====================================
{}
",
        snapshot_options.iter().map(|(k, v)| format!("{k}: {v}")).collect::<Vec<_>>().join("\n"),
    );
    let input_marker =
        "\n=====================================input======================================\n";
    let mut search_from = 0;
    let expected = loop {
        let pos = after_filename[search_from..].find(&options_pattern)?;
        let after_options = search_from + pos + options_pattern.len();
        let input_pos = after_filename[after_options..].find(input_marker)?;
        let between = &after_filename[after_options..after_options + input_pos];
        if !between.contains('\n') {
            break &after_filename[after_options + input_pos..];
        }
        search_from = after_options;
    };

    let output_start_line =
        "=====================================output=====================================\n";
    let output_started = expected.find(output_start_line)?;
    let output_end_line =
        "\n================================================================================";
    let output_ended = expected.find(output_end_line)?;

    let output = expected[output_started..output_ended]
        .trim_start_matches(output_start_line)
        .trim_end_matches(output_end_line);

    Some(output.to_string())
}

/// Apply the same escape rules as Prettier does.
/// If Prettier's snapshot contains `<LF>`, `<CR>` or `<CRLF>`, we also need to visualize.
fn replace_escape_and_eol(input: &str, need_eol_visualized: bool) -> String {
    let input = input
        .cow_replace("\\", "\\\\")
        .cow_replace("`", "\\`")
        .cow_replace("${", "\\${")
        .into_owned();

    if need_eol_visualized {
        let mut chars = input.chars();
        let mut result = String::new();

        while let Some(char) = chars.next() {
            match char {
                '\u{a}' => result.push_str("<LF>\n"),
                '\u{d}' => {
                    let next = chars.clone().next();
                    if next == Some('\u{a}') {
                        result.push_str("<CRLF>\n");
                        chars.next();
                    } else {
                        result.push_str("<CR>\n");
                    }
                }
                _ => {
                    result.push(char);
                }
            }
        }

        return result;
    }

    input
}

/// One `runFormatTest(import.meta, parsers, opts)` call from a spec file.
struct SpecCall {
    /// The literal options from the call's third argument, as an [`OptionSet`]
    /// (same shape the fixture harness feeds `parse_options`).
    options: OptionSet,
    /// `(key, raw source text)` pairs used to locate the matching snapshot
    /// section (Prettier renders them verbatim into the snapshot header).
    snapshot_options: Vec<(String, String)>,
}

fn string_elements(arr: &oxc_ast::ast::ArrayExpression) -> Vec<String> {
    arr.elements
        .iter()
        .filter_map(|el| match el {
            ArrayExpressionElement::StringLiteral(literal) => Some(literal.value.to_string()),
            _ => None,
        })
        .collect()
}

/// Returns the matching calls plus whether ANY `runFormatTest()` call was seen
/// (even for other parsers) — an all-absent spec signals a suite layout change.
fn parse_spec(spec: &Path, exact_parser: Option<&str>) -> (Vec<SpecCall>, bool) {
    let mut parser = SpecParser { exact_parser, ..SpecParser::default() };
    parser.parse(spec);
    (parser.calls, parser.saw_run_format_test)
}

#[derive(Default)]
struct SpecParser<'a> {
    source_text: String,
    parsers: Vec<String>,
    calls: Vec<SpecCall>,
    exact_parser: Option<&'a str>,
    saw_run_format_test: bool,
}

impl SpecParser<'_> {
    fn parse(&mut self, spec: &Path) {
        let spec_content = fs::read_to_string(spec).unwrap_or_default();

        self.source_text.clone_from(&spec_content);

        let allocator = Allocator::default();
        let mut source_type = SourceType::from_path(spec).unwrap_or_default();
        if source_type.is_javascript() {
            source_type = source_type.with_jsx(true);
        }

        let mut ret = Parser::new(&allocator, &spec_content, source_type).parse();
        assert!(ret.diagnostics.is_empty());
        self.visit_program(&mut ret.program);
    }
}

impl VisitMut<'_> for SpecParser<'_> {
    // Some test cases use a variable to store the parsers.
    //
    // ```js
    // const parser = ["babel"];
    //
    // runFormatTest(import.meta, parser, {});
    // runFormatTest(import.meta, parser, { semi: false });
    // ```
    fn visit_variable_declarator(&mut self, decl: &mut VariableDeclarator<'_>) {
        let Some(name) = decl.id.get_identifier_name() else { return };
        if !matches!(name.as_str(), "parser" | "parsers") {
            return;
        }

        debug_assert!(self.parsers.is_empty(), "`parsers` is already defined");
        if let Some(Expression::ArrayExpression(arr_expr)) = &decl.init {
            self.parsers = string_elements(arr_expr);
        }
    }

    // The `runFormatTest()` function is used on prettier's test cases.
    // We need to collect all calls and get the options and parsers.
    fn visit_call_expression(&mut self, expr: &mut CallExpression<'_>) {
        let Some(ident) = expr.callee.get_identifier_reference() else { return };
        if ident.name != "runFormatTest" {
            return;
        }
        self.saw_run_format_test = true;

        let mut snapshot_options: Vec<(String, String)> = vec![];
        let mut parsers = vec![];

        // Get parsers
        if let Some(argument) = expr.arguments.get(1) {
            let Some(argument_expr) = argument.as_expression() else {
                return;
            };

            // If inlined array
            if let Expression::ArrayExpression(arr_expr) = argument_expr {
                parsers = string_elements(arr_expr);
            }
            // If variable
            if let Expression::Identifier(_) = argument_expr {
                debug_assert!(
                    !self.parsers.is_empty(),
                    "`parsers` is not collected, check variable name"
                );
                parsers.clone_from(&self.parsers);
            }
        } else {
            return;
        }

        // A single `format.test.js` may list several parsers (e.g. `with-comment/`),
        // so languages with an exact parser name keep only their own calls.
        if let Some(exact) = self.exact_parser
            && !parsers.iter().any(|p| p == exact)
        {
            return;
        }

        // Collect the literal options; non-literal values (e.g. the `errors`
        // object) never influence formatting and are left out.
        let mut options = OptionSet::new();
        if let Some(Argument::ObjectExpression(obj_expr)) = expr.arguments.get(2) {
            obj_expr.properties.iter().for_each(|item| {
                if let ObjectPropertyKind::ObjectProperty(obj_prop) = item
                    && let Some(name) = obj_prop.key.static_name()
                {
                    let value = match &obj_prop.value {
                        Expression::BooleanLiteral(literal) => {
                            Some(serde_json::Value::Bool(literal.value))
                        }
                        Expression::NumericLiteral(literal) => {
                            // Integral options (printWidth/tabWidth) must round-trip
                            // through `as_u64`, so avoid `Number::from_f64`.
                            #[expect(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                            if literal.value.fract() == 0.0 && literal.value >= 0.0 {
                                Some(serde_json::Value::Number((literal.value as u64).into()))
                            } else {
                                serde_json::Number::from_f64(literal.value)
                                    .map(serde_json::Value::Number)
                            }
                        }
                        Expression::StringLiteral(literal) => {
                            Some(serde_json::Value::String(literal.value.to_string()))
                        }
                        _ => None,
                    };
                    if let Some(value) = value {
                        options.insert(name.to_string(), value);
                    }

                    if name != "errors" {
                        snapshot_options.push((
                            name.to_string(),
                            obj_prop.value.span().source_text(&self.source_text).to_string(),
                        ));
                    }
                }
            });
        }

        debug_assert!(!parsers.is_empty(), "`parsers` should not be empty");
        snapshot_options.push((
            "parsers".to_string(),
            format!(
                "[{}]",
                parsers.iter().map(|p| format!("\"{p}\"")).collect::<Vec<_>>().join(", ")
            ),
        ));

        // Prettier omits `printWidth` from the options block when it equals the
        // default (80); the value is only shown in the trailing visualization line.
        snapshot_options.sort_by(|a, b| a.0.cmp(&b.0));

        self.calls.push(SpecCall { options, snapshot_options });
    }
}
