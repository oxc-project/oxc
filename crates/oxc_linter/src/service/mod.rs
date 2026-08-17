use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use rustc_hash::FxHashMap;

use oxc_allocator::AllocatorPool;
use oxc_diagnostics::DiagnosticSender;

use crate::{Linter, RuleTimingStore, suppression::DiffManager};

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

/// Allocator pools for a lint run.
///
/// Callers that own JS plugin worker startup (the CLI and the language server) build these
/// themselves, because the pools' buffer ids have to be routable to already-running workers.
/// Rust-only callers let [`Runtime`] pick pools for them.
pub struct AllocatorPools {
    /// Pool used for parsing and linting.
    pub parse: AllocatorPool,
    /// Pool used only to copy ASTs into fixed-size arenas before handing them to JS plugins.
    /// `None` unless both JS plugins and the `import` plugin are enabled.
    pub js: Option<AllocatorPool>,
}

pub struct LintService {
    runtime: Runtime,
}

impl LintService {
    pub fn new(linter: Linter, options: LintServiceOptions) -> Self {
        let runtime = Runtime::new(linter, options);
        Self { runtime }
    }

    /// Create a [`LintService`] which uses `pools` instead of creating its own.
    pub fn new_with_allocator_pools(
        linter: Linter,
        options: LintServiceOptions,
        pools: AllocatorPools,
    ) -> Self {
        let runtime = Runtime::new_with_allocator_pools(linter, options, pools);
        Self { runtime }
    }

    /// # Panics
    pub fn run<const TIMINGS: bool>(
        &self,
        file_system: &(dyn RuntimeFileSystem + Sync + Send),
        paths: Vec<Arc<OsStr>>,
        tx_error: &DiagnosticSender,
        diff_manager: &Arc<DiffManager>,
        rule_timing_store: Option<&RuleTimingStore>,
    ) {
        self.runtime.run::<TIMINGS>(file_system, paths, tx_error, diff_manager, rule_timing_store);
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

    pub fn collect_parse_diagnostics(
        &self,
        file_system: &(dyn RuntimeFileSystem + Sync + Send),
        paths: Vec<Arc<OsStr>>,
        tx_error: &DiagnosticSender,
    ) {
        self.runtime.collect_parse_diagnostics(file_system, paths, tx_error);
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
