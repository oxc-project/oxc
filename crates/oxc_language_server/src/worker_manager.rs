use std::{
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use serde_json::Value;
use tokio::sync::{OnceCell, RwLock, RwLockReadGuard};
use tower_lsp_server::{
    jsonrpc::{Error, Result},
    ls_types::{MessageType, Pattern, Registration, Unregistration, Uri, WorkspaceFolder},
};
use tracing::{debug, warn};

use crate::{
    ClientMessage,
    capabilities::DiagnosticMode,
    file_system::ResolvedPath,
    tool::ToolBuilder,
    worker::WorkspaceWorker,
    working_directories::{
        ResolvedWorkingDirectories, resolve_working_directories, sub_worker_options,
        working_directories_need_walk,
    },
};

/// The index of the root which contains `file_path` with the longest match.
fn find_most_specific_root<'a>(
    roots: impl Iterator<Item = (usize, &'a Path)>,
    file_path: &Path,
) -> Option<usize> {
    roots
        .filter(|(_, root_path)| file_path.starts_with(root_path))
        .max_by_key(|(_, root_path)| root_path.as_os_str().len())
        .map(|(index, _)| index)
}

/// The outcome of [`WorkerManager::sync_sub_workers`].
#[derive(Default)]
pub struct SubWorkerSync {
    /// Sub workers which are not a working directory anymore, the caller has to shut them down.
    pub removed: Vec<WorkspaceWorker>,
    /// Roots of the sub workers which were created.
    pub added_roots: Vec<Uri>,
    /// Roots whose excluded set changed, so their tool has to be rebuilt. They are already rebuilt
    /// when the caller set `rebuild_now`.
    pub rebuild_roots: Vec<Uri>,
    /// Watcher registrations of the created sub workers, and of the rebuilt workers whose
    /// patterns changed.
    pub registrations: Vec<Registration>,
    /// Watcher unregistrations of the rebuilt workers whose patterns changed.
    pub unregistrations: Vec<Unregistration>,
    /// Messages to show to the client.
    pub client_messages: Vec<ClientMessage>,
    /// Whether the `auto` flag of the workspace folder worker flipped, which changes its watcher
    /// patterns independently of what its tool decides.
    pub auto_changed: bool,
}

enum WorkerGuardInner<'a> {
    Vec(RwLockReadGuard<'a, Vec<WorkspaceWorker>>),
    Single(&'a WorkspaceWorker),
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
            WorkerGuardInner::Single(single_guard) => single_guard,
        }
    }
}

/// The mode that the [`WorkerManager`] is operating in, which determines how it manages workers and delegates the task to the tool.
pub enum ManagerMode {
    // the manager works in 2 modes, when no workspaces are configured, it creates workers dynamically for file URIs.
    // When workspaces are reconfigured (added or removed by the client), it creates workers for those and ignores file URIs outside of them.
    DynamicNoWorkspaces(
        // toggle for single file / workspace mode
        AtomicBool,
    ),
    // The manager will create workers dynamically for file URIs. It also supports workspaces configured by the client, but does not require them.
    // This is useful for tasks on URIs that are outside of any configured workspace.
    // At first it is `None` until the diagnostic mode is known, then it is set to a worker with the root URI `file:///` and the same diagnostic mode as the other workers.
    DynamicWithWorkspaces(Box<OnceCell<WorkspaceWorker>>),
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

    /// Create a new [`WorkerManager`] with `DynamicWithWorkspaces` mode.
    pub fn new_dynamic(tool_builder: Arc<dyn ToolBuilder>) -> Self {
        Self {
            mode: ManagerMode::DynamicWithWorkspaces(Box::new(OnceCell::new())),
            tool_builder,
            workers: RwLock::new(vec![]),
        }
    }

    // ── Starting / Stopping ───────────────────────────────────────────────────────

