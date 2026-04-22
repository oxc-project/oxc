use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use serde_json::Value;
use tokio::sync::{RwLock, RwLockReadGuard};
use tower_lsp_server::{
    jsonrpc::{Error, Result},
    ls_types::{Registration, Unregistration, Uri, WorkspaceFolder},
};
use tracing::debug;

use crate::{
    capabilities::DiagnosticMode, file_system::ResolvedPath, tool::ToolBuilder,
    worker::WorkspaceWorker,
};

enum WorkerGuardInner<'a> {
    Vec(RwLockReadGuard<'a, Vec<WorkspaceWorker>>),
    #[cfg(test)]
    Single(RwLockReadGuard<'a, WorkspaceWorker>),
}

/// A RAII guard that holds a shared read lock over the workers list and exposes
/// a reference to a single [`WorkspaceWorker`] inside it.
///
/// Obtained from [`WorkerManager::get_worker_for_uri`].
/// The read lock is held for as long as this guard is alive.
pub struct WorkerGuard<'a> {
    guard: WorkerGuardInner<'a>,
    index: usize,
}

impl std::ops::Deref for WorkerGuard<'_> {
    type Target = WorkspaceWorker;

    fn deref(&self) -> &Self::Target {
        match &self.guard {
            WorkerGuardInner::Vec(vec_guard) => &vec_guard[self.index],
            #[cfg(test)]
            WorkerGuardInner::Single(single_guard) => single_guard,
        }
    }
}

/// The mode that the [`WorkerManager`] is operating in, which determines how it manages workers and delegates the task to the tool.
pub enum ManagerMode {
    // the manager requires an explicit workspace to operate
    // these workspaces are managed by the client and communicated via `initialize` + `didChangeWorkspaceFolders`
    #[expect(dead_code)] // needs to be implemented
    RequireWorkspace,
    // the manager works in 2 modes, when no workspaces are configured, it creates workers dynamically for file URIs.
    // When workspaces are reconfigured (added or removed by the client), it creates workers for those and ignores file URIs outside of them.
    DynamicNoWorkspaces(
        // toggle for single file / workspace mode
        AtomicBool,
    ),
    // The manager will create workers dynamically for file URIs. It also supports workspaces configured by the client, but does not require them.
    // This is useful for tasks on URIs that are outside of any configured workspace.
    #[cfg(test)] // needs to be implemented
    DynamicWithWorkspaces(Box<RwLock<WorkspaceWorker>>),
}

/// Manages the lifecycle of [`WorkspaceWorker`]s for the language server.
///
/// Responsibilities:
/// - Storing and providing access to all active workers (one per workspace root).
/// - Tracking whether the server is in *single-file mode* (no workspace folders
///   were provided during `initialize`).
/// - Creating workers via the stored [`ToolBuilder`]s so callers do not need
///   direct access to those builders.
/// - Finding the most-specific worker for any given file URI.
/// - Dynamically creating / tearing down workers in single-file mode.
/// - Handling workspace-folder additions and removals atomically.
///
/// **Workers are never cloned** – they are expensive and each root URI must
/// have at most one live worker at any point in time.
pub struct WorkerManager {
    workers: RwLock<Vec<WorkspaceWorker>>,
    mode: ManagerMode,
    tool_builder: Arc<dyn ToolBuilder>,
}

impl WorkerManager {
    /// Create a new [`WorkerManager`] with no workers and single-file mode disabled.
    pub fn new(tool_builder: Arc<dyn ToolBuilder>) -> Self {
        Self {
            workers: RwLock::new(vec![]),
            mode: ManagerMode::DynamicNoWorkspaces(AtomicBool::new(false)),
            tool_builder,
        }
    }

    #[cfg(test)]
    pub fn new_with_mode(tool_builder: Arc<dyn ToolBuilder>, mode: ManagerMode) -> Self {
        Self { mode, tool_builder, workers: RwLock::new(vec![]) }
    }

