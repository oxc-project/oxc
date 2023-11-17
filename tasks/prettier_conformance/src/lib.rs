mod spec;

use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use spec::SpecParser;
use walkdir::WalkDir;

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_prettier::{Prettier, PrettierOptions};
use oxc_span::{Atom, SourceType};
use oxc_tasks_common::project_root;

// #[test]
// #[cfg(any(coverage, coverage_nightly))]
// fn test() {
// TestRunner::new(TestRunnerOptions::default()).run();
// }

#[derive(Default)]
pub struct TestRunnerOptions {
    pub filter: Option<String>,
}

/// The test runner which walks the prettier repository and searches for formatting tests.
pub struct TestRunner {
    options: TestRunnerOptions,
    spec: SpecParser,
}

fn root() -> PathBuf {
    project_root().join("tasks/prettier_conformance")
}

fn fixtures_root() -> PathBuf {
    project_root().join(root()).join("prettier/tests/format/js")
}

impl TestRunner {
    pub fn new(options: TestRunnerOptions) -> Self {
        Self { options, spec: SpecParser::default() }
    }

    /// # Panics
    #[allow(clippy::cast_precision_loss)]
    pub fn run(mut self) {
        let fixture_root = fixtures_root();
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

        let dir_set: HashSet<_> = dirs.iter().cloned().collect();
        dirs = dir_set.into_iter().collect();

        dirs.sort_unstable();

        let mut total = 0;
        let mut failed = vec![];

        for dir in &dirs {
            // Get jsfmt.spec.js
            let mut spec_path = dir.join("jsfmt.spec.js");
            while !spec_path.exists() {
                spec_path = dir.parent().unwrap().join("jsfmt.spec.js");
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
                    self.options
                        .filter
                        .as_ref()
                        .map_or(true, |name| e.path().to_string_lossy().contains(name))
                        && !e
                            .path()
                            .file_name()
                            .is_some_and(|name| name.to_string_lossy().contains("jsfmt.spec.js"))
                })
                .map(|e| e.path().to_path_buf())
                .collect();

            self.spec.parse(&spec_path);
            total += inputs.len();
            inputs.sort_unstable();
            self.test_snapshot(dir, &spec_path, &inputs, &mut failed);
        }

        let passed = total - failed.len();
        let percentage = (passed as f64 / total as f64) * 100.0;
        let heading = format!("Compatibility: {passed}/{total} ({percentage:.2}%)");
        println!("{heading}");

        if self.options.filter.is_none() {
            let failed = failed.join("\n");
            let snapshot = format!("{heading}\n\n# Failed\n{failed}");
            fs::write(root().join("prettier.snap.md"), snapshot).unwrap();
        }
    }

    fn test_snapshot(
        &self,
        dir: &Path,
        spec_path: &Path,
        inputs: &[PathBuf],
        failed: &mut Vec<String>,
    ) {
        let fixture_root = fixtures_root();
        let mut write_dir_info = true;
        for path in inputs {
            let input = fs::read_to_string(path).unwrap();

            let result = self.spec.calls.iter().all(|spec| {
                let snapshot = self.get_single_snapshot(path, &input, spec.0, &spec.1);
                if snapshot.trim().is_empty() {
                    return false;
                }

                let expected_file =
                    spec_path.parent().unwrap().join("__snapshots__/jsfmt.spec.js.snap");

                let expected = fs::read_to_string(expected_file).unwrap();

                if inputs.is_empty() {
                    return false;
                }

                expected.contains(&snapshot)
            });

            if !result {
                let mut dir_info = String::new();
                if write_dir_info {
                    dir_info.push_str(
                        format!(
                            "\n### {}\n",
                            dir.strip_prefix(&fixture_root).unwrap().to_string_lossy()
                        )
                        .as_str(),
                    );
                    write_dir_info = false;
                }

                failed.push(format!(
                    "{dir_info}* {}",
                    path.strip_prefix(&fixture_root).unwrap().to_string_lossy()
                ));
            }
        }
    }

    fn get_single_snapshot(
        &self,
        path: &Path,
        input: &str,
        prettier_options: PrettierOptions,
        snapshot_options: &[(Atom, String)],
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
            .join(", ");

        let title_snapshot_options = format!("- {{{snapshot_line}}} ",);

        let title = format!(
            "exports[`{filename} {}format 1`] = `",
            if snapshot_line.is_empty() { String::new() } else { title_snapshot_options }
        );

        let output = Self::prettier(path, input, prettier_options);
        let snapshot_options = snapshot_options
            .iter()
            .map(|(k, v)| format!("{k}: {v}"))
            .collect::<Vec<_>>()
            .join("\n");

        if self.options.filter.is_some() {
            println!("Input path: {}", path.to_string_lossy());
            println!("Input:");
            println!("{input}");
            println!("Output:");
            println!("{output}");
        }

        let space_line = " ".repeat(prettier_options.print_width);

        format!(
            r#"
{title}
====================================options=====================================
{snapshot_options}
{space_line}| printWidth
=====================================input======================================
{input}
=====================================output=====================================
{output}
================================================================================
`;"#
        )
    }

    fn prettier(path: &Path, source_text: &str, prettier_options: PrettierOptions) -> String {
        let allocator = Allocator::default();
        let source_type = SourceType::from_path(path).unwrap();
        let ret = Parser::new(&allocator, source_text, source_type).parse();
        Prettier::new(&allocator, source_text, ret.trivias, prettier_options).build(&ret.program)
    }
}
