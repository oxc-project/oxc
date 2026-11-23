//! A cache of per-file results: a file whose `stat` is unchanged keeps its last result without
//! being read ([`Strategy::Metadata`]); one that changed, or every file with
//! [`Strategy::Content`], is read and keeps a result put for the same content, by the last run
//! or by any run of any cache in the same [`CacheDir`].
//!
//! A result is kept under a key the caller makes of everything it depends on besides the file's
//! path and content: the tool's build and options. A result that can't be keyed so isn't put.

use std::{
    fmt,
    path::Path,
    sync::{
        Arc, Mutex, MutexGuard, OnceLock,
        atomic::{AtomicBool, Ordering},
    },
};

use hashbrown::HashTable;
use rustc_hash::{FxBuildHasher, FxHashSet};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::{
    digest::Digest,
    dir::CacheDir,
    stat::{Observation, Stat, Timestamp},
    store::{Generation, Retention, Store},
};

/// Where a [`Cache`] keeps every result by key, in its [`CacheDir`]: shared by its caches.
const STORE: &str = "store";

/// When a file's last result can be kept without reading it.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Strategy {
    /// When its `stat` is the one recorded, outside the racy window: the file isn't read.
    #[default]
    Metadata,
    /// Never: every file is read and hashed, and kept by content only.
    Content,
}

/// A file's result, as a run put it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry<R> {
    /// The path, as the caller names it.
    pub path: Box<str>,
    /// Everything the result depends on besides the file's path and content.
    pub key: Digest,
    /// The file's `stat` when it was read.
    pub seen: Observation,
    /// The digest of the content the result is for.
    pub content: Digest,
    /// What the caller computed for that content, under that key.
    pub result: R,
}

/// One of the last run's entries.
struct Last<R> {
    entry: Entry<R>,
    /// Whether this run kept it by its `stat`: set without a lock or a copy, as every file of a
    /// run that changed nothing is kept so.
    kept: AtomicBool,
}

/// Per-file results kept across runs, shared by a run's threads. See the crate's README for an
/// example.
pub struct Cache<R> {
    /// The directory this cache, its siblings and their shared store are in.
    dir: Arc<CacheDir>,
    /// The file in `dir` this cache's entries are kept in.
    name: String,
    /// When a file's last result can be kept without reading it.
    strategy: Strategy,
    /// When the run started: the time every `stat` it records counts as taken.
    started: Timestamp,
    /// How long the store keeps results no run used.
    retention: Retention,
    /// The last run's entries, by path: a table of entries hashed by their own path, so each
    /// path is allocated once (a large project has many).
    last: HashTable<Last<R>>,
    /// Read on the first lookup by content: a run answered by `stat` alone never reads it.
    store: OnceLock<Store>,
    /// This run's entries but those [kept](Last::kept) by `stat`, and the store keys it used
    /// and added: with the kept ones, the next run's `last`, and what [`Self::save`] commits to
    /// the store.
    found: Mutex<(Vec<Entry<R>>, Generation)>,
}

impl<R> fmt::Debug for Cache<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Cache")
            .field("name", &self.name)
            .field("strategy", &self.strategy)
            .field("last", &self.last.len())
            .finish_non_exhaustive()
    }
}

impl<R: Clone + PartialEq + Serialize + DeserializeOwned> Cache<R> {
    /// Opens the cache kept in `name` of `dir`, with the last run's results, and the
    /// [`Strategy::Metadata`] strategy until set otherwise.
    pub fn open(dir: Arc<CacheDir>, name: impl Into<String>) -> Self {
        let name = name.into();
        let entries: Vec<Entry<R>> = dir.read(&name).unwrap_or_default();
        let mut last = HashTable::with_capacity(entries.len());
        for entry in entries {
            let hash = Self::hash(&entry.path);
            let last_of = |last: &Last<R>| Self::hash(&last.entry.path);
            last.insert_unique(hash, Last { entry, kept: AtomicBool::new(false) }, last_of);
        }
        Self {
            last,
            dir,
            name,
            strategy: Strategy::default(),
            started: Timestamp::now(),
            retention: Retention::default(),
            store: OnceLock::new(),
            found: Mutex::default(),
        }
    }

