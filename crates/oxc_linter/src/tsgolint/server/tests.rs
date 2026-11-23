//! Protocol tests for [`TsrsLintServer`] against a fake server, the Node script
//! `fixtures/tsrs_lint_server/tsrs-lint`.
//!
//! Each test runs the fake in its own directory: the fake reads its behavior from `behavior.json`
//! there and appends its start and every frame it receives to `frames.jsonl`.

use std::{
    path::{Path, PathBuf},
    process::Command,
    sync::{Arc, mpsc},
    time::{Duration, Instant},
};

use serde_json::{Value, json};

use super::{
    CANCEL, DID_CLOSE, DID_OPEN, FILES_CHANGED, FileChangeKind, HELLO, LINT, LintError, SHUTDOWN,
    SHUTDOWN_TIMEOUT, TsrsLintServer,
};
use crate::tsgolint::{Config, Rule, TsGoLintDiagnostic};

/// How long to wait for the fake to receive a frame.
const WAIT_TIMEOUT: Duration = Duration::from_secs(10);

struct Fixture {
    dir: tempfile::TempDir,
}

impl Fixture {
    /// A directory for the fake with `behavior`, or `None` when `node` is not available.
    #[expect(clippy::print_stderr, reason = "reports a skipped test")]
    fn new(behavior: &Value) -> Option<Self> {
        let has_node = Command::new("node")
            .arg("--version")
            .output()
            .is_ok_and(|output| output.status.success());
        if !has_node {
            eprintln!("skipping tsrs-lint server client test: `node` is not available");
            return None;
        }
        let dir = tempfile::tempdir().expect("failed to create a temporary directory");
        std::fs::write(dir.path().join("behavior.json"), behavior.to_string())
            .expect("failed to write behavior.json");
        Some(Self { dir })
    }

    fn server(&self) -> TsrsLintServer {
        let executable =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/tsrs_lint_server/tsrs-lint");
        TsrsLintServer::new(executable, self.dir.path().to_path_buf())
    }

    /// A source file path in the fixture directory. The fake never reads it.
    fn file(&self, name: &str) -> String {
        self.dir.path().join(name).to_string_lossy().into_owned()
    }

    /// Every event the fake logged, in order.
    fn events(&self) -> Vec<Value> {
        let Ok(log) = std::fs::read_to_string(self.dir.path().join("frames.jsonl")) else {
            return vec![];
        };
        log.lines().map(|line| serde_json::from_str(line).expect("invalid log line")).collect()
    }

    /// The types of the frames the fake received, in order.
    fn frame_types(&self) -> Vec<u8> {
        self.events()
            .iter()
            .filter_map(|event| event["type"].as_u64())
            .map(|frame_type| u8::try_from(frame_type).unwrap())
            .collect()
    }

    /// The messages of the frames of `frame_type` the fake received, in order.
    fn messages(&self, frame_type: u8) -> Vec<Value> {
        self.events()
            .into_iter()
            .filter(|event| event["type"].as_u64() == Some(u64::from(frame_type)))
            .map(|event| event["message"].clone())
            .collect()
    }

    /// How many fake processes started.
    fn starts(&self) -> usize {
        self.events().iter().filter(|event| event["event"] == "start").count()
    }

