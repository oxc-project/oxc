#![allow(clippy::print_stdout)]

mod ignore_list;
pub mod options;
mod spec;

use std::path::{Path, PathBuf};

use cow_utils::CowUtils;
use rustc_hash::FxHashSet;
use similar::TextDiff;
use walkdir::WalkDir;

use oxc_allocator::Allocator;
use oxc_parser::{ParseOptions, Parser};
use oxc_prettier::{Prettier, PrettierOptions};
use oxc_span::SourceType;

use crate::{ignore_list::IGNORE_TESTS, options::TestRunnerOptions, spec::parse_spec};

#[test]
#[cfg(any(coverage, coverage_nightly))]
fn test() {
    use crate::options::TestLanguage;
    TestRunner::new(TestRunnerOptions { filter: None, language: TestLanguage::Js }).run();
    TestRunner::new(TestRunnerOptions { filter: None, language: TestLanguage::Ts }).run();
}

fn root() -> PathBuf {
    oxc_tasks_common::project_root().join("tasks").join("prettier_conformance")
}

fn fixtures_root() -> PathBuf {
    root().join("prettier").join("tests").join("format")
}

fn snap_root() -> PathBuf {
    root().join("snapshots")
}

const FORMAT_TEST_SPEC_NAME: &str = "format.test.js";
const SNAPSHOT_DIR_NAME: &str = "__snapshots__";
const SNAPSHOT_FILE_NAME: &str = "format.test.js.snap";

pub struct TestRunner {
    options: TestRunnerOptions,
}

impl TestRunner {
    pub fn new(options: TestRunnerOptions) -> Self {
        Self { options }
    }

    /// # Panics
    pub fn run(&self) {
        let test_lang = self.options.language.as_str();
        let test_dirs = collect_test_dirs(&self.options.language.fixtures_roots(&fixtures_root()));

        // If filter is set, only run the specified test for debug
        if self.options.filter.is_some() {
            for dir in &test_dirs {
                let inputs = collect_test_files(dir, self.options.filter.as_ref());
                // If filter is set, many of the tests can be skipped
                if !inputs.is_empty() {
                    // This will print the diff
                    let _failed_test_files = test_snapshots(dir, &inputs, true);
                }
            }

            return;
        }

        // Otherwise, run all tests and generate coverage reports
        let mut total_tested_file_count = 0;
        let mut total_failed_file_count = 0;
        let mut failed_reports = String::new();
        failed_reports.push_str("# Failed\n");
        failed_reports.push('\n');
        failed_reports.push_str("| Spec path | Failed or Passed | Match ratio |\n");
        failed_reports.push_str("| :-------- | :--------------: | :---------: |\n");
        for dir in &test_dirs {
            let inputs = collect_test_files(dir, None);
            let failed_test_files = test_snapshots(dir, &inputs, false);

            total_tested_file_count += inputs.len();
            total_failed_file_count += failed_test_files.len();

            for (path, (failed, passed, ratio)) in failed_test_files {
                failed_reports.push_str(&format!(
                    "| {} | {}{} | {:.2}% |\n",
                    path.strip_prefix(fixtures_root()).unwrap().to_string_lossy(),
                    "💥".repeat(failed),
                    "✨".repeat(passed),
                    ratio * 100.0
                ));
            }
        }

        let passed = total_tested_file_count - total_failed_file_count;
        #[expect(clippy::cast_precision_loss)]
        let percentage = (passed as f64 / total_tested_file_count as f64) * 100.0;
        let summary = format!(
            "{test_lang} compatibility: {passed}/{total_tested_file_count} ({percentage:.2}%)"
        );

        // Print summary
        println!("{summary}");
        // And generate coverage reports
        let snapshot = format!("{summary}\n\n{failed_reports}");
        std::fs::write(snap_root().join(format!("prettier.{test_lang}.snap.md")), snapshot)
            .unwrap();
    }
}