    /// Sets when a file's last result can be kept without reading it.
    #[must_use]
    pub fn strategy(mut self, strategy: Strategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Sets how long the store keeps results no run used.
    #[must_use]
    pub fn retention(mut self, retention: Retention) -> Self {
        self.retention = retention;
        self
    }

    /// Sets when the run started, if before the cache was opened. Every `stat` it records counts
    /// as taken then: it must be before any file is read.
    #[must_use]
    pub fn started_at(mut self, started: Timestamp) -> Self {
        self.started = started;
        self
    }

    /// Returns the last run's entries, in no particular order.
    pub fn entries(&self) -> impl Iterator<Item = &Entry<R>> {
        self.last.iter().map(|last| &last.entry)
    }

    /// The last run's entry for the file named `name`.
    fn last(&self, name: &str) -> Option<&Last<R>> {
        self.last.find(Self::hash(name), |last| &*last.entry.path == name)
    }

    /// A path's hash in [`Self::last`].
    fn hash(name: &str) -> u64 {
        std::hash::BuildHasher::hash_one(&FxBuildHasher, name)
    }

    /// Returns the last run's result for the file at `path`, which now has `stat`, under `key`,
    /// if the file is unchanged by its `stat` and the strategy allows that. Such a file needn't
    /// be read. A path that isn't UTF-8 is never cached.
    pub fn get_by_stat(&self, path: &Path, key: Digest, stat: Stat) -> Option<R> {
        if self.strategy == Strategy::Content {
            return None;
        }
        let last = self.last(path.to_str()?)?;
        if last.entry.key != key || !last.entry.seen.holds_for(&stat) {
            return None;
        }
        last.kept.store(true, Ordering::Relaxed);
        Some(last.entry.result.clone())
    }

    /// Returns the result for the file at `path`, read with `stat` and hashing to `content`,
    /// under `key`: the last run's, or one any run put for that key and content.
    pub fn get_by_content(
        &self,
        path: &Path,
        key: Digest,
        stat: Stat,
        content: Digest,
    ) -> Option<R> {
        let name = path.to_str()?;
        let result = match self.last(name).map(|last| &last.entry) {
            Some(last) if last.key == key && last.content == content => last.result.clone(),
            _ => {
                let store_key = Self::store_key(name, key, content);
                let result = postcard::from_bytes(self.store().get(&store_key)?).ok()?;
                self.found().1.used.push(store_key);
                result
            }
        };
        self.push(name, key, stat, content, result.clone());
        Some(result)
    }

    /// Puts `result` for the file at `path`, read with `stat` and hashing to `content`, under
    /// `key`, for this cache's next run and any cache's run.
    pub fn put(&self, path: &Path, key: Digest, stat: Stat, content: Digest, result: R) {
        let Some(name) = path.to_str() else { return };
        if let Ok(value) = postcard::to_allocvec(&result) {
            self.found().1.added.push((Self::store_key(name, key, content), value.into()));
        }
        self.push(name, key, stat, content, result);
    }

    /// Saves this run's results, if they aren't the last run's, and commits the results it used
    /// and put to the store.
    pub fn save(&self) {
        let (found, generation) = std::mem::take(&mut *self.found());
        let kept = || self.last.iter().filter(|last| last.kept.load(Ordering::Relaxed));
        // Every file kept by its `stat` and nothing else: the last run's entries, as they are.
        let unchanged = found.is_empty() && kept().count() == self.last.len();
        if !unchanged {
            // A file found again after it was kept is as found.
            let found_paths: FxHashSet<&str> = found.iter().map(|entry| &*entry.path).collect();
            let mut entries: Vec<&Entry<R>> = kept()
                .map(|last| &last.entry)
                .filter(|entry| !found_paths.contains(&*entry.path))
                .chain(&found)
                .collect();
            entries.sort_unstable_by(|a, b| a.path.cmp(&b.path));
            self.dir.write(&self.name, &entries);
        }
        if generation.is_empty() {
            return;
        }
        // Merged into the store as it is on disk now, so runs in parallel lose as little as possible.
        let mut store: Store = self.dir.read(STORE).unwrap_or_default();
        if store.commit(&generation, self.retention) {
            self.dir.write(STORE, &store);
        }
    }

    /// A result's key in the store, shared by every cache in the directory: the caller's key,
    /// the path and the content, so a hit is for the same file, contents and dependencies.
    fn store_key(path: &str, key: Digest, content: Digest) -> Digest {
        Digest::of(&(key, path, content))
    }

    /// The store, read from disk the first time it's needed.
    fn store(&self) -> &Store {
        self.store.get_or_init(|| self.dir.read(STORE).unwrap_or_default())
    }

    /// What this run found so far, locked for one update.
    fn found(&self) -> MutexGuard<'_, (Vec<Entry<R>>, Generation)> {
        self.found.lock().unwrap()
    }

