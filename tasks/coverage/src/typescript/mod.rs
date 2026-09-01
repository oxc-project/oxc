use std::path::{Path, PathBuf};

use crate::workspace_root;

pub mod constants;
mod diagnostics_code_collector;
pub mod error_baseline;
pub mod meta;
pub mod scanner;
pub mod transpile_runner;
pub mod type_symbol_baseline;

pub use diagnostics_code_collector::save_reviewed_tsc_diagnostics_codes;

pub(crate) const CASES_PATH: &str = "typescript/tsc/testdata/tests/cases";
pub(crate) const BASELINES_PATH: &str = "typescript/tsc/testdata/baselines/reference";

pub(crate) fn baseline_root(test_path: &Path) -> PathBuf {
    let mut components = test_path.components();
    components
        .find(|component| component.as_os_str() == "cases")
        .expect("TypeScript test path should contain `cases`");
    let suite = components.next().expect("TypeScript test path should contain a suite");
    workspace_root().join(BASELINES_PATH).join(suite)
}