    // ── Starting / Stopping ───────────────────────────────────────────────────────

    pub async fn start_manager(&self, workers: Vec<WorkspaceWorker>) {
        *self.workers.write().await = workers;

        // for dynamic workspaces we need to start them manually
        #[cfg(test)]
        if let ManagerMode::DynamicWithWorkspaces(worker) = &self.mode {
            worker.read().await.start_worker(serde_json::Value::Null).await;
        }
    }

    /// Shut down all workers and clear the worker list.
    /// Returns the URIs for which diagnostics should be cleared.
    pub async fn stop_manager(&self) -> Vec<Uri> {
        let mut clear_uris = vec![];
        let workers = {
            let mut workers = self.workers.write().await;
            std::mem::take(&mut *workers)
        };
        for worker in workers {
            // shutdown each worker and collect the URIs to clear diagnostics.
            // unregistering file watchers is not necessary, because the client will do it automatically on shutdown.
            // some clients (`helix`) do not expect any requests after shutdown is sent.
            let (worker_uris, _) = worker.shutdown().await;
            clear_uris.extend(worker_uris);
        }

        #[cfg(test)]
        if let ManagerMode::DynamicWithWorkspaces(worker) = &self.mode {
            let (worker_uris, _) = worker.read().await.shutdown().await;
            clear_uris.extend(worker_uris);
        }

        clear_uris
    }

    // ── State accessors ───────────────────────────────────────────────────────