    /// Adds this run's entry for a file read with `stat`, observed at the start of the run.
    fn push(&self, path: &str, key: Digest, stat: Stat, content: Digest, result: R) {
        let seen = Observation::new(stat, self.started);
        self.found().0.push(Entry { path: path.into(), key, seen, content, result });
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, time::Duration};

    use quickcheck::{Arbitrary, Gen, QuickCheck, TestResult};

    use super::*;

    const CONTENTS: [&str; 3] = ["aaaa", "bbbb", "cc"];

    /// Answers from a `stat`, across every case: the property must exercise them.
    static BY_STAT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

    /// What a tool computes for `content` under `key`: what every answer must equal. It differs
    /// by key, so an answer for another key is caught.
    fn compute(key: u8, content: &[u8]) -> bool {
        (usize::from(key) + usize::from(content[0])) % 2 == 0
    }

    #[derive(Debug, Clone, Copy)]
    enum Step {
        /// Write one of [`CONTENTS`].
        Write(usize),
        /// Write one of [`CONTENTS`] and put the modification time back: only `ctime` tells,
        /// which Windows doesn't have (a same-size write that puts it back goes unseen there).
        #[cfg(unix)]
        WriteKeepingMtime(usize),
        /// Run under one of two keys.
        Run(u8),
        /// Forget the last run, as another cache in the directory would: only the store answers.
        Forget,
        /// Wait past the racy window, so the next run can trust the `stat` it records.
        Wait,
    }

    impl Arbitrary for Step {
        fn arbitrary(g: &mut Gen) -> Self {
            let content = usize::arbitrary(g) % CONTENTS.len();
            match u8::arbitrary(g) % 8 {
                0 => Self::Write(content),
                #[cfg(unix)]
                1 => Self::WriteKeepingMtime(content),
                2 => Self::Forget,
                3 | 4 => Self::Wait,
                _ => Self::Run(u8::arbitrary(g) % 2),
            }
        }
    }

