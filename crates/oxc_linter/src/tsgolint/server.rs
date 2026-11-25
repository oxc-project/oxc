//! Client for `tsrs-lint server`: a long-lived type-aware linter process used by the language server.
//!
//! The process keeps the programs of the workspace's projects between requests, so a request only
//! re-parses what changed and re-checks what it needs. Messages use the headless framing in both
//! directions: `u32 LE JSON length | u8 message type | UTF-8 JSON`.
//!
//! A new process first answers a handshake with the protocol version it speaks. oxlint and
//! `tsrs-lint` are installed separately, so a process that speaks another version, or a
//! `tsrs-lint` without server mode, is reported once and not started again.
//!
//! A process that exits is started again on the next request, but at most [`MAX_STARTS`] times
//! within [`START_WINDOW`]: a process that keeps exiting is not started again either.
//!
//! A process that sends no frame for a request within [`REQUEST_TIMEOUT`] is killed, so a stuck
//! process does not block the language server forever.
//!
//! A request whose results can no longer be used, because a file it lints was closed, is
//! canceled: the server stops it and answers with a canceled error.
//!
//! [`TsrsLintServer::stop`] (or dropping the last reference) asks the process to shut down, and
//! kills it if it has not exited within [`SHUTDOWN_TIMEOUT`]. Its pending requests are canceled.
//!
//! Changes on disk are collected and sent as one `FilesChanged` frame before the next frame, so a
//! burst of watcher events (an install, a build) reaches the process as one batch.

use std::{
    collections::VecDeque,
    ffi::OsStr,
    io::{BufReader, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, ChildStdout},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc::{self, RecvTimeoutError},
    },
    time::{Duration, Instant},
};

use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info, warn};

use super::{
    Config, MessageType, TsGoLintDiagnostic, TsGoLintMessage, decode_message, read_frame,
    type_aware_command,
};

#[cfg(all(test, unix))]
mod tests;

/// The version of the server protocol this client speaks. The server must speak the same one.
const PROTOCOL_VERSION: u32 = 1;

/// How long a new process may take to answer the handshake.
const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(30);

/// How long a process may take to exit after it is asked to shut down.
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(2);

/// How long a request may wait for its next frame. A process that does not answer in time is
/// killed, and the next request starts another one.
const REQUEST_TIMEOUT: Duration = Duration::from_mins(5);

/// At most this many processes are started within [`START_WINDOW`].
const MAX_STARTS: usize = 3;
const START_WINDOW: Duration = Duration::from_mins(1);

// Client -> server message types.
const DID_OPEN: u8 = 16;
const DID_CHANGE: u8 = 17;
const DID_CLOSE: u8 = 18;
const FILES_CHANGED: u8 = 19;
const LINT: u8 = 20;
const SHUTDOWN: u8 = 21;
const HELLO: u8 = 22;
const CANCEL: u8 = 23;

// Server -> client message types: the one-shot protocol's, the end of a request, and the
// handshake's answer.
const ERROR: u8 = MessageType::Error as u8;
const DIAGNOSTIC: u8 = MessageType::Diagnostic as u8;
const DONE: u8 = 3;
const SERVER_HELLO: u8 = 4;

/// Kind of a file change on disk, as LSP's `FileChangeType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FileChangeKind {
    Created = 1,
    Changed = 2,
    Deleted = 3,
}

/// Whether `path` names the `tsrs-lint` executable, which supports the long-lived server mode.
/// `tsgolint` does not.
pub fn is_tsrs_lint_executable(path: &Path) -> bool {
    path.file_stem()
        .and_then(OsStr::to_str)
        .is_some_and(|stem| stem.eq_ignore_ascii_case("tsrs-lint"))
}

