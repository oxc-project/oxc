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
pub use runtime::{OsFileSystem, RuntimeFileSystem};
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

    /// # Panics
    pub fn run(
        &self,
        file_system: &(dyn RuntimeFileSystem + Sync + Send),
        paths: Vec<Arc<OsStr>>,
        tx_error: &DiagnosticSender,
    ) {
        self.runtime.run(file_system, paths, tx_error);
    }

    pub fn set_disable_directives_map(
        &mut self,
        map: Arc<Mutex<FxHashMap<PathBuf, crate::disable_directives::DisableDirectives>>>,
    ) {
        self.runtime.set_disable_directives_map(map);
    }

    pub fn run_source(
        &self,
        file_system: &(dyn RuntimeFileSystem + Sync + Send),
        paths: Vec<Arc<OsStr>>,
    ) -> Vec<crate::Message> {
        self.runtime.run_source(file_system, paths)
    }

    /// For tests
    #[cfg(test)]
    pub(crate) fn run_test_source(
        &self,
        file_system: &(dyn RuntimeFileSystem + Sync + Send),
        paths: Vec<Arc<OsStr>>,
        check_syntax_errors: bool,
        tx_error: &DiagnosticSender,
    ) -> Vec<crate::Message> {
        self.runtime.run_test_source(file_system, paths, check_syntax_errors, tx_error)
    }
}
