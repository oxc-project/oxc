use std::{
    fs,
    path::{Path, PathBuf},
};

use walkdir::{DirEntry, WalkDir};

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_prettier::{Prettier, PrettierOptions};
use oxc_span::SourceType;
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
}

fn root() -> PathBuf {
    project_root().join("tasks/prettier_conformance")
}

fn fixtures_root() -> PathBuf {
    project_root().join(root()).join("prettier/tests/format/js")
}

impl TestRunner {
    pub fn new(options: TestRunnerOptions) -> Self {
        Self { options }
    }

    /// # Panics
    #[allow(clippy::cast_precision_loss)]
    pub fn run(self) {
        let fixture_root = fixtures_root();
        // Read the first level of directories that contain `__snapshots__`
        let mut dirs = WalkDir::new(&fixture_root)
            .min_depth(1)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| {
                self.options
                    .filter
                    .as_ref()
                    .map_or(true, |name| e.path().to_string_lossy().contains(name))
            })
            .filter(|e| e.file_type().is_dir() && e.path().join("__snapshots__").exists())
            .map(DirEntry::into_path)
            .collect::<Vec<_>>();

        dirs.sort_unstable();

        let mut failed = vec![];

        for dir in &dirs {
            // Get jsfmt.spec.js and all the other input files
            let (specs, mut inputs): (Vec<PathBuf>, Vec<PathBuf>) = WalkDir::new(dir)
                .min_depth(1)
                .max_depth(1)
                .into_iter()
                .filter_map(Result::ok)
                .filter(|e| !e.file_type().is_dir())
                .map(|e| e.path().to_path_buf())
                .partition(|path| path.file_name().is_some_and(|name| name == "jsfmt.spec.js"));

            inputs.sort_unstable();
            if !self.test_snapshot(&specs[0], &inputs) {
                failed.push(format!(
                    "* {}",
                    dir.strip_prefix(&fixture_root).unwrap().to_string_lossy()
                ));
            }
        }

        let total = dirs.len();
        let passed = total - failed.len();
        let percentage = (passed as f64 / total as f64) * 100.0;
        let heading = format!("Compatibility: {passed}/{total} ({percentage:.2}%)");
        println!("{heading}");

        if self.options.filter.is_none() {
            let failed = failed.join("\n");
            let snapshot = format!("{heading}\n\n# Failed\n\n{failed}");
            fs::write(root().join("prettier.snap.md"), snapshot).unwrap();
        }
    }

    fn test_snapshot(&self, spec: &Path, inputs: &[PathBuf]) -> bool {
        let inputs = inputs
            .iter()
            .map(|path| {
                let input = fs::read_to_string(path).unwrap();
                self.get_single_snapshot(path, &input)
            })
            .collect::<Vec<_>>()
            .join("\n");

        let snapshot = format!("// Jest Snapshot v1, https://goo.gl/fbAQLP\n{inputs}");

        let expected_file = spec.parent().unwrap().join("__snapshots__/jsfmt.spec.js.snap");
        let expected = fs::read_to_string(expected_file).unwrap();

        snapshot == expected
    }

    fn get_single_snapshot(&self, path: &Path, input: &str) -> String {
        let filename = path.file_name().unwrap().to_string_lossy();
        let output = Self::prettier(path, input);

        if self.options.filter.is_some() {
            println!("Input:");
            println!("{input}");
            println!("Output:");
            println!("{output}");
        }

        format!(
            r#"
exports[`{filename} format 1`] = `
====================================options=====================================
parsers: ["babel", "flow", "typescript"]
printWidth: 80
                                                                                | printWidth
=====================================input======================================
{input}
=====================================output=====================================
{output}
================================================================================
`;
"#
        )
    }

    fn prettier(path: &Path, source_text: &str) -> String {
        let allocator = Allocator::default();
        let source_type = SourceType::from_path(path).unwrap();
        let ret = Parser::new(&allocator, source_text, source_type).parse();
        Prettier::new(&allocator, PrettierOptions::default()).build(&ret.program)
    }
}
