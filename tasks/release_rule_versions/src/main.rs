#![expect(clippy::print_stdout, clippy::print_stderr)]

use std::{path::PathBuf, process::ExitCode};

use pico_args::Arguments;

use oxc_release_rule_versions::{check_rule_versions, rewrite_rule_versions};
use oxc_tasks_common::project_root;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let mut args = Arguments::from_env();
    let command = args.subcommand().map_err(|error| error.to_string())?;
    let root = args
        .opt_value_from_str::<_, PathBuf>("--root")
        .map_err(|error| error.to_string())?
        .unwrap_or_else(project_root);

    match command.as_deref() {
        Some("rewrite") => {
            let release_version = args
                .value_from_str::<_, String>("--release-version")
                .map_err(|error| error.to_string())?;
            let report = rewrite_rule_versions(&root, &release_version)
                .map_err(|error| error.to_string())?;
            println!("rewrote {} rule versions", report.rewritten_rules.len());
            Ok(())
        }
        Some("check") => {
            let violations = check_rule_versions(&root).map_err(|error| error.to_string())?;
            if violations.is_empty() {
                println!("rule versions are valid");
                return Ok(());
            }

            for violation in violations {
                eprintln!("{violation}");
            }
            Err("stable rules still use version = \"next\"".to_string())
        }
        _ => Err("usage: cargo run -p oxc_release_rule_versions -- <rewrite|check> [options]"
            .to_string()),
    }
}
