use std::{ffi::OsStr, path::PathBuf, sync::Arc};

use oxc_linter::rules::RuleEnum;

/// State required to initialize the `tsgolint` linter.
#[derive(Debug, Clone)]
pub struct TsGoLintState {
    /// Current working directory to run `tsgolint` in
    pub cwd: PathBuf,
    /// The paths of files to lint
    #[expect(dead_code)]
    pub paths: Vec<Arc<OsStr>>,
    /// The rules to run when linting
    #[expect(dead_code)]
    pub rules: Vec<RuleEnum>,
}
