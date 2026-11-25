use std::{fs, process::Command, sync::OnceLock};

use lazy_regex::Regex;
use oxc_tasks_common::agent;

use crate::workspace_root;

/// Cases V8 itself expects to fail, skipped so the runtime suite reports oxc's
/// failures rather than the engine's.
///
/// The list is V8's `test262.status` at the V8 version bundled in the `node` on
/// `PATH`, since that engine executes the cases. V8's `main` branch is wrong in
/// both directions: it un-skips cases V8 fixed after Node's branch cut, and it
/// skips cases for test262 files newer than the pinned checkout.
///
/// The parsed list is cached per V8 version in a gitignored file.
///
/// # Panics
/// Panics if `node` cannot be run or the status file cannot be fetched.
pub fn get_v8_test262_failure_paths() -> &'static Vec<String> {
    static STATUS: OnceLock<Vec<String>> = OnceLock::new();
    STATUS.get_or_init(|| {
        let version = node_v8_version();
        let path = workspace_root().join(format!("src/runtime/v8_test262_{version}.status"));
        if path.exists() {
            return fs::read_to_string(&path).unwrap().lines().map(ToString::to_string).collect();
        }

        let status = fetch_status(&version);
        let mut tests = Regex::new(r"'(.+)': \[(FAIL|SKIP)\]")
            .unwrap()
            .captures_iter(&status)
            .filter_map(|capture| capture.get(1))
            .filter(|m| m.as_str() != "*")
            .map(|m| m.as_str().to_string())
            .collect::<Vec<_>>();
        tests.sort_unstable();
        tests.dedup();
        fs::write(path, tests.join("\n")).unwrap();
        tests
    })
}

/// `process.versions.v8` without Node's `-node.N` suffix, e.g. `14.6.202.34`.
fn node_v8_version() -> String {
    let output = Command::new("node")
        .args(["-p", "process.versions.v8"])
        .output()
        .expect("Failed to run node - ensure Node.js is installed");
    assert!(output.status.success(), "`node -p process.versions.v8` failed");
    let version = String::from_utf8(output.stdout).unwrap();
    version.trim().split('-').next().unwrap().to_string()
}

/// V8 tags every release, so the file at `version` is the list that shipped
/// with Node's engine. A version without a tag falls back to `main`.
fn fetch_status(version: &str) -> String {
    let url = |git_ref: &str| {
        format!("https://raw.githubusercontent.com/v8/v8/{git_ref}/test/test262/test262.status")
    };
    match agent().get(&url(version)).call() {
        Ok(mut res) => res.body_mut().read_to_string().unwrap(),
        Err(err) => {
            println!("V8 test262.status for {version} is unavailable ({err}), using main instead");
            agent().get(&url("main")).call().unwrap().body_mut().read_to_string().unwrap()
        }
    }
}
