use std::{
    convert::AsRef,
    hash::{Hash, Hasher},
    num::NonZeroUsize,
    ops::Deref,
    path::{Path, PathBuf},
    sync::{Arc, OnceLock},
};

use dashmap::DashMap;
use identity_hash::BuildIdentityHasher;
use rustc_hash::FxHasher;
use sharded_slab::Slab;

use crate::{package_json::PackageJson, FileMetadata, FileSystem, ResolveError};

pub struct Cache<Fs> {
    pub(crate) fs: Fs,
    // Using IdentityHasher to avoid double hashing in the `get` + `insert` case.
    cache: DashMap<u64, CacheValue, BuildIdentityHasher<u64>>,
    // Using Slab for compact storage in a multi-threaded environment,
    // which also shrinks the size of CacheValue.
    symlink: Slab<PathBuf>,
    package_json: Slab<Result<Arc<PackageJson>, ResolveError>>,
}

impl<Fs: FileSystem> Default for Cache<Fs> {
    fn default() -> Self {
        Self {
            fs: Fs::default(),
            cache: DashMap::default(),
            symlink: Slab::default(),
            package_json: Slab::default(),
        }
    }
}

impl<Fs: FileSystem> Cache<Fs> {
    pub fn new(fs: Fs) -> Self {
        Self { fs, ..Self::default() }
    }

    /// # Panics
    ///
    /// * Path is file but does not have a parent
    pub fn dirname<'a>(&self, cache_value: &'a CacheValue) -> &'a CacheValue {
        if cache_value.is_file(&self.fs) {
            cache_value.parent.as_ref().unwrap()
        } else {
            cache_value
        }
    }

    pub fn value(&self, path: &Path) -> CacheValue {
        let hash = {
            let mut hasher = FxHasher::default();
            path.hash(&mut hasher);
            hasher.finish()
        };
        if let Some(cache_entry) = self.cache.get(&hash) {
            return cache_entry.value().clone();
        }
        let parent = path.parent().map(|p| self.value(p));
        let data =
            CacheValue(Arc::new(CacheValueImpl::new(path.to_path_buf().into_boxed_path(), parent)));
        self.cache.insert(hash, data.clone());
        data
    }
}

#[derive(Debug, Clone)]
pub struct CacheValue(Arc<CacheValueImpl>);

impl Deref for CacheValue {
    type Target = CacheValueImpl;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl AsRef<CacheValueImpl> for CacheValue {
    fn as_ref(&self) -> &CacheValueImpl {
        self.0.as_ref()
    }
}

#[derive(Debug)]
pub struct CacheValueImpl {
    path: Box<Path>,
    parent: Option<CacheValue>,
    meta: OnceLock<Option<FileMetadata>>,
    symlink: OnceLock<Option<NonZeroUsize>>,
    package_json: OnceLock<Option<NonZeroUsize>>,
}

impl CacheValueImpl {
    fn new(path: Box<Path>, parent: Option<CacheValue>) -> Self {
        Self {
            path,
            parent,
            meta: OnceLock::new(),
            symlink: OnceLock::new(),
            package_json: OnceLock::new(),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn to_path_buf(&self) -> PathBuf {
        self.path.to_path_buf()
    }

    fn meta<Fs: FileSystem>(&self, fs: &Fs) -> Option<FileMetadata> {
        *self.meta.get_or_init(|| fs.metadata(&self.path).ok())
    }

    pub fn is_file<Fs: FileSystem>(&self, fs: &Fs) -> bool {
        self.meta(fs).is_some_and(|meta| meta.is_file)
    }

    pub fn is_dir<Fs: FileSystem>(&self, fs: &Fs) -> bool {
        self.meta(fs).is_some_and(|meta| meta.is_dir)
    }

    pub fn symlink<Fs: FileSystem>(&self, cache: &Cache<Fs>) -> Option<PathBuf> {
        self.symlink
            .get_or_init(|| {
                let Some(path) = cache.fs.canonicalize(&self.path).ok() else { return None };
                let index = cache.symlink.insert(path).expect("slab exceeded maximum items");
                Some(unsafe { NonZeroUsize::new_unchecked(index + 1) })
            })
            .map(|index| cache.symlink.get(index.get() - 1).unwrap().clone())
    }

    /// Find package.json of a path by traversing parent directories.
    ///
    /// # Errors
    ///
    /// * [ResolveError::JSON]
    pub fn find_package_json<Fs: FileSystem>(
        &self,
        cache: &Cache<Fs>,
    ) -> Result<Option<Arc<PackageJson>>, ResolveError> {
        let mut cache_value = self;
        // Go up a directory when querying a file, this avoids a file read from example.js/package.json
        if cache_value.is_file(&cache.fs) {
            if let Some(cv) = &cache_value.parent {
                cache_value = cv.as_ref();
            }
        }
        let mut cache_value = Some(cache_value);
        while let Some(cv) = cache_value {
            if let Some(package_json) = cv.package_json(cache).transpose()? {
                return Ok(Some(Arc::clone(&package_json)));
            }
            cache_value = cv.parent.as_deref();
        }
        Ok(None)
    }

    /// Get package.json of the given path.
    ///
    /// # Errors
    ///
    /// * [ResolveError::JSON]
    pub fn package_json<Fs: FileSystem>(
        &self,
        cache: &Cache<Fs>,
    ) -> Option<Result<Arc<PackageJson>, ResolveError>> {
        // Change to `get_or_try_init` once it is stable
        self.package_json
            .get_or_init(|| {
                let package_json_path = self.path.join("package.json");
                let Some(package_json_str) = cache.fs.read_to_string(&package_json_path).ok()
                else {
                    return None;
                };
                let package_json_result =
                    PackageJson::parse(package_json_path.clone(), &package_json_str)
                        .map(Arc::new)
                        .map_err(|error| {
                            ResolveError::from_serde_json_error(package_json_path, &error)
                        });
                let index = cache
                    .package_json
                    .insert(package_json_result)
                    .expect("slab exceeded maximum items");
                Some(unsafe { NonZeroUsize::new_unchecked(index + 1) })
            })
            .map(|index| cache.package_json.get(index.get() - 1).unwrap().clone())
    }
}
