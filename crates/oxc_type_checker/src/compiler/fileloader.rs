//! Port of typescript-go's `internal/compiler/fileloader.go`.
//!
//! [`FileLoader::process_all_program_files`] is the entry point (tsgo `processAllProgramFiles`):
//! it normalizes and deduplicates the root file names into [`ProcessedFiles`].

use std::path::{Path, PathBuf};

use oxc_index::IndexVec;
use rustc_hash::FxHashMap;

use crate::tspath;

use super::program::FileId;

/// A program's collected root files — the subset of tsgo's `processedFiles` needed before
/// parsing: the ordered file list plus a path -> id lookup.
///
/// tsgo stores `files []*ast.SourceFile` + `filesByPath map[tspath.Path]*ast.SourceFile`; here
/// `files` holds the normalized root paths (parsed source files replace them once parsing lands)
/// and `files_by_path` maps each path to its [`FileId`].
#[derive(Debug, Default)]
pub(super) struct ProcessedFiles {
    /// Root files, in include order (tsgo `files`).
    pub(super) files: IndexVec<FileId, PathBuf>,
    /// Normalized path -> file id (tsgo `filesByPath`).
    pub(super) files_by_path: FxHashMap<PathBuf, FileId>,
}

/// Collects a program's root files, mirroring tsgo's `fileLoader`.
///
/// tsgo's `fileLoader` also parses each file and owns the resolver, supported-extension lists, and
/// lib/project-reference bookkeeping; those arrive with later steps.
pub(super) struct FileLoader<'a> {
    current_directory: &'a Path,
    processed: ProcessedFiles,
}

impl<'a> FileLoader<'a> {
    /// Port of tsgo's `processAllProgramFiles`: collect the root files into the program's file
    /// list, normalizing and deduplicating each path.
    ///
    /// tsgo also parses each file here (and discovers imports, lib files, and project references);
    /// those are later steps.
    pub(super) fn process_all_program_files(
        current_directory: &'a Path,
        root_files: &[PathBuf],
    ) -> ProcessedFiles {
        let mut loader = Self::new(current_directory);
        for root_file in root_files {
            loader.add_root_file(root_file);
        }
        loader.processed
    }

    fn new(current_directory: &'a Path) -> Self {
        Self { current_directory, processed: ProcessedFiles::default() }
    }

    /// tsgo `fileLoader.addRootFileTask`: add a root file, keyed by its normalized absolute path,
    /// skipping a path that was already added.
    fn add_root_file(&mut self, file_name: &Path) {
        let path = tspath::to_path(self.current_directory, file_name);
        if self.processed.files_by_path.contains_key(&path) {
            return;
        }
        let id = self.processed.files.push(path.clone());
        self.processed.files_by_path.insert(path, id);
    }
}
