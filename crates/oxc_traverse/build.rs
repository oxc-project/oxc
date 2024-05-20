use std::{env, process::Command};

fn main() {
    // Re-run if NodeJS build script or AST types change
    println!("cargo:rerun-if-changed=scripts");
    println!("cargo:rerun-if-changed=../oxc_ast/src/ast");

    // Exit if on CI.
    // The built files should be checked into git, so want to run tests etc on what's actually in repo,
    // rather than regenerating them.
    match env::var("CI") {
        Ok(value) if value == "true" => return,
        _ => {}
    }

    // Run NodeJS build script
    let status = Command::new("node")
        .arg("./scripts/build.mjs")
        .status()
        .expect("Failed to run NodeJS build script");
    assert!(status.success(), "Failed to run NodeJS build script");
}
