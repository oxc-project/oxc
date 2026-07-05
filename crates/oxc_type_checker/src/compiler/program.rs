//! Port of typescript-go's `internal/compiler/program.go`.

use std::path::{Path, PathBuf};

use oxc_index::{IndexSlice, define_nonmax_u32_index_type};

use super::fileloader::{ProcessedFiles, process_all_program_files};

define_nonmax_u32_index_type! {
    /// Index of a root file within a [`Program`].
    ///
    /// typescript-go has no integer file id — it keys files by their normalized `tspath.Path`.
    /// This typed index is an oxc-side addition so files can be referenced by a cheap `u32` (and,
    /// later, declarations by `(FileId, SymbolId)`).
    pub struct FileId;
}

/// A program's collected root files. Mirrors tsgo's `Program` (which embeds `processedFiles`).
///
/// This is the in-memory model the type checker will run over. Today it holds only the root file
/// list; parsing and binding the files, import resolution, and checking are later steps.
#[derive(Debug)]
pub struct Program {
    processed: ProcessedFiles,
}

impl Program {
    /// Port of tsgo's `NewProgram`: collect `root_files` into the program's file list,
    /// normalizing each path and deduplicating (relative paths resolve against
    /// `current_directory`).
    pub fn new(current_directory: &Path, root_files: &[PathBuf]) -> Self {
        Self { processed: process_all_program_files(current_directory, root_files) }
    }

    /// All root files, in include order (tsgo `Program.SourceFiles`).
    pub fn files(&self) -> &IndexSlice<FileId, [PathBuf]> {
        &self.processed.files
    }

    /// The file with the given [`FileId`].
    pub fn file(&self, id: FileId) -> &Path {
        &self.processed.files[id]
    }

    /// The [`FileId`] for a normalized path, if the program contains it (tsgo
    /// `Program.GetSourceFileByPath`).
    pub fn file_id(&self, path: &Path) -> Option<FileId> {
        self.processed.files_by_path.get(path).copied()
    }

    /// The number of root files.
    pub fn len(&self) -> usize {
        self.processed.files.len()
    }

    /// Whether the program has no root files.
    pub fn is_empty(&self) -> bool {
        self.processed.files.is_empty()
    }
}
