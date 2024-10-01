#![allow(clippy::print_stdout, clippy::print_stderr)]

mod constants;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use constants::PLUGINS;
use indexmap::IndexMap;
use oxc_tasks_common::{normalize_path, project_root, Snapshot};
use test_case::TestCaseKind;
use walkdir::WalkDir;

mod driver;
mod test_case;

#[test]
#[cfg(any(coverage, coverage_nightly))]
fn test() {
    TestRunner::new(TestRunnerOptions::default()).run();
}

#[derive(Default, Clone)]
pub struct TestRunnerOptions {
    pub filter: Option<String>,
    pub exec: bool,
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

fn oxc_test_root() -> PathBuf {
    conformance_root().join("tests")
}

fn fixture_root() -> PathBuf {
    conformance_root().join("fixtures")
}

const CONFORMANCE_SNAPSHOT: &str = "babel.snap.md";
const OXC_CONFORMANCE_SNAPSHOT: &str = "oxc.snap.md";
const EXEC_SNAPSHOT: &str = "babel_exec.snap.md";
const OXC_EXEC_SNAPSHOT: &str = "oxc_exec.snap.md";

struct SnapshotOption {
    paths: IndexMap<String, Vec<TestCaseKind>>,
    dest: PathBuf,
}

impl SnapshotOption {
    fn new(paths: IndexMap<String, Vec<TestCaseKind>>, file_name: &'static str) -> Self {
        Self { paths, dest: snap_root().join(file_name) }
    }
}

impl TestRunner {
    pub fn new(options: TestRunnerOptions) -> Self {
        let snapshot = Snapshot::new(&babel_root(), /* show_commit */ true);
        Self { options, snapshot }
    }

    /// # Panics
    pub fn run(self) {
        for (root, snapshot, exec_snapshot) in &[
            (packages_root(), CONFORMANCE_SNAPSHOT, EXEC_SNAPSHOT),
            (oxc_test_root(), OXC_CONFORMANCE_SNAPSHOT, OXC_EXEC_SNAPSHOT),
        ] {
            let (transform_paths, exec_files) =
                Self::glob_files(root, self.options.filter.as_ref());
            self.generate_snapshot(root, SnapshotOption::new(transform_paths, snapshot));

            if self.options.exec {
                let fixture_root = fixture_root();
                if !fixture_root.exists() {
                    fs::create_dir(&fixture_root).unwrap();
                }
                self.generate_snapshot(root, SnapshotOption::new(exec_files, exec_snapshot));
                let _ = fs::remove_dir_all(fixture_root);
            }
        }
    }

    fn glob_files(
        root: &Path,
        filter: Option<&String>,
    ) -> (IndexMap<String, Vec<TestCaseKind>>, IndexMap<String, Vec<TestCaseKind>>) {
        let cwd = root.parent().unwrap_or(root);
        // use `IndexMap` to keep the order of the test cases the same in insert order.
        let mut transform_files = IndexMap::<String, Vec<TestCaseKind>>::new();
        let mut exec_files = IndexMap::<String, Vec<TestCaseKind>>::new();

        for case in PLUGINS {
            let root = root.join(case).join("test/fixtures");
            let (mut transform_paths, mut exec_paths): (Vec<TestCaseKind>, Vec<TestCaseKind>) =
                WalkDir::new(root)
                    .into_iter()
                    .filter_map(Result::ok)
                    .filter_map(|e| {
                        let path = e.path();
                        if let Some(filter) = filter {
                            if !path.to_string_lossy().contains(filter) {
                                return None;
                            }
                        }
                        TestCaseKind::new(cwd, path).filter(|test_case| !test_case.skip_test_case())
                    })
                    .partition(|p| matches!(p, TestCaseKind::Transform(_)));

            transform_paths.sort_unstable_by(|a, b| a.path().cmp(b.path()));
            exec_paths.sort_unstable_by(|a, b| a.path().cmp(b.path()));

            if !transform_paths.is_empty() {
                transform_files.insert((*case).to_string(), transform_paths);
            }
            if !exec_paths.is_empty() {
                exec_files.insert((*case).to_string(), exec_paths);
            }
        }

        (transform_files, exec_files)
    }

    fn generate_snapshot(&self, root: &Path, option: SnapshotOption) {
        let SnapshotOption { paths, dest } = option;
        let mut snapshot = String::new();
        let mut total = 0;
        let mut all_passed = vec![];
        let mut all_passed_count = 0;

        for (case, test_cases) in paths {
            let case_root = root.join(&case).join("test/fixtures");
            let num_of_tests = test_cases.len();
            total += num_of_tests;

            // Run the test
            let (passed, failed): (Vec<TestCaseKind>, Vec<TestCaseKind>) = test_cases
                .into_iter()
                .map(|mut test_case| {
                    test_case.test(self.options.filter.is_some());
                    test_case
                })
                .partition(|test_case| test_case.errors().is_empty());
            all_passed_count += passed.len();

            // Snapshot
            if failed.is_empty() {
                all_passed.push(case);
            } else {
                snapshot.push_str("# ");
                snapshot.push_str(&case);
                snapshot.push_str(&format!(" ({}/{})\n", passed.len(), num_of_tests));
                for test_case in failed {
                    snapshot.push_str("* ");
                    snapshot.push_str(&normalize_path(
                        test_case.path().strip_prefix(&case_root).unwrap(),
                    ));
                    let errors = test_case.errors();
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
            self.snapshot.save(&dest, &snapshot);
        }
    }
}

struct TestRunnerEnv;

impl TestRunnerEnv {
    fn template(code: &str) -> String {
        format!(
            r#"
                import {{expect, test}} from 'bun:test';
                test("exec", () => {{
                    {code}
                }})
            "#
        )
    }

    fn get_test_result(path: &Path) -> String {
        let output = Command::new("bun")
            .current_dir(path.parent().unwrap())
            .args(["test", path.file_name().unwrap().to_string_lossy().as_ref()])
            .output()
            .expect("Try install bun: https://bun.sh/docs/installation");

        let content = if output.stderr.is_empty() { &output.stdout } else { &output.stderr };
        String::from_utf8_lossy(content).to_string()
    }

    fn run_test(path: &Path) -> bool {
        Self::get_test_result(path).contains("1 pass")
    }
}
