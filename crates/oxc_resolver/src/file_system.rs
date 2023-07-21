use std::{
    fs, io,
    path::{Path, PathBuf},
};

pub trait FileSystem: Default + Send + Sync {
    /// See [std::fs::read_to_string]
    ///
    /// # Errors
    ///
    /// * Any [io::Error]
    fn read_to_string<P: AsRef<Path>>(&self, path: P) -> io::Result<String>;

    /// See [std::fs::metadata]
    ///
    /// # Errors
    ///
    /// This function will return an error in the following situations, but is not
    /// limited to just these cases:
    ///
    /// * The user lacks permissions to perform `metadata` call on `path`.
    /// * `path` does not exist.
    fn metadata<P: AsRef<Path>>(&self, path: P) -> io::Result<FileMetadata>;

    /// See [std::fs::canonicalize]
    ///
    /// # Errors
    ///
    /// This function will return an error in the following situations, but is not
    /// limited to just these cases:
    ///
    /// * `path` does not exist.
    /// * A non-final component in path is not a directory.
    fn canonicalize<P: AsRef<Path>>(&self, path: P) -> io::Result<PathBuf>;
}

#[derive(Debug, Clone, Copy)]
pub struct FileMetadata {
    pub(crate) is_file: bool,
}

impl FileMetadata {
    pub fn new(is_file: bool) -> Self {
        Self { is_file }
    }
}

/// Operating System
#[derive(Default)]
pub struct FileSystemOs;

impl FileSystem for FileSystemOs {
    fn read_to_string<P: AsRef<Path>>(&self, path: P) -> io::Result<String> {
        fs::read_to_string(path)
    }

    fn metadata<P: AsRef<Path>>(&self, path: P) -> io::Result<FileMetadata> {
        fs::metadata(path).map(|metadata| FileMetadata { is_file: metadata.is_file() })
    }

    fn canonicalize<P: AsRef<Path>>(&self, path: P) -> io::Result<PathBuf> {
        dunce::canonicalize(path)
    }
}
