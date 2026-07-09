//! Port of typescript-go's `internal/compiler/program.go`.

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use oxc_index::{IndexSlice, define_nonmax_u32_index_type};

use crate::tsoptions::ParsedConfig;

use super::{
    fileloader::{FileLoader, ProcessedFiles},
    source_file::SourceFile,
};

define_nonmax_u32_index_type! {
    /// Index of a [`SourceFile`] within a [`Program`].
    ///
    /// typescript-go has no integer file id — it keys files by their normalized `tspath.Path`.
    /// This typed index is an oxc-side addition so files can be referenced by a cheap `u32` (and,
    /// later, declarations by `(FileId, SymbolId)`).
    pub struct FileId;
}

/// The inputs a [`Program`] is created from, mirroring tsgo's `ProgramOptions`.
///
/// tsgo's `Host` is created internally from `current_directory`; `config` stands in for tsgo's
/// `Config` when a tsconfig drives the compilation.
#[derive(Debug)]
pub struct ProgramOptions {
    /// The directory relative paths resolve against (tsgo `Host.GetCurrentDirectory`).
    pub current_directory: PathBuf,
    /// The root files to load (tsgo `Config.FileNames()`).
    pub root_files: Vec<PathBuf>,
    /// The parsed project tsconfig (tsgo `Config`), driving module resolution
    /// (`paths`/`baseUrl`) and the compiler-option gates. `None` when files are compiled
    /// directly without a config file.
    pub config: Option<Arc<ParsedConfig>>,
}

/// A program: the source files loaded from a set of root files (roots + their transitive imports).
/// Mirrors tsgo's `Program` (which embeds `processedFiles`).
///
/// This is the in-memory model the type checker will run over. Files are parsed (AST + module
/// record) but not yet bound; type checking is a later step.
#[derive(Debug)]
pub struct Program {
    opts: ProgramOptions,
    processed: ProcessedFiles,
}

impl Program {
    /// Port of tsgo's `NewProgram`: parse the root files, follow their imports to load every
    /// dependent file (in parallel), and collect them.
    pub fn new(opts: ProgramOptions) -> Self {
        let processed = FileLoader::process_all_program_files(&opts);
        Self { opts, processed }
    }

    /// The options the program was created from (tsgo `Program.Options`).
    pub fn options(&self) -> &ProgramOptions {
        &self.opts
    }

    /// All source files in program order — a post-order walk of the import graph from the root
    /// files, so dependencies come before their importers (tsgo `Program.SourceFiles`).
    pub fn files(&self) -> &IndexSlice<FileId, [SourceFile]> {
        &self.processed.files
    }

    /// The source file with the given [`FileId`].
    pub fn file(&self, id: FileId) -> &SourceFile {
        &self.processed.files[id]
    }

    /// The [`FileId`] for a normalized path, if the program contains it (tsgo
    /// `Program.GetSourceFileByPath`).
    pub fn file_id(&self, path: &Path) -> Option<FileId> {
        self.processed.files_by_path.get(path).copied()
    }

    /// The file `specifier` resolves to from within `id`, if it was loaded (tsgo
    /// `Program.GetResolvedModule`).
    pub fn resolved_module(&self, id: FileId, specifier: &str) -> Option<FileId> {
        self.processed.resolved_modules[id].get(specifier).copied()
    }

    /// Paths that could not be loaded — unreadable files and files with unsupported extensions
    /// (tsgo `missingFiles`).
    pub fn missing_files(&self) -> &[PathBuf] {
        &self.processed.missing_files
    }

    /// The number of source files.
    pub fn len(&self) -> usize {
        self.processed.files.len()
    }

    /// Whether the program has no source files.
    pub fn is_empty(&self) -> bool {
        self.processed.files.is_empty()
    }
}