    /// Acquire a shared read lock over the worker list.
    /// Does not include the dynamic worker in `DynamicWithWorkspaces` mode, which must be accessed by `read_dynamic_worker`.
    pub async fn read_workspace_workers(&self) -> RwLockReadGuard<'_, Vec<WorkspaceWorker>> {
        self.workers.read().await
    }

    /// Acquire a shared read lock over the dynamic worker in `DynamicWithWorkspaces` mode, if enabled.
    #[cfg_attr(not(test), expect(clippy::unused_async))] // when removing the test-only mode, this method will need to perform async initialization for the dynamic worker
    pub async fn read_dynamic_worker(&self) -> Option<RwLockReadGuard<'_, WorkspaceWorker>> {
        #[cfg(test)]
        if let ManagerMode::DynamicWithWorkspaces(worker) = &self.mode {
            return Some(worker.read().await);
        }
        None
    }

    /// Access the tool builder.
    pub fn read_tool_builder(&self) -> &Arc<dyn ToolBuilder> {
        &self.tool_builder
    }

    /// Returns `true` when the server was started without any workspace folders.
    pub fn is_single_file_mode(&self) -> bool {
        matches!(&self.mode, ManagerMode::DynamicNoWorkspaces(flag) if flag.load(Ordering::Relaxed))
    }

    /// Overwrite the single-file-mode flag.
    pub fn set_single_file_mode(&self, value: bool) {
        if let ManagerMode::DynamicNoWorkspaces(flag) = &self.mode {
            flag.store(value, Ordering::Relaxed);
        }
    }

    // ── Worker creation ───────────────────────────────────────────────────────

    /// Append new workers to the list (used after `didChangeWorkspaceFolders`).
    pub async fn add_workers(&self, workers: Vec<WorkspaceWorker>) {
        self.workers.write().await.extend(workers);
    }

    /// Build a new [`WorkspaceWorker`] for the given root URI without starting
    /// it.  Call [`WorkspaceWorker::start_worker`] afterwards.
    pub fn create_worker(&self, root_uri: Uri, diagnostic_mode: DiagnosticMode) -> WorkspaceWorker {
        WorkspaceWorker::new(root_uri, Arc::clone(&self.tool_builder), diagnostic_mode)
    }

    // ── Lookup helpers (associated functions) ─────────────────────────────────

    /// Return the index of the most specific workspace worker for a given URI,
    /// or `None` when no worker covers `uri`.
    ///
    /// For non-`file://` URIs the first worker (index `0`) is returned when
    /// the list is non-empty, mirroring the behaviour of rust-analyzer and
    /// typescript-language-server.
    fn find_worker_index_for_uri(workers: &[WorkspaceWorker], uri: &Uri) -> Option<usize> {
        if uri.scheme().as_str() != "file" {
            return if workers.is_empty() { None } else { Some(0) };
        }

        let resolved_path = ResolvedPath::try_from(uri).ok()?;
        let file_path = resolved_path.as_path();

        workers
            .iter()
            .enumerate()
            .filter_map(|(i, worker)| {
                let resolved_path = ResolvedPath::try_from(worker.get_root_uri()).ok()?;
                let root_path = resolved_path.as_path();
                if file_path.starts_with(root_path) {
                    Some((i, root_path.as_os_str().len()))
                } else {
                    None
                }
            })
            .max_by_key(|(_, len)| *len)
            .map(|(i, _)| i)
    }

    // SAFETY: call this method only when you are sure, that we are not in `DynamicWithWorkspaces` mode,
    // or else it will return [`None`] for URIs that are outside of any workspace.
    #[expect(clippy::unused_self)] // we will use it later
    fn find_worker_for_uri<'a>(
        &self,
        workers: &'a [WorkspaceWorker],
        uri: &Uri,
    ) -> Option<&'a WorkspaceWorker> {
        let index = Self::find_worker_index_for_uri(workers, uri)?;
        Some(&workers[index])
    }

    /// Find the most specific workspace worker for a given URI.
    ///
    /// When multiple workers are responsible for a URI (e.g., in nested
    /// workspaces), this returns the worker with the longest matching path.
    ///
    /// For non-`file://` URIs the first worker in the list is returned,
    /// mirroring the behaviour of rust-analyzer and
    /// typescript-language-server.
    pub async fn get_worker_for_uri(&self, uri: &Uri) -> Option<WorkerGuard<'_>> {
        {
            let guard = self.workers.read().await;
            if let Some(index) = Self::find_worker_index_for_uri(&guard, uri) {
                return Some(WorkerGuard { guard: WorkerGuardInner::Vec(guard), index });
            }
        }

        #[cfg(test)]
        if let ManagerMode::DynamicWithWorkspaces(worker) = &self.mode {
            // In DynamicWithWorkspaces mode, if no worker matches the URI, fallback to the dynamic worker.
            return Some(WorkerGuard {
                guard: WorkerGuardInner::Single(worker.read().await),
                index: 0,
            });
        }

        None
    }

    /// Return the URI for the parent directory of a `file://` URI, or `None`
    /// when the URI has no parent or cannot be converted to a path.
    fn get_parent_dir_uri(file_uri: &Uri) -> Option<Uri> {
        let file_path = file_uri.to_file_path()?;
        let parent = file_path.parent()?;
        Uri::from_file_path(parent)
    }

    /// Validate that every URI in `workspaces` can be resolved to a local file
    /// path.  Returns an LSP error on the first invalid URI.
    pub fn assert_workspaces_are_valid_paths(workspaces: &[Uri]) -> Result<()> {
        for uri in workspaces {
            if uri.to_file_path().is_none() {
                return Err(Error::invalid_params(format!(
                    "workspace URI is not a valid file path: {}",
                    uri.as_str()
                )));
            }
        }
        Ok(())
    }

    // ── Workspace-folder change handling ──────────────────────────────────────

    /// Update the worker list to reflect workspace folder additions/removals.
    ///
    /// This method acquires the write lock briefly with no async I/O:
    ///
    /// * If folders are **added** while in single-file mode, that mode is
    ///   exited and all dynamically-created workers are drained and returned
    ///   for shutdown.
    /// * Workers for **removed** folders are extracted and returned for
    ///   shutdown.
    /// * If the resulting list is empty *and* no folders are being added, the
    ///   server enters single-file mode.
    ///
    /// Returns the workers that the caller must shut down (after releasing any
    /// locks held).
    pub async fn update_workspace_folders(
        &self,
        added: &[WorkspaceFolder],
        removed: &[WorkspaceFolder],
    ) -> Vec<WorkspaceWorker> {
        let mut workers_to_shutdown: Vec<WorkspaceWorker> = vec![];
        let mut workers = self.workers.write().await;

        // Transition out of single-file mode when real workspace folders arrive.
        if !added.is_empty() && self.is_single_file_mode() {
            self.set_single_file_mode(false);
            workers_to_shutdown.extend(workers.drain(..));
        }

        for folder in removed {
            if let Some(idx) = workers.iter().position(|w| w.get_root_uri() == &folder.uri) {
                workers_to_shutdown.push(workers.swap_remove(idx));
            }
        }

        // If there are no remaining workers and nothing new is coming, enter
        // single-file mode so subsequent `didOpen` calls create workers
        // dynamically.
        if workers.is_empty() && added.is_empty() {
            self.set_single_file_mode(true);
        }

        workers_to_shutdown
    }

    // ── Single-file mode operations ───────────────────────────────────────────

    /// Ensure a [`WorkspaceWorker`] exists for the parent directory of the
    /// given `file://` URI when the server is in single-file mode.
    ///
    /// The method is a no-op when:
    /// * the server is not in single-file mode, or
    /// * a suitable worker already exists, or
    /// * a concurrent call races to insert the same worker first.
    ///
    /// Returns `Some(registrations)` with the file-system watcher registrations
    /// that the caller should forward to the client.  Returns `None` when no
    /// new worker was inserted.
    pub async fn ensure_worker_for_file_uri(
        &self,
        uri: &Uri,
        diagnostic_mode: DiagnosticMode,
        dynamic_watchers: bool,
    ) -> Option<Vec<Registration>> {
        // Bail out immediately if we are not in single-file mode.
        if !self.is_single_file_mode() {
            return None;
        }

        let parent_uri = Self::get_parent_dir_uri(uri)?;

        // Fast path: avoid a write lock when a suitable worker already exists.
        {
            let workers = self.workers.read().await;
            if self.find_worker_for_uri(&workers, uri).is_some() {
                return None;
            }
        }

        debug!("single file mode: creating workspace worker for {}", parent_uri.as_str());
        let worker =
            WorkspaceWorker::new(parent_uri, Arc::clone(&self.tool_builder), diagnostic_mode);
        worker.start_worker(Value::Null).await;
        let registrations = if dynamic_watchers { worker.init_watchers().await } else { vec![] };

        // Acquire the write lock to insert the worker.  Re-check both the mode
        // flag and the worker list because a concurrent call (e.g., another
        // `didOpen` or `didChangeWorkspaceFolders`) may have beaten us here.
        let mut worker = Some(worker);
        {
            let mut workers = self.workers.write().await;
            if self.is_single_file_mode() && self.find_worker_for_uri(&workers, uri).is_none() {
                workers.push(worker.take().unwrap());
            }
        }

        // If we lost the race, release the worker's resources and signal to
        // the caller that no new registrations are needed.
        if let Some(discarded) = worker {
            discarded.shutdown().await;
            return None;
        }

        Some(registrations)
    }

    /// In single-file mode, shut down and remove the [`WorkspaceWorker`] whose
    /// root URI matches `worker_root_uri` when no open files remain associated
    /// with that workspace.
    ///
    /// `open_uris` should be a snapshot of the currently open file URIs (read
    /// from the in-memory file system *before* acquiring the workers write
    /// lock, to avoid cross-lock deadlocks).
    ///
    /// Returns `Some((uris_to_clear, unregistrations))` when the worker was
    /// shut down, `None` when there are still open files or the worker was not
    /// found.
    pub async fn try_shutdown_empty_workspace(
        &self,
        worker_root_uri: &Uri,
        open_uris: &[Uri],
    ) -> Option<(Vec<Uri>, Vec<Unregistration>)> {
        // Bail out immediately if we are not in single-file mode.
        if !self.is_single_file_mode() {
            return None;
        }

        let worker = {
            let mut workers = self.workers.write().await;

            let has_open_files = open_uris.iter().any(|open_uri| {
                self.find_worker_for_uri(&workers, open_uri)
                    .is_some_and(|w| w.get_root_uri() == worker_root_uri)
            });

            if has_open_files {
                return None;
            }

            let idx = workers.iter().position(|w| w.get_root_uri() == worker_root_uri)?;
            debug!("single file mode: shutting down empty workspace {}", worker_root_uri.as_str());
            workers.swap_remove(idx)
        }; // write lock released here

        let (uris, unregistrations) = worker.shutdown().await;
        Some((uris, unregistrations))
    }
}

