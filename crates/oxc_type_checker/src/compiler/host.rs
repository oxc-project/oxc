//! Port of typescript-go's `internal/compiler/host.go`.
//!
//! The [`CompilerHost`] abstracts the environment a program is loaded from — the current
//! directory and the file system. [`CompilerHost::get_source_file`] reads a file's text and
//! parses it, mirroring tsgo's `compilerHost.GetSourceFile`.

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
/// Mirrors tsgo's `CompilerHost` interface / `compilerHost` struct. The file system is an
/// [`oxc_resolver::FileSystem`] (tsgo's `vfs.FS`), held behind `Arc<dyn ..>` so it can be
/// shared and reused by later resolution steps.
pub struct CompilerHost {
    current_directory: PathBuf,
    fs: Arc<dyn FileSystem>,
}

impl CompilerHost {
    /// Create a host rooted at `current_directory`, backed by the OS file system
    /// ([`oxc_resolver::FileSystemOs`], tsgo's `osvfs`).
    pub fn new(current_directory: PathBuf) -> Self {
        Self { current_directory, fs: Arc::new(FileSystemOs) }
    }

    /// Create a host with an explicit file system.
    pub fn with_file_system(current_directory: PathBuf, fs: Arc<dyn FileSystem>) -> Self {
        Self { current_directory, fs }
    }

    /// The directory that relative paths are resolved against (tsgo `GetCurrentDirectory`).
    pub fn current_directory(&self) -> &Path {
        &self.current_directory
    }

    /// Read and parse the file described by `opts`, mirroring tsgo's
    /// `compilerHost.GetSourceFile`: read the text through the file system, then parse it.
    ///
    /// Returns `None` if the file cannot be read (tsgo returns `nil`).
    pub fn get_source_file(&self, opts: SourceFileParseOptions) -> Option<SourceFile> {
        let source_text = self.fs.read_to_string(&opts.file_name).ok()?;
        // Derive the JS/TS dialect from the extension (tsgo's `ScriptKind`/`LanguageVariant`).
        // Extensions oxc's `SourceType` cannot model — notably `.json`, which `get_file_names`
        // includes in the project and tsgo parses as JSON — fall back to the default dialect so
        // the file still appears in the program's file list; faithful JSON parsing (tsgo's
        // `parseJSONText`) and the `isSupportedExtension` gate for truly-unsupported extensions
        // are deferred.
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
