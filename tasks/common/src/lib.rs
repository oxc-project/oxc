#![expect(clippy::print_stdout, clippy::disallowed_methods)]
use std::path::{Path, PathBuf};

mod cli;
mod config;
mod diff;
mod fs;
mod http;
mod logging;
mod request;
mod snapshot;
mod test_file;
mod test_runner;

pub use diff::print_diff_in_terminal;

pub use crate::{
    cli::{CommonCliOptions, get_subcommand, parse_common_args},
    config::{ConfigParser, ESLintConfigParser, TestConfigParser},
    fs::{FileOperations, PathOperations, create_safe_file},
    http::{HttpClient, HttpConfig, download_file, download_files},
    logging::{print_success, print_error, print_warning, print_info, print_dim, print_progress, print_file_operation, print_download},
    request::agent,
    snapshot::Snapshot,
    test_file::*,
    test_runner::{TestRunner, TestRunnerOptions, StandardTestOptions, configure_thread_pool},
};

/// # Panics
/// Invalid Project Root
pub fn project_root() -> PathBuf {
    project_root::get_project_root().unwrap()
}

/// Normalizes the path when on Windows to using forward slash delimiters.
pub fn normalize_path<P: AsRef<Path>>(path: P) -> String {
    path.as_ref().to_string_lossy().replace('\\', "/")
}