#[cfg(test)]
mod tests {
    #[cfg(target_os = "windows")]
    use std::path::PathBuf;
    use std::sync::Arc;

    use tokio::sync::RwLock;
    use tower_lsp_server::ls_types::Uri;

    use crate::{
        DiagnosticMode, ToolBuilder, tests::FakeToolBuilder, worker::WorkspaceWorker,
        worker_manager::WorkerManager,
    };

    fn create_builder() -> Arc<dyn ToolBuilder> {
        Arc::new(FakeToolBuilder::default()) as Arc<dyn ToolBuilder>
    }

    #[cfg(target_os = "windows")]
    fn path_from_fixture(fixture: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures").join(fixture)
    }

    #[tokio::test]
    async fn test_get_worker_for_uri_nested_workspaces() {
        let workspace = WorkspaceWorker::new(
            "file:///path/to/workspace".parse().unwrap(),
            create_builder(),
            DiagnosticMode::None,
        );
        let workspace_deeper = WorkspaceWorker::new(
            "file:///path/to/workspace/deeper".parse().unwrap(),
            create_builder(),
            DiagnosticMode::None,
        );
        let workers = vec![workspace, workspace_deeper];
        let manager = WorkerManager::new(create_builder());
        manager.start_manager(workers).await;

        // File in deeper workspace should match the deeper worker
        let file_in_deeper: Uri = "file:///path/to/workspace/deeper/file.js".parse().unwrap();
        let worker = manager.get_worker_for_uri(&file_in_deeper).await;
        assert!(worker.is_some());
        assert_eq!(worker.unwrap().get_root_uri().as_str(), "file:///path/to/workspace/deeper");

        // File in parent workspace should match the parent worker
        let file_in_parent: Uri = "file:///path/to/workspace/file.js".parse().unwrap();
        let worker = manager.get_worker_for_uri(&file_in_parent).await;
        assert!(worker.is_some());
        assert_eq!(worker.unwrap().get_root_uri().as_str(), "file:///path/to/workspace");

        // File outside both workspaces should not match any worker
        let file_outside: Uri = "file:///path/to/other/file.js".parse().unwrap();
        let worker = manager.get_worker_for_uri(&file_outside).await;
        assert!(worker.is_none());
    }

