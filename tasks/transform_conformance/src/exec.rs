use std::{path::Path, process::Command};

use crate::{conformance_root, TestRunner};

impl TestRunner {
    pub(crate) fn run_vitest(&self, dir: &str, dest: &Path) {
        let version = String::from("node: ")
            + &String::from_utf8(Command::new("node").arg("--version").output().unwrap().stdout)
                .unwrap();
        let output = Command::new("node")
            .current_dir(conformance_root())
            .env("NO_COLOR", "1")
            .args([
                "--run",
                "vitest",
                "--",
                "run",
                "--reporter=./reporter.mjs",
                "--exclude=\"\"",
                "--no-color",
                dir,
            ])
            .output()
            .unwrap();
        let content = if output.stderr.is_empty() { output.stdout } else { output.stderr };
        let output = String::from_utf8(content).unwrap();
        let output = version + &output;
        self.snapshot.save(dest, &output);
    }
}
