mod module_cache;
mod runtime;

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use oxc_diagnostics::DiagnosticSender;
use rayon::{iter::ParallelBridge, prelude::ParallelIterator};

use crate::Linter;

use runtime::Runtime;

pub struct LintServiceOptions {
    /// Current working directory
    cwd: Box<Path>,

    /// All paths to lint
    paths: Vec<Box<Path>>,

    /// TypeScript `tsconfig.json` path for reading path alias and project references
    tsconfig: Option<PathBuf>,

    cross_module: bool,
}

impl LintServiceOptions {
    #[must_use]
    pub fn new<T>(cwd: T, paths: Vec<Box<Path>>) -> Self
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

#[derive(Clone)]
pub struct LintService {
    runtime: Arc<Runtime>,
}

impl LintService {
    pub fn new(linter: Linter, options: LintServiceOptions) -> Self {
        let runtime = Arc::new(Runtime::new(linter, options));
        Self { runtime }
    }

    #[cfg(test)]
    pub(crate) fn from_linter(linter: Linter, options: LintServiceOptions) -> Self {
        let runtime = Arc::new(Runtime::new(linter, options));
        Self { runtime }
    }

    pub fn linter(&self) -> &Linter {
        &self.runtime.linter
    }

    pub fn number_of_dependencies(&self) -> usize {
        self.runtime.number_of_dependencies()
    }

    /// # Panics
    pub fn run(&self, tx_error: &DiagnosticSender) {
        self.runtime
            .iter_paths()
            .par_bridge()
            .for_each_with(&self.runtime, |runtime, path| runtime.process_path(path, tx_error));
        tx_error.send(None).unwrap();
    }

    /// For tests
    #[cfg(test)]
    pub(crate) fn run_source<'a>(
        &self,
        allocator: &'a oxc_allocator::Allocator,
        source_text: &'a str,
        check_syntax_errors: bool,
        tx_error: &DiagnosticSender,
    ) -> Vec<crate::Message<'a>> {
        self.runtime
            .iter_paths()
            .flat_map(|path| {
                let source_type = oxc_span::SourceType::from_path(path).unwrap();
                self.runtime.init_cache_state(path);
                self.runtime.process_source(
                    path,
                    allocator,
                    source_text,
                    source_type,
                    check_syntax_errors,
                    tx_error,
                )
            })
            .collect::<Vec<_>>()
    }
}
