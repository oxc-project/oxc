use once_cell::sync::OnceCell as OnceLock;
use std::{
    convert::AsRef,
    hash::{BuildHasherDefault, Hash, Hasher},
    ops::Deref,
    path::{Path, PathBuf},
    sync::Arc,
};

use dashmap::DashMap;
use rustc_hash::FxHasher;

use crate::{
    package_json::PackageJson, FileMetadata, FileSystem, ResolveError, ResolveOptions, TsConfig,
};

#[derive(Default)]
pub struct Cache<Fs> {
    pub(crate) fs: Fs,
    // Using IdentityHasher to avoid double hashing in the `get` + `insert` case.
    cache: DashMap<u64, CachedPath, BuildHasherDefault<IdentityHasher>>,
    tsconfigs: DashMap<u64, Arc<TsConfig>, BuildHasherDefault<IdentityHasher>>,
}

#[derive(Default)]
struct IdentityHasher(u64);

impl Hasher for IdentityHasher {
    fn write(&mut self, _: &[u8]) {
        panic!("Invalid use of IdentityHasher")
    }
    fn write_u64(&mut self, n: u64) {
        self.0 = n;
    }
    fn finish(&self) -> u64 {
        self.0
    }
}

impl<Fs: FileSystem> Cache<Fs> {
    pub fn new(fs: Fs) -> Self {
        Self { fs, ..Self::default() }
    }

    pub fn clear(&self) {
        self.cache.clear();
        self.tsconfigs.clear();
    }

    pub fn value(&self, path: &Path) -> CachedPath {
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
            CachedPath(Arc::new(CachedPathImpl::new(path.to_path_buf().into_boxed_path(), parent)));
        self.cache.insert(hash, data.clone());
        data
    }

    pub fn tsconfig(
        &self,
        tsconfig_path: &Path,
        callback: impl FnOnce(&mut TsConfig) -> Result<(), ResolveError>, // callback for modifying tsconfig with `extends`
    ) -> Result<Arc<TsConfig>, ResolveError> {
        let hash = {
            let mut hasher = FxHasher::default();
            tsconfig_path.hash(&mut hasher);
            hasher.finish()
        };
        self.tsconfigs
            .entry(hash)
            .or_try_insert_with(|| {
                let mut tsconfig_string = self
                    .fs
                    .read_to_string(tsconfig_path)
                    .map_err(|_| ResolveError::NotFound(tsconfig_path.to_path_buf()))?;
                let mut tsconfig =
                    TsConfig::parse(tsconfig_path, &mut tsconfig_string).map_err(|error| {
                        ResolveError::from_serde_json_error(tsconfig_path.to_path_buf(), &error)
                    })?;
                callback(&mut tsconfig)?;
                Ok(Arc::new(tsconfig))
            })
            .map(|r| Arc::clone(r.value()))
    }
}

#[derive(Clone)]
pub struct CachedPath(Arc<CachedPathImpl>);

impl Deref for CachedPath {
    type Target = CachedPathImpl;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl AsRef<CachedPathImpl> for CachedPath {
    fn as_ref(&self) -> &CachedPathImpl {
        self.0.as_ref()
    }
}

pub struct CachedPathImpl {
    path: Box<Path>,
    parent: Option<CachedPath>,
    meta: OnceLock<Option<FileMetadata>>,
    symlink: OnceLock<Option<PathBuf>>,
    package_json: OnceLock<Option<Arc<PackageJson>>>,
}

impl CachedPathImpl {
    fn new(path: Box<Path>, parent: Option<CachedPath>) -> Self {
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

    pub fn parent(&self) -> Option<&CachedPath> {
        self.parent.as_ref()
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

    pub fn symlink<Fs: FileSystem>(&self, fs: &Fs) -> Option<PathBuf> {
        self.symlink.get_or_init(|| fs.canonicalize(&self.path).ok()).clone()
    }

    /// Find package.json of a path by traversing parent directories.
    ///
    /// # Errors
    ///
    /// * [ResolveError::JSON]
    pub fn find_package_json<Fs: FileSystem>(
        &self,
        fs: &Fs,
        options: &ResolveOptions,
    ) -> Result<Option<Arc<PackageJson>>, ResolveError> {
        let mut cache_value = self;
        // Go up a directory when querying a file, this avoids a file read from example.js/package.json
        if cache_value.is_file(fs) {
            if let Some(cv) = &cache_value.parent {
                cache_value = cv.as_ref();
            }
        }
        let mut cache_value = Some(cache_value);
        while let Some(cv) = cache_value {
            if let Some(package_json) = cv.package_json(fs, options)? {
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
        fs: &Fs,
        options: &ResolveOptions,
    ) -> Result<Option<Arc<PackageJson>>, ResolveError> {
        // Change to `std::sync::OnceLock::get_or_try_init` when it is stable.
        self.package_json
            .get_or_try_init(|| {
                let package_json_path = self.path.join("package.json");
                let Ok(package_json_string) = fs.read_to_string(&package_json_path) else {
                    return Ok(None);
                };
                PackageJson::parse(package_json_path.clone(), &package_json_string, options)
                    .map(Arc::new)
                    .map(Some)
                    .map_err(|error| ResolveError::from_serde_json_error(package_json_path, &error))
            })
            .cloned()
    }
}
