#![allow(clippy::print_stdout, clippy::print_stderr, clippy::disallowed_methods)]
mod ignore_list;
mod spec;

use std::{
    fs,
    path::{Path, PathBuf},
};

use oxc_allocator::Allocator;
use oxc_parser::{ParseOptions, Parser};
use oxc_prettier::{Prettier, PrettierOptions};
use oxc_span::SourceType;
use oxc_tasks_common::project_root;
use rustc_hash::FxHashSet;
use walkdir::WalkDir;

use crate::{
    ignore_list::{JS_IGNORE_TESTS, TS_IGNORE_TESTS},
    spec::SpecParser,
};

#[test]
#[cfg(any(coverage, coverage_nightly))]
fn test() {
    TestRunner::new(TestLanguage::Js, TestRunnerOptions::default()).run();
    TestRunner::new(TestLanguage::Ts, TestRunnerOptions::default()).run();
}

pub enum TestLanguage {
    Js,
    Ts,
}

impl TestLanguage {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Js => "js",
            Self::Ts => "ts",
        }
    }
}

#[derive(Default, Clone)]
pub struct TestRunnerOptions {
    pub filter: Option<String>,
}

/// The test runner which walks the prettier repository and searches for formatting tests.
pub struct TestRunner {
    language: TestLanguage,
    fixtures_root: PathBuf,
    ignore_tests: &'static [&'static str],
    options: TestRunnerOptions,
    spec: SpecParser,
}

fn root() -> PathBuf {
    project_root().join("tasks").join("prettier_conformance")
}

fn fixtures_root() -> PathBuf {
    root().join("prettier").join("tests").join("format")
}

fn snap_root() -> PathBuf {
    root().join("snapshots")
}

const SNAP_NAME: &str = "format.test.js";
const SNAP_RELATIVE_PATH: &str = "__snapshots__/format.test.js.snap";
const LF: char = '\u{a}';
const CR: char = '\u{d}';

impl TestRunner {
    pub fn new(language: TestLanguage, options: TestRunnerOptions) -> Self {
        let fixtures_root = fixtures_root().join(match language {
            TestLanguage::Js => "js",
            TestLanguage::Ts => "typescript",
        });
        let ignore_tests = match language {
            TestLanguage::Js => JS_IGNORE_TESTS,
            TestLanguage::Ts => TS_IGNORE_TESTS,
        };
        Self { language, fixtures_root, ignore_tests, options, spec: SpecParser::default() }
    }

    /// # Panics
    #[expect(clippy::cast_precision_loss)]
    pub fn run(mut self) {
        let fixture_root = &self.fixtures_root;
        // Read the first level of directories that contain `__snapshots__`
        let mut dirs = WalkDir::new(fixture_root)
            .min_depth(1)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| {
                self.options
                    .filter
                    .as_ref()
                    .map_or(true, |name| e.path().to_string_lossy().contains(name))
            })
            .filter(|e| !self.ignore_tests.iter().any(|s| e.path().to_string_lossy().contains(s)))
            .map(|e| {
                let mut path = e.into_path();
                if path.is_file() {
                    if let Some(parent_path) = path.parent() {
                        path = parent_path.into();
                    }
                }
                path
            })
            .filter(|path| path.join("__snapshots__").exists())
            .collect::<Vec<_>>();

        let dir_set: FxHashSet<_> = dirs.iter().cloned().collect();
        dirs = dir_set.into_iter().collect();

        dirs.sort_unstable();

        let mut total = 0;
        let mut failed = vec![];

        for dir in &dirs {
            // Get jsfmt.spec.js
            let mut spec_path = dir.join(SNAP_NAME);
            while !spec_path.exists() {
                spec_path = dir.parent().unwrap().join(SNAP_NAME);
            }

            if !spec_path.exists() {
                continue;
            }

            // Get all the other input files
            let mut inputs: Vec<PathBuf> = WalkDir::new(dir)
                .min_depth(1)
                .max_depth(1)
                .into_iter()
                .filter_map(Result::ok)
                .filter(|e| !e.file_type().is_dir())
                .filter(|e| {
                    !self.ignore_tests.iter().any(|s| e.path().to_string_lossy().contains(s))
                })
                .filter(|e| {
                    self.options
                        .filter
                        .as_ref()
                        .map_or(true, |name| e.path().to_string_lossy().contains(name))
                        && !e
                            .path()
                            .file_name()
                            .is_some_and(|name| name.to_string_lossy().contains(SNAP_NAME))
                })
                .map(|e| e.path().to_path_buf())
                .collect();

            self.spec.parse(&spec_path);
            debug_assert!(
                !self.spec.calls.is_empty(),
                "There is no `runFormatTest()` in {}, please check if it is correct?",
                spec_path.to_string_lossy()
            );
            total += inputs.len();
            inputs.sort_unstable();
            self.test_snapshot(dir, &spec_path, &inputs, &mut failed);
        }

