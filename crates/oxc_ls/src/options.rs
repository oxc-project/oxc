use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct LintOptions {
    pub paths: Vec<PathBuf>,
    pub fix: bool,
    pub ignore_path: PathBuf,
    pub no_ignore: bool,
    pub ignore_pattern: Vec<String>,
}