/// A `tsrs-lint server` process for one workspace, spawned on first use and respawned after it
/// exits. It is shared by every linter built for the workspace.
pub struct TsrsLintServer {
    executable_path: PathBuf,
    cwd: PathBuf,
    /// The running process. Its lock orders the frames sent to it, and is never held while a
    /// process starts: document updates from the editor do not wait for a handshake.
    connection: Mutex<Option<Arc<Connection>>>,
    /// Held while a process starts, so only one starts at a time.
    start: Mutex<()>,
    /// Set while a process starts, so the changes on disk from then on are sent to it.
    starting: AtomicBool,
    /// Why the server is not started again, once it cannot be used.
    disabled: Mutex<Option<String>>,
    /// When the recent processes were started, oldest first.
    starts: Mutex<VecDeque<Instant>>,
    /// Why the last process exited.
    last_exit: Arc<Mutex<Option<String>>>,
    /// The changes on disk not sent yet, with the latest kind of each path.
    file_changes: Mutex<FxHashMap<PathBuf, FileChangeKind>>,
    /// The open documents, sent again to a respawned process.
    open_documents: Mutex<FxHashMap<PathBuf, OpenDocument>>,
    next_id: AtomicU64,
    /// [`REQUEST_TIMEOUT`], shorter in tests.
    request_timeout: Duration,
}

impl std::fmt::Debug for TsrsLintServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TsrsLintServer")
            .field("executable_path", &self.executable_path)
            .field("cwd", &self.cwd)
            .finish_non_exhaustive()
    }
}

struct OpenDocument {
    version: i32,
    text: Arc<str>,
}

enum Event {
    Diagnostic(TsGoLintDiagnostic),
    Done,
    Error(String),
    Canceled,
}

/// A request waiting for its terminal frame.
struct PendingRequest {
    sender: mpsc::Sender<Event>,
    /// The files it lints, as sent.
    file_paths: Vec<String>,
}

type Pending = Arc<Mutex<FxHashMap<u64, PendingRequest>>>;

/// Why a lint request has no results.
#[derive(Debug)]
pub(super) enum LintError {
    /// The request was canceled before it finished, or not sent because the server has other
    /// text for one of its files than the text being linted.
    Canceled,
    Failed(String),
}

impl From<String> for LintError {
    fn from(error: String) -> Self {
        Self::Failed(error)
    }
}

struct Connection {
    child: Mutex<Child>,
    stdin: Mutex<ChildStdin>,
    /// The senders of the requests waiting for their terminal frame, by request id.
    pending: Pending,
    alive: Arc<AtomicBool>,
    /// Set once the process is asked to shut down, so its exit is expected.
    stopping: Arc<AtomicBool>,
    pid: u32,
}

impl Connection {
    fn is_alive(&self) -> bool {
        self.alive.load(Ordering::Acquire)
    }

    /// Cancels the pending requests that lint a file matching `matches`. Each still gets its
    /// terminal frame: a canceled error, or its results if it finished first.
    fn cancel_requests(&self, matches: impl Fn(&str) -> bool) {
        let ids: Vec<u64> = self
            .pending
            .lock()
            .expect("tsrs-lint pending mutex poisoned")
            .iter()
            .filter(|(_, request)| request.file_paths.iter().any(|path| matches(path)))
            .map(|(id, _)| *id)
            .collect();
        for id in ids {
            debug!("canceling tsrs-lint server request {id}");
            let _ = self.send(CANCEL, &CancelMessage { id });
        }
    }

    fn send<T: Serialize>(&self, message_type: u8, message: &T) -> Result<(), String> {
        let json = serde_json::to_vec(message).map_err(|e| e.to_string())?;
        let length = u32::try_from(json.len()).map_err(|e| e.to_string())?;
        let mut stdin = self.stdin.lock().expect("tsrs-lint stdin mutex poisoned");
        let result = stdin
            .write_all(&length.to_le_bytes())
            .and_then(|()| stdin.write_all(&[message_type]))
            .and_then(|()| stdin.write_all(&json))
            .and_then(|()| stdin.flush());
        result.map_err(|e| {
            self.alive.store(false, Ordering::Release);
            format!("Failed to write to tsrs-lint server: {e}")
        })
    }
}

