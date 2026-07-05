//! Port of typescript-go's `internal/compiler/filesparser.go`.
//!
//! [`FilesParser::run`] parses a set of [`ParseTask`]s and collects them into the program's file
//! store. tsgo runs the tasks on a work group (parallel unless single-threaded) and recurses into
//! each file's imports/references (`subTasks`); this port is single-threaded and roots-only — it
//! loads each task once, deduplicating by normalized path.

use std::path::PathBuf;

use rustc_hash::FxHashSet;

use crate::tspath;

use super::{
    fileloader::ProcessedFiles,
    host::CompilerHost,
    source_file::{SourceFile, SourceFileParseOptions},
};

/// A unit of work: parse one file. Mirrors tsgo's `parseTask`.
pub(super) struct ParseTask {
    /// The file's absolute, normalized name (tsgo `normalizedFilePath`).
    normalized_file_path: PathBuf,
    /// The file's identity key (tsgo `path`), assigned in [`FilesParser::parse`]. Distinct from
    /// `normalized_file_path` once case canonicalization lands; equal until then.
    path: PathBuf,
    /// The parsed file, set by [`ParseTask::load`]. `None` if the file could not be read.
    file: Option<SourceFile>,
    /// Whether this task duplicates an already-seen path and should be skipped when collecting
    /// (tsgo tracks this via `loadedTask`).
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
    fn load(&mut self, host: &CompilerHost) {
        let opts = SourceFileParseOptions {
            file_name: self.normalized_file_path.clone(),
            path: self.path.clone(),
        };
        self.file = host.get_source_file(opts);
        // TODO(next step): discover `self.file`'s imports/references and queue them as subtasks
        // (tsgo `parseTask.load` -> `fileLoader.resolveImportsAndModuleAugmentations`).
    }
}

/// Parses and collects [`ParseTask`]s, mirroring tsgo's `filesParser`.
#[derive(Default)]
pub(super) struct FilesParser {
    /// Paths already loaded, so each file is parsed once (tsgo `taskDataByPath`).
    task_data_by_path: FxHashSet<PathBuf>,
}

impl FilesParser {
    /// tsgo `filesParser.parse` + `getProcessedFiles`: load every task once (deduplicating by
    /// path), then gather the results into [`ProcessedFiles`].
    pub(super) fn run(mut self, host: &CompilerHost, mut tasks: Vec<ParseTask>) -> ProcessedFiles {
        self.parse(host, &mut tasks);
        Self::collect(tasks)
    }

    /// tsgo `filesParser.start`: give each task its identity key, then load it — unless a task
    /// with the same key was already loaded, in which case mark it a duplicate.
    fn parse(&mut self, host: &CompilerHost, tasks: &mut [ParseTask]) {
        for task in tasks {
            task.path = tspath::to_path(host.current_directory(), &task.normalized_file_path);
            if self.task_data_by_path.insert(task.path.clone()) {
                task.load(host);
            } else {
                task.duplicate = true;
            }
        }
    }

    /// tsgo `filesParser.collectFiles`: append files in task order, skipping duplicates and
    /// recording files that could not be read as missing.
    fn collect(tasks: Vec<ParseTask>) -> ProcessedFiles {
        let mut processed = ProcessedFiles::default();
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
        processed
    }
}
