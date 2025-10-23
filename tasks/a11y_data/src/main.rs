#![expect(clippy::print_stdout)]
use std::process::Command;

use oxc_a11y_data::generate;
use oxc_tasks_common::project_root;

pub fn main() {
    let cwd = project_root().join("tasks/a11y_data");

    if !cwd.join("abstractRoles.json").exists()
        || !cwd.join("interactiveElementRoleSchemas.json").exists()
        || !cwd.join("interactiveRoles.json").exists()
        || !cwd.join("noninteractiveAxObjectSchema.json").exists()
        || !cwd.join("noninteractiveElementRoleSchemas.json").exists()
        || !cwd.join("noninteractiveRoles.json").exists() {
        println!("Generating accessibility data...");
        Command::new("pnpm").current_dir(&cwd).args(["run", "init"]).output().unwrap();
    }

    generate();
}
