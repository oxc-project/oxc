use std::{
    fs, io,
    path::{Path, PathBuf},
};

/// File System abstraction used for `ResolverGeneric`.
pub trait FileSystem: Send + Sync {
    /// See [std::fs::read_to_string]
    ///
    /// # Errors
    ///
    /// * See [std::fs::read_to_string]
    /// ## Warning
    /// Use `&Path` instead of a generic `P: AsRef<Path>` here,
    /// because object safety requirements, it is especially useful, when
    /// you want to store multiple `dyn FileSystem` in a `Vec` or use a `ResolverGeneric<Fs>` in
    /// napi env.
    fn read_to_string(&self, path: &Path) -> io::Result<String>;

    /// See [std::fs::metadata]
    ///
    /// # Errors
    /// See [std::fs::metadata]
    /// ## Warning
    /// Use `&Path` instead of a generic `P: AsRef<Path>` here,
    /// because object safety requirements, it is especially useful, when
    /// you want to store multiple `dyn FileSystem` in a `Vec` or use a `ResolverGeneric<Fs>` in
    /// napi env.
    fn metadata(&self, path: &Path) -> io::Result<FileMetadata>;

    /// See [std::fs::symlink_metadata]
    ///
    /// # Errors
    ///
    /// See [std::fs::symlink_metadata]
    /// ## Warning
    /// Use `&Path` instead of a generic `P: AsRef<Path>` here,
    /// because object safety requirements, it is especially useful, when
    /// you want to store multiple `dyn FileSystem` in a `Vec` or use a `ResolverGeneric<Fs>` in
    /// napi env.
    fn symlink_metadata(&self, path: &Path) -> io::Result<FileMetadata>;

    /// See [std::fs::canonicalize]
    ///
    /// # Errors
    ///
    /// See [std::fs::read_link]
    /// ## Warning
    /// Use `&Path` instead of a generic `P: AsRef<Path>` here,
    /// because object safety requirements, it is especially useful, when
    /// you want to store multiple `dyn FileSystem` in a `Vec` or use a `ResolverGeneric<Fs>` in
    /// napi env.
    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf>;
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
    fn read_to_string(&self, path: &Path) -> io::Result<String> {
        fs::read_to_string(path)
    }

    fn metadata(&self, path: &Path) -> io::Result<FileMetadata> {
        fs::metadata(path).map(FileMetadata::from)
    }

    fn symlink_metadata(&self, path: &Path) -> io::Result<FileMetadata> {
        fs::symlink_metadata(path).map(FileMetadata::from)
    }

    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf> {
        dunce::canonicalize(path)
    }
}
