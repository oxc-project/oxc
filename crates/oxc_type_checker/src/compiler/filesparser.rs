//! Port of typescript-go's `internal/compiler/filesparser.go`.
//!
//! [`FilesParser::parse`] loads all root files and their transitive imports, mirroring tsgo's
//! `filesParser` walk — but driven by rayon instead of a work group (per the request). A single
//! graph thread owns the dedup set and drains results; rayon workers do the parse + import
//! resolution. Dedup is therefore lock-free (single-threaded), following `oxc_linter`'s runtime.
//!
//! [`FilesParser::get_processed_files`] then assigns [`FileId`](super::FileId)s in tsgo's
//! `collectFiles` order — a post-order walk of the task graph from the roots, so a file's
//! dependencies come before it — which also keeps the output deterministic despite rayon's
//! nondeterministic arrival order.

use std::{
    any::Any,
    panic::{self, AssertUnwindSafe},
    path::PathBuf,
    sync::mpsc,
};

use rustc_hash::{FxHashMap, FxHashSet};

use crate::tspath::has_extension;

use super::{
    fileloader::{FileLoader, ProcessedFiles, SubTask},
    source_file::{SourceFile, SourceFileParseOptions},
};

/// A unit of work: load one file. Mirrors tsgo's `parseTask` (pre-load state).
struct ParseTask {
    path: PathBuf,
}

impl ParseTask {
    /// tsgo `parseTask.load`: gate unsupported extensions, read + parse the file, then resolve
    /// its imports and module augmentations into sub-tasks.
    fn load(self, loader: &FileLoader) -> LoadedTask {
        // tsgo drops files with unsupported extensions before parsing (they still count as
        // missing); extensionless files are loaded as-is.
        let file_name = self.path.to_string_lossy();
        if has_extension(&file_name) && !loader.is_supported_extension(&file_name) {
            return LoadedTask { path: self.path, source_file: None, sub_tasks: Vec::new() };
        }

        let opts = SourceFileParseOptions { file_name: self.path.clone(), path: self.path.clone() };
        match loader.host().get_source_file(opts) {
            Some(source_file) => {
                let sub_tasks = loader.resolve_references(&source_file);
                LoadedTask { path: self.path, source_file: Some(source_file), sub_tasks }
            }
            None => LoadedTask { path: self.path, source_file: None, sub_tasks: Vec::new() },
        }
    }
}

/// A loaded [`ParseTask`]: the parsed [`SourceFile`] (`None` if unreadable or unsupported) and
/// the file's resolved references in resolution order — tsgo's `subTasks` and `resolutionsInFile`
/// combined into one [`SubTask`] list.
struct LoadedTask {
    path: PathBuf,
    source_file: Option<SourceFile>,
    sub_tasks: Vec<SubTask>,
}

/// What a worker reports back: a loaded task, or the payload of a panic that occurred while
/// loading it — so the graph thread can re-raise the panic rather than hang.
enum WorkerResult {
    Loaded(Box<LoadedTask>),
    Panicked(Box<dyn Any + Send>),
}

/// Drives parallel loading and collects the result, mirroring tsgo's `filesParser`.
#[derive(Default)]
pub(super) struct FilesParser {
    /// Every loaded task, keyed by normalized path (tsgo `taskDataByPath`).
    tasks_by_path: FxHashMap<PathBuf, LoadedTask>,
    /// The normalized root paths, in root order (tsgo walks `rootTasks`).
    root_paths: Vec<PathBuf>,
}

