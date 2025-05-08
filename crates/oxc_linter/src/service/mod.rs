use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    sync::Arc,
};

use oxc_diagnostics::DiagnosticSender;
use runtime::Runtime;
pub use runtime::RuntimeFileSystem;

use crate::Linter;

mod runtime;

#[cfg(feature = "language_server")]
pub mod offset_to_position;

pub struct LintServiceOptions {
    /// Current working directory
    cwd: Box<Path>,

    /// All paths to lint
    paths: Vec<Arc<OsStr>>,

    /// TypeScript `tsconfig.json` path for reading path alias and project references
    tsconfig: Option<PathBuf>,

    cross_module: bool,
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

pub struct LintService<'l> {
    runtime: Runtime<'l>,
}

impl<'l> LintService<'l> {
    pub fn new(linter: &'l Linter, options: LintServiceOptions) -> Self {
        let runtime = Runtime::new(linter, options);
        Self { runtime }
    }

    #[must_use]
    pub fn with_file_system(
        mut self,
        file_system: Box<dyn RuntimeFileSystem + Sync + Send>,
    ) -> Self {
        self.runtime = self.runtime.with_file_system(file_system);
        self
    }

    /// # Panics
    pub fn run(&mut self, tx_error: &DiagnosticSender) {
        self.runtime.run(tx_error);
        tx_error.send(None).unwrap();
    }

    #[cfg(feature = "language_server")]
    pub fn run_source<'a>(
        &mut self,
        allocator: &'a oxc_allocator::Allocator,
    ) -> Vec<crate::MessageWithPosition<'a>> {
        self.runtime.run_source(allocator)
    }

    /// For tests
    #[cfg(test)]
    pub(crate) fn run_test_source<'a>(
        &mut self,
        allocator: &'a oxc_allocator::Allocator,
        check_syntax_errors: bool,
        tx_error: &DiagnosticSender,
    ) -> Vec<crate::Message<'a>> {
        self.runtime.run_test_source(allocator, check_syntax_errors, tx_error)
    }
}
