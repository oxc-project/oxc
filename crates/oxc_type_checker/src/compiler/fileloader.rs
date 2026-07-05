//! Port of typescript-go's `internal/compiler/fileloader.go`.
//!
//! [`process_all_program_files`] is the entry point (tsgo `processAllProgramFiles`): a
//! [`FileLoader`] turns the root file names into [`ParseTask`]s, then hands them to a
//! [`FilesParser`] which parses and collects them into [`ProcessedFiles`].

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

/// Turns a program's root file names into [`ParseTask`]s, mirroring tsgo's `fileLoader`.
///
/// tsgo's `fileLoader` also owns the resolver, supported-extension lists, and lib/project-reference
/// bookkeeping; those arrive with later steps.
pub(super) struct FileLoader<'h> {
    host: &'h CompilerHost,
    root_tasks: Vec<ParseTask>,
}

impl<'h> FileLoader<'h> {
    fn new(host: &'h CompilerHost) -> Self {
        Self { host, root_tasks: Vec::new() }
    }

    /// tsgo `processAllProgramFiles`: queue every root file as a task, then parse and collect
    /// them into the program's file store.
    fn load(mut self, root_files: &[PathBuf]) -> ProcessedFiles {
        for root_file in root_files {
            self.add_root_file_task(root_file);
        }
        FilesParser::default().run(self.host, self.root_tasks)
    }

    /// tsgo `fileLoader.addRootFileTask`: queue a root file for parsing, keyed by its normalized
    /// absolute path.
    fn add_root_file_task(&mut self, file_name: &Path) {
        let normalized = tspath::to_path(self.host.current_directory(), file_name);
        self.root_tasks.push(ParseTask::new(normalized));
    }
}

/// Port of tsgo's `processAllProgramFiles`.
///
/// Import discovery, lib files, project references, and automatic type directives are deferred —
/// see [`ParseTask::load`](super::filesparser).
pub(super) fn process_all_program_files(
    host: &CompilerHost,
    root_files: &[PathBuf],
) -> ProcessedFiles {
    FileLoader::new(host).load(root_files)
}