impl FilesParser {
    /// Load `root_files` and every file reachable through their imports, in parallel
    /// (tsgo `filesParser.parse`).
    pub(super) fn parse(&mut self, loader: &FileLoader, root_files: &[PathBuf]) {
        rayon::scope(|scope| {
            let (tx, rx) = mpsc::channel::<WorkerResult>();
            // `encountered` dedups by normalized path; owned solely by this (graph) thread, so no
            // locking. `pending` counts spawned-but-not-yet-collected tasks; every worker reports
            // back exactly once (even on panic), so it always reaches 0.
            let mut encountered = FxHashSet::<PathBuf>::default();
            let mut pending = 0usize;

            // Seed the roots.
            for root in root_files {
                let path = loader.to_path(root);
                self.root_paths.push(path.clone());
                if encountered.insert(path.clone()) {
                    pending += 1;
                    spawn_load(scope, loader, path, tx.clone());
                }
            }

            // Drain results, enqueuing newly-discovered dependencies. While the channel is empty
            // the graph thread donates itself to the pool via `yield_now` instead of idling.
            while pending > 0 {
                let result = match rx.try_recv() {
                    Ok(result) => result,
                    Err(mpsc::TryRecvError::Empty) => {
                        rayon::yield_now();
                        continue;
                    }
                    Err(mpsc::TryRecvError::Disconnected) => break,
                };
                pending -= 1;
                let task = match result {
                    WorkerResult::Loaded(task) => *task,
                    // A worker panicked (e.g. a parser bug on some file). Re-raise it here rather
                    // than spinning the drain loop forever or silently dropping the file.
                    WorkerResult::Panicked(payload) => panic::resume_unwind(payload),
                };
                for (_specifier, dep_path) in &task.sub_tasks {
                    if encountered.insert(dep_path.clone()) {
                        pending += 1;
                        spawn_load(scope, loader, dep_path.clone(), tx.clone());
                    }
                }
                self.tasks_by_path.insert(task.path.clone(), task);
            }
        });
    }

    /// tsgo `filesParser.getProcessedFiles`: assign [`FileId`](super::FileId)s by walking the
    /// task graph from the roots in post-order (`collectFiles` collects a task's sub-tasks
    /// before the task itself), then link each file's resolutions to the ids.
    pub(super) fn get_processed_files(&mut self) -> ProcessedFiles {
        /// A task whose sub-tasks are being walked; the task itself is collected once the
        /// cursor passes the last one.
        struct Frame {
            task: LoadedTask,
            next_sub_task: usize,
        }

        let mut processed = ProcessedFiles::default();
        let mut sub_tasks_by_id: Vec<Vec<SubTask>> = Vec::new();
        let mut stack: Vec<Frame> = Vec::new();
        // The walk visits every encountered path exactly once: `tasks_by_path.remove` marks a
        // task visited, and paths without a task (already taken) are skipped. Iterative rather
        // than recursive (as in tsgo) so a deep import chain cannot overflow the stack.
        for root in std::mem::take(&mut self.root_paths) {
            if let Some(task) = self.tasks_by_path.remove(&root) {
                stack.push(Frame { task, next_sub_task: 0 });
            }
            while let Some(frame) = stack.last_mut() {
                if let Some((_, dep_path)) = frame.task.sub_tasks.get(frame.next_sub_task) {
                    let dep_path = dep_path.clone();
                    frame.next_sub_task += 1;
                    if let Some(task) = self.tasks_by_path.remove(&dep_path) {
                        stack.push(Frame { task, next_sub_task: 0 });
                    }
                } else {
                    let task = stack.pop().unwrap().task;
                    match task.source_file {
                        Some(source_file) => {
                            let id = processed.files.push(source_file);
                            processed.files_by_path.insert(task.path, id);
                            sub_tasks_by_id.push(task.sub_tasks);
                        }
                        None => processed.missing_files.push(task.path),
                    }
                }
            }
        }

        // Link the module-graph edges: import specifier -> dependency FileId (dropping
        // dependencies that failed to load; triple-slash references have no specifier).
        processed.resolved_modules = sub_tasks_by_id
            .into_iter()
            .map(|sub_tasks| {
                sub_tasks
                    .into_iter()
                    .filter_map(|(specifier, dep_path)| {
                        let specifier = specifier?;
                        processed.files_by_path.get(&dep_path).map(|&id| (specifier, id))
                    })
                    .collect()
            })
            .collect();
        processed
    }
}

/// Spawn a rayon worker that loads `path` and reports back to the graph thread exactly once — the
/// loaded task, or the panic payload if loading panicked. Reporting on panic (instead of just
/// unwinding the worker) keeps `pending` accurate and lets the graph thread re-raise the panic,
/// rather than the drain loop hanging on a task that never reports.
fn spawn_load<'scope>(
    scope: &rayon::Scope<'scope>,
    loader: &'scope FileLoader,
    path: PathBuf,
    tx: mpsc::Sender<WorkerResult>,
) {
    scope.spawn(move |_| {
        let result = match panic::catch_unwind(AssertUnwindSafe(|| ParseTask { path }.load(loader)))
        {
            Ok(task) => WorkerResult::Loaded(Box::new(task)),
            Err(payload) => WorkerResult::Panicked(payload),
        };
        // The receiver is only gone once the graph thread has stopped draining (it is finishing, or
        // itself unwinding a re-raised panic), so a failed send here can be dropped.
        let _ = tx.send(result);
    });
}
