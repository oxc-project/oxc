//! Port of typescript-go's `internal/compiler/host.go`.
//!
//! The [`CompilerHost`] abstracts the current directory and the file system.
//! [`CompilerHost::get_source_file`] reads a file's text and parses it (tsgo
//! `compilerHost.GetSourceFile`). It is `Send + Sync` so rayon workers can share one by `&`.

use std::{
    fmt,
    path::{Path, PathBuf},
    sync::Arc,
};

use oxc_resolver::{FileSystem, FileSystemOs};
use oxc_span::SourceType;

use super::source_file::{SourceFile, SourceFileParseOptions};

/// The host a [`Program`](super::program::Program) is loaded from.
///
/// Mirrors tsgo's `CompilerHost`. The file system is an [`oxc_resolver::FileSystem`] (tsgo's
/// `vfs.FS`) behind `Arc<dyn ..>`.
pub struct CompilerHost {
    current_directory: PathBuf,
    fs: Arc<dyn FileSystem>,
}

impl CompilerHost {
    /// Create a host rooted at `current_directory`, backed by the OS file system.
    pub fn new(current_directory: PathBuf) -> Self {
        Self { current_directory, fs: Arc::new(FileSystemOs) }
    }

    /// The directory relative paths are resolved against (tsgo `GetCurrentDirectory`).
    pub fn current_directory(&self) -> &Path {
        &self.current_directory
    }

    /// Whether `path` is an existing regular file (tsgo `host.FS().FileExists`).
    pub fn file_exists(&self, path: &Path) -> bool {
        self.fs.metadata(path).is_ok_and(oxc_resolver::FileMetadata::is_file)
    }

    /// Read and parse the file described by `opts`, mirroring tsgo's `compilerHost.GetSourceFile`:
    /// read the text through the file system, then parse it.
    ///
    /// Returns `None` if the file cannot be read (tsgo returns `nil`).
    pub fn get_source_file(&self, opts: SourceFileParseOptions) -> Option<SourceFile> {
        let source_text = self.fs.read_to_string(&opts.file_name).ok()?;
        // Derive the JS/TS dialect from the extension. Extensions oxc's `SourceType` cannot model
        // (notably `.json`) fall back to the default so the file still enters the program; faithful
        // JSON handling is deferred.
        let source_type = SourceType::from_path(&opts.file_name).unwrap_or_default();
        Some(SourceFile::parse(opts, source_text, source_type))
    }
}

impl fmt::Debug for CompilerHost {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CompilerHost")
            .field("current_directory", &self.current_directory)
            .finish_non_exhaustive()
    }
}