impl Connection {
    /// Kills a process that stopped answering. Its pending requests end with an error, and the
    /// next request starts another process.
    fn kill(&self, reason: &str) {
        warn!("killing tsrs-lint server (pid {}): {reason}", self.pid);
        self.alive.store(false, Ordering::Release);
        let mut child = self.child.lock().expect("tsrs-lint child mutex poisoned");
        let _ = child.kill();
        let _ = child.wait();
    }
}

impl Connection {
    /// Asks the process to shut down, and kills it if it does not exit in time. Its pending
    /// requests end as canceled.
    fn shut_down(&self) {
        self.stopping.store(true, Ordering::Release);
        if self.is_alive() {
            debug!("asking tsrs-lint server (pid {}) to shut down", self.pid);
            let _ = self.send(SHUTDOWN, &serde_json::json!({}));
        }
        let mut child = self.child.lock().expect("tsrs-lint child mutex poisoned");
        let deadline = Instant::now() + SHUTDOWN_TIMEOUT;
        while Instant::now() < deadline {
            match child.try_wait() {
                Ok(Some(status)) => {
                    debug!("tsrs-lint server (pid {}) exited with {status}", self.pid);
                    return;
                }
                Ok(None) => std::thread::sleep(Duration::from_millis(10)),
                Err(_) => break,
            }
        }
        warn!(
            "tsrs-lint server (pid {}) did not exit within {} seconds, killing it",
            self.pid,
            SHUTDOWN_TIMEOUT.as_secs()
        );
        let _ = child.kill();
        let _ = child.wait();
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        self.shut_down();
    }
}

#[derive(Serialize)]
struct DocumentMessage<'a> {
    path: &'a str,
    version: i32,
    text: &'a str,
}

#[derive(Serialize)]
struct CloseMessage<'a> {
    path: &'a str,
}

#[derive(Serialize)]
struct CancelMessage {
    id: u64,
}

#[derive(Serialize)]
struct FilesChangedMessage<'a> {
    changes: Vec<FileChange<'a>>,
}

#[derive(Serialize)]
struct FileChange<'a> {
    path: &'a str,
    kind: u8,
}

#[derive(Serialize)]
struct LintMessage<'a> {
    id: u64,
    configs: &'a [Config],
    fix: bool,
    fix_suggestions: bool,
}

#[derive(Serialize)]
struct HelloMessage {
    protocol_version: u32,
}

#[derive(Deserialize)]
struct HelloPayload {
    protocol_version: u32,
    #[serde(default)]
    version: String,
}

#[derive(Deserialize)]
struct IdPayload {
    id: Option<u64>,
    /// On an error frame: the request was canceled.
    #[serde(default)]
    canceled: bool,
}

/// Why a process could not be started.
enum SpawnError {
    /// This executable can never serve this client: it is not started again.
    Incompatible(String),
    Failed(String),
}

#[derive(Deserialize)]
struct DonePayload {
    id: u64,
}

impl TsrsLintServer {
    /// Creates a server for the workspace at `cwd`. The process starts on the first lint request.
    pub fn new(executable_path: PathBuf, cwd: PathBuf) -> Self {
        Self {
            executable_path,
            cwd,
            connection: Mutex::new(None),
            start: Mutex::new(()),
            starting: AtomicBool::new(false),
            disabled: Mutex::new(None),
            starts: Mutex::new(VecDeque::new()),
            last_exit: Arc::default(),
            file_changes: Mutex::default(),
            open_documents: Mutex::new(FxHashMap::default()),
            next_id: AtomicU64::new(1),
            request_timeout: REQUEST_TIMEOUT,
        }
    }