        let language = self.language.as_str();
        let passed = total - failed.len();
        let percentage = (passed as f64 / total as f64) * 100.0;
        let heading = format!("{language} compatibility: {passed}/{total} ({percentage:.2}%)");
        println!("{heading}");

        if self.options.filter.is_none() {
            let failed = failed.join("\n");
            let snapshot = format!("{heading}\n\n# Failed\n{failed}");
            let filename = format!("prettier.{language}.snap.md");
            fs::write(snap_root().join(filename), snapshot).unwrap();
        }
    }

    fn test_snapshot(
        &self,
        dir: &Path,
        spec_path: &Path,
        inputs: &[PathBuf],
        failed: &mut Vec<String>,
    ) {
        let mut write_dir_info = true;
        for path in inputs {
            let input = fs::read_to_string(path).unwrap();

            let result = self.spec.calls.iter().all(|spec| {
                let expected_file = spec_path.parent().unwrap().join(SNAP_RELATIVE_PATH);
                let expected = fs::read_to_string(expected_file).unwrap();
                let snapshot = self.get_single_snapshot(path, &input, spec.0, &spec.1, &expected);
                if snapshot.trim().is_empty() {
                    return false;
                }

                if inputs.is_empty() {
                    return false;
                }

                expected.contains(&snapshot)
            });

            if self.spec.calls.is_empty() || !result {
                let mut dir_info = String::new();
                if write_dir_info {
                    dir_info.push_str(
                        format!(
                            "\n### {}\n",
                            dir.strip_prefix(&self.fixtures_root).unwrap().to_string_lossy()
                        )
                        .as_str(),
                    );
                    write_dir_info = false;
                }

                failed.push(format!(
                    "{dir_info}* {}",
                    path.strip_prefix(&self.fixtures_root).unwrap().to_string_lossy()
                ));
            }
        }
    }

    fn visualize_end_of_line(content: &str) -> String {
        let mut chars = content.chars();
        let mut result = String::new();

        loop {
            let current = chars.next();
            let Some(char) = current else {
                break;
            };

            match char {
                LF => result.push_str("<LF>\n"),
                CR => {
                    let next = chars.clone().next();
                    if next == Some(LF) {
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
        result
    }

    fn get_single_snapshot(
        &self,
        path: &Path,
        input: &str,
        prettier_options: PrettierOptions,
        snapshot_options: &[(String, String)],
        snap_content: &str,
    ) -> String {
        let filename = path.file_name().unwrap().to_string_lossy();

        let snapshot_line = snapshot_options
            .iter()
            .filter(|k| {
                if k.0 == "parsers" {
                    false
                } else if k.0 == "printWidth" {
                    return k.1 != "80";
                } else {
                    true
                }
            })
            .map(|(k, v)| format!("\"{k}\":{v}"))
            .collect::<Vec<_>>()
            .join(",");

        let title_snapshot_options = format!("- {{{snapshot_line}}} ",);

        let title = format!(
            "exports[`{filename} {}format 1`] = `",
            if snapshot_line.is_empty() { String::new() } else { title_snapshot_options }
        );

        let need_eol_visualized = snap_content.contains("<LF>");
        let output = Self::prettier(path, input, prettier_options);
        let output = Self::escape_and_convert_snap_string(&output, need_eol_visualized);
        let input = Self::escape_and_convert_snap_string(input, need_eol_visualized);
        let snapshot_options = snapshot_options
            .iter()
            .map(|(k, v)| format!("{k}: {v}"))
            .collect::<Vec<_>>()
            .join("\n");

        let space_line = " ".repeat(prettier_options.print_width);
        let snapshot_without_output = format!(
            r#"
{title}
====================================options=====================================
{snapshot_options}
{space_line}| printWidth
=====================================input======================================
{input}"#
        );

        let snapshot_output = format!(
            r#"
=====================================output=====================================
{output}

================================================================================
`;"#
        );

        // put it here but not in below if-statement to help detect no matched input cases.
        let expected = Self::get_expect(snap_content, &snapshot_without_output).unwrap_or_default();

        if self.options.filter.is_some() {
            println!("Input path: {}", path.to_string_lossy());
            if !snapshot_line.is_empty() {
                println!("Options: \n{snapshot_line}\n");
            }
            println!("Input:");
            println!("{input}");
            println!("Output:");
            println!("{output}");
            println!("Diff:");
            println!("{}", Self::get_diff(&output, &expected));
        }

        format!("{snapshot_without_output}{snapshot_output}")
    }

    fn get_expect(expected: &str, input: &str) -> Option<String> {
        let input_started = expected.find(input)?;
        let expected = &expected[input_started..];
        let output_start_line =
            "=====================================output=====================================\n";
        let output_end_line =
            "================================================================================";
        let output_started = expected.find(output_start_line)?;
        let output_ended = expected.find(output_end_line)?;
        let output = expected[output_started..output_ended]
            .trim_start_matches(output_start_line)
            .trim_end_matches(output_end_line);
        Some(output.to_string())
    }

    fn get_diff(output: &str, expect: &str) -> String {
        let output = output.trim().lines().collect::<Vec<_>>();
        let expect = expect.trim().lines().collect::<Vec<_>>();
        let length = output.len().max(expect.len());
        let mut result = String::new();

        for i in 0..length {
            let left = output.get(i).unwrap_or(&"");
            let right = expect.get(i).unwrap_or(&"");

            let s = if left == right {
                format!("{left: <80} | {right: <80}\n")
            } else {
                format!("{left: <80} X {right: <80}\n")
            };

            result.push_str(&s);
        }

        result
    }

    fn escape_and_convert_snap_string(input: &str, need_eol_visualized: bool) -> String {
        let input = input.replace('\\', "\\\\").replace('`', "\\`").replace("${", "\\${");
        if need_eol_visualized {
            Self::visualize_end_of_line(&input)
        } else {
            input
        }
    }

    fn prettier(path: &Path, source_text: &str, prettier_options: PrettierOptions) -> String {
        let allocator = Allocator::default();
        let source_type = SourceType::from_path(path).unwrap();
        let ret = Parser::new(&allocator, source_text, source_type)
            .with_options(ParseOptions { preserve_parens: false, ..ParseOptions::default() })
            .parse();
        Prettier::new(&allocator, prettier_options).build(&ret.program)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::{fixtures_root, TestRunner, SNAP_RELATIVE_PATH};

    fn get_expect_in_arrays(input_name: &str) -> String {
        let base = fixtures_root().join("js/arrays");
        let expect_file = fs::read_to_string(base.join(SNAP_RELATIVE_PATH)).unwrap();
        let input = fs::read_to_string(base.join(input_name)).unwrap();
        TestRunner::get_expect(&expect_file, &input).unwrap()
    }

    #[ignore]
    #[test]
    fn test_get_expect() {
        let expected = get_expect_in_arrays("empty.js");
        assert_eq!(
            expected,
            "const a =
  someVeryVeryVeryVeryVeryVeryVeryVeryVeryVeryVeryVeLong.Expression || [];
const b =
  someVeryVeryVeryVeryVeryVeryVeryVeryVeryVeryVeryVeLong.Expression || {};

"
        );
    }

    #[ignore]
    #[test]
    fn test_get_diff() {
        let expected = get_expect_in_arrays("empty.js");
        let output = "
const a =
  someVeryVeryVeryVeryVeryVeryVeryVeryVeryVeryVeryVeLong.Expression ||
  []
;
const b =
  someVeryVeryVeryVeryVeryVeryVeryVeryVeryVeryVeryVeLong.Expression ||
  {}
;";
        let diff = TestRunner::get_diff(output, &expected);
        let expected_diff = "
const a =                                                                        | const a =
  someVeryVeryVeryVeryVeryVeryVeryVeryVeryVeryVeryVeLong.Expression ||           X   someVeryVeryVeryVeryVeryVeryVeryVeryVeryVeryVeryVeLong.Expression || [];
  []                                                                             X const b =
;                                                                                X   someVeryVeryVeryVeryVeryVeryVeryVeryVeryVeryVeryVeLong.Expression || {};
const b =                                                                        X
  someVeryVeryVeryVeryVeryVeryVeryVeryVeryVeryVeryVeLong.Expression ||           X
  {}                                                                             X
;                                                                                X";

        assert_eq!(diff.trim(), expected_diff.trim());
    }
}
