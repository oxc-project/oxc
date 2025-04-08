use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::Linter;

use super::{DiagnosticSender, runtime::Runtime};

pub struct LintServiceOptions {
    /// Current working directory
    pub(crate) cwd: Box<Path>,

    /// All paths to lint
    pub(crate) paths: Vec<Arc<OsStr>>,

    /// TypeScript `tsconfig.json` path for reading path alias and project references
    pub(crate) tsconfig: Option<PathBuf>,

    pub(crate) cross_module: bool,
}

impl LintServiceOptions {
    #[must_use]
    pub fn new<T>(cwd: T, paths: Vec<Arc<OsStr>>) -> Self
    where
        T: Into<Box<Path>>,
    {
        Self { cwd: cwd.into(), paths, tsconfig: None, cross_module: false }
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

    #[cfg(test)]
    pub(crate) fn from_linter(linter: Linter, options: LintServiceOptions) -> Self {
        let runtime = Runtime::new(linter, options);
        Self { runtime }
    }

    pub fn linter(&self) -> &Linter {
        &self.runtime.linter
    }

    /// # Panics
    pub fn run(&mut self, tx_error: &DiagnosticSender) {
        self.runtime.run(tx_error);
        tx_error.send(None).unwrap();
    }

    /// For tests
    #[cfg(test)]
    pub(crate) fn run_source<'a>(
        &mut self,
        allocator: &'a oxc_allocator::Allocator,
        source_text: &str,
        check_syntax_errors: bool,
        tx_error: &DiagnosticSender,
    ) -> Vec<crate::Message<'a>> {
        self.runtime.run_source(allocator, source_text, check_syntax_errors, tx_error)
    }
}