/// Read the first level of directories that contain `__snapshots__` and `format.test.js`
/// ```text
/// js/arrows <------------------------------- THIS
/// ├── __snapshots__
/// ├── arrow-chain-with-trailing-comments.js
/// ├── arrow_function_expression.js
/// ├── format.test.js
/// ├── semi <-------------------------------- AND THIS
/// │   ├── __snapshots__
/// │   ├── format.test.js
/// │   └── semi.js
/// └── tuple-and-record.js
/// ```
fn collect_test_dirs(fixture_roots: &Vec<PathBuf>) -> Vec<PathBuf> {
    let mut test_dirs = FxHashSet::default();

    for fixture_root in fixture_roots {
        let dirs = WalkDir::new(fixture_root)
            .min_depth(1)
            .into_iter()
            .filter_map(Result::ok)
            .map(|e| {
                let mut path = e.into_path();
                if path.is_file() {
                    if let Some(parent_path) = path.parent() {
                        path = parent_path.into();
                    }
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

/// Read all test files in the directory with applying ignore + filter
/// ```text
/// js/arrows
/// ├── __snapshots__
/// ├── arrow-chain-with-trailing-comments.js <---- THIS
/// ├── arrow_function_expression.js <------------- AND THIS
/// ├── format.test.js
/// └── tuple-and-record.js <---------------------- AND THIS
/// ```
fn collect_test_files(dir: &Path, filter: Option<&String>) -> Vec<PathBuf> {
    let mut test_files: Vec<PathBuf> = WalkDir::new(dir)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| !e.file_type().is_dir())
        .filter(|e| e.path().file_name().is_none_or(|name| name != FORMAT_TEST_SPEC_NAME))
        .filter(|e| !IGNORE_TESTS.iter().any(|s| e.path().to_string_lossy().contains(s)))
        .filter(|e| filter.map_or(true, |name| e.path().to_string_lossy().contains(name)))
        .map(|e| e.path().to_path_buf())
        .collect();
    test_files.sort_unstable();

    test_files
}

/// Run `oxc_prettier` and compare the output with the Prettier's snapshot
fn test_snapshots(
    dir: &Path,
    test_files: &Vec<PathBuf>,
    has_debug_filter: bool,
) -> Vec<(PathBuf, (usize, usize, f32))> {
    // Parse all `runFormatTest()` calls and collect format options
    let spec_path = &dir.join(FORMAT_TEST_SPEC_NAME);
    let spec_calls = parse_spec(spec_path);
    debug_assert!(
        !spec_calls.is_empty(),
        "There is no `runFormatTest()` in {}, please check if it is correct?",
        spec_path.to_string_lossy()
    );

    let snapshots =
        std::fs::read_to_string(dir.join(SNAPSHOT_DIR_NAME).join(SNAPSHOT_FILE_NAME)).unwrap();

    let mut failed_test_files = vec![];
    for path in test_files {
        // Single source text is used for multiple options
        let source_text = std::fs::read_to_string(path).unwrap();

        let mut failed_count = 0;
        let mut total_diff_ratio = 0.0;
        // Check every combination of options!
        for (prettier_options, snapshot_options) in &spec_calls {
            // Single snapshot file contains multiple test cases, so need to find the right one
            let expected = find_output_from_snapshots(
                &snapshots,
                path.file_name().unwrap().to_string_lossy().as_ref(),
                snapshot_options,
                prettier_options.print_width,
            )
            .unwrap();

            let actual = replace_escape_and_eol(
                &run_oxc_prettier(
                    &source_text,
                    SourceType::from_path(path).unwrap(),
                    *prettier_options,
                ),
                expected.contains("LF>") || expected.contains("<CR"),
            );

            let result = expected == actual;
            let diff = TextDiff::from_lines(&expected, &actual);

            if !result {
                failed_count += 1;
                total_diff_ratio += diff.ratio();
            }

            if has_debug_filter {
                let print_with_border = |title: &str| {
                    let w = prettier_options.print_width;
                    println!("--- {title} {}", "-".repeat(w - title.len() - 5));
                };

                println!(
                    "{} Test: {}",
                    if result { "✨" } else { "💥" },
                    path.strip_prefix(fixtures_root()).unwrap().to_string_lossy(),
                );
                println!(
                    "Options: {{ {} }}",
                    snapshot_options
                        .iter()
                        .filter(|(k, _)| k != "parsers")
                        .map(|(k, v)| format!("{k}: {v}"))
                        .collect::<Vec<_>>()
                        .join(", ")
                );

                if !result {
                    print_with_border("Input");
                    println!("{source_text}");
                    print_with_border(&format!("PrettierOutput: {}LoC", expected.lines().count()));
                    println!("{expected}");
                    print_with_border(&format!("OxcOutput: {}LoC", actual.lines().count()));
                    println!("{actual}");
                    print_with_border("Diff");
                    oxc_tasks_common::print_diff_in_terminal(&diff);
                }
                println!();
            }
        }

        if failed_count != 0 {
            let total_count = spec_calls.len();
            let passed_count = total_count - failed_count;
            #[expect(clippy::cast_precision_loss)]
            let max_diff_ratio = total_count as f32;
            failed_test_files.push((
                path.clone(),
                (failed_count, passed_count, total_diff_ratio / max_diff_ratio),
            ));
        }
    }

    failed_test_files
}

/// Extract single output section from snapshot file which contains multiple test cases.
///
/// Format is like below:
/// ```
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
    print_width: usize,
) -> Option<String> {
    let filename_started = snap_content.find(&format!("exports[`{file_name} "))?;
    let expected = &snap_content[filename_started..];

    let options_started = expected.find(&format!(
        "====================================options=====================================
{}
{}| printWidth
=====================================input======================================
",
        snapshot_options.iter().map(|(k, v)| format!("{k}: {v}")).collect::<Vec<_>>().join("\n"),
        " ".repeat(print_width)
    ))?;
    let expected = &expected[options_started..];

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

fn run_oxc_prettier(
    source_text: &str,
    source_type: SourceType,
    prettier_options: PrettierOptions,
) -> String {
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, source_text, source_type)
        .with_options(ParseOptions { preserve_parens: false, ..ParseOptions::default() })
        .parse();
    Prettier::new(&allocator, prettier_options).build(&ret.program)
}
