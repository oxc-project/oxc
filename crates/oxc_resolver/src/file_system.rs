use std::{fs, path::Path, sync::Arc};

use dashmap::DashMap;

use crate::{package_json::PackageJson, ResolveError};

#[derive(Debug, Clone, Copy)]
pub struct FileMetadata {
    is_file: bool,
}

/// [File System](https://doc.rust-lang.org/stable/std/fs/) with caching
#[derive(Default)]
pub struct FileSystem {
    cache: DashMap<Box<Path>, Option<FileMetadata>>,
    package_json_cache: DashMap<Box<Path>, Result<Arc<PackageJson>, ResolveError>>,
}

impl FileSystem {
    /// <https://doc.rust-lang.org/stable/std/fs/fn.metadata.html>
    pub fn metadata(&self, path: &Path) -> Option<FileMetadata> {
        if let Some(result) = self.cache.get(path) {
            return *result;
        }
        let file_metadata =
            fs::metadata(path).ok().map(|metadata| FileMetadata { is_file: metadata.is_file() });
        self.cache.insert(path.to_path_buf().into_boxed_path(), file_metadata);
        file_metadata
    }

    pub fn is_file(&self, path: &Path) -> bool {
        self.metadata(path).is_some_and(|m| m.is_file)
    }

    pub fn read_package_json(&self, path: &Path) -> Result<Arc<PackageJson>, ResolveError> {
        if let Some(result) = self.package_json_cache.get(path) {
            return result.value().clone();
        }
        let package_json_string = fs::read_to_string(path).unwrap();
        let result = PackageJson::parse(path.to_path_buf(), &package_json_string)
            .map(Arc::new)
            .map_err(|error| ResolveError::from_serde_json_error(path.to_path_buf(), &error));
        self.package_json_cache.insert(path.to_path_buf().into_boxed_path(), result.clone());
        result
    }
}
