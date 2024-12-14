#![allow(clippy::print_stdout, clippy::print_stderr)]

mod constants;
mod driver;
mod exec;
mod test_case;

use std::{
    fs,
    path::{Path, PathBuf},
};

use constants::PLUGINS;
use indexmap::IndexMap;
use oxc_tasks_common::{normalize_path, project_root, Snapshot};
use test_case::{TestCase, TestCaseKind};
use walkdir::WalkDir;

#[test]
#[cfg(any(coverage, coverage_nightly))]
fn test() {
    TestRunner::new(TestRunnerOptions::default()).run();
}

#[derive(Default, Clone)]
pub struct TestRunnerOptions {
    pub debug: bool,
    pub filter: Option<String>,
    pub exec: bool,
    /// If it's true, will override the output of dismatch test cases,
    /// and write it down to `overrides` folder
    pub r#override: bool,
}

/// The test runner which walks the babel repository and searches for transformation tests.
pub struct TestRunner {
    options: TestRunnerOptions,
    snapshot: Snapshot,
}

fn babel_root() -> PathBuf {
    project_root().join("tasks").join("coverage").join("babel")
}

fn packages_root() -> PathBuf {
    babel_root().join("packages")
}

fn conformance_root() -> PathBuf {
    project_root().join("tasks").join("transform_conformance")
}

fn snap_root() -> PathBuf {
    conformance_root().join("snapshots")
}

fn override_root() -> PathBuf {
    conformance_root().join("overrides")
}

fn oxc_test_root() -> PathBuf {
    conformance_root().join("tests")
}

fn fixture_root() -> PathBuf {
    conformance_root().join("fixtures")
}

impl TestRunner {
    pub fn new(options: TestRunnerOptions) -> Self {
        let snapshot = Snapshot::new(&babel_root(), /* show_commit */ true);
        Self { options, snapshot }
    }

    /// # Panics
    pub fn run(self) {
        for (root, name) in &[(packages_root(), "babel"), (oxc_test_root(), "oxc")] {
            let snapshot = format!("{name}.snap.md");
            let exec_snapshot = format!("{name}_exec.snap.md");
            let fixture_root = fixture_root().join(name);
            if self.options.exec {
                let _ = fs::remove_dir_all(&fixture_root);
                let _ = fs::create_dir_all(&fixture_root);
            }
            let transform_paths = Self::generate_test_cases(root, &self.options);
            self.generate_snapshot(root, &snap_root().join(snapshot), transform_paths);
            if self.options.exec {
                self.run_vitest(&format!("./fixtures/{name}"), &snap_root().join(exec_snapshot));
            }
        }
    }

    fn generate_test_cases(
        root: &Path,
        options: &TestRunnerOptions,
    ) -> IndexMap<String, Vec<TestCase>> {
        let cwd = root.parent().unwrap_or(root);
        // use `IndexMap` to keep the order of the test cases the same in insert order.
        let mut transform_files = IndexMap::<String, Vec<TestCase>>::new();

        for case in PLUGINS {
            let root = root.join(case).join("test/fixtures");

            let mut cases = WalkDir::new(root)
                .into_iter()
                .filter_map(Result::ok)
                .filter(|e| {
                    if let Some(filter) = &options.filter {
                        if !e.path().to_string_lossy().contains(filter) {
                            return false;
                        }
                    }
                    true
                })
                .filter_map(|e| TestCase::new(cwd, e.path()))
                .filter(|test_case| !test_case.skip_test_case())
                .map(|mut case| {
                    case.test(options);
                    case
                })
                .collect::<Vec<_>>();

            cases.sort_unstable_by(|a, b| a.path.cmp(&b.path));

            let transform_cases = cases
                .into_iter()
                .filter(|case| case.kind == TestCaseKind::Conformance)
                .collect::<Vec<_>>();
            if !transform_cases.is_empty() {
                transform_files.insert((*case).to_string(), transform_cases);
            }
        }

        transform_files
    }

    fn generate_snapshot(&self, root: &Path, dest: &Path, paths: IndexMap<String, Vec<TestCase>>) {
        let mut snapshot = String::new();
        let mut total = 0;
        let mut all_passed = vec![];
        let mut all_passed_count = 0;

        for (case, test_cases) in paths {
            let case_root = root.join(&case).join("test/fixtures");
            let num_of_tests = test_cases.len();
            total += num_of_tests;

            // Run the test
            let (passed, failed): (Vec<TestCase>, Vec<TestCase>) =
                test_cases.into_iter().partition(|test_case| test_case.errors.is_empty());
            all_passed_count += passed.len();

            // Snapshot
            if failed.is_empty() {
                all_passed.push(case);
            } else {
                snapshot.push_str("# ");
                snapshot.push_str(&case);
                snapshot.push_str(&format!(" ({}/{})\n", passed.len(), num_of_tests));
                for test_case in failed {
                    if self.options.r#override {
                        test_case.write_override_output();
                    }
                    snapshot.push_str("* ");
                    snapshot.push_str(&normalize_path(
                        test_case.path.strip_prefix(&case_root).unwrap(),
                    ));
                    let errors = test_case.errors;
                    if !errors.is_empty() {
                        snapshot.push('\n');
                        for error in errors {
                            snapshot.push_str(&error.message);
                            snapshot.push('\n');
                        }
                        snapshot.push('\n');
                    }
                }
                snapshot.push('\n');
            }
        }

        if self.options.filter.is_none() {
            let all_passed =
                all_passed.into_iter().map(|s| format!("* {s}")).collect::<Vec<_>>().join("\n");
            let snapshot = format!(
                "Passed: {all_passed_count}/{total}\n\n# All Passed:\n{all_passed}\n\n\n{snapshot}"
            );
            self.snapshot.save(dest, &snapshot);
        }
    }
}