    #[tokio::test]
    async fn test_get_worker_for_uri_similar_names() {
        let workspace = WorkspaceWorker::new(
            "file:///path/to/workspace".parse().unwrap(),
            create_builder(),
            DiagnosticMode::None,
        );
        let workspace2 = WorkspaceWorker::new(
            "file:///path/to/workspace-2".parse().unwrap(),
            create_builder(),
            DiagnosticMode::None,
        );
        let workers = vec![workspace, workspace2];
        let manager = WorkerManager::new(create_builder());
        manager.start_manager(workers).await;

        // File in workspace-2 should match workspace-2 only
        let file_in_workspace2: Uri = "file:///path/to/workspace-2/file.js".parse().unwrap();
        let worker = manager.get_worker_for_uri(&file_in_workspace2).await;
        assert!(worker.is_some());
        assert_eq!(worker.unwrap().get_root_uri().as_str(), "file:///path/to/workspace-2");

        // File in workspace should match workspace only
        let file_in_workspace: Uri = "file:///path/to/workspace/file.js".parse().unwrap();
        let worker = manager.get_worker_for_uri(&file_in_workspace).await;
        assert!(worker.is_some());
        assert_eq!(worker.unwrap().get_root_uri().as_str(), "file:///path/to/workspace");
    }

    #[tokio::test]
    async fn test_get_worker_for_uri_single_workspace() {
        let workspace = WorkspaceWorker::new(
            "file:///path/to/workspace".parse().unwrap(),
            create_builder(),
            DiagnosticMode::None,
        );
        let workers = vec![workspace];
        let manager = WorkerManager::new(create_builder());
        manager.start_manager(workers).await;

        // File in workspace should match
        let file_in_workspace: Uri = "file:///path/to/workspace/file.js".parse().unwrap();
        let worker = manager.get_worker_for_uri(&file_in_workspace).await;
        assert!(worker.is_some());
        assert_eq!(worker.unwrap().get_root_uri().as_str(), "file:///path/to/workspace");

        // File outside workspace should not match
        let file_outside: Uri = "file:///path/to/other/file.js".parse().unwrap();
        let worker = manager.get_worker_for_uri(&file_outside).await;
        assert!(worker.is_none());
    }

