use std::path::{Path, PathBuf};

mod babel;
mod diff;
mod request;
mod test_file;

pub use crate::{babel::BabelOptions, request::agent, test_file::*};
pub use diff::print_diff_in_terminal;

/// # Panics
/// Invalid Project Root
pub fn project_root() -> PathBuf {
    project_root::get_project_root().unwrap()
}

/// Normalizes the path when on Windows to using forward slash delimiters.
pub fn normalize_path<P: AsRef<Path>>(path: P) -> String {
    path.as_ref().display().to_string().replace('\\', "/")
}
