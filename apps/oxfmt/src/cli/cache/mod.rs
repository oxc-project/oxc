//! `--cache`: remember which files are formatted, so a later run reads and formats only what
//! changed.
//!
//! Invariants:
//! - A run with the cache prints, exits and writes what the same run without it would.
//! - A file is reported unchanged without formatting only under a key covering the build,
//!   its options, its path and its content. A result that called into Node (Prettier, its
//!   plugins, the Tailwind sorter) depends on files besides the one formatted: it isn't kept.
//! - A `stat` is trusted only outside its racy window (see [`oxc_cache::Observation::holds_for`]);
//!   observations are always timed at the start of the run, before anything is read.
//! - The cache never changes an answer by failing: anything unreadable is a miss,
//!   anything unwritable is skipped.

mod files;

use std::path::{Path, PathBuf};

use serde::Serialize;

pub use files::{CachedFile, FileCache, FileCacheRequest};

/// The cache's on-disk format: another version's directory is never read.
const FORMAT: &str = "v1";
/// Run shapes kept, by most recent write: each keeps its last run's files in `files/`.
const MAX_RUNS: usize = 256;

/// What decides which files a run formats, not its output mode or thread count: runs of one
/// shape share their last run's files. Every file's key has its options, so this only keeps
/// runs of different shapes from replacing each other's.
#[derive(Serialize)]
pub struct RunShape<'a> {
    /// The current directory: paths are relative to it.
    pub cwd: &'a Path,
    /// The path arguments.
    pub paths: &'a [PathBuf],
    /// `--config`.
    pub config: Option<&'a Path>,
    /// `--disable-nested-config`.
    pub disable_nested_config: bool,
    /// `--ignore-path`.
    pub ignore_paths: &'a [PathBuf],
    /// `--with-node-modules`.
    pub with_node_modules: bool,
}
