use std::{ffi::OsStr, path::Path, sync::Arc};

use oxc_diagnostics::DiagnosticSender;

use crate::Linter;

mod runtime;
use runtime::Runtime;
pub use runtime::RuntimeFileSystem;

#[cfg(feature = "language_server")]
pub mod offset_to_position;

pub struct LintServiceOptions {
    /// Current working directory
    cwd: Box<Path>,
    /// Whether to enable cross-module resolution.
    cross_module: bool,
}

impl LintServiceOptions {
    #[must_use]
    pub fn new<T>(cwd: T) -> Self
    where
        T: Into<Box<Path>>,
    {
        Self { cwd: cwd.into(), cross_module: false }
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
    pub fn new(
        linter: Linter,
        allocator_pool: oxc_allocator::AllocatorPool,
        options: LintServiceOptions,
    ) -> Self {
        let runtime = Runtime::new(linter, allocator_pool, options);
        Self { runtime }
    }

    #[must_use]
    pub fn with_file_system(
        &mut self,
        file_system: Box<dyn RuntimeFileSystem + Sync + Send>,
    ) -> &mut Self {
        self.runtime.with_file_system(file_system);
        self
    }

    #[must_use]
    pub fn with_paths(&mut self, paths: Vec<Arc<OsStr>>) -> &mut Self {
        self.runtime.with_paths(paths);
        self
    }

    /// # Panics
    pub fn run(&mut self, tx_error: &DiagnosticSender) {
        self.runtime.run(tx_error);
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
