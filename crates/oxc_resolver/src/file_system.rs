use std::{
    fs,
    path::{Path, PathBuf},
};

use dashmap::DashMap;

#[derive(Debug, Clone, Copy)]
pub struct FileMetadata {
    is_file: bool,
}

/// [File System](https://doc.rust-lang.org/stable/std/fs/) with caching
#[derive(Default)]
pub struct FileSystem {
    cache: DashMap<PathBuf, Option<FileMetadata>>,
}

impl FileSystem {
    /// <https://doc.rust-lang.org/stable/std/fs/fn.metadata.html>
    pub fn metadata<P: AsRef<Path>>(&self, path: P) -> Option<FileMetadata> {
        let path = path.as_ref();
        if let Some(result) = self.cache.get(path) {
            return *result;
        }
        let file_metadata =
            fs::metadata(path).ok().map(|metadata| FileMetadata { is_file: metadata.is_file() });
        self.cache.insert(path.to_path_buf(), file_metadata);
        file_metadata
    }

    pub fn is_file<P: AsRef<Path>>(&self, path: P) -> bool {
        self.metadata(path).is_some_and(|m| m.is_file)
    }
}