    #[tokio::test]
    async fn test_get_worker_for_uri_no_workers() {
        let workers: Vec<WorkspaceWorker> = vec![];
        let manager = WorkerManager::new(create_builder());
        manager.start_manager(workers).await;

        let file: Uri = "file:///path/to/workspace/file.js".parse().unwrap();
        let worker = manager.get_worker_for_uri(&file).await;
        assert!(worker.is_none());
    }

    #[tokio::test]
    async fn test_get_worker_for_uri_vscode_user_data_single_workspace() {
        let workspace = WorkspaceWorker::new(
            "file:///path/to/workspace".parse().unwrap(),
            create_builder(),
            DiagnosticMode::None,
        );
        let workers = vec![workspace];

        // non file URI should use first workspace
        let vscode_userdata_file: Uri = "vscode-userdata:///Untitled-1".parse().unwrap();
        let manager = WorkerManager::new(create_builder());
        manager.start_manager(workers).await;
        let worker = manager.get_worker_for_uri(&vscode_userdata_file).await;
        assert!(worker.is_some());
        assert_eq!(worker.unwrap().get_root_uri().as_str(), "file:///path/to/workspace");
    }

    #[tokio::test]
    async fn test_get_worker_for_uri_untitled_single_workspace() {
        let workspace = WorkspaceWorker::new(
            "file:///path/to/workspace".parse().unwrap(),
            create_builder(),
            DiagnosticMode::None,
        );
        let workers = vec![workspace];

        // non file URI should use first workspace
        let untitled_file: Uri = "untitled:///Untitled-1".parse().unwrap();
        let manager = WorkerManager::new(create_builder());
        manager.start_manager(workers).await;
        let worker = manager.get_worker_for_uri(&untitled_file).await;
        assert!(worker.is_some());
        assert_eq!(worker.unwrap().get_root_uri().as_str(), "file:///path/to/workspace");
    }

