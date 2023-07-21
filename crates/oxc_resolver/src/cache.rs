use std::{hash::BuildHasherDefault, path::Path, sync::Arc};

use dashmap::DashMap;
use rustc_hash::FxHasher;

use crate::{package_json::PackageJson, FileMetadata, FileSystem, ResolveError};

pub type PackageJsonCache = DashMap<
    Box<Path>,
    Result<Option<Arc<PackageJson>>, ResolveError>,
    BuildHasherDefault<FxHasher>,
>;

pub struct Cache<Fs> {
    fs: Fs,
    cache: DashMap<Box<Path>, Option<FileMetadata>, BuildHasherDefault<FxHasher>>,
    package_json_cache: PackageJsonCache,
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
        let file_metadata = self.fs.symlink_metadata(path).ok();
        self.cache.insert(path.to_path_buf().into_boxed_path(), file_metadata);
        file_metadata
    }

    pub fn is_file(&self, path: &Path) -> bool {
        self.metadata_cached(path).is_some_and(|m| m.is_file)
    }

    /// # Errors
    ///
    /// * [ResolveError::JSONError]
    pub fn read_package_json(&self, path: &Path) -> Result<Option<Arc<PackageJson>>, ResolveError> {
        if let Some(result) = self.package_json_cache.get(path) {
            return result.value().clone();
        }
        let Ok(package_json_string) = self.fs.read_to_string(path) else {
            self.package_json_cache.insert(path.to_path_buf().into_boxed_path(), Ok(None));
            return Ok(None);
        };
        let result = PackageJson::parse(path.to_path_buf(), &package_json_string)
            .map(|package_json| Some(Arc::new(package_json)))
            .map_err(|error| ResolveError::from_serde_json_error(path.to_path_buf(), &error));
        self.package_json_cache.insert(path.to_path_buf().into_boxed_path(), result.clone());
        result
    }
}
