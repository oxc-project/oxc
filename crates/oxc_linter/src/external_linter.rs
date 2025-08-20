use std::{fmt::Debug, pin::Pin, sync::Arc};

use serde::{Deserialize, Serialize};

use oxc_allocator::Allocator;

/// Callback functions for a JS worker thread.
pub struct ExternalLinterWorkerCallbacks {
    pub load_plugins: ExternalLinterLoadPluginsCb,
    pub lint_file: ExternalLinterLintFileCb,
}

/// Initialize JS worker threads.
pub type ExternalLinterInitWorkerThreadsCb = Arc<
    dyn Fn(
            u32,
        ) -> Pin<
            Box<dyn Future<Output = Result<Box<[ExternalLinterWorkerCallbacks]>, String>> + Send>,
        > + Send
        + Sync
        + 'static,
>;

/// Load a JS plugin on main thread.
pub type ExternalLinterLoadPluginCb = Arc<
    dyn Fn(String) -> Pin<Box<dyn Future<Output = Result<PluginLoadResult, String>> + Send>>
        + Send
        + Sync
        + 'static,
>;

/// Load multiple JS plugins on a worker thread.
pub type ExternalLinterLoadPluginsCb = Arc<
    dyn Fn(Vec<String>) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send>>
        + Send
        + Sync
        + 'static,
>;

/// Lint a file on a worker thread.
pub type ExternalLinterLintFileCb = Arc<
    dyn Fn(String, Vec<u32>, &Allocator, usize) -> Result<Vec<LintFileResult>, String>
        + Sync
        + Send,
>;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum PluginLoadResult {
    #[serde(rename_all = "camelCase")]
    Success {
        name: String,
        offset: usize,
        rule_names: Vec<String>,
    },
    Failure(String),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LintFileResult {
    pub rule_index: u32,
    pub message: String,
    pub loc: Loc,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Loc {
    pub start: u32,
    pub end: u32,
}

#[derive(Clone)]
#[cfg_attr(not(all(feature = "oxlint2", not(feature = "disable_oxlint2"))), expect(dead_code))]
pub struct ExternalLinter {
    init_worker_threads: ExternalLinterInitWorkerThreadsCb,
    load_plugin: ExternalLinterLoadPluginCb,
    lint_file_fns: Box<[ExternalLinterLintFileCb]>,
}

impl ExternalLinter {
    pub fn new(
        init_worker_threads: ExternalLinterInitWorkerThreadsCb,
        load_plugin: ExternalLinterLoadPluginCb,
    ) -> Self {
        Self { init_worker_threads, load_plugin, lint_file_fns: Box::new([]) }
    }

    /// Initialize JS worker threads.
    ///
    /// # Panics
    ///
    /// Panics if either:
    /// * The current thread is not a Tokio runtime thread.
    /// * The JS worker threads failed to initialize.
    #[cfg(all(feature = "oxlint2", not(feature = "disable_oxlint2")))]
    pub fn init_worker_threads(&self, thread_count: u32) -> Box<[ExternalLinterWorkerCallbacks]> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on((self.init_worker_threads)(thread_count))
        })
        .unwrap()
    }

    #[cfg(not(all(feature = "oxlint2", not(feature = "disable_oxlint2"))))]
    #[expect(unused_variables, clippy::unused_self)]
    pub fn init_worker_threads(&self, thread_count: u32) -> Box<[ExternalLinterWorkerCallbacks]> {
        unreachable!();
    }

    /// Set the lint file callbacks for each worker thread.
    pub fn set_lint_file_on_threads(
        &mut self,
        lint_file_on_threads: Box<[ExternalLinterLintFileCb]>,
    ) {
        self.lint_file_fns = lint_file_on_threads;
    }

    /// Load a JS plugin on main thread.
    ///
    /// # Errors
    /// Returns `Err` if error on JS side.
    ///
    /// # Panics
    /// Panics if the current thread is not a Tokio runtime thread.
    #[cfg(all(feature = "oxlint2", not(feature = "disable_oxlint2")))]
    pub fn load_plugin(&self, plugin_path: &str) -> Result<PluginLoadResult, String> {
        let plugin_path = plugin_path.to_string();
        tokio::task::block_in_place(move || {
            tokio::runtime::Handle::current().block_on((self.load_plugin)(plugin_path))
        })
    }

    #[cfg(not(all(feature = "oxlint2", not(feature = "disable_oxlint2"))))]
    #[expect(unused_variables, clippy::unused_self, clippy::missing_errors_doc)]
    pub fn load_plugin(&self, plugin_path: &str) -> Result<PluginLoadResult, String> {
        unreachable!();
    }

    /// Lint a file.
    ///
    /// # Errors
    ///
    /// Returns `Err` if:
    /// * Error on JS side.
    /// * Current thread is not a Rayon thread.
    #[expect(clippy::unnecessary_safety_comment)]
    pub fn lint_file(
        &self,
        file_path: String,
        rule_ids: Vec<u32>,
        allocator: &Allocator,
    ) -> Result<Vec<LintFileResult>, String> {
        let thread_id = rayon::current_thread_index()
            .ok_or_else(|| String::from("Current thread must be a Rayon thread"))?;

        let lint_file = &self.lint_file_fns[thread_id];
        // SAFETY: We have verified that the current thread is a Rayon thread.
        // `rayon::current_thread_index()` always returns a number less than the number of threads in pool.
        lint_file(file_path, rule_ids, allocator, thread_id)
    }
}

impl Debug for ExternalLinter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExternalLinter").finish()
    }
}
