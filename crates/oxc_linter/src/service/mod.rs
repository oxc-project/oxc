use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use rustc_hash::FxHashMap;

use oxc_diagnostics::DiagnosticSender;

use crate::Linter;

mod runtime;
use runtime::Runtime;
pub use runtime::RuntimeFileSystem;
#[derive(Clone)]
pub struct LintServiceOptions {
    /// Current working directory
    cwd: Box<Path>,
    /// TypeScript `tsconfig.json` path for reading path alias and project references
    tsconfig: Option<PathBuf>,

    cross_module: bool,
}

impl LintServiceOptions {
    #[must_use]
    pub fn new<T>(cwd: T) -> Self
    where
        T: Into<Box<Path>>,
    {
        Self { cwd: cwd.into(), tsconfig: None, cross_module: false }
    }

    #[inline]
    #[must_use]
    pub fn with_tsconfig<T>(mut self, tsconfig: T) -> Self
    where
        T: Into<PathBuf>,
    {
        let tsconfig = tsconfig.into();
        // Should this be canonicalized?
        let tsconfig = if tsconfig.is_relative() { self.cwd.join(tsconfig) } else { tsconfig };
        debug_assert!(tsconfig.is_file());

        self.tsconfig = Some(tsconfig);
        self
    }

    #[inline]
    #[must_use]
    pub fn with_cross_module(mut self, cross_module: bool) -> Self {
        self.cross_module = cross_module;
        self
    }

    #[inline]
    pub fn cwd(&self) -> &Path {
        &self.cwd
    }
}

pub struct LintService {
    runtime: Runtime,
}

impl LintService {
    pub fn new(linter: Linter, options: LintServiceOptions) -> Self {
        let runtime = Runtime::new(linter, options);
        Self { runtime }
    }

    pub fn with_file_system(
        &mut self,
        file_system: Box<dyn RuntimeFileSystem + Sync + Send>,
    ) -> &mut Self {
        self.runtime.with_file_system(file_system);
        self
    }

    pub fn with_paths(&mut self, paths: Vec<Arc<OsStr>>) -> &mut Self {
        self.runtime.with_paths(paths);
        self
    }

    /// # Panics
    pub fn run(&mut self, tx_error: &DiagnosticSender) {
        self.runtime.run(tx_error);
    }

    pub fn set_disable_directives_map(
        &mut self,
        map: Arc<Mutex<FxHashMap<PathBuf, crate::disable_directives::DisableDirectives>>>,
    ) {
        self.runtime.set_disable_directives_map(map);
    }

    #[cfg(feature = "language_server")]
    pub fn run_source(&mut self) -> Vec<crate::Message> {
        self.runtime.run_source()
    }

    /// For tests
    #[cfg(test)]
    pub(crate) fn run_test_source(
        &mut self,
        check_syntax_errors: bool,
        tx_error: &DiagnosticSender,
    ) -> Vec<crate::Message> {
        self.runtime.run_test_source(check_syntax_errors, tx_error)
    }
}
