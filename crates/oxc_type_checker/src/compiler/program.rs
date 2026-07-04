//! Port of typescript-go's `internal/compiler/program.go`.

use std::path::{Path, PathBuf};

use oxc_index::IndexSlice;

use super::{
    fileloader::{ProcessedFiles, process_all_program_files},
    host::CompilerHost,
    source_file::{FileId, SourceFile},
};

/// A program: the source files collected from a set of root files, plus the host they were
/// loaded from. Mirrors tsgo's `Program` (which embeds `processedFiles`).
///
/// This is the in-memory model the type checker will run over. Today it holds only the parsed +
/// bound root files; import resolution, lib files, and checking are later steps.
#[derive(Debug)]
pub struct Program {
    host: CompilerHost,
    processed: ProcessedFiles,
}

impl Program {
    /// Port of tsgo's `NewProgram`: read, parse, and bind each of `root_files`, collecting them
    /// into the program's file store.
    pub fn new(host: CompilerHost, root_files: &[PathBuf]) -> Self {
        let processed = process_all_program_files(&host, root_files);
        Self { host, processed }
    }

    /// The host the program was loaded from.
    pub fn host(&self) -> &CompilerHost {
        &self.host
    }

    /// All source files, in include order (tsgo `Program.SourceFiles`).
    pub fn source_files(&self) -> &IndexSlice<FileId, [SourceFile]> {
        &self.processed.files
    }

    /// The source file with the given [`FileId`].
    pub fn source_file(&self, id: FileId) -> &SourceFile {
        &self.processed.files[id]
    }

    /// The source file with the given normalized path (tsgo `Program.GetSourceFileByPath`).
    pub fn get_source_file_by_path(&self, path: &Path) -> Option<&SourceFile> {
        self.processed.files_by_path.get(path).map(|&id| &self.processed.files[id])
    }

    /// The number of source files.
    pub fn len(&self) -> usize {
        self.processed.files.len()
    }

    /// Whether the program has no source files.
    pub fn is_empty(&self) -> bool {
        self.processed.files.is_empty()
    }

    /// Root files that could not be read (tsgo `missingFiles`).
    pub fn missing_files(&self) -> &[PathBuf] {
        &self.processed.missing_files
    }
}
