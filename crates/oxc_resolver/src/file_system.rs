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
    /// * See [std::fs::read_to_string]
    fn read_to_string<P: AsRef<Path>>(&self, path: P) -> io::Result<String>;

    /// See [std::fs::metadata]
    ///
    /// # Errors
    ///
    /// See [std::fs::metadata]
    fn metadata<P: AsRef<Path>>(&self, path: P) -> io::Result<FileMetadata>;

    /// See [std::fs::symlink_metadata]
    ///
    /// # Errors
    ///
    /// See [std::fs::symlink_metadata]
    fn symlink_metadata<P: AsRef<Path>>(&self, path: P) -> io::Result<FileMetadata>;

    /// See [std::fs::canonicalize]
    ///
    /// # Errors
    ///
    /// See [std::fs::read_link]
    fn canonicalize<P: AsRef<Path>>(&self, path: P) -> io::Result<PathBuf>;
}

/// Metadata information about a file.
#[derive(Debug, Clone, Copy)]
pub struct FileMetadata {
    pub(crate) is_file: bool,
    pub(crate) is_dir: bool,
    pub(crate) is_symlink: bool,
}

impl FileMetadata {
    pub fn new(is_file: bool, is_dir: bool, is_symlink: bool) -> Self {
        Self { is_file, is_dir, is_symlink }
    }
}

impl From<fs::Metadata> for FileMetadata {
    fn from(metadata: fs::Metadata) -> Self {
        Self::new(metadata.is_file(), metadata.is_dir(), metadata.is_symlink())
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

    fn symlink_metadata<P: AsRef<Path>>(&self, path: P) -> io::Result<FileMetadata> {
        fs::symlink_metadata(path).map(FileMetadata::from)
    }

    fn canonicalize<P: AsRef<Path>>(&self, path: P) -> io::Result<PathBuf> {
        dunce::canonicalize(path)
    }
}
