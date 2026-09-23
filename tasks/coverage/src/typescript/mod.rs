use std::{
    fs,
    path::{Path, PathBuf},
};

use rustc_hash::{FxHashMap, FxHashSet};

use crate::workspace_root;

pub mod constants;
mod diagnostics_code_collector;
pub mod error_baseline;
pub mod meta;
pub mod scanner;
pub mod transpile_runner;
pub mod type_symbol_baseline;

pub use diagnostics_code_collector::save_reviewed_tsc_diagnostics_codes;

pub const CASES_PATH: &str = "typescript/tsc/testdata/tests/cases";
pub const BASELINES_PATH: &str = "typescript/tsc/testdata/baselines/reference";

pub fn baseline_root(test_path: &Path) -> PathBuf {
    let mut components = test_path.components();
    components
        .find(|component| component.as_os_str() == "cases")
        .expect("TypeScript test path should contain `cases`");
    let suite = components.next().expect("TypeScript test path should contain a suite");
    workspace_root().join(BASELINES_PATH).join(suite)
}

/// Tests for which upstream actually produced a reference baseline. Native TSC leaves skipped
/// fixtures in the cases directory, so a missing error baseline alone does not imply success.
pub struct ReferenceBaselines(FxHashMap<PathBuf, FxHashSet<String>>);

impl ReferenceBaselines {
    pub fn new() -> Self {
        let suites = ["compiler", "conformance"]
            .into_iter()
            .map(|suite| {
                let root = workspace_root().join(BASELINES_PATH).join(suite);
                let tests = fs::read_dir(&root)
                    .expect("TypeScript reference baselines should exist")
                    .map(|entry| entry.expect("TypeScript reference baseline should be readable"))
                    .filter_map(|entry| {
                        let name = entry.file_name();
                        reference_test_name(name.to_str()?).map(str::to_owned)
                    })
                    .collect();
                (root, tests)
            })
            .collect();
        Self(suites)
    }

    pub fn contains(&self, test_path: &Path) -> bool {
        let Some(name) = test_path.file_stem().and_then(|name| name.to_str()) else {
            return false;
        };
        self.0.get(&baseline_root(test_path)).is_some_and(|tests| tests.contains(name))
    }
}

fn reference_test_name(name: &str) -> Option<&str> {
    let stem = [
        ".errors.txt",
        ".trace.json",
        ".sourcemap.txt",
        ".js.map",
        ".js",
        ".types",
        ".symbols",
        ".contentmapper",
    ]
    .into_iter()
    .find_map(|suffix| name.strip_suffix(suffix))?;
    Some(stem.split_once('(').map_or(stem, |(name, _)| name))
}

#[test]
fn reference_baseline_names() {
    assert_eq!(reference_test_name("decoratorOnClass6.es6.js"), Some("decoratorOnClass6.es6"));
    assert_eq!(
        reference_test_name("example(strict=true,target=es2015).errors.txt"),
        Some("example")
    );
    assert_eq!(reference_test_name("example.js.map"), Some("example"));
    assert_eq!(reference_test_name("example.types"), Some("example"));
    assert_eq!(reference_test_name("example.trace.json"), Some("example"));
    assert_eq!(reference_test_name("example.ts"), None);
}
