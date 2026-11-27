//! `--cache`: remembers what was already formatted, keyed by what decides the output:
//! the oxfmt version, the format strategy, the resolved options and the file's content.
//!
//! - A run still walks and loads configs. A file whose `stat` is the one it had when last known
//!   formatted isn't read either, once that `stat` is old enough to trust (see [`RACY_NS`]).
//! - A result that called into Node (Prettier, the Tailwind sorter) may depend on files outside
//!   that key (plugins, the Tailwind stylesheet), so it isn't cached.
//! - Anything unreadable is a miss, anything unwritable is skipped.
//!
//! NOTE: The version stands for the build, so a local build of the same version may reuse
//! results of another one. Clear the cache directory after rebuilding oxfmt.

use std::{
    cell::RefCell,
    fs,
    io::Write,
    path::{Path, PathBuf},
    ptr,
    sync::{Arc, Mutex, Weak},
    time::SystemTime,
};

use rustc_hash::FxHashSet;

use crate::core::{FormatPlan, oxfmtrc::FormatConfig, utils};

type Key = [u8; 32];

/// Bump when the key or the file format changes.
const FILE: &str = "formatted-v1";
/// Keys kept at most (32 bytes each, 16 MiB): about 2x kibana, a content and a `stat` key per file.
const MAX_KEYS: usize = 1 << 19;
/// A `stat` whose timestamps are this close to the start of the run isn't trusted:
/// a later write in the same timestamp tick wouldn't change it (Git's "racy" rule).
/// Coarser than every local file system's timestamps (FAT's are 2 s).
const RACY_NS: i128 = 2_000_000_000;

pub struct FormatCache {
    dir: PathBuf,
    /// When the run started, before any file was read, in nanoseconds since the Unix epoch.
    started: i128,
    /// Keys of contents (and `stat`s) known formatted, as the last run left them.
    known: FxHashSet<Key>,
    hits: Mutex<Vec<Key>>,
    added: Mutex<Vec<Key>>,
}

impl FormatCache {
    /// Opens the cache at `location`, `node_modules/.cache/oxfmt` in `cwd` by default.
    pub fn open(cwd: &Path, location: Option<&Path>) -> Self {
        let started = nanos(SystemTime::now()).unwrap_or(i128::MIN);
        let dir = location.map_or_else(
            || cwd.join("node_modules/.cache/oxfmt"),
            |location| utils::normalize_relative_path(cwd, location),
        );
        // A truncated or corrupt file only yields keys that match nothing.
        let known = fs::read(dir.join(FILE))
            .map_or_default(|bytes| bytes.as_chunks::<32>().0.iter().copied().collect());
        Self { dir, started, known, hits: Mutex::default(), added: Mutex::default() }
    }

    /// The key of formatting `source` as `plan` says.
    pub fn key(plan: &FormatPlan, source: &str) -> Key {
        let mut hasher = plan_hasher(plan);
        hasher.update(source.as_bytes());
        *hasher.finalize().as_bytes()
    }

    /// The key of formatting the file at `plan.path` as it is now, without reading it:
    /// its path and `stat`. `None` if its `stat` is too recent to trust, or can't be taken.
    ///
    /// Take it before reading the file, so a write during the read changes the next `stat`.
    pub fn stat_key(&self, plan: &FormatPlan) -> Option<Key> {
        let (times, ids) = stat(&fs::metadata(&plan.path).ok()?)?;
        if times.iter().any(|time| time + RACY_NS >= self.started) {
            return None;
        }
        let mut fields = [0; 96];
        for (bytes, field) in
            fields.as_chunks_mut::<16>().0.iter_mut().zip(times.iter().chain(&ids))
        {
            bytes.copy_from_slice(&field.to_le_bytes());
        }
        let mut hasher = plan_hasher(plan);
        // No source starts with `0xFF` (not UTF-8), so this never hashes what `key` does.
        hasher.update(&[0xFF]);
        hasher.update(plan.path.as_os_str().as_encoded_bytes());
        // A fixed length after the path: no separator needed.
        hasher.update(&fields);
        Some(*hasher.finalize().as_bytes())
    }

    pub fn is_formatted(&self, key: Key) -> bool {
        let hit = self.known.contains(&key);
        if hit {
            self.hits.lock().unwrap().push(key);
        }
        hit
    }

    pub fn set_formatted(&self, keys: impl IntoIterator<Item = Key>) {
        self.added.lock().unwrap().extend(keys);
    }

    /// Saves the keys, if this run added any.
    pub fn save(self) {
        let (hits, added) = (self.hits.into_inner().unwrap(), self.added.into_inner().unwrap());
        tracing::debug!(target: "oxfmt::cache", hits = hits.len(), added = added.len(), "save");
        if added.is_empty() {
            return;
        }
        let keys = keys_to_keep(self.known, hits, added, MAX_KEYS);
        let mut bytes = Vec::with_capacity(keys.len() * 32);
        for key in &keys {
            bytes.extend_from_slice(key);
        }
        // Written beside it and renamed over, so a concurrent run reads either file whole.
        let tmp = self.dir.join(format!("{FILE}.{}.tmp", std::process::id()));
        let _ = fs::create_dir_all(&self.dir)
            .and_then(|()| fs::write(&tmp, bytes))
            .and_then(|()| fs::rename(&tmp, self.dir.join(FILE)));
    }
}