    /// Uses `timeout` instead of [`REQUEST_TIMEOUT`].
    #[cfg(test)]
    fn with_request_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = timeout;
        self
    }

    /// The executable this server runs.
    pub fn executable_path(&self) -> &Path {
        &self.executable_path
    }

    /// Shuts the process down, and does not start another one. Pending requests end as
    /// canceled: their results are no longer shown.
    ///
    /// # Panics
    /// Panics if an internal mutex is poisoned.
    pub fn stop(&self) {
        let connection = {
            let mut connection =
                self.connection.lock().expect("tsrs-lint connection mutex poisoned");
            info!("stopping tsrs-lint server for `{}`", self.cwd.display());
            *self.disabled.lock().expect("tsrs-lint disabled mutex poisoned") =
                Some("tsrs-lint server was stopped".to_string());
            connection.take()
        };
        // A pending request holds the connection too: shut the process down now, not when the
        // last reference is dropped.
        if let Some(connection) = connection {
            connection.shut_down();
        }
    }

    /// Whether the server is not started again: its executable cannot serve this client, its
    /// process kept exiting, or it was stopped. A new server for the same executable starts
    /// afresh.
    ///
    /// # Panics
    /// Panics if an internal mutex is poisoned.
    pub fn is_disabled(&self) -> bool {
        self.disabled.lock().expect("tsrs-lint disabled mutex poisoned").is_some()
    }

    /// A document was opened in the editor.
    pub fn did_open(&self, path: &Path, version: i32, text: Arc<str>) {
        self.update_document(DID_OPEN, path, version, text);
    }

    /// An open document changed in the editor, including unsaved edits.
    pub fn did_change(&self, path: &Path, version: i32, text: Arc<str>) {
        self.update_document(DID_CHANGE, path, version, text);
    }

    fn update_document(&self, message_type: u8, path: &Path, version: i32, text: Arc<str>) {
        // The connection lock orders the update with a concurrent respawn.
        let connection = self.connection.lock().expect("tsrs-lint connection mutex poisoned");
        let path_str = path.to_string_lossy();
        if let Some(connection) = connection.as_ref().filter(|connection| connection.is_alive()) {
            self.send_file_changes(connection);
            let _ = connection
                .send(message_type, &DocumentMessage { path: &path_str, version, text: &text });
        }
        self.open_documents
            .lock()
            .expect("tsrs-lint open documents mutex poisoned")
            .insert(path.to_path_buf(), OpenDocument { version, text });
    }

    /// A document was closed in the editor.
    ///
    /// # Panics
    /// Panics if an internal mutex is poisoned.
    pub fn did_close(&self, path: &Path) {
        let connection = self.connection.lock().expect("tsrs-lint connection mutex poisoned");
        let removed = self
            .open_documents
            .lock()
            .expect("tsrs-lint open documents mutex poisoned")
            .remove(path)
            .is_some();
        if removed
            && let Some(connection) = connection.as_ref().filter(|connection| connection.is_alive())
        {
            let path = path.to_string_lossy();
            // The results of a request for a closed file are not shown.
            connection.cancel_requests(|file_path| file_path == path);
            self.send_file_changes(connection);
            let _ = connection.send(DID_CLOSE, &CloseMessage { path: &path });
        }
    }

    /// Files changed on disk. A process that is not running reads them from disk when it starts.
    ///
    /// # Panics
    /// Panics if an internal mutex is poisoned.
    pub fn files_changed(&self, changes: &[(PathBuf, FileChangeKind)]) {
        if changes.is_empty() {
            return;
        }
        let connection = self.connection.lock().expect("tsrs-lint connection mutex poisoned");
        if connection.as_ref().is_some_and(|connection| connection.is_alive())
            || self.starting.load(Ordering::Acquire)
        {
            self.file_changes
                .lock()
                .expect("tsrs-lint file changes mutex poisoned")
                .extend(changes.iter().cloned());
        }
    }

    /// Sends the collected changes on disk as one frame. Called with the connection lock held,
    /// before any other frame, so the process sees them in order.
    fn send_file_changes(&self, connection: &Connection) {
        let changes = std::mem::take(
            &mut *self.file_changes.lock().expect("tsrs-lint file changes mutex poisoned"),
        );
        if changes.is_empty() {
            return;
        }
        let paths: Vec<_> =
            changes.iter().map(|(path, kind)| (path.to_string_lossy(), *kind)).collect();
        let message = FilesChangedMessage {
            changes: paths
                .iter()
                .map(|(path, kind)| FileChange { path, kind: *kind as u8 })
                .collect(),
        };
        let _ = connection.send(FILES_CHANGED, &message);
    }

    /// Lints the files of `configs` and returns their diagnostics.
    ///
    /// `sources` is the text oxlint lints each file with, by path. The server lints the text it
    /// was last sent for an open file, and the request carries no version: when an edit reached
    /// the server since `sources` was read, its results would not match `sources`, so the request
    /// is not sent and ends as canceled. The edit's own lint follows.
    pub(super) fn lint(
        &self,
        configs: &[Config],
        sources: &FxHashMap<String, String>,
        fix: bool,
        fix_suggestions: bool,
    ) -> Result<Vec<TsGoLintDiagnostic>, LintError> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let (sender, events) = mpsc::channel();
        let file_paths =
            configs.iter().flat_map(|config| config.file_paths.iter().cloned()).collect();
        let connection = self.ensure_connection()?;
        {
            let slot = self.connection.lock().expect("tsrs-lint connection mutex poisoned");
            // The server may have been stopped, or its process replaced, since it was ensured.
            let disabled = self.disabled.lock().expect("tsrs-lint disabled mutex poisoned").clone();
            if let Some(reason) = disabled {
                return Err(LintError::Failed(reason));
            }
            if !slot.as_ref().is_some_and(|current| Arc::ptr_eq(current, &connection)) {
                return Err(LintError::Failed("tsrs-lint server exited".to_string()));
            }
            // The connection lock orders this check with the document updates.
            if let Some(path) = self.stale_document(sources) {
                debug!(
                    "tsrs-lint server request {id} not sent: `{path}` changed since it was read"
                );
                return Err(LintError::Canceled);
            }
            self.send_file_changes(&connection);
            connection
                .pending
                .lock()
                .expect("tsrs-lint pending mutex poisoned")
                .insert(id, PendingRequest { sender, file_paths });
            // The reader may have stopped after the liveness check: it would never answer this request.
            if !connection.is_alive() {
                connection.pending.lock().expect("tsrs-lint pending mutex poisoned").remove(&id);
                return Err(LintError::Failed("tsrs-lint server exited".to_string()));
            }
            let message = LintMessage { id, configs, fix, fix_suggestions };
            if let Err(error) = connection.send(LINT, &message) {
                connection.pending.lock().expect("tsrs-lint pending mutex poisoned").remove(&id);
                return Err(LintError::Failed(error));
            }
        }

        let started = Instant::now();
        let mut diagnostics = vec![];
        let result = loop {
            match events.recv_timeout(self.request_timeout) {
                Ok(Event::Diagnostic(diagnostic)) => diagnostics.push(diagnostic),
                Ok(Event::Done) => break Ok(diagnostics),
                Ok(Event::Error(error)) => break Err(LintError::Failed(error)),
                Ok(Event::Canceled) => break Err(LintError::Canceled),
                Err(RecvTimeoutError::Timeout) => {
                    let error = format!(
                        "tsrs-lint server did not answer request {id} within {} seconds",
                        self.request_timeout.as_secs()
                    );
                    connection.kill(&error);
                    break Err(LintError::Failed(error));
                }
                Err(RecvTimeoutError::Disconnected) => {
                    break Err(LintError::Failed("tsrs-lint server exited".to_string()));
                }
            }
        };
        connection.pending.lock().expect("tsrs-lint pending mutex poisoned").remove(&id);
        let elapsed = started.elapsed().as_millis();
        match &result {
            Ok(diagnostics) => debug!(
                "tsrs-lint server request {id}: {} diagnostics in {elapsed} ms",
                diagnostics.len()
            ),
            Err(LintError::Canceled) => {
                debug!("tsrs-lint server request {id} canceled after {elapsed} ms");
            }
            Err(LintError::Failed(error)) => {
                warn!("tsrs-lint server request {id} failed after {elapsed} ms: {error}");
            }
        }
        result
    }

    /// An open document whose text the server has is not the one in `sources`.
    fn stale_document<'a>(&self, sources: &'a FxHashMap<String, String>) -> Option<&'a str> {
        let documents =
            self.open_documents.lock().expect("tsrs-lint open documents mutex poisoned");
        sources.iter().find_map(|(path, text)| {
            documents
                .get(Path::new(path))
                .is_some_and(|document| *document.text != **text)
                .then_some(path.as_str())
        })
    }

    /// Returns the running process, spawning it (and sending it the open documents) when there is
    /// none or it exited.
    ///
    /// The connection lock is not held while a process starts, which can take up to
    /// [`HANDSHAKE_TIMEOUT`]: document updates meanwhile are recorded, and sent with the open
    /// documents once the process runs.
    fn ensure_connection(&self) -> Result<Arc<Connection>, String> {
        if let Some(connection) = self.live_connection() {
            return Ok(connection);
        }
        let _start = self.start.lock().expect("tsrs-lint start mutex poisoned");
        // Another request may have started one meanwhile.
        if let Some(connection) = self.live_connection() {
            return Ok(connection);
        }
        // Drop an exited process before starting the next one, outside the connection lock: it
        // may take up to `SHUTDOWN_TIMEOUT` to exit.
        let exited = self.connection.lock().expect("tsrs-lint connection mutex poisoned").take();
        drop(exited);
        {
            let mut disabled = self.disabled.lock().expect("tsrs-lint disabled mutex poisoned");
            if let Some(reason) = disabled.as_ref() {
                return Err(reason.clone());
            }
            if !self.record_start() {
                let last_exit = self.last_exit.lock().expect("tsrs-lint last exit mutex poisoned");
                let reason = format!(
                    "tsrs-lint server stopped {MAX_STARTS} times within {} seconds, so type-aware \
                     linting is off for this workspace until its configuration changes or the \
                     language server restarts. Last error: {}",
                    START_WINDOW.as_secs(),
                    last_exit.as_deref().unwrap_or("unknown")
                );
                error!("{reason}");
                *disabled = Some(reason.clone());
                return Err(reason);
            }
        }
        // The next process reads the disk afresh: only the changes from now on are sent to it.
        self.file_changes.lock().expect("tsrs-lint file changes mutex poisoned").clear();
        self.starting.store(true, Ordering::Release);
        let spawned = self.spawn();
        let mut slot = self.connection.lock().expect("tsrs-lint connection mutex poisoned");
        self.starting.store(false, Ordering::Release);
        let connection = match spawned {
            Ok(connection) => Arc::new(connection),
            Err(SpawnError::Incompatible(reason)) => {
                error!("{reason}");
                *self.disabled.lock().expect("tsrs-lint disabled mutex poisoned") =
                    Some(reason.clone());
                return Err(reason);
            }
            Err(SpawnError::Failed(error)) => {
                warn!("{error}");
                *self.last_exit.lock().expect("tsrs-lint last exit mutex poisoned") =
                    Some(error.clone());
                return Err(error);
            }
        };
        // The server may have been stopped while the process started.
        let stopped = self.disabled.lock().expect("tsrs-lint disabled mutex poisoned").clone();
        let sent = match stopped {
            Some(reason) => Err(reason),
            None => self.send_open_documents(&connection),
        };
        if let Err(error) = sent {
            // Dropping the connection shuts the process down: not with the lock held.
            drop(slot);
            drop(connection);
            return Err(error);
        }
        *slot = Some(Arc::clone(&connection));
        Ok(connection)
    }

    /// The running process, if it is alive.
    fn live_connection(&self) -> Option<Arc<Connection>> {
        self.connection
            .lock()
            .expect("tsrs-lint connection mutex poisoned")
            .as_ref()
            .filter(|connection| connection.is_alive())
            .map(Arc::clone)
    }

    /// Sends the open documents to a new process. Called with the connection lock held, so no
    /// document update is missed.
    fn send_open_documents(&self, connection: &Connection) -> Result<(), String> {
        let documents =
            self.open_documents.lock().expect("tsrs-lint open documents mutex poisoned");
        for (path, document) in documents.iter() {
            connection.send(
                DID_OPEN,
                &DocumentMessage {
                    path: &path.to_string_lossy(),
                    version: document.version,
                    text: &document.text,
                },
            )?;
        }
        Ok(())
    }

    /// Records a process start, unless [`MAX_STARTS`] processes were started within
    /// [`START_WINDOW`]. Returns whether one may start.
    fn record_start(&self) -> bool {
        let now = Instant::now();
        let mut starts = self.starts.lock().expect("tsrs-lint starts mutex poisoned");
        while starts.front().is_some_and(|start| now.duration_since(*start) >= START_WINDOW) {
            starts.pop_front();
        }
        if starts.len() >= MAX_STARTS {
            return false;
        }
        starts.push_back(now);
        true
    }

    /// Starts a process and completes the handshake with it.
    fn spawn(&self) -> Result<Connection, SpawnError> {
        let path = self.executable_path.display();
        info!("starting tsrs-lint server `{path}` in `{}`", self.cwd.display());
        let mut child = type_aware_command(&self.executable_path, "server")
            .current_dir(&self.cwd)
            .spawn()
            .map_err(|e| {
                SpawnError::Failed(format!(
                    "Failed to spawn tsrs-lint server from path `{path}`, with error: {e}"
                ))
            })?;
        let stdin = child.stdin.take().expect("Failed to open tsrs-lint stdin");
        let stdout = child.stdout.take().expect("Failed to open tsrs-lint stdout");
        let pending: Pending = Arc::default();
        let alive = Arc::new(AtomicBool::new(true));
        let stopping = Arc::new(AtomicBool::new(false));
        let pid = child.id();
        let (hello_sender, hello) = mpsc::channel();
        {
            let pending = Arc::clone(&pending);
            let alive = Arc::clone(&alive);
            let stopping = Arc::clone(&stopping);
            let last_exit = Arc::clone(&self.last_exit);
            std::thread::Builder::new()
                .name("tsrs-lint-reader".into())
                .spawn(move || {
                    let exit_error = read_loop(stdout, &pending, &alive, &stopping, &hello_sender);
                    if stopping.load(Ordering::Acquire) {
                        debug!("tsrs-lint server (pid {pid}) stopped: {exit_error}");
                    } else {
                        warn!("tsrs-lint server (pid {pid}) stopped unexpectedly: {exit_error}");
                    }
                    *last_exit.lock().expect("tsrs-lint last exit mutex poisoned") =
                        Some(exit_error);
                })
                .map_err(|e| {
                    SpawnError::Failed(format!("Failed to start the tsrs-lint reader thread: {e}"))
                })?;
        }
        let connection = Connection {
            child: Mutex::new(child),
            stdin: Mutex::new(stdin),
            pending,
            alive,
            stopping,
            pid,
        };
        // A `tsrs-lint` without server mode exits without reading its input, so the write may
        // fail: its answer, or the end of its output, tells what happened.
        let _ = connection.send(HELLO, &HelloMessage { protocol_version: PROTOCOL_VERSION });
        match hello.recv_timeout(HANDSHAKE_TIMEOUT) {
            Ok(hello) if hello.protocol_version == PROTOCOL_VERSION => {
                info!(
                    "tsrs-lint server {} (pid {pid}, protocol {PROTOCOL_VERSION}) started",
                    hello.version
                );
                Ok(connection)
            }
            Ok(hello) => Err(SpawnError::Incompatible(format!(
                "`{path}` is tsrs-lint {} and speaks server protocol {}, but this oxlint speaks \
                 protocol {PROTOCOL_VERSION}. Install matching versions of oxlint and tsrs-lint.",
                hello.version, hello.protocol_version
            ))),
            Err(RecvTimeoutError::Disconnected) => Err(SpawnError::Incompatible(format!(
                "`{path}` exited before the server handshake. It may be a tsrs-lint without \
                 `tsrs-lint server`: this oxlint needs one that speaks server protocol \
                 {PROTOCOL_VERSION}."
            ))),
            Err(RecvTimeoutError::Timeout) => Err(SpawnError::Failed(format!(
                "`{path}` did not answer the server handshake within {} seconds",
                HANDSHAKE_TIMEOUT.as_secs()
            ))),
        }
    }
}