    /// Waits until the fake received a frame of `frame_type`.
    fn wait_for_frame(&self, frame_type: u8) {
        let deadline = Instant::now() + WAIT_TIMEOUT;
        while !self.frame_types().contains(&frame_type) {
            assert!(
                Instant::now() < deadline,
                "the fake did not receive a frame of type {frame_type} within {WAIT_TIMEOUT:?}"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}

fn configs(file_path: &str) -> Vec<Config> {
    vec![Config {
        file_paths: vec![file_path.to_string()],
        rules: vec![Rule { name: "no-floating-promises".to_string(), options: None }],
    }]
}

fn lint(server: &TsrsLintServer, file_path: &str) -> Result<Vec<TsGoLintDiagnostic>, LintError> {
    server.lint(&configs(file_path), false, false)
}

fn failure(result: Result<Vec<TsGoLintDiagnostic>, LintError>) -> String {
    match result {
        Err(LintError::Failed(error)) => error,
        Err(LintError::Canceled) => panic!("expected a failure, got a cancellation"),
        Ok(diagnostics) => panic!("expected a failure, got {} diagnostics", diagnostics.len()),
    }
}

#[test]
fn lint_routes_diagnostics_by_request_id() {
    let Some(fixture) = Fixture::new(&json!({})) else { return };
    let server = fixture.server();
    let file = fixture.file("index.ts");

    let diagnostics = lint(&server, &file).expect("lint failed");

    // The fake also sends a diagnostic for another request id, which is not this request's.
    assert_eq!(diagnostics.len(), 1);
    let TsGoLintDiagnostic::Rule(diagnostic) = &diagnostics[0] else {
        panic!("expected a rule diagnostic");
    };
    assert_eq!(diagnostic.rule, "no-floating-promises");
    assert_eq!(diagnostic.message.description, "fake diagnostic for request 1");
    assert_eq!(diagnostic.file_path, PathBuf::from(&file));

    assert_eq!(fixture.events()[0]["argv"], json!(["server"]));
    assert_eq!(fixture.messages(HELLO), vec![json!({ "protocol_version": 1 })]);
    let lints = fixture.messages(LINT);
    assert_eq!(lints.len(), 1);
    assert_eq!(
        lints[0],
        json!({
            "id": 1,
            "configs": [{ "file_paths": [file], "rules": [{ "name": "no-floating-promises" }] }],
            "fix": false,
            "fix_suggestions": false,
        })
    );
    assert!(!server.is_disabled());

    // The running process serves the next request.
    let diagnostics = lint(&server, &file).expect("second lint failed");
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(fixture.starts(), 1);
}

#[test]
fn other_protocol_version_disables_the_server() {
    let Some(fixture) = Fixture::new(&json!({ "protocol_version": 2 })) else { return };
    let server = fixture.server();
    let file = fixture.file("index.ts");

    let error = failure(lint(&server, &file));
    assert!(error.contains("speaks server protocol 2"), "{error}");
    assert!(error.contains("this oxlint speaks protocol 1"), "{error}");
    assert!(server.is_disabled());

    // The server is not started again.
    let second = failure(lint(&server, &file));
    assert_eq!(second, error);
    assert_eq!(fixture.starts(), 1);
    assert_eq!(fixture.messages(HELLO).len(), 1);
    assert_eq!(fixture.messages(LINT), [] as [Value; 0]);
}

#[test]
fn executable_without_server_mode_disables_the_server() {
    let Some(fixture) = Fixture::new(&json!({ "start": "exit" })) else { return };
    let server = fixture.server();
    let file = fixture.file("index.ts");

    let error = failure(lint(&server, &file));
    assert!(error.contains("exited before the server handshake"), "{error}");
    assert!(server.is_disabled());

    let second = failure(lint(&server, &file));
    assert_eq!(second, error);
    assert_eq!(fixture.starts(), 1);
}

#[test]
fn process_that_keeps_exiting_is_not_started_again() {
    let Some(fixture) = Fixture::new(&json!({ "lint": "exit" })) else { return };
    let server = fixture.server();
    let file = fixture.file("index.ts");

    for attempt in 1..=3 {
        let error = failure(lint(&server, &file));
        assert!(!error.contains("times within"), "attempt {attempt}: {error}");
        assert!(!server.is_disabled(), "attempt {attempt}");
        assert_eq!(fixture.starts(), attempt);
    }

    let error = failure(lint(&server, &file));
    assert!(error.contains("stopped 3 times within 60 seconds"), "{error}");
    assert!(error.contains("Last error: tsrs-lint server exited"), "{error}");
    assert!(server.is_disabled());
    assert_eq!(fixture.starts(), 3);
    assert_eq!(fixture.messages(LINT).len(), 3);
}

#[test]
fn closing_a_document_cancels_its_request() {
    let Some(fixture) = Fixture::new(&json!({ "lint": "hold" })) else { return };
    let server = Arc::new(fixture.server());
    let file = fixture.file("index.ts");
    server.did_open(Path::new(&file), 1, Arc::from("Promise.resolve();\n"));

    let (sender, result) = mpsc::channel();
    {
        let server = Arc::clone(&server);
        let file = file.clone();
        std::thread::spawn(move || sender.send(lint(&server, &file)));
    }
    fixture.wait_for_frame(LINT);
    server.did_close(Path::new(&file));

    let result = result.recv_timeout(WAIT_TIMEOUT).expect("the request did not end");
    assert!(matches!(result, Err(LintError::Canceled)), "{result:?}");
    fixture.wait_for_frame(DID_CLOSE);

    // The open document reached the new process before the request, and the cancellation
    // before the close.
    assert_eq!(fixture.frame_types(), vec![HELLO, DID_OPEN, LINT, CANCEL, DID_CLOSE]);
    assert_eq!(
        fixture.messages(DID_OPEN),
        vec![json!({ "path": file, "version": 1, "text": "Promise.resolve();\n" })]
    );
    let id = &fixture.messages(LINT)[0]["id"];
    assert_eq!(fixture.messages(CANCEL), vec![json!({ "id": id })]);
    assert_eq!(fixture.messages(DID_CLOSE), vec![json!({ "path": file })]);
    assert!(!server.is_disabled());
}

#[test]
fn file_changes_are_sent_as_one_batch_before_the_next_frame() {
    let Some(fixture) = Fixture::new(&json!({})) else { return };
    let server = fixture.server();
    let file = fixture.file("index.ts");
    let a = PathBuf::from(fixture.file("a.ts"));
    let b = PathBuf::from(fixture.file("b.ts"));

    // A process that is not running reads the disk when it starts: nothing is sent.
    server.files_changed(&[(a.clone(), FileChangeKind::Changed)]);
    lint(&server, &file).expect("first lint failed");
    assert_eq!(fixture.messages(FILES_CHANGED), [] as [Value; 0]);

    server.files_changed(&[(a.clone(), FileChangeKind::Changed)]);
    server.files_changed(&[(b.clone(), FileChangeKind::Created)]);
    server.files_changed(&[(a.clone(), FileChangeKind::Deleted)]);
    // Nothing is sent until the next frame.
    std::thread::sleep(Duration::from_millis(100));
    assert_eq!(fixture.messages(FILES_CHANGED), [] as [Value; 0]);

    lint(&server, &file).expect("second lint failed");

    assert_eq!(fixture.frame_types(), vec![HELLO, LINT, FILES_CHANGED, LINT]);
    let batches = fixture.messages(FILES_CHANGED);
    assert_eq!(batches.len(), 1);
    let mut changes: Vec<(String, u64)> = batches[0]["changes"]
        .as_array()
        .expect("changes is not an array")
        .iter()
        .map(|change| {
            (change["path"].as_str().unwrap().to_string(), change["kind"].as_u64().unwrap())
        })
        .collect();
    changes.sort();
    // The latest kind of each path.
    assert_eq!(
        changes,
        vec![
            (a.to_string_lossy().into_owned(), FileChangeKind::Deleted as u64),
            (b.to_string_lossy().into_owned(), FileChangeKind::Created as u64),
        ]
    );
}

#[test]
fn stop_shuts_the_process_down() {
    let Some(fixture) = Fixture::new(&json!({})) else { return };
    let server = fixture.server();
    let file = fixture.file("index.ts");
    lint(&server, &file).expect("lint failed");
    let pid = fixture.events()[0]["pid"].as_u64().expect("no pid in the start event");

    let started = Instant::now();
    server.stop();

    // The process exited on its own after the shutdown request, without being killed.
    assert!(started.elapsed() < SHUTDOWN_TIMEOUT, "stop took {:?}", started.elapsed());
    assert_eq!(fixture.messages(SHUTDOWN), vec![json!({})]);
    assert_eq!(fixture.events().last().unwrap()["event"], "exit");
    let alive = Command::new("kill")
        .args(["-0", &pid.to_string()])
        .output()
        .is_ok_and(|output| output.status.success());
    assert!(!alive, "the process {pid} is still running");

    assert!(server.is_disabled());
    let error = failure(lint(&server, &file));
    assert_eq!(error, "tsrs-lint server was stopped");
    assert_eq!(fixture.starts(), 1);
}
