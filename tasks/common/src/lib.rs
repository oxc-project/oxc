use std::{env, path::PathBuf};

mod test_file;

pub use self::test_file::*;

/// # Panics
///
/// * Invalid current_dir
pub fn project_root() -> PathBuf {
    project_root::get_project_root().unwrap_or_else(|e| {
        eprintln!("Using current working directory because {e}");
        env::current_dir().unwrap()
    })
}
