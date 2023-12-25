use indexmap::IndexMap;
use oxc_tasks_common::{normalize_path, project_root};
use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};
use test_case::TestCaseKind;
use walkdir::WalkDir;

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
}

fn root() -> PathBuf {
    project_root().join("tasks/coverage/babel/packages")
}

fn snap_root() -> PathBuf {
    project_root().join("tasks/transform_conformance")
}

fn fixture_root() -> PathBuf {
    snap_root().join("fixtures")
}

const CASES: &[&str] = &[
    // ES2024
    "babel-plugin-transform-unicode-sets-regex",
    // ES2022
    "babel-plugin-transform-class-properties",
    "babel-plugin-transform-class-static-block",
    "babel-plugin-transform-private-methods",
    "babel-plugin-transform-private-property-in-object",
    // [Syntax] "babel-plugin-transform-syntax-top-level-await",
    // ES2021
    "babel-plugin-transform-logical-assignment-operators",
    "babel-plugin-transform-numeric-separator",
    // ES2020
    "babel-plugin-transform-export-namespace-from",
    "babel-plugin-transform-dynamic-import",
    "babel-plugin-transform-nullish-coalescing-operator",
    "babel-plugin-transform-optional-chaining",
    // [Syntax] "babel-plugin-transform-syntax-bigint",
    // [Syntax] "babel-plugin-transform-syntax-dynamic-import",
    // [Syntax] "babel-plugin-transform-syntax-import-meta",
    // ES2019
    "babel-plugin-transform-optional-catch-binding",
    "babel-plugin-transform-json-strings",
    // ES2018
    "babel-plugin-transform-async-generator-functions",
    "babel-plugin-transform-object-rest-spread",
    // [Regex] "babel-plugin-transform-unicode-property-regex",
    "babel-plugin-transform-dotall-regex",
    // [Regex] "babel-plugin-transform-named-capturing-groups-regex",
    // ES2017
    "babel-plugin-transform-async-to-generator",
    // ES2016
    "babel-plugin-transform-exponentiation-operator",
    // ES2015
    "babel-plugin-transform-arrow-functions",
    "babel-plugin-transform-function-name",
    "babel-plugin-transform-shorthand-properties",
    "babel-plugin-transform-sticky-regex",
    "babel-plugin-transform-unicode-regex",
    "babel-plugin-transform-template-literals",
    "babel-plugin-transform-duplicate-keys",
    "babel-plugin-transform-instanceof",
    // ES3
    "babel-plugin-transform-property-literals",
    // TypeScript
    "babel-plugin-transform-typescript",
    // React
    "babel-plugin-transform-react-jsx",
];

const EXCLUDE_TESTS: &[&str] = &["babel-plugin-transform-typescript/test/fixtures/enum"];

const CONFORMANCE_SNAPSHOT: &str = "babel.snap.md";
const EXEC_SNAPSHOT: &str = "babel_exec.snap.md";

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
        Self { options }
    }

    /// # Panics
    pub fn run(self) {
        let root = root();
        let (transform_paths, exec_files) = Self::glob_files(&root, self.options.filter.as_ref());
        self.generate_snapshot(SnapshotOption::new(transform_paths, CONFORMANCE_SNAPSHOT));

        if self.options.exec {
            let fixture_root = fixture_root();
            if !fixture_root.exists() {
                fs::create_dir(&fixture_root).unwrap();
            }
            self.generate_snapshot(SnapshotOption::new(exec_files, EXEC_SNAPSHOT));
        }
    }

    fn glob_files(
        root: &Path,
        filter: Option<&String>,
    ) -> (IndexMap<String, Vec<TestCaseKind>>, IndexMap<String, Vec<TestCaseKind>>) {
        // use `IndexMap` to keep the order of the test cases the same in insert order.
        let mut transform_files = IndexMap::<String, Vec<TestCaseKind>>::new();
        let mut exec_files = IndexMap::<String, Vec<TestCaseKind>>::new();

        for case in CASES {
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

                        if EXCLUDE_TESTS.iter().any(|p| path.to_string_lossy().contains(p)) {
                            return None;
                        }

                        let test_case = TestCaseKind::from_path(path);
                        if let Some(test_case) = test_case {
                            if test_case.skip_test_case() {
                                return None;
                            }

                            return Some(test_case);
                        }
                        None
                    })
                    .partition(|p| matches!(p, TestCaseKind::Transform(_)));

            transform_paths.sort_unstable_by(|a, b| a.path().cmp(b.path()));
            exec_paths.sort_unstable_by(|a, b| a.path().cmp(b.path()));

            transform_files.insert((*case).to_string(), transform_paths);
            exec_files.insert((*case).to_string(), exec_paths);
        }

        (transform_files, exec_files)
    }

    fn generate_snapshot(&self, option: SnapshotOption) {
        let SnapshotOption { paths, dest } = option;
        let mut snapshot = String::new();
        let mut total = 0;
        let mut all_passed = vec![];
        let mut all_passed_count = 0;

        for (case, test_cases) in paths {
            // Skip empty test cases, e.g. some cases do not have `exec.js` file.
            if test_cases.is_empty() {
                continue;
            }

            let case_root = root().join(&case).join("test/fixtures");
            let num_of_tests = test_cases.len();
            total += num_of_tests;

            // Run the test
            let (passed, failed): (Vec<TestCaseKind>, Vec<TestCaseKind>) = test_cases
                .into_iter()
                .partition(|test_case| test_case.test(self.options.filter.is_some()));
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
                    snapshot.push('\n');
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
            let mut file = File::create(dest).unwrap();
            file.write_all(snapshot.as_bytes()).unwrap();
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