/// Reads frames until the process exits, routing each to the request it belongs to. Returns why
/// it stopped.
fn read_loop(
    stdout: ChildStdout,
    pending: &Pending,
    alive: &AtomicBool,
    stopping: &AtomicBool,
    hello: &mpsc::Sender<HelloPayload>,
) -> String {
    let mut reader = BufReader::new(stdout);
    let exit_error = loop {
        let (message_type, payload) = match read_frame(&mut reader) {
            Ok(Some(frame)) => frame,
            Ok(None) => break "tsrs-lint server exited".to_string(),
            Err(error) => break format!("Failed to read from tsrs-lint server: {error}"),
        };
        match message_type {
            SERVER_HELLO => match serde_json::from_slice::<HelloPayload>(&payload) {
                Ok(payload) => {
                    let _ = hello.send(payload);
                }
                Err(error) => break format!("Failed to parse tsrs-lint hello payload: {error}"),
            },
            DONE => match serde_json::from_slice::<DonePayload>(&payload) {
                Ok(done) => send(pending, done.id, Event::Done),
                Err(error) => break format!("Failed to parse tsrs-lint done payload: {error}"),
            },
            ERROR | DIAGNOSTIC => {
                // The frames of the one-shot protocol, with the id of the request they answer.
                let (id, canceled) = match serde_json::from_slice::<IdPayload>(&payload) {
                    Ok(IdPayload { id, canceled }) => (id, canceled),
                    Err(error) => break format!("Failed to parse tsrs-lint payload: {error}"),
                };
                match (decode_message(message_type, &payload), id) {
                    (Ok(TsGoLintMessage::Diagnostic(diagnostic)), Some(id)) => {
                        send(pending, id, Event::Diagnostic(diagnostic));
                    }
                    (Ok(TsGoLintMessage::Error(_)), Some(id)) if canceled => {
                        send(pending, id, Event::Canceled);
                    }
                    (Ok(TsGoLintMessage::Error(error)), Some(id)) => {
                        send(pending, id, Event::Error(error.error));
                    }
                    (Ok(TsGoLintMessage::Error(error)), None) => {
                        break format!("tsrs-lint server failed: {}", error.error);
                    }
                    (Ok(_), _) => {}
                    (Err(error), _) => break error.to_string(),
                }
            }
            // Unknown message types are skipped, so a newer server can add messages.
            _ => {}
        }
    };
    alive.store(false, Ordering::Release);
    let waiting: Vec<_> =
        pending.lock().expect("tsrs-lint pending mutex poisoned").drain().collect();
    // The requests of a process asked to shut down are canceled: their results are not shown.
    let stopped = stopping.load(Ordering::Acquire);
    for (_, request) in waiting {
        let event = if stopped { Event::Canceled } else { Event::Error(exit_error.clone()) };
        let _ = request.sender.send(event);
    }
    exit_error
}

fn send(pending: &Pending, id: u64, event: Event) {
    let pending = pending.lock().expect("tsrs-lint pending mutex poisoned");
    if let Some(request) = pending.get(&id) {
        let _ = request.sender.send(event);
    }
}
