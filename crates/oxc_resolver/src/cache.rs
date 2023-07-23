use std::{
    hash::BuildHasherDefault,
    path::{Path, PathBuf},
    sync::{Arc, OnceLock},
};

use dashmap::DashMap;
use rustc_hash::FxHasher;

use crate::{package_json::PackageJson, FileMetadata, FileSystem, ResolveError};

pub struct Cache<Fs> {
    fs: Fs,
    cache: DashMap<PathBuf, Arc<CacheValue>, BuildHasherDefault<FxHasher>>,
}

impl<Fs: FileSystem> Default for Cache<Fs> {
    fn default() -> Self {
        Self { fs: Fs::default(), cache: DashMap::default() }
    }
}

impl<Fs: FileSystem> Cache<Fs> {
    pub fn new(fs: Fs) -> Self {
        Self { fs, ..Self::default() }
    }

    pub fn is_file(&self, path: &Path) -> bool {
        self.cache_value(path).is_file(&self.fs)
    }

    pub fn is_dir(&self, path: &Path) -> bool {
        self.cache_value(path).is_dir(&self.fs)
    }

    /// # Panics
    ///
    /// * Path is file but does not have a parent
    pub fn dirname(&self, path: &Path) -> PathBuf {
        (if self.is_file(path) { path.parent().unwrap() } else { path }).to_path_buf()
    }

    pub fn canonicalize(&self, path: &Path) -> Option<PathBuf> {
        self.cache_value(path).symlink(&self.fs)
    }

    /// Get package.json of the given path.
    ///
    /// # Errors
    ///
    /// * [ResolveError::JSON]
    pub fn get_package_json(&self, path: &Path) -> Result<Option<Arc<PackageJson>>, ResolveError> {
        self.cache_value(path).package_json(&self.fs).transpose()
    }

    /// Find package.json of a path by traversing parent directories.
    ///
    /// # Errors
    ///
    /// * [ResolveError::JSON]
    pub fn find_package_json(&self, path: &Path) -> Result<Option<Arc<PackageJson>>, ResolveError> {
        let mut cache_value = Some(self.cache_value(path));
        while let Some(cv) = cache_value {
            if let Some(package_json) = cv.package_json(&self.fs).transpose()? {
                return Ok(Some(Arc::clone(&package_json)));
            }
            cache_value = cv.parent.clone();
        }
        Ok(None)
    }

    fn cache_value(&self, path: &Path) -> Arc<CacheValue> {
        if let Some(cache_entry) = self.cache.get(path) {
            return Arc::clone(cache_entry.value());
        }
        let parent = path.parent().map(|p| self.cache_value(p));
        let data = Arc::new(CacheValue::new(path.to_path_buf(), parent));
        self.cache.insert(path.to_path_buf(), Arc::clone(&data));
        data
    }
}

#[derive(Debug, Clone)]
pub struct CacheValue {
    path: PathBuf,
    parent: Option<Arc<CacheValue>>,
    meta: OnceLock<Option<FileMetadata>>,
    symlink: OnceLock<Option<PathBuf>>,
    package_json: OnceLock<Option<Result<Arc<PackageJson>, ResolveError>>>,
}

impl CacheValue {
    fn new(path: PathBuf, parent: Option<Arc<Self>>) -> Self {
        Self {
            path,
            parent,
            meta: OnceLock::new(),
            symlink: OnceLock::new(),
            package_json: OnceLock::new(),
        }
    }

    fn meta<Fs: FileSystem>(&self, fs: &Fs) -> Option<FileMetadata> {
        *self.meta.get_or_init(|| fs.metadata(&self.path).ok())
    }

    fn is_file<Fs: FileSystem>(&self, fs: &Fs) -> bool {
        self.meta(fs).is_some_and(|meta| meta.is_file)
    }

    fn is_dir<Fs: FileSystem>(&self, fs: &Fs) -> bool {
        self.meta(fs).is_some_and(|meta| meta.is_dir)
    }

    fn symlink<Fs: FileSystem>(&self, fs: &Fs) -> Option<PathBuf> {
        self.symlink.get_or_init(|| fs.canonicalize(&self.path).ok()).clone()
    }

    fn package_json<Fs: FileSystem>(
        &self,
        fs: &Fs,
    ) -> Option<Result<Arc<PackageJson>, ResolveError>> {
        // Change to `get_or_try_init` once it is stable
        self.package_json
            .get_or_init(|| {
                let package_json_path = self.path.join("package.json");
                fs.read_to_string(&package_json_path).ok().map(|package_json_string| {
                    PackageJson::parse(package_json_path.clone(), &package_json_string)
                        .map(Arc::new)
                        .map_err(|error| {
                            ResolveError::from_serde_json_error(package_json_path, &error)
                        })
                })
            })
            .clone()
    }
}
