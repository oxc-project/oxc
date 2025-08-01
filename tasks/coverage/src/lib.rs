#![expect(clippy::print_stdout, clippy::disallowed_methods)]

// Core
mod runtime;
mod suite;
mod task_runners;
// Suites
mod babel;
mod misc;
mod test262;
mod typescript;

mod driver;
mod tools;

use std::path::PathBuf;

use oxc_tasks_common::project_root;

pub use crate::driver::Driver;

pub fn workspace_root() -> PathBuf {
    project_root().join("tasks").join("coverage")
}

fn snap_root() -> PathBuf {
    workspace_root().join("snapshots")
}

#[derive(Debug, Default)]
pub struct AppArgs {
    pub debug: bool,
    pub filter: Option<String>,
    pub detail: bool,
    /// Print mismatch diff
    pub diff: bool,
}

impl AppArgs {
    fn should_print_detail(&self) -> bool {
        self.filter.is_some() || self.detail
    }

    pub fn run_default(&self) {
        task_runners::run_default(self);
    }

    pub fn run_parser(&self) {
        task_runners::run_parser(self);
    }

    pub fn run_semantic(&self) {
        task_runners::run_semantic(self);
    }

    pub fn run_codegen(&self) {
        task_runners::run_codegen(self);
    }

    pub fn run_formatter(&self) {
        task_runners::run_formatter(self);
    }

    pub fn run_transformer(&self) {
        task_runners::run_transformer(self);
    }

    pub fn run_transpiler(&self) {
        task_runners::run_transpiler(self);
    }

    pub fn run_estree(&self) {
        task_runners::run_estree(self);
    }

    /// # Panics
    pub fn run_runtime(&self) {
        task_runners::run_runtime(self);
    }

    pub fn run_minifier(&self) {
        task_runners::run_minifier(self);
    }
}

#[test]
#[cfg(any(coverage, coverage_nightly))]
fn test() {
    AppArgs::default().run_default()
}
