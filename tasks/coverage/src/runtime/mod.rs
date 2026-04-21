//! Runtime tests - Test262 execution with Node.js
//!
//! These tests run generated code in a Node.js subprocess to verify correctness.
//! Requires `node` to be installed and the runtime server to be running.

mod test262_status;

use std::process::{Command, Stdio};

use crate::workspace_root;

/// Run runtime tests
///
/// # Panics
/// Panics if the Node.js runtime server fails to start.
pub fn run(_filter: Option<&str>, _detail: bool) {
    let path = workspace_root().join("src/runtime/runtime.js").to_string_lossy().to_string();

    println!("runtime Summary:");
    println!("Starting Node.js runtime server...");

    let mut runtime_process = Command::new("node")
        .args(["--experimental-vm-modules", &path])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("Failed to start runtime.js - ensure Node.js is installed");

    // TODO: Implement async test execution
    // The runtime tests require async execution to communicate with the Node.js server
    // For now, we just start the server and exit
    println!("Runtime tests require async execution - not yet implemented in simplified runner");

    let _ = runtime_process.kill();
    let _ = runtime_process.wait();
}