    /// One run: every answer, from `stat`, the last run or the store, is what computing gives.
    fn run(dir: &Arc<CacheDir>, path: &Path, strategy: Strategy, key: u8) -> Option<&'static str> {
        let cache = Cache::open(Arc::clone(dir), "files").strategy(strategy);
        let source = fs::read(path).unwrap();
        let expected = compute(key, &source);
        let (key, stat) = (Digest::of(&key), Stat::of_path(path).unwrap());
        let wrong = if let Some(result) = cache.get_by_stat(path, key, stat) {
            BY_STAT.fetch_add(1, Ordering::Relaxed);
            if strategy == Strategy::Content {
                Some("content strategy trusted a stat")
            } else {
                (result != expected).then_some("stale result by stat")
            }
        } else {
            let content = Digest::of_bytes(&source);
            if let Some(result) = cache.get_by_content(path, key, stat, content) {
                (result != expected).then_some("wrong result by content")
            } else {
                cache.put(path, key, stat, content, expected);
                None
            }
        };
        cache.save();
        wrong
    }

    #[test]
    fn every_answer_is_what_computing_gives() {
        fn property(steps: Vec<Step>, content: bool) -> TestResult {
            let strategy = if content { Strategy::Content } else { Strategy::Metadata };
            let tmp = tempfile::tempdir().unwrap();
            let dir = Arc::new(CacheDir::open(&tmp.path().join("cache")).unwrap());
            let path = tmp.path().join("f");
            fs::write(&path, CONTENTS[0]).unwrap();
            for step in steps {
                match step {
                    Step::Write(i) => fs::write(&path, CONTENTS[i]).unwrap(),
                    #[cfg(unix)]
                    Step::WriteKeepingMtime(i) => {
                        let mtime = fs::metadata(&path).unwrap().modified().unwrap();
                        fs::write(&path, CONTENTS[i]).unwrap();
                        let file = fs::File::options().write(true).open(&path).unwrap();
                        file.set_modified(mtime).unwrap();
                    }
                    Step::Forget => {
                        let _ = fs::remove_file(tmp.path().join("cache/files"));
                    }
                    Step::Wait => {
                        std::thread::sleep(crate::stat::RACY_WINDOW + Duration::from_millis(10));
                    }
                    Step::Run(key) => {
                        if let Some(wrong) = run(&dir, &path, strategy, key) {
                            return TestResult::error(wrong);
                        }
                    }
                }
            }
            TestResult::passed()
        }
        QuickCheck::new()
            .tests(150)
            .rng(Gen::new(24))
            .quickcheck(property as fn(Vec<Step>, bool) -> TestResult);
        assert!(BY_STAT.load(Ordering::Relaxed) > 0, "no answer came from a `stat`");
    }

    /// The case the property finds only sometimes: a same-size write that puts the mtime back,
    /// after a `stat` was trusted. Only `ctime` tells.
    #[cfg(unix)]
    #[test]
    fn a_same_size_write_keeping_mtime_is_seen() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = Arc::new(CacheDir::open(tmp.path()).unwrap());
        let path = tmp.path().join("f");
        fs::write(&path, CONTENTS[0]).unwrap();
        std::thread::sleep(crate::stat::RACY_WINDOW + Duration::from_millis(10));
        let (key, content) = (Digest::of(&1), Digest::of_bytes(CONTENTS[0].as_bytes()));

        let cache = Cache::open(Arc::clone(&dir), "files");
        cache.put(&path, key, Stat::of_path(&path).unwrap(), content, 1u8);
        cache.save();
        let cache = Cache::<u8>::open(Arc::clone(&dir), "files");
        assert_eq!(cache.get_by_stat(&path, key, Stat::of_path(&path).unwrap()), Some(1));

        let mtime = fs::metadata(&path).unwrap().modified().unwrap();
        fs::write(&path, CONTENTS[1]).unwrap();
        fs::File::options().write(true).open(&path).unwrap().set_modified(mtime).unwrap();
        assert_eq!(cache.get_by_stat(&path, key, Stat::of_path(&path).unwrap()), None);
    }

    #[test]
    fn another_cache_in_the_directory_finds_a_result_by_key_and_content() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = Arc::new(CacheDir::open(tmp.path()).unwrap());
        let (f, g) = (tmp.path().join("f"), tmp.path().join("g"));
        fs::write(&f, "x").unwrap();
        fs::write(&g, "x").unwrap();
        let (stat, content) = (Stat::of_path(&f).unwrap(), Digest::of_bytes(b"x"));

        let cache = Cache::open(Arc::clone(&dir), "a");
        cache.put(&f, Digest::of(&1), stat, content, 7u8);
        cache.save();

        let cache = Cache::<u8>::open(Arc::clone(&dir), "b");
        assert_eq!(cache.get_by_content(&f, Digest::of(&1), stat, content), Some(7));
        assert_eq!(cache.get_by_content(&g, Digest::of(&1), stat, content), None, "another path");
        assert_eq!(cache.get_by_content(&f, Digest::of(&2), stat, content), None, "another key");
        let other = Digest::of_bytes(b"y");
        assert_eq!(cache.get_by_content(&f, Digest::of(&1), stat, other), None, "another content");
    }

    #[cfg(unix)]
    #[test]
    fn a_path_that_isnt_utf8_is_never_cached() {
        use std::{ffi::OsStr, os::unix::ffi::OsStrExt};
        let tmp = tempfile::tempdir().unwrap();
        let dir = Arc::new(CacheDir::open(tmp.path()).unwrap());
        let file = tmp.path().join("f");
        fs::write(&file, "x").unwrap();
        let (stat, content) = (Stat::of_path(&file).unwrap(), Digest::of_bytes(b"x"));
        let path = Path::new(OsStr::from_bytes(b"\xff"));
        let cache = Cache::open(Arc::clone(&dir), "files");
        cache.put(path, Digest::of(&1), stat, content, 1u8);
        assert_eq!(cache.get_by_content(path, Digest::of(&1), stat, content), None);
        cache.save();
        assert!(Cache::<u8>::open(dir, "files").entries().next().is_none());
    }
}
