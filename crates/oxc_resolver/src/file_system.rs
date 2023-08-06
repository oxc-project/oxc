use std::{
    fs, io,
    path::{Path, PathBuf},
};

/// File System abstraction used for `ResolverGeneric`.
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

/// Metadata information about a file.
#[derive(Debug, Clone, Copy)]
pub struct FileMetadata {
    pub(crate) is_file: bool,
    pub(crate) is_dir: bool,
}

impl FileMetadata {
    pub fn new(is_file: bool, is_dir: bool) -> Self {
        Self { is_file, is_dir }
    }
}

impl From<fs::Metadata> for FileMetadata {
    fn from(metadata: fs::Metadata) -> Self {
        Self::new(metadata.is_file(), metadata.is_dir())
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
        fs::metadata(path).map(FileMetadata::from)
    }

    fn canonicalize<P: AsRef<Path>>(&self, path: P) -> io::Result<PathBuf> {
        dunce::canonicalize(path)
    }
}
