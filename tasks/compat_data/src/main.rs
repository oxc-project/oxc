#![allow(clippy::print_stdout)]
use std::process::Command;

use oxc_compat_data::generate;
use oxc_tasks_common::project_root;

fn main() {
    let cwd = project_root().join("tasks/compat_data");

    if !cwd.join("compat-table").exists() {
        println!("Cloning compat-table ...");
        Command::new("pnpm").current_dir(&cwd).args(["run", "init"]).output().unwrap();
    }

    let output = Command::new("pnpm").current_dir(cwd).args(["run", "build"]).output().unwrap();
    if !output.status.success() {
        println!("{}", String::from_utf8(output.stderr).unwrap());
    }

    generate();
}
