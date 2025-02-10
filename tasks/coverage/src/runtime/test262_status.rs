use std::{fs, sync::OnceLock};

use oxc_tasks_common::agent;
use regex::Regex;

use crate::workspace_root;

/// Generate v8 test262 status file, which is used to skip failed tests
/// see <https://chromium.googlesource.com/v8/v8/+/refs/heads/main/test/test262/test262.status>
///
/// # Panics
pub fn get_v8_test262_failure_paths() -> &'static Vec<String> {
    static STATUS: OnceLock<Vec<String>> = OnceLock::new();
    STATUS.get_or_init(|| {
        let path = workspace_root().join("src/runtime/v8_test262.status");

        let lines = if path.exists() {
            fs::read_to_string(&path).unwrap().lines().map(ToString::to_string).collect::<Vec<_>>()
        } else {
            let res = agent()
                .get("http://raw.githubusercontent.com/v8/v8/main/test/test262/test262.status")
                .call()
                .unwrap()
                .body_mut()
                .read_to_string()
                .unwrap();
            let mut tests = Regex::new(r"'(.+)': \[(FAIL|SKIP)\]")
                .unwrap()
                .captures_iter(&res)
                .filter_map(|capture| capture.get(1))
                .filter(|m| m.as_str() != "*")
                .map(|m| m.as_str().to_string())
                .collect::<Vec<_>>();
            tests.sort_unstable();
            tests.dedup();
            fs::write(path, tests.join("\n")).unwrap();
            tests
        };

        lines
    })
}
