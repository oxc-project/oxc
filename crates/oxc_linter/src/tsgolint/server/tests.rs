//! Protocol tests for [`TsrsLintServer`] against a fake server, the Node script
//! `fixtures/tsrs_lint_server/tsrs-lint`.
//!
//! Each test runs the fake in its own directory: the fake reads its behavior from `behavior.json`
//! there and appends its start and every frame it receives to `frames.jsonl`.

use std::{
    path::{Path, PathBuf},
    process::Command,
};

use serde_json::{Value, json};

use super::{HELLO, LINT, TsrsLintServer};
use crate::tsgolint::{Config, Rule, TsGoLintDiagnostic};

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
}

fn configs(file_path: &str) -> Vec<Config> {
    vec![Config {
        file_paths: vec![file_path.to_string()],
        rules: vec![Rule { name: "no-floating-promises".to_string(), options: None }],
    }]
}

fn lint(server: &TsrsLintServer, file_path: &str) -> Result<Vec<TsGoLintDiagnostic>, String> {
    server.lint(&configs(file_path), false, false)
}

fn failure(result: Result<Vec<TsGoLintDiagnostic>, String>) -> String {
    match result {
        Err(error) => error,
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

    let second = failure(lint(&server, &file));
    assert_eq!(second, error);
    assert_eq!(fixture.starts(), 1);
}