    #[tokio::test]
    async fn test_get_worker_for_uri_untitled_multiple_workspaces() {
        let workspace1 = WorkspaceWorker::new(
            "file:///path/to/workspace1".parse().unwrap(),
            create_builder(),
            DiagnosticMode::None,
        );
        let workspace2 = WorkspaceWorker::new(
            "file:///path/to/workspace2".parse().unwrap(),
            create_builder(),
            DiagnosticMode::None,
        );
        let workers = vec![workspace1, workspace2];

        // non file URI should use first workspace (not second)
        let untitled_file: Uri = "untitled:///Untitled-1".parse().unwrap();
        let manager = WorkerManager::new(create_builder());
        manager.start_manager(workers).await;
        let worker = manager.get_worker_for_uri(&untitled_file).await;
        assert!(worker.is_some());
        assert_eq!(worker.unwrap().get_root_uri().as_str(), "file:///path/to/workspace1");
    }

    #[tokio::test]
    async fn test_get_worker_for_uri_untitled_no_workspace() {
        let workers: Vec<WorkspaceWorker> = vec![];

        // Untitled file with no workspaces should return None
        let untitled_file: Uri = "untitled:///Untitled-1".parse().unwrap();
        let manager = WorkerManager::new(create_builder());
        manager.start_manager(workers).await;
        let worker = manager.get_worker_for_uri(&untitled_file).await;
        assert!(worker.is_none());
    }

    #[tokio::test]
    async fn test_get_worker_for_uri_untitled_with_nested_workspaces() {
        let workspace = WorkspaceWorker::new(
            "file:///path/to/workspace".parse().unwrap(),
            create_builder(),
            DiagnosticMode::None,
        );
        let workspace_deeper = WorkspaceWorker::new(
            "file:///path/to/workspace/deeper".parse().unwrap(),
            create_builder(),
            DiagnosticMode::None,
        );
        let workers = vec![workspace, workspace_deeper];

        // Untitled file should use first workspace (not nested one)
        let untitled_file: Uri = "untitled:///Untitled-1".parse().unwrap();
        let manager = WorkerManager::new(create_builder());
        manager.start_manager(workers).await;
        let worker = manager.get_worker_for_uri(&untitled_file).await;
        assert!(worker.is_some());
        assert_eq!(worker.unwrap().get_root_uri().as_str(), "file:///path/to/workspace");

        // File URIs should still use path-based matching
        let file_in_deeper: Uri = "file:///path/to/workspace/deeper/file.js".parse().unwrap();
        let worker = manager.get_worker_for_uri(&file_in_deeper).await;
        assert!(worker.is_some());
        assert_eq!(worker.unwrap().get_root_uri().as_str(), "file:///path/to/workspace/deeper");
    }

    #[test]
    #[cfg(not(target_os = "windows"))] // UNIX paths not supported on Windows
    fn test_get_parent_dir_uri() {
        // Typical file URI
        let file: Uri = "file:///path/to/dir/file.js".parse().unwrap();
        let parent = WorkerManager::get_parent_dir_uri(&file).unwrap();
        assert_eq!(parent.as_str(), "file:///path/to/dir");

        // File directly under root
        let root_file: Uri = "file:///file.js".parse().unwrap();
        let parent = WorkerManager::get_parent_dir_uri(&root_file).unwrap();
        // Parent of /file.js is /
        assert_eq!(parent.to_file_path().unwrap().to_string_lossy(), "/");

        // File URI pointing to the root ("/") has no parent — get_parent_dir_uri should return None.
        let no_path_file: Uri = "file:///".parse().unwrap();
        // Path is "/", so parent() returns None
        assert!(WorkerManager::get_parent_dir_uri(&no_path_file).is_none());
    }

    #[tokio::test]
    #[cfg(target_os = "windows")]
    async fn test_get_workspace_folder_case_insensitivity() {
        let fixture = path_from_fixture("same_path_different_uri");
        let root_path = PathBuf::from(
            fixture
                .to_string_lossy()
                .replace("same_path_different_uri", "Same_Path_different_uri")
                .replace("fixtures", "Fixtures"),
        );

        let workspace = WorkspaceWorker::new(
            Uri::from_file_path(root_path).unwrap(),
            create_builder(),
            DiagnosticMode::None,
        );
        let workers = vec![workspace];
        let manager = WorkerManager::new(create_builder());
        manager.start_manager(workers).await;

        // File with different case should still match on Windows
        let file: Uri = Uri::from_file_path(fixture.join("text.txt")).unwrap();
        let worker = manager.get_worker_for_uri(&file).await;
        assert!(worker.is_some());
    }