    /// Start the manager with the given workers and diagnostic mode.
    ///
    /// On dynamic workspaces, this will also start a worker for the root URI `file:///` with the given diagnostic mode.
    /// It can return a list of [`ClientMessage`] that should be sent to the client.
    ///
    /// # Panics
    /// If `file:///` cannot be converted to `Uri`, which should never happen.
    pub async fn start_manager(
        &self,
        workers: Vec<WorkspaceWorker>,
        diagnostic_mode: DiagnosticMode,
    ) -> Vec<ClientMessage> {
        *self.workers.write().await = workers;

        // for dynamic workspaces we need to start them manually
        if let ManagerMode::DynamicWithWorkspaces(cell) = &self.mode {
            debug_assert!(
                cell.get().is_none(),
                "dynamic worker should not be set when starting the manager"
            );

            let worker = WorkspaceWorker::new(
                "file:///".parse().unwrap(),
                Arc::clone(&self.tool_builder),
                diagnostic_mode,
            );
            let client_messages = worker.start_worker(serde_json::Value::Null).await;

            let _ = cell.set(worker);

            return client_messages;
        }

        vec![]
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

        if let ManagerMode::DynamicWithWorkspaces(worker) = &self.mode {
            let dynamic_worker = worker.get();
            if let Some(dynamic_worker) = dynamic_worker {
                let (worker_uris, _) = dynamic_worker.shutdown().await;
                clear_uris.extend(worker_uris);
            }
        }

        clear_uris
    }

    // ── State accessors ───────────────────────────────────────────────────────

