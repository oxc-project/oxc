//! Port of typescript-go's `internal/compiler/filesparser.go`.
//!
//! [`FilesParser`] drives parsing of a set of [`ParseTask`]s and collects the results into the
//! program's file store. tsgo runs the tasks on a work group (parallel unless single-threaded)
//! and recurses into each file's imports/references (`subTasks`); this port is single-threaded
//! and roots-only — it loads each task once, deduplicating by normalized path.

use std::path::PathBuf;

use rustc_hash::FxHashSet;

use super::{
    fileloader::{FileLoader, ProcessedFiles},
    source_file::{SourceFile, SourceFileParseOptions},
};

/// A unit of work: parse one file. Mirrors tsgo's `parseTask`.
pub(super) struct ParseTask {
    /// The file's absolute, normalized name (tsgo `normalizedFilePath`).
    normalized_file_path: PathBuf,
    /// The file's identity key (tsgo `path`), computed in [`FilesParser::start`].
    path: PathBuf,
    /// The parsed file, set by [`ParseTask::load`]. `None` if the file could not be read.
    file: Option<SourceFile>,
    /// Whether this task duplicates an already-seen path and should be skipped when collecting
    /// (tsgo tracks this via `loadedTask`). tsgo's `loaded`/`startedSubTasks` gating arrives
    /// with the deferred subtask recursion.
    duplicate: bool,
}

impl ParseTask {
    pub(super) fn new(normalized_file_path: PathBuf) -> Self {
        Self { normalized_file_path, path: PathBuf::new(), file: None, duplicate: false }
    }

    /// Parse this task's file, mirroring tsgo's `parseTask.load`: read + parse via the host.
    ///
    /// tsgo then discovers the file's imports/references and queues them as `subTasks` (via
    /// `resolveImportsAndModuleAugmentations`); that step is deferred here — we only load roots.
    fn load(&mut self, loader: &FileLoader<'_>) {
        let opts = SourceFileParseOptions {
            file_name: self.normalized_file_path.clone(),
            path: self.path.clone(),
        };
        self.file = loader.host().get_source_file(opts);
        // TODO(next step): discover `self.file`'s imports/references and queue them as subtasks
        // (tsgo `parseTask.load` -> `fileLoader.resolveImportsAndModuleAugmentations`).
    }
}

/// Drives parsing of a set of [`ParseTask`]s, mirroring tsgo's `filesParser`.
#[derive(Default)]
pub(super) struct FilesParser {
    /// Paths already seen, so each file is parsed once (tsgo `taskDataByPath`).
    task_data_by_path: FxHashSet<PathBuf>,
}

impl FilesParser {
    pub(super) fn new() -> Self {
        Self::default()
    }

    /// Load every task, mirroring tsgo's `filesParser.parse` -> `start` (without the work group
    /// or subtask recursion). Tasks are loaded in place and returned for collection.
    pub(super) fn parse(
        &mut self,
        loader: &FileLoader<'_>,
        mut tasks: Vec<ParseTask>,
    ) -> Vec<ParseTask> {
        self.start(loader, &mut tasks);
        tasks
    }

    fn start(&mut self, loader: &FileLoader<'_>, tasks: &mut [ParseTask]) {
        for task in tasks {
            task.path = loader.to_path(&task.normalized_file_path);
            if self.task_data_by_path.insert(task.path.clone()) {
                task.load(loader);
            } else {
                // A file with this path was already loaded; skip it when collecting.
                task.duplicate = true;
            }
        }
    }

    /// Assemble the loaded tasks into the program's file store, mirroring tsgo's
    /// `filesParser.getProcessedFiles` -> `collectFiles`: append files in task order,
    /// deduplicating by path, and recording files that could not be read as missing.
    pub(super) fn get_processed_files(tasks: Vec<ParseTask>) -> ProcessedFiles {
        let mut processed = ProcessedFiles::default();
        Self::collect_files(&mut processed, tasks);
        processed
    }

    fn collect_files(processed: &mut ProcessedFiles, tasks: Vec<ParseTask>) {
        for task in tasks {
            if task.duplicate {
                continue;
            }
            let ParseTask { normalized_file_path, path, file, .. } = task;
            match file {
                Some(file) => {
                    let file_id = processed.files.push(file);
                    processed.files_by_path.insert(path, file_id);
                }
                None => processed.missing_files.push(normalized_file_path),
            }
        }
    }
}