/// What the output depends on besides the file: the build, the strategy and the options.
fn plan_hasher(plan: &FormatPlan) -> blake3::Hasher {
    let mut hasher = blake3::Hasher::new();
    hasher.update(env!("CARGO_PKG_VERSION").as_bytes());
    hasher.update(&[0]);
    let _ = write!(hasher, "{:?}", plan.strategy);
    hasher.update(&[0]);
    hasher.update(&options_digest(&plan.config));
    hasher
}

/// A file's `stat`, as what a write changes: its modification and inode change times,
/// only trusted once older than the run, and its size, inode, device and mode
/// (a file replaced by a rename has another inode).
///
/// NOTE: Without an inode change time (not on Unix), a write to the same file that keeps
/// the size and puts `mtime` back isn't seen.
fn stat(meta: &fs::Metadata) -> Option<([i128; 2], [i128; 4])> {
    let mtime = nanos(meta.modified().ok()?)?;
    #[cfg(unix)]
    let (ctime, ino, dev, mode) = {
        use std::os::unix::fs::MetadataExt;
        // Tools that put `mtime` back can't set it.
        let ctime = i128::from(meta.ctime()) * 1_000_000_000 + i128::from(meta.ctime_nsec());
        (ctime, meta.ino(), meta.dev(), meta.mode())
    };
    // The creation time tells a file replaced by a rename.
    // TODO: Add `ChangeTime` once `windows_change_time` is stable.
    #[cfg(windows)]
    let (ctime, ino, dev, mode) = {
        use std::os::windows::fs::MetadataExt;
        (mtime, meta.creation_time(), 0_u64, meta.file_attributes())
    };
    #[cfg(not(any(unix, windows)))]
    let (ctime, ino, dev, mode) = (mtime, 0_u64, 0_u64, 0_u32);
    Some(([mtime, ctime], [meta.len(), ino, dev, u64::from(mode)].map(i128::from)))
}

fn nanos(time: SystemTime) -> Option<i128> {
    let since = time.duration_since(SystemTime::UNIX_EPOCH).ok()?;
    i128::try_from(since.as_nanos()).ok()
}

/// The digest of `config`, hashed once per thread for files sharing a resolved config.
fn options_digest(config: &Arc<FormatConfig>) -> Key {
    thread_local! {
        // `Weak` keeps the allocation, so another config can't take its address.
        static LAST: RefCell<(Weak<FormatConfig>, Key)> = const { RefCell::new((Weak::new(), [0; 32])) };
    }
    LAST.with_borrow_mut(|(last, digest)| {
        if !ptr::eq(last.as_ptr(), Arc::as_ptr(config)) {
            // Hashed whole: many small writes to the hasher are 2x slower.
            let json = serde_json::to_vec(&**config).unwrap_or_default();
            *last = Arc::downgrade(config);
            *digest = *blake3::hash(&json).as_bytes();
        }
        *digest
    })
}

/// The keys to write: all of them, or past `max`, this run's and then arbitrary older ones.
fn keys_to_keep(
    mut known: FxHashSet<Key>,
    hits: Vec<Key>,
    added: Vec<Key>,
    max: usize,
) -> FxHashSet<Key> {
    if known.len() + added.len() <= max {
        known.extend(added);
        return known;
    }
    let mut keys: FxHashSet<Key> = hits.into_iter().chain(added).collect();
    let room = max.saturating_sub(keys.len());
    keys.extend(known.into_iter().take(room));
    keys
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(n: u8) -> Key {
        [n; 32]
    }

    #[test]
    fn saved_keys_are_known_next_run() {
        let dir = tempfile::tempdir().unwrap();
        let cache = FormatCache::open(dir.path(), Some(dir.path()));
        assert!(!cache.is_formatted(key(1)));
        cache.set_formatted([key(1)]);
        cache.save();

        let cache = FormatCache::open(dir.path(), Some(dir.path()));
        assert!(cache.is_formatted(key(1)));
        assert!(!cache.is_formatted(key(2)));
    }

    #[test]
    fn a_truncated_file_keeps_only_whole_keys() {
        let dir = tempfile::tempdir().unwrap();
        let bytes: Vec<u8> = [key(1), key(2)].iter().flatten().copied().chain([3; 5]).collect();
        fs::write(dir.path().join(FILE), bytes).unwrap();

        let cache = FormatCache::open(dir.path(), Some(dir.path()));
        assert_eq!(cache.known.len(), 2);
        assert!(cache.is_formatted(key(1)) && cache.is_formatted(key(2)));
    }

    #[test]
    fn past_the_cap_this_runs_keys_are_kept_first() {
        let known: FxHashSet<Key> = (1..=5).map(key).collect();
        let all = keys_to_keep(known.clone(), vec![key(1)], vec![key(6)], 6);
        assert_eq!(all.len(), 6);
        let pruned = keys_to_keep(known, vec![key(1)], vec![key(6)], 4);
        assert!((3..=4).contains(&pruned.len()));
        assert!(pruned.contains(&key(1)) && pruned.contains(&key(6)));
    }
}