    /// Acquire a shared read lock over the worker list.
    /// Does not include the dynamic worker in `DynamicWithWorkspaces` mode, which must be accessed by `read_dynamic_worker`.
    pub async fn read_workspace_workers(&self) -> RwLockReadGuard<'_, Vec<WorkspaceWorker>> {
        self.workers.read().await
    }

    /// Return the dynamic worker from `DynamicWithWorkspaces` mode, if enabled.
    pub fn read_dynamic_worker(&self) -> Option<&WorkspaceWorker> {
        if let ManagerMode::DynamicWithWorkspaces(worker) = &self.mode {
            return worker.get();
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

    // ── `workingDirectories` ──────────────────────────────────────────────────

    /// Resolve the `workingDirectories` option away from the runtime.
    ///
    /// A glob or the `auto` detection walks the workspace folder, which is blocking file system
    /// work and must not run on an async worker thread.
    /// # Errors
    /// When the blocking task could not be joined. The caller must then keep the state it has:
    /// resolving to an empty set would tear every sub worker down for a transient failure.
    async fn resolve_working_directories(
        &self,
        root_uri: &Uri,
        options: &serde_json::Value,
        folder_roots: &[Uri],
    ) -> std::result::Result<ResolvedWorkingDirectories, ClientMessage> {
        // the workspace folders the client opened below this one belong to their own worker
        let nested_roots = Self::nested_folder_roots(root_uri, folder_roots);

        // A literal entry is a handful of `is_dir()` calls and an absent option is no work at all,
        // both are resolved inline. Only a glob or the `auto` detection walks the workspace folder.
        if !working_directories_need_walk(options) {
            return Ok(resolve_working_directories(
                root_uri,
                options,
                self.tool_builder.as_ref(),
                &nested_roots,
            ));
        }

        let tool_builder = Arc::clone(&self.tool_builder);
        let task_root_uri = root_uri.clone();
        let options = options.clone();

        tokio::task::spawn_blocking(move || {
            resolve_working_directories(
                &task_root_uri,
                &options,
                tool_builder.as_ref(),
                &nested_roots,
            )
        })
        .await
        .map_err(|err| {
            warn!("resolving the working directories of {} failed: {err}", root_uri.as_str());
            ClientMessage {
                r#type: MessageType::WARNING,
                message: format!(
                    "`workingDirectories` could not be resolved for {}, the previous project roots are kept",
                    root_uri.as_str()
                ),
            }
        })
    }

    /// The roots of the workspace folders the client opened strictly below `root_uri`.
    ///
    /// The caller passes every workspace folder root it knows about, which is deliberately not
    /// read from [`Self::workers`] here: the handlers which start a worker hold that lock, or
    /// start folders which are not registered yet, and re-entering a write-fair [`RwLock`] from
    /// inside a read guard deadlocks as soon as a writer is queued.
    fn nested_folder_roots(root_uri: &Uri, folder_roots: &[Uri]) -> Vec<PathBuf> {
        let Ok(resolved) = ResolvedPath::try_from(root_uri) else {
            return vec![];
        };
        let root_path = resolved.as_path();

        let mut nested = folder_roots
            .iter()
            .filter_map(|uri| ResolvedPath::try_from(uri).ok())
            .map(|resolved| resolved.as_path().to_path_buf())
            .filter(|path| path != root_path && path.starts_with(root_path))
            .collect::<Vec<_>>();
        nested.sort_unstable();
        nested.dedup();

        nested
    }

    /// The roots of every workspace folder worker, the sub workers excluded.
    ///
    /// The lock is taken and dropped here, so the result can be handed to a call which starts or
    /// reconciles workers.
    pub async fn folder_roots(&self) -> Vec<Uri> {
        self.workers
            .read()
            .await
            .iter()
            .filter(|worker| !worker.is_sub_worker())
            .map(|worker| worker.get_root_uri().clone())
            .collect()
    }

    /// Resolve every root once, so the comparisons below do not canonicalize them again.
    fn resolve_roots(roots: &[Uri]) -> Vec<(Uri, PathBuf)> {
        roots
            .iter()
            .filter_map(|root| {
                ResolvedPath::try_from(root)
                    .ok()
                    .map(|resolved| (root.clone(), resolved.as_path().to_path_buf()))
            })
            .collect()
    }

    /// Compute the excluded roots of `root_path`: the resolved roots strictly below it.
    ///
    /// A working directory can contain another working directory, in which case the outer one must
    /// exclude the inner one just like the workspace folder excludes both.
    fn excluded_roots_below(roots: &[(Uri, PathBuf)], root_path: &Path) -> Vec<Uri> {
        roots
            .iter()
            .filter(|(_, other_path)| other_path != root_path && other_path.starts_with(root_path))
            .map(|(uri, _)| uri.clone())
            .collect()
    }

    /// Whether a change to this file can add or remove a working directory: a `package.json` for
    /// the `auto` detection, or one of the configuration file names of the tool.
    ///
    /// This is deliberately not the watcher patterns of the tool: a setup with an explicit
    /// `configPath` watches a single file but still gains a working directory when a config
    /// appears somewhere else.
    pub fn is_project_root_marker(&self, uri: &Uri) -> bool {
        let Some(name) = uri.path().as_str().rsplit('/').next() else {
            return false;
        };

        name == "package.json" || self.tool_builder.config_file_names().contains(&name)
    }

    /// Create and start one sub worker per resolved working directory.
    ///
    /// `claimed_roots` holds the roots which already have a worker: a client workspace folder is
    /// never shadowed by a sub worker for the same directory.
    async fn start_sub_workers(
        &self,
        parent_uri: &Uri,
        roots: &[Uri],
        options: &serde_json::Value,
        diagnostic_mode: &DiagnosticMode,
        claimed_roots: &[Uri],
    ) -> (Vec<WorkspaceWorker>, Vec<ClientMessage>) {
        let sub_options = sub_worker_options(options);
        let mut client_messages = vec![];
        let mut sub_workers = Vec::with_capacity(roots.len());

        let resolved_roots = Self::resolve_roots(roots);
        let claimed_paths = Self::resolve_roots(claimed_roots);

        for (root_uri, root_path) in &resolved_roots {
            if claimed_paths.iter().any(|(_, claimed)| claimed == root_path) {
                debug!(
                    "skipping working directory {}, it already has its own worker",
                    root_uri.as_str()
                );
                continue;
            }

            debug!("starting working directory worker for {}", root_uri.as_str());
            let sub_worker = WorkspaceWorker::new_sub_worker(
                root_uri.clone(),
                parent_uri.clone(),
                Arc::clone(&self.tool_builder),
                diagnostic_mode.clone(),
            );
            // a working directory nested inside another one is owned by its own worker
            sub_worker
                .set_working_directories(
                    Self::excluded_roots_below(&resolved_roots, root_path),
                    false,
                )
                .await;
            client_messages.extend(sub_worker.start_worker(sub_options.clone()).await);
            sub_workers.push(sub_worker);
        }

        (sub_workers, client_messages)
    }

    /// Resolve the `workingDirectories` option of a workspace folder, start its worker and start
    /// one sub worker per resolved working directory.
    ///
    /// The working directories have to be known **before** the workspace folder worker is started,
    /// so its tool can exclude them from its eager discovery (nested configs, ignore files, ...).
    ///
    /// `claimed_roots` holds the roots which already have a worker, so a directory which the client
    /// opened as its own workspace folder does not get a second, shadowing worker, and
    /// `folder_roots` every workspace folder root the caller knows about, registered or not.
    ///
    /// This method deliberately never touches `Self::workers`: `initialized` calls it while it
    /// holds the read guard, and re-entering a write-fair [`RwLock`] from inside a read guard
    /// deadlocks as soon as a writer is queued.
    ///
    /// The returned sub workers are already started, the caller has to insert them into the worker
    /// list (directly, or through [`Self::add_workers`]).
    pub async fn start_folder_worker(
        &self,
        folder_worker: &WorkspaceWorker,
        options: serde_json::Value,
        diagnostic_mode: &DiagnosticMode,
        claimed_roots: &[Uri],
        folder_roots: &[Uri],
    ) -> (Vec<WorkspaceWorker>, Vec<ClientMessage>) {
        let resolved = match self
            .resolve_working_directories(folder_worker.get_root_uri(), &options, folder_roots)
            .await
        {
            Ok(resolved) => resolved,
            Err(message) => {
                // the worker still starts, only without working directories
                let mut client_messages = vec![message];
                client_messages.extend(folder_worker.start_worker(options).await);
                return (vec![], client_messages);
            }
        };
        let mut client_messages = resolved.client_messages();

        folder_worker.set_working_directories(resolved.roots.clone(), resolved.auto).await;
        client_messages.extend(folder_worker.start_worker(options.clone()).await);

        let (sub_workers, messages) = self
            .start_sub_workers(
                folder_worker.get_root_uri(),
                &resolved.roots,
                &options,
                diagnostic_mode,
                claimed_roots,
            )
            .await;
        client_messages.extend(messages);

        (sub_workers, client_messages)
    }

    /// Recompute the `workingDirectories` of the workspace folder worker with the root
    /// `parent_uri` and reconcile its sub workers: sub workers which disappeared are removed and
    /// returned for shutdown, new ones are created, started and inserted, surviving ones are left
    /// untouched.
    ///
    /// `report_warnings` should only be `true` when the option value itself changed, so a watched
    /// file event does not repeat the same validation warnings over and over.
    pub async fn sync_sub_workers(
        &self,
        parent_uri: &Uri,
        options: &serde_json::Value,
        diagnostic_mode: &DiagnosticMode,
        dynamic_watchers: bool,
        report_warnings: bool,
        rebuild_now: bool,
    ) -> SubWorkerSync {
        // the lock is not held here, so the folder roots are read before resolving
        let folder_roots = self.folder_roots().await;
        let resolved =
            match self.resolve_working_directories(parent_uri, options, &folder_roots).await {
                Ok(resolved) => resolved,
                Err(message) => {
                    // keep every worker as it is, the option could not be resolved this time
                    return SubWorkerSync {
                        client_messages: vec![message],
                        ..SubWorkerSync::default()
                    };
                }
            };
        let mut sync = SubWorkerSync {
            client_messages: if report_warnings { resolved.client_messages() } else { vec![] },
            ..SubWorkerSync::default()
        };

        // The sub workers which have to exist are the resolved roots which the client did not
        // open as a workspace folder itself. A client folder always wins, so there is exactly one
        // worker, and one watcher registration, per root at any time.
        //
        // The comparison is made against the workers which are actually there, not only against
        // the roots stored on the parent: adding or removing a workspace folder on one of those
        // roots changes what has to exist without changing the option value.
        let parent_path = ResolvedPath::try_from(parent_uri).ok();
        let parent_path = parent_path.as_ref().map(ResolvedPath::as_path);

        let (expected, parent_roots_changed) = {
            let workers = self.workers.read().await;
            let Some(parent) = workers
                .iter()
                .find(|worker| !worker.is_sub_worker() && worker.has_root_path(parent_path))
            else {
                return sync;
            };

            let stored_roots = parent.get_working_directories().await;
            // always store them, the `auto` flag can change without changing the resolved roots
            sync.auto_changed =
                parent.set_working_directories(resolved.roots.clone(), resolved.auto).await;

            let resolved_roots = Self::resolve_roots(&resolved.roots);
            let expected = resolved_roots
                .iter()
                .filter(|(_, root_path)| {
                    !workers.iter().any(|worker| {
                        !worker.is_sub_worker() && worker.has_root_path(Some(root_path))
                    })
                })
                .map(|(uri, _)| uri.clone())
                .collect::<Vec<_>>();

            let current = workers
                .iter()
                .filter(|worker| worker.has_parent_path(parent_path))
                .filter_map(WorkspaceWorker::get_root_path)
                .collect::<Vec<_>>();
            let expected_paths = Self::resolve_roots(&expected);

            let roots_changed = stored_roots != resolved.roots;
            let unchanged = !roots_changed
                && current.len() == expected.len()
                && current
                    .iter()
                    .all(|root| expected_paths.iter().any(|(_, expected)| expected == root));

            if unchanged {
                return sync;
            }

            (expected, roots_changed)
        };

        // remove the sub workers which are not expected anymore, either because the option does
        // not resolve to them or because the client opened a workspace folder on their root
        {
            let expected_paths = Self::resolve_roots(&expected);
            let mut workers = self.workers.write().await;
            let mut index = 0;
            while index < workers.len() {
                let worker = &workers[index];
                if worker.has_parent_path(parent_path)
                    && !expected_paths
                        .iter()
                        .any(|(_, expected)| worker.has_root_path(Some(expected)))
                {
                    sync.removed.push(workers.remove(index));
                } else {
                    index += 1;
                }
            }
        }

        // create the sub workers which are new
        let claimed_roots = self
            .workers
            .read()
            .await
            .iter()
            .map(|worker| worker.get_root_uri().clone())
            .collect::<Vec<_>>();
        let (added, messages) = self
            .start_sub_workers(
                parent_uri,
                &resolved.roots,
                options,
                diagnostic_mode,
                &claimed_roots,
            )
            .await;
        sync.client_messages.extend(messages);

        for sub_worker in &added {
            sync.added_roots.push(sub_worker.get_root_uri().clone());
            if dynamic_watchers {
                sync.registrations.extend(sub_worker.init_watchers().await);
            }
        }
        self.add_workers(added).await;

        // The build context of the workspace folder worker changed, and so did the one of every
        // surviving sub worker whose nested working directories changed. Their options did not
        // change, so their tool would otherwise keep the configs and ignore files of a directory
        // which is now owned by another worker.
        sync.rebuild_roots = self.prepare_rebuilds(parent_uri, &sync, parent_roots_changed).await;

        if rebuild_now {
            let (client_messages, unregistrations, registrations) =
                self.rebuild_workers(&sync.rebuild_roots).await;
            sync.client_messages.extend(client_messages);
            sync.unregistrations.extend(unregistrations);
            sync.registrations.extend(registrations);
        }

        sync
    }

    /// Store the new exclusions on the workers whose [`BuildContext`](crate::BuildContext)
    /// changed after a reconciliation, and return their roots.
    ///
    /// The workspace folder worker is first, so a caller which rebuilds them in order has an up to
    /// date folder worker before it lints the documents a removed sub worker orphaned.
    async fn prepare_rebuilds(
        &self,
        parent_uri: &Uri,
        sync: &SubWorkerSync,
        parent_roots_changed: bool,
    ) -> Vec<Uri> {
        let workers = self.workers.read().await;

        let parent_path = ResolvedPath::try_from(parent_uri).ok();
        let parent_path = parent_path.as_ref().map(ResolvedPath::as_path);

        let Some(parent) = workers
            .iter()
            .find(|worker| !worker.is_sub_worker() && worker.has_root_path(parent_path))
        else {
            return vec![];
        };

        let mut roots = vec![];
        // the folder worker only changed when its own set of excluded roots changed
        if parent_roots_changed {
            roots.push(parent.get_root_uri().clone());
        }

        // The full resolved set, including the roots which a client workspace folder claimed: a
        // working directory has to exclude everything below it, regardless of which worker
        // serves it.
        let resolved_roots = Self::resolve_roots(&parent.get_working_directories().await);
        let added_paths = Self::resolve_roots(&sync.added_roots);

        for worker in workers.iter() {
            if !worker.has_parent_path(parent_path) {
                continue;
            }
            // a worker which was just created already has the right context
            if added_paths.iter().any(|(_, added)| worker.has_root_path(Some(added))) {
                continue;
            }
            let Some(worker_path) = worker.get_root_path() else {
                continue;
            };

            let excluded_roots = Self::excluded_roots_below(&resolved_roots, worker_path);
            if worker.get_working_directories().await == excluded_roots {
                continue;
            }

            worker.set_working_directories(excluded_roots, false).await;
            roots.push(worker.get_root_uri().clone());
        }

        roots
    }

    /// Rebuild the tools of the workers rooted at `roots`, in order.
    ///
    /// Returns the messages for the client and the watcher registrations of the workers whose
    /// patterns changed with the rebuild.
    pub async fn rebuild_workers(
        &self,
        roots: &[Uri],
    ) -> (Vec<ClientMessage>, Vec<Unregistration>, Vec<Registration>) {
        let mut client_messages = vec![];
        let mut unregistrations = vec![];
        let mut registrations = vec![];

        let workers = self.workers.read().await;
        for (root, root_path) in Self::resolve_roots(roots) {
            let Some(worker) = workers.iter().find(|worker| worker.has_root_path(Some(&root_path)))
            else {
                continue;
            };

            debug!("rebuilding the worker {}", root.as_str());
            let (messages, unregistered, registered) = worker.rebuild_tool().await;
            client_messages.extend(messages);
            unregistrations.extend(unregistered);
            registrations.extend(registered);
        }

        (client_messages, unregistrations, registrations)
    }

    /// Re-register the file system watchers of the given roots, without rebuilding their tools.
    ///
    /// Needed for a worker whose tool restarted on its own: the tool reported no watcher change,
    /// but its patterns can depend on the excluded roots it was rebuilt with.
    pub async fn refresh_worker_watchers(
        &self,
        roots: &[(Uri, Vec<Pattern>)],
    ) -> (Vec<Unregistration>, Vec<Registration>) {
        let mut unregistrations = vec![];
        let mut registrations = vec![];

        let workers = self.workers.read().await;
        for (root, patterns_before) in roots {
            let Ok(root_path) = ResolvedPath::try_from(root) else {
                continue;
            };
            let Some(worker) =
                workers.iter().find(|worker| worker.has_root_path(Some(root_path.as_path())))
            else {
                continue;
            };

            let (unregistered, registered) =
                worker.refresh_watchers_if_changed(patterns_before).await;
            unregistrations.extend(unregistered);
            registrations.extend(registered);
        }

        (unregistrations, registrations)
    }

    /// Route every URI to the worker responsible for it, in a single pass.
    ///
    /// The result is indexed like `uris` and holds the index of the worker, or `None` when no
    /// worker covers the URI. Handlers which need the owned documents of several workers route
    /// once and group by index with [`Self::owned_uris`].
    pub fn route_uris(workers: &[WorkspaceWorker], uris: &[Uri]) -> Vec<Option<usize>> {
        uris.iter().map(|uri| Self::find_worker_index_for_uri(workers, uri)).collect()
    }

    /// The URIs which [`Self::route_uris`] assigned to `index`.
    pub fn owned_uris(routes: &[Option<usize>], uris: &[Uri], index: usize) -> Vec<Uri> {
        uris.iter()
            .zip(routes)
            .filter(|(_, route)| **route == Some(index))
            .map(|(uri, _)| uri.clone())
            .collect()
    }

    /// Every workspace folder worker whose root contains `uri`, with its options.
    ///
    /// A file can sit in several nested workspace folders and each of them may declare its own
    /// `workingDirectories`, so a marker file has to reconcile all of them, not only the nearest.
    pub async fn folder_workers_containing(&self, uri: &Uri) -> Vec<(Uri, serde_json::Value)> {
        let workers = self.workers.read().await;
        let Ok(resolved_path) = ResolvedPath::try_from(uri) else {
            return vec![];
        };
        let file_path = resolved_path.as_path();

        let mut folders = vec![];
        for worker in workers.iter().filter(|worker| !worker.is_sub_worker()) {
            if !worker.get_root_path().is_some_and(|root| file_path.starts_with(root)) {
                continue;
            }
            folders.push((worker.get_root_uri().clone(), worker.get_options().await));
        }

        folders
    }

    /// Whether the event only matches a watcher pattern a workspace folder worker registered for
    /// itself, the `**/package.json` of the `auto` detection.
    ///
    /// Such an event is not a tool configuration change, so it must not reach the tool of that
    /// folder nor the tool of any of its sub workers.
    ///
    /// Every workspace folder above the file is asked, not only the nearest one: a `package.json`
    /// inside a nested client folder is still the detection marker of the `auto` folder above it,
    /// and the nested folder, which declares nothing, must not turn it into a tool event.
    pub async fn is_detection_only_event(&self, uri: &Uri) -> bool {
        let workers = self.workers.read().await;
        let Ok(resolved_path) = ResolvedPath::try_from(uri) else {
            return false;
        };
        let file_path = resolved_path.as_path();

        let mut is_detection = false;
        for worker in workers.iter().filter(|worker| !worker.is_sub_worker()) {
            if !worker.get_root_path().is_some_and(|root| file_path.starts_with(root)) {
                continue;
            }
            // a tool which watches the file itself always wins, the event is a real config change
            if worker.matches_tool_pattern(uri).await {
                return false;
            }
            is_detection |= worker.matches_detection_pattern(uri).await;
        }

        is_detection
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
            // these are in-memory files that don't have a file path
            return (!workers.is_empty()).then_some(0);
        }

        let resolved_path = ResolvedPath::try_from(uri).ok()?;

        // the roots are resolved once, when the worker is created
        find_most_specific_root(
            workers.iter().enumerate().filter_map(|(index, worker)| {
                worker.get_root_path().map(|root_path| (index, root_path))
            }),
            resolved_path.as_path(),
        )
    }

    // SAFETY: call this method only when you are sure, that we are not in `DynamicWithWorkspaces` mode,
    // or else it will return [`None`] for URIs that are outside of any workspace.
    fn find_worker_for_uri<'a>(
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

        if let ManagerMode::DynamicWithWorkspaces(worker) = &self.mode {
            let Some(worker) = worker.get() else {
                debug!(
                    "dynamic worker is not initialized yet, cannot find worker for URI {}",
                    uri.as_str()
                );
                return None;
            };
            // In DynamicWithWorkspaces mode, if no worker matches the URI, fallback to the dynamic worker.
            return Some(WorkerGuard { guard: WorkerGuardInner::Single(worker), index: 0 });
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
    ///
    /// # Errors
    /// * If any URI in `workspaces` cannot be converted to a file path, an error is returned indicating which URI was invalid.
    pub fn assert_workspaces_are_valid_paths(workspaces: Vec<&Uri>) -> Result<()> {
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
            let folder_path = ResolvedPath::try_from(&folder.uri).ok();
            let folder_path = folder_path.as_ref().map(ResolvedPath::as_path);

            // removing a workspace folder also removes its `workingDirectories` sub workers
            let mut index = 0;
            while index < workers.len() {
                let worker = &workers[index];
                if worker.has_parent_path(folder_path)
                    || (!worker.is_sub_worker() && worker.has_root_path(folder_path))
                {
                    workers_to_shutdown.push(workers.remove(index));
                } else {
                    index += 1;
                }
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
    ) -> (Option<Registration>, Vec<ClientMessage>) {
        // Bail out immediately if we are not in single-file mode.
        if !self.is_single_file_mode() {
            return (None, vec![]);
        }

        let Some(parent_uri) = Self::get_parent_dir_uri(uri) else {
            return (None, vec![]);
        };

        // Fast path: avoid a write lock when a suitable worker already exists.
        {
            let workers = self.workers.read().await;
            if Self::find_worker_for_uri(&workers, uri).is_some() {
                return (None, vec![]);
            }
        }

        debug!("single file mode: creating workspace worker for {}", parent_uri.as_str());
        let worker =
            WorkspaceWorker::new(parent_uri, Arc::clone(&self.tool_builder), diagnostic_mode);
        let client_messages = worker.start_worker(Value::Null).await;
        let registration = if dynamic_watchers { worker.init_watchers().await } else { None };

        // Acquire the write lock to insert the worker.  Re-check both the mode
        // flag and the worker list because a concurrent call (e.g., another
        // `didOpen` or `didChangeWorkspaceFolders`) may have beaten us here.
        let mut worker = Some(worker);
        {
            let mut workers = self.workers.write().await;
            if self.is_single_file_mode() && Self::find_worker_for_uri(&workers, uri).is_none() {
                #[expect(clippy::missing_panics_doc)]
                // We wrapped the worker in `Some` to avoid moving it before acquiring the lock, so it should always be `Some` here.
                workers.push(worker.take().expect(
                    "freshly created worker should be available when storing it in the list",
                ));
            }
        }

        // If we lost the race, release the worker's resources and signal to
        // the caller that no new registrations are needed.
        if let Some(discarded) = worker {
            discarded.shutdown().await;
            return (None, vec![]);
        }

        (registration, client_messages)
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
                Self::find_worker_for_uri(&workers, open_uri)
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

    /// `initialized` starts the workspace folder workers while it holds the read guard of the
    /// worker list, and a notification from the client can queue a writer at any moment. Tokio's
    /// `RwLock` is write fair, so a second `read()` taken from inside the guard would never be
    /// granted: starting a worker must not touch the list at all.
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_starting_a_worker_under_the_read_guard_does_not_deadlock() {
        let root: Uri = "file:///path/to/workspace".parse().unwrap();
        let manager = Arc::new(WorkerManager::new(create_builder()));
        manager
            .start_manager(
                vec![WorkspaceWorker::new(root.clone(), create_builder(), DiagnosticMode::None)],
                DiagnosticMode::None,
            )
            .await;

        let guard = manager.read_workspace_workers().await;

        // a `didChangeWorkspaceFolders` landing right now queues a writer
        let writing = Arc::clone(&manager);
        let writer = tokio::spawn(async move {
            writing
                .add_workers(vec![WorkspaceWorker::new(
                    "file:///path/to/other".parse().unwrap(),
                    create_builder(),
                    DiagnosticMode::None,
                )])
                .await;
        });
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;

        let started = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            manager.start_folder_worker(
                &guard[0],
                serde_json::json!({ "workingDirectories": ["packages/a"] }),
                &DiagnosticMode::None,
                std::slice::from_ref(&root),
                std::slice::from_ref(&root),
            ),
        )
        .await;
        assert!(started.is_ok(), "starting a worker under the read guard deadlocked");

        drop(guard);
        writer.await.unwrap();
    }

    /// Semantics rule 7: the `**/package.json` of the `auto` detection is not a tool event. The
    /// nearest workspace folder of the marker can be a nested client folder which declares
    /// nothing, so every folder above the file decides, not only that one.
    #[tokio::test]
    async fn test_a_marker_inside_a_nested_folder_stays_a_detection_event() {
        let root: Uri = "file:///path/to/workspace".parse().unwrap();
        let nested: Uri = "file:///path/to/workspace/packages/a".parse().unwrap();

        let folder = WorkspaceWorker::new(root.clone(), create_builder(), DiagnosticMode::None);
        folder
            .start_worker(serde_json::json!({ "workingDirectories": [{ "mode": "auto" }] }))
            .await;
        folder.set_working_directories(vec![], true).await;

        let inner = WorkspaceWorker::new(nested, create_builder(), DiagnosticMode::None);
        inner.start_worker(serde_json::json!({})).await;

        let manager = WorkerManager::new(create_builder());
        manager.start_manager(vec![folder, inner], DiagnosticMode::None).await;

        let marker: Uri = "file:///path/to/workspace/packages/a/package.json".parse().unwrap();
        assert!(manager.is_detection_only_event(&marker).await);

        // a file a tool watches is a real configuration change, regardless of which folder
        // watches it
        let config: Uri = "file:///path/to/workspace/packages/a/tool.config".parse().unwrap();
        assert!(!manager.is_detection_only_event(&config).await);
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
        manager.start_manager(workers, DiagnosticMode::None).await;

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
        manager.start_manager(workers, DiagnosticMode::None).await;

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
        manager.start_manager(workers, DiagnosticMode::None).await;

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
        manager.start_manager(workers, DiagnosticMode::None).await;

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
        manager.start_manager(workers, DiagnosticMode::None).await;
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
        manager.start_manager(workers, DiagnosticMode::None).await;
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
        manager.start_manager(workers, DiagnosticMode::None).await;
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
        manager.start_manager(workers, DiagnosticMode::None).await;
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
        manager.start_manager(workers, DiagnosticMode::None).await;
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
        manager.start_manager(workers, DiagnosticMode::None).await;

        // File with different case should still match on Windows
        let file: Uri = Uri::from_file_path(fixture.join("text.txt")).unwrap();
        let worker = manager.get_worker_for_uri(&file).await;
        assert!(worker.is_some());
    }

    #[tokio::test]
    async fn test_get_worker_for_uri_dynamic_with_workspaces_no_workspaces() {
        let manager = WorkerManager::new_dynamic(create_builder());
        manager.start_manager(vec![], DiagnosticMode::None).await;

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
        let manager = WorkerManager::new_dynamic(create_builder());
        manager
            .start_manager(
                vec![WorkspaceWorker::new(
                    "file:///path/to/workspace".parse().unwrap(),
                    create_builder(),
                    DiagnosticMode::None,
                )],
                DiagnosticMode::None,
            )
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
