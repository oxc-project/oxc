use std::{
    collections::BTreeMap,
    fmt::Debug,
    fs,
    hash::Hasher,
    io::{self, Write},
    path::{MAIN_SEPARATOR, Path, PathBuf},
    time::UNIX_EPOCH,
};

use rustc_hash::FxHasher;
use serde::{Deserialize, Serialize};

use super::diagnostic::CachedDiagnostic;

/// Bump when the cache file format changes.
const FORMAT_VERSION: u32 = 1;

const DEFAULT_CACHE_FILE: &str = ".oxlintcache";

/// The cache file. Entries are keyed by the file's display path, the same name that
/// diagnostics for the file carry.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CacheFile {
    version: u32,
    /// Fingerprint of everything besides the file itself that affects lint results.
    settings: String,
    pub files: BTreeMap<String, CacheEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheEntry {
    pub size: u64,
    /// Modification time as `[seconds, nanoseconds]` since the Unix epoch.
    pub mtime: (u64, u32),
    /// Content hash, only present when written with `--cache-strategy content`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub diagnostics: Vec<CachedDiagnostic>,
}

impl CacheEntry {
    pub fn stat(&self) -> FileStat {
        FileStat { size: self.size, mtime: self.mtime }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileStat {
    pub size: u64,
    pub mtime: (u64, u32),
}

impl FileStat {
    pub fn read(path: &Path) -> Option<Self> {
        let metadata = fs::metadata(path).ok()?;
        let mtime = metadata.modified().ok()?.duration_since(UNIX_EPOCH).ok()?;
        Some(Self { size: metadata.len(), mtime: (mtime.as_secs(), mtime.subsec_nanos()) })
    }
}

impl CacheFile {
    pub fn new(settings: String, files: BTreeMap<String, CacheEntry>) -> Self {
        Self { version: FORMAT_VERSION, settings, files }
    }

    /// Load the entries of the cache file at `path`.
    /// A missing, unreadable or outdated cache file is treated as empty.
    pub fn load(path: &Path, settings: &str) -> BTreeMap<String, CacheEntry> {
        fs::read(path)
            .ok()
            .and_then(|bytes| serde_json::from_slice::<Self>(&bytes).ok())
            .filter(|file| file.version == FORMAT_VERSION && file.settings == settings)
            .map(|file| file.files)
            .unwrap_or_default()
    }

    /// Write the cache file atomically, so that concurrent or interrupted runs never leave a
    /// partially written file behind.
    pub fn save(&self, path: &Path) -> io::Result<()> {
        let dir = path.parent().unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(dir)?;
        let mut file = tempfile::NamedTempFile::new_in(dir)?;
        file.write_all(&serde_json::to_vec(self)?)?;
        file.persist(path)?;
        Ok(())
    }
}

/// Resolve `--cache-location` against `cwd`, like ESLint: a directory gets a cache file named
/// after a hash of `cwd`.
pub fn cache_path(location: Option<&Path>, cwd: &Path) -> PathBuf {
    let Some(location) = location else {
        return cwd.join(DEFAULT_CACHE_FILE);
    };
    let path = cwd.join(location);
    let is_dir =
        path.is_dir() || location.as_os_str().to_string_lossy().ends_with(['/', MAIN_SEPARATOR]);
    if is_dir {
        path.join(format!(".cache_{}", hash_bytes(cwd.as_os_str().as_encoded_bytes())))
    } else {
        path
    }
}

/// Fingerprint the settings that affect lint results of every file.
///
/// `settings` is hashed through its `Debug` output, which is deterministic for the resolved
/// configuration and includes every rule with its options. The output can change between
/// versions, so the oxlint version is part of the fingerprint.
pub fn settings_hash(settings: &dyn Debug, tsconfig: Option<&Path>) -> String {
    let mut input = format!("{FORMAT_VERSION}\n{}\n{settings:?}\n", env!("CARGO_PKG_VERSION"));
    if let Some(tsconfig) = tsconfig {
        input.push_str(&fs::read_to_string(tsconfig).unwrap_or_default());
    }
    hash_bytes(input.as_bytes())
}

/// Stable 128-bit hash, as hex. Not cryptographic; only used to detect changes.
pub fn hash_bytes(bytes: &[u8]) -> String {
    let hash = |seed| {
        let mut hasher = FxHasher::with_seed(seed);
        hasher.write(bytes);
        hasher.finish()
    };
    format!("{:016x}{:016x}", hash(0), hash(0x9e37_79b9))
}
