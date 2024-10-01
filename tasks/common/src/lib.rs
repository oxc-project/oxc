#![allow(clippy::print_stdout, clippy::disallowed_methods)]
use std::path::{Path, PathBuf};

mod diff;
mod request;
mod snapshot;
mod test_file;

pub use diff::print_diff_in_terminal;

pub use crate::{request::agent, snapshot::Snapshot, test_file::*};

/// # Panics
/// Invalid Project Root
pub fn project_root() -> PathBuf {
    project_root::get_project_root().unwrap()
}

/// Normalizes the path when on Windows to using forward slash delimiters.
pub fn normalize_path<P: AsRef<Path>>(path: P) -> String {
    path.as_ref().to_string_lossy().replace('\\', "/")
}
