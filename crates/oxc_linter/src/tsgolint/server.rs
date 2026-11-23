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
//! A request whose results can no longer be used, because a file it lints was closed, is
//! canceled: the server stops it and answers with a canceled error.
//!
//! [`TsrsLintServer::stop`] (or dropping the last reference) asks the process to shut down, and
//! kills it if it has not exited within [`SHUTDOWN_TIMEOUT`].

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
    connection: Mutex<Option<Arc<Connection>>>,
    /// Why the server is not started again, once it cannot be used.
    disabled: Mutex<Option<String>>,
    /// When the recent processes were started, oldest first.
    starts: Mutex<VecDeque<Instant>>,
    /// Why the last process exited.
    last_exit: Arc<Mutex<Option<String>>>,
    /// The open documents, sent again to a respawned process.
    open_documents: Mutex<FxHashMap<PathBuf, OpenDocument>>,
    next_id: AtomicU64,
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
    /// The request was canceled before it finished.
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

impl Drop for Connection {
    /// Asks the process to shut down, and kills it if it does not exit in time.
    fn drop(&mut self) {
        if self.is_alive() {
            let _ = self.send(SHUTDOWN, &serde_json::json!({}));
        }
        let child = self.child.get_mut().expect("tsrs-lint child mutex poisoned");
        let deadline = Instant::now() + SHUTDOWN_TIMEOUT;
        while Instant::now() < deadline {
            match child.try_wait() {
                Ok(Some(_)) => return,
                Ok(None) => std::thread::sleep(Duration::from_millis(10)),
                Err(_) => break,
            }
        }
        let _ = child.kill();
        let _ = child.wait();
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
            disabled: Mutex::new(None),
            starts: Mutex::new(VecDeque::new()),
            last_exit: Arc::default(),
            open_documents: Mutex::new(FxHashMap::default()),
            next_id: AtomicU64::new(1),
        }
    }

    /// The executable this server runs.
    pub fn executable_path(&self) -> &Path {
        &self.executable_path
    }

    /// Shuts the process down, and does not start another one. Pending requests end with an
    /// error.
    ///
    /// # Panics
    /// Panics if an internal mutex is poisoned.
    pub fn stop(&self) {
        let mut connection = self.connection.lock().expect("tsrs-lint connection mutex poisoned");
        *self.disabled.lock().expect("tsrs-lint disabled mutex poisoned") =
            Some("tsrs-lint server was stopped".to_string());
        // Dropping the connection shuts the process down.
        connection.take();
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
        if let Some(connection) = connection.as_ref().filter(|connection| connection.is_alive()) {
            let paths: Vec<_> = changes.iter().map(|(path, _)| path.to_string_lossy()).collect();
            let message = FilesChangedMessage {
                changes: paths
                    .iter()
                    .zip(changes)
                    .map(|(path, (_, kind))| FileChange { path, kind: *kind as u8 })
                    .collect(),
            };
            let _ = connection.send(FILES_CHANGED, &message);
        }
    }

    /// Lints the files of `configs` and returns their diagnostics.
    pub(super) fn lint(
        &self,
        configs: &[Config],
        fix: bool,
        fix_suggestions: bool,
    ) -> Result<Vec<TsGoLintDiagnostic>, LintError> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let (sender, events) = mpsc::channel();
        let file_paths =
            configs.iter().flat_map(|config| config.file_paths.iter().cloned()).collect();
        let connection = {
            let mut connection =
                self.connection.lock().expect("tsrs-lint connection mutex poisoned");
            let connection = self.ensure_connection(&mut connection)?;
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
            connection
        };

        let mut diagnostics = vec![];
        let result = loop {
            match events.recv() {
                Ok(Event::Diagnostic(diagnostic)) => diagnostics.push(diagnostic),
                Ok(Event::Done) => break Ok(diagnostics),
                Ok(Event::Error(error)) => break Err(LintError::Failed(error)),
                Ok(Event::Canceled) => break Err(LintError::Canceled),
                Err(_) => break Err(LintError::Failed("tsrs-lint server exited".to_string())),
            }
        };
        connection.pending.lock().expect("tsrs-lint pending mutex poisoned").remove(&id);
        result
    }

    /// Returns the running process, spawning it (and sending it the open documents) when there is
    /// none or it exited.
    fn ensure_connection(
        &self,
        slot: &mut Option<Arc<Connection>>,
    ) -> Result<Arc<Connection>, String> {
        if let Some(connection) = slot.as_ref().filter(|connection| connection.is_alive()) {
            return Ok(Arc::clone(connection));
        }
        let mut disabled = self.disabled.lock().expect("tsrs-lint disabled mutex poisoned");
        if let Some(reason) = disabled.as_ref() {
            return Err(reason.clone());
        }
        // Drop an exited process before starting the next one.
        *slot = None;
        if !self.record_start() {
            let last_exit = self.last_exit.lock().expect("tsrs-lint last exit mutex poisoned");
            let reason = format!(
                "tsrs-lint server stopped {MAX_STARTS} times within {} seconds, so type-aware \
                 linting is off for this workspace until its configuration changes or the \
                 language server restarts. Last error: {}",
                START_WINDOW.as_secs(),
                last_exit.as_deref().unwrap_or("unknown")
            );
            *disabled = Some(reason.clone());
            return Err(reason);
        }
        let connection = match self.spawn() {
            Ok(connection) => Arc::new(connection),
            Err(SpawnError::Incompatible(reason)) => {
                *disabled = Some(reason.clone());
                return Err(reason);
            }
            Err(SpawnError::Failed(error)) => {
                *self.last_exit.lock().expect("tsrs-lint last exit mutex poisoned") =
                    Some(error.clone());
                return Err(error);
            }
        };
        drop(disabled);
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
        drop(documents);
        *slot = Some(Arc::clone(&connection));
        Ok(connection)
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
        let (hello_sender, hello) = mpsc::channel();
        {
            let pending = Arc::clone(&pending);
            let alive = Arc::clone(&alive);
            let last_exit = Arc::clone(&self.last_exit);
            std::thread::Builder::new()
                .name("tsrs-lint-reader".into())
                .spawn(move || {
                    let exit_error = read_loop(stdout, &pending, &alive, &hello_sender);
                    *last_exit.lock().expect("tsrs-lint last exit mutex poisoned") =
                        Some(exit_error);
                })
                .map_err(|e| {
                    SpawnError::Failed(format!("Failed to start the tsrs-lint reader thread: {e}"))
                })?;
        }
        let connection =
            Connection { child: Mutex::new(child), stdin: Mutex::new(stdin), pending, alive };
        // A `tsrs-lint` without server mode exits without reading its input, so the write may
        // fail: its answer, or the end of its output, tells what happened.
        let _ = connection.send(HELLO, &HelloMessage { protocol_version: PROTOCOL_VERSION });
        match hello.recv_timeout(HANDSHAKE_TIMEOUT) {
            Ok(hello) if hello.protocol_version == PROTOCOL_VERSION => Ok(connection),
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
    for (_, request) in waiting {
        let _ = request.sender.send(Event::Error(exit_error.clone()));
    }
    exit_error
}

fn send(pending: &Pending, id: u64, event: Event) {
    let pending = pending.lock().expect("tsrs-lint pending mutex poisoned");
    if let Some(request) = pending.get(&id) {
        let _ = request.sender.send(event);
    }
}