    #[tokio::test]
    async fn test_get_worker_for_uri_dynamic_with_workspaces_no_workspaces() {
        let dynamic_worker = WorkspaceWorker::new(
            "file:///".parse().unwrap(),
            create_builder(),
            DiagnosticMode::None,
        );
        let manager = WorkerManager::new_with_mode(
            create_builder(),
            crate::worker_manager::ManagerMode::DynamicWithWorkspaces(Box::new(RwLock::new(
                dynamic_worker,
            ))),
        );
        manager.start_manager(vec![]).await;

        // File in workspace should match dynamic worker
        let file: Uri = "file:///any/path/file.js".parse().unwrap();
        let worker = manager.get_worker_for_uri(&file).await;
        assert!(worker.is_some());
        assert_eq!(worker.unwrap().get_root_uri().as_str(), "file:///");

        // Non-file URI should also match dynamic worker
        let non_file_uri: Uri = "untitled:///Untitled-1".parse().unwrap();
        let worker = manager.get_worker_for_uri(&non_file_uri).await;
        assert!(worker.is_some());
        assert_eq!(worker.unwrap().get_root_uri().as_str(), "file:///");
    }

    #[tokio::test]
    async fn test_get_worker_for_uri_dynamic_with_workspaces_with_workspaces() {
        let dynamic_worker = WorkspaceWorker::new(
            "file:///".parse().unwrap(),
            create_builder(),
            DiagnosticMode::None,
        );
        let manager = WorkerManager::new_with_mode(
            create_builder(),
            crate::worker_manager::ManagerMode::DynamicWithWorkspaces(Box::new(RwLock::new(
                dynamic_worker,
            ))),
        );

        // File in workspace should match dynamic worker
        let file: Uri = "file:///any/path/file.js".parse().unwrap();
        let worker = manager.get_worker_for_uri(&file).await;
        assert!(worker.is_some());
        assert_eq!(worker.unwrap().get_root_uri().as_str(), "file:///");

        // Non-file URI should also match dynamic worker
        let non_file_uri: Uri = "untitled:///Untitled-1".parse().unwrap();
        let worker = manager.get_worker_for_uri(&non_file_uri).await;
        assert!(worker.is_some());
        assert_eq!(worker.unwrap().get_root_uri().as_str(), "file:///");

        manager
            .start_manager(vec![WorkspaceWorker::new(
                "file:///path/to/workspace".parse().unwrap(),
                create_builder(),
                DiagnosticMode::None,
            )])
            .await;

        // Files outside workspace should still match dynamic worker
        let file_outside: Uri = "file:///other/path/file.js".parse().unwrap();
        let worker = manager.get_worker_for_uri(&file_outside).await;
        assert!(worker.is_some());
        assert_eq!(worker.unwrap().get_root_uri().as_str(), "file:///");

        // Files inside workspace should match the workspace worker
        let file_inside: Uri = "file:///path/to/workspace/file.js".parse().unwrap();
        let worker = manager.get_worker_for_uri(&file_inside).await;
        assert!(worker.is_some());
        assert_eq!(worker.unwrap().get_root_uri().as_str(), "file:///path/to/workspace");

        // Non-file URI should also match first workspace (not dynamic worker)
        let non_file_uri: Uri = "untitled:///Untitled-1".parse().unwrap();
        let worker = manager.get_worker_for_uri(&non_file_uri).await;
        assert!(worker.is_some());
        assert_eq!(worker.unwrap().get_root_uri().as_str(), "file:///path/to/workspace");
    }
}
