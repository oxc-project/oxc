//! The files known formatted, shared by the formatting threads: a [`Cache`] keyed by the build
//! and each file's options.

use std::{
    cell::RefCell,
    path::{Path, PathBuf},
    sync::{Arc, Weak},
};

use serde::{Deserialize, Serialize};

use oxc_cache::{BuildId, Cache, CacheDir, Digest, Stat, Timestamp};

use super::{FORMAT, MAX_RUNS, RunShape};
use crate::core::{FormatPlan, oxfmtrc::FormatConfig, utils};

/// What the cache keeps: a file is formatted. An unformatted file isn't put: it's formatted
/// again, as printing or writing it needs anyway.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Formatted;

/// The files known formatted, for one run with `--cache`.
pub struct FileCache {
    /// The cache directory: `files/` for each run shape's files, `store`, `builds`.
    dir: Arc<CacheDir>,
    /// The current directory: files are kept by paths relative to it.
    cwd: PathBuf,
    /// The running oxfmt: its version and its binary.
    build: Digest,
    /// The files known formatted.
    cache: Cache<Formatted>,
}

/// A file the format service is about to read.
pub struct CachedFile<'a> {
    /// The run's cache.
    files: &'a FileCache,
    /// Its path, relative to the current directory.
    path: &'a Path,
    /// The build and its options.
    key: Digest,
    /// Its `stat` when the format service received it, before reading it.
    stat: Stat,
}

/// A cache to open for a run, decided before anything is read.
pub struct FileCacheRequest {
    /// When the run started: every `stat` it records counts as observed then.
    started: Timestamp,
    /// The cache directory, `node_modules/.cache/oxfmt` in the current directory by default.
    root: PathBuf,
    /// The current directory.
    cwd: PathBuf,
    /// The run's shape, naming the file its last run's files are kept in.
    shape: Digest,
}

impl FileCacheRequest {
    /// The cache at `location` for a run of `shape`, starting now. `None` if the shape can't be
    /// keyed (a path that isn't UTF-8): the run is then uncached.
    pub fn new(location: Option<&Path>, shape: &RunShape<'_>) -> Option<Self> {
        // Taken before anything is read: every stat recorded by this run counts as observed now.
        let started = Timestamp::now();
        let root = match location {
            Some(location) => utils::normalize_relative_path(shape.cwd, location),
            None => shape.cwd.join("node_modules/.cache/oxfmt"),
        };
        Some(Self { started, root, cwd: shape.cwd.to_path_buf(), shape: Digest::try_of(shape)? })
    }

    /// Opens the cache: reads the last run's files. Done off the main thread, while the configs
    /// load. `None` if the build can't be identified or the directory can't be created: the
    /// run is then uncached.
    pub fn open(self) -> Option<Arc<FileCache>> {
        let dir = Arc::new(CacheDir::open(&self.root.join(FORMAT))?);
        let build = Digest::of(&(env!("CARGO_PKG_VERSION"), BuildId::current(&dir, "builds")?));
        let name = format!("files/{}", Digest::of(&(build, self.shape)));
        let cache = Cache::open(Arc::clone(&dir), name).started_at(self.started);
        Some(Arc::new(FileCache { dir, cwd: self.cwd, build, cache }))
    }
}

impl FileCache {
    /// The file at `path`; `None` if it's gone or its path isn't UTF-8 (such a file is never
    /// cached).
    pub fn lookup<'a>(&'a self, path: &'a Path, plan: &FormatPlan) -> Option<CachedFile<'a>> {
        let rel = path.strip_prefix(&self.cwd).unwrap_or(path);
        rel.to_str()?;
        let stat = Stat::of_path(path)?;
        Some(CachedFile { files: self, path: rel, key: self.key(plan), stat })
    }

    /// What a file's verdict depends on besides its path and content: the build, the strategy
    /// and the options.
    fn key(&self, plan: &FormatPlan) -> Digest {
        thread_local! {
            // The last config's digest, by its address: files in a directory share one. `Weak`
            // holds the allocation, so another config can't take the address.
            static LAST: RefCell<(Weak<FormatConfig>, Digest)> =
                RefCell::new((Weak::new(), Digest::of_bytes(&[])));
        }
        let config = LAST.with_borrow_mut(|(last, digest)| {
            if !std::ptr::eq(last.as_ptr(), Arc::as_ptr(&plan.config)) {
                *last = Arc::downgrade(&plan.config);
                *digest = Digest::of(&*plan.config);
            }
            *digest
        });
        let strategy = format!("{:?}", plan.strategy);
        Digest::of_bytes(&[self.build.as_ref(), config.as_ref(), strategy.as_bytes()].concat())
    }

    /// Saves the files known formatted. Called once the format service is done.
    pub fn save(&self) {
        self.cache.save();
        self.dir.prune("files", MAX_RUNS);
    }
}

impl CachedFile<'_> {
    /// Whether the file is known formatted from its `stat` alone.
    pub fn formatted_by_stat(&self) -> bool {
        self.files.cache.get_by_stat(self.path, self.key, self.stat).is_some()
    }

    /// Whether the file, read and hashing to `content`, is known formatted: by the last run,
    /// or by any run of another shape.
    pub fn formatted_by_content(&self, content: Digest) -> bool {
        self.files.cache.get_by_content(self.path, self.key, self.stat, content).is_some()
    }

    /// Record that the file, hashing to `content`, is formatted.
    pub fn formatted(&self, content: Digest) {
        self.files.cache.put(self.path, self.key, self.stat, content, Formatted);
    }
}
