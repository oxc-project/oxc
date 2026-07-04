//! Port of typescript-go's `internal/compiler/fileloader.go`.
//!
//! [`process_all_program_files`] is the entry point (tsgo `processAllProgramFiles`): it builds a
//! [`FileLoader`], queues the root files as tasks, parses them, and collects the result into
//! [`ProcessedFiles`].

use std::path::{Path, PathBuf};

use oxc_index::IndexVec;
use rustc_hash::FxHashMap;

use crate::tspath;

use super::{
    filesparser::{FilesParser, ParseTask},
    host::CompilerHost,
    source_file::{FileId, SourceFile},
};

/// The collected files of a program — the subset of tsgo's `processedFiles` needed to hold the
/// parsed root files.
///
/// tsgo stores `files []*ast.SourceFile` + `filesByPath map[tspath.Path]*ast.SourceFile`; here
/// `files` is a typed [`IndexVec`] keyed by [`FileId`] and `files_by_path` maps the normalized
/// path to that id. tsgo's other `processedFiles` tables (resolved modules, metadata, lib files,
/// redirects, …) arrive with later steps.
#[derive(Debug, Default)]
pub(super) struct ProcessedFiles {
    /// Parsed files, in include order (tsgo `files`).
    pub(super) files: IndexVec<FileId, SourceFile>,
    /// Normalized path -> file id (tsgo `filesByPath`).
    pub(super) files_by_path: FxHashMap<PathBuf, FileId>,
    /// Root files that could not be read (tsgo `missingFiles`).
    pub(super) missing_files: Vec<PathBuf>,
}

/// Loads a program's files, mirroring tsgo's `fileLoader`.
pub(super) struct FileLoader<'h> {
    host: &'h CompilerHost,
    root_tasks: Vec<ParseTask>,
}

impl<'h> FileLoader<'h> {
    fn new(host: &'h CompilerHost) -> Self {
        Self { host, root_tasks: Vec::new() }
    }

    /// The host files are read from.
    pub(super) fn host(&self) -> &CompilerHost {
        self.host
    }

    /// tsgo `fileLoader.toPath`: a file's identity key, normalized against the host's current
    /// directory.
    pub(super) fn to_path(&self, file_name: &Path) -> PathBuf {
        tspath::to_path(self.host.current_directory(), file_name)
    }

    /// tsgo `fileLoader.addRootFileTask`: queue a root file for parsing.
    fn add_root_file_task(&mut self, file_name: &Path) {
        let normalized = self.to_path(file_name);
        self.root_tasks.push(ParseTask::new(normalized));
    }
}

/// Port of tsgo's `processAllProgramFiles`: build the loader, queue the root files, parse them,
/// and collect the result.
///
/// Import discovery, lib files, project references, and automatic type directives are deferred —
/// see [`ParseTask::load`](super::filesparser).
pub(super) fn process_all_program_files(
    host: &CompilerHost,
    root_files: &[PathBuf],
) -> ProcessedFiles {
    let mut loader = FileLoader::new(host);
    for root_file in root_files {
        loader.add_root_file_task(root_file);
    }
    let root_tasks = std::mem::take(&mut loader.root_tasks);
    let mut files_parser = FilesParser::new();
    let root_tasks = files_parser.parse(&loader, root_tasks);
    FilesParser::get_processed_files(root_tasks)
}
