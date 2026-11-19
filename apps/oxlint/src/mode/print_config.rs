use oxc_linter::ConfigStore;

use crate::{cli::CliRunResult, lint::print_and_flush_stdout};

pub fn run_print_config(
    config_store: &ConfigStore,
    path: Option<&Path>,
    stdout: &mut dyn std::io::Write,
) -> CliRunResult {
    let config_file = match path {
        Some(path) => config_store.to_config_file_for_path(path),
        None => config_store.to_config_file(),
    };
    print_and_flush_stdout(stdout, &config_file);
    print_and_flush_stdout(stdout, "\n");

    CliRunResult::PrintConfigResult
}
use std::path::Path;
