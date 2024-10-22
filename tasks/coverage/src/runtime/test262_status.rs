use std::{fs, sync::OnceLock, time::Duration};

use oxc_tasks_common::agent;

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
                .timeout(Duration::from_secs(10))
                .call()
                .unwrap()
                .into_string()
                .unwrap();
            let mut tests = vec![];
            regex::Regex::new(r"'(.+)': \[(FAIL|SKIP)\]").unwrap().captures_iter(&res).for_each(
                |caps| {
                    if let Some(name) = caps.get(1).map(|f| f.as_str()) {
                        if !name.eq("*") {
                            tests.push(name.to_string());
                        }
                    }
                },
            );
            tests.sort_unstable();
            tests.dedup();
            fs::write(path, tests.join("\n")).unwrap();
            tests
        };

        lines
    })
}
