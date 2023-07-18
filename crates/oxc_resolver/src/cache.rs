use std::{hash::BuildHasherDefault, path::Path, sync::Arc};

use dashmap::DashMap;
use rustc_hash::FxHasher;

use crate::{package_json::PackageJson, FileMetadata, FileSystem, ResolveError};

pub struct Cache<Fs> {
    fs: Fs,
    cache: DashMap<Box<Path>, Option<FileMetadata>, BuildHasherDefault<FxHasher>>,
    package_json_cache:
        DashMap<Box<Path>, Result<Arc<PackageJson>, ResolveError>, BuildHasherDefault<FxHasher>>,
}

impl<Fs: FileSystem> Default for Cache<Fs> {
    fn default() -> Self {
        Self {
            fs: Fs::default(),
            cache: DashMap::default(),
            package_json_cache: DashMap::default(),
        }
    }
}

impl<Fs: FileSystem> Cache<Fs> {
    pub fn new(fs: Fs) -> Self {
        Self { fs, ..Self::default() }
    }

    fn metadata_cached(&self, path: &Path) -> Option<FileMetadata> {
        if let Some(result) = self.cache.get(path) {
            return *result;
        }
        let file_metadata = self.fs.metadata(path).ok();
        self.cache.insert(path.to_path_buf().into_boxed_path(), file_metadata);
        file_metadata
    }

    pub fn is_file(&self, path: &Path) -> bool {
        self.metadata_cached(path).is_some_and(|m| m.is_file)
    }

    /// # Errors
    ///
    /// * [ResolveError::JSONError]
    ///
    /// # Panics
    ///
    /// * Failed to read the file (TODO: remove this)
    pub fn read_package_json(&self, path: &Path) -> Result<Arc<PackageJson>, ResolveError> {
        if let Some(result) = self.package_json_cache.get(path) {
            return result.value().clone();
        }
        // TODO: handle file read error
        let package_json_string = self.fs.read_to_string(path).unwrap();
        let result = PackageJson::parse(path.to_path_buf(), &package_json_string)
            .map(Arc::new)
            .map_err(|error| ResolveError::from_serde_json_error(path.to_path_buf(), &error));
        self.package_json_cache.insert(path.to_path_buf().into_boxed_path(), result.clone());
        result
    }
}
