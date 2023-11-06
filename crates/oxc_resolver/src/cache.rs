use once_cell::sync::OnceCell as OnceLock;
use std::{
    borrow::{Borrow, Cow},
    convert::AsRef,
    fmt,
    hash::{BuildHasherDefault, Hash, Hasher},
    io,
    ops::Deref,
    path::{Path, PathBuf},
    sync::Arc,
};

use dashmap::{DashMap, DashSet};
use rustc_hash::FxHasher;

use crate::{
    package_json::PackageJson, path::PathUtil, FileMetadata, FileSystem, ResolveError,
    ResolveOptions, TsConfig,
};

#[derive(Default)]
pub struct Cache<Fs> {
    pub(crate) fs: Fs,
    cache: DashSet<CachedPath, BuildHasherDefault<IdentityHasher>>,
    tsconfigs: DashMap<PathBuf, Arc<TsConfig>, BuildHasherDefault<FxHasher>>,
}

impl<Fs: FileSystem + Default> Cache<Fs> {
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
        if let Some(cache_entry) = self.cache.get((hash, path).borrow() as &dyn CacheKey) {
            return cache_entry.clone();
        }
        let parent = path.parent().map(|p| self.value(p));
        let data = CachedPath(Arc::new(CachedPathImpl::new(
            hash,
            path.to_path_buf().into_boxed_path(),
            parent,
        )));
        self.cache.insert(data.clone());
        data
    }

    pub fn tsconfig(
        &self,
        path: &Path,
        callback: impl FnOnce(&mut TsConfig) -> Result<(), ResolveError>, // callback for modifying tsconfig with `extends`
    ) -> Result<Arc<TsConfig>, ResolveError> {
        if let Some(tsconfig_ref) = self.tsconfigs.get(path) {
            return Ok(Arc::clone(tsconfig_ref.value()));
        }
        let meta = self.fs.metadata(path).ok();
        let tsconfig_path = if meta.is_some_and(|m| m.is_file) {
            Cow::Borrowed(path)
        } else if meta.is_some_and(|m| m.is_dir) {
            Cow::Owned(path.join("tsconfig.json"))
        } else {
            let mut os_string = path.to_path_buf().into_os_string();
            os_string.push(".json");
            Cow::Owned(PathBuf::from(os_string))
        };
        let mut tsconfig_string = self
            .fs
            .read_to_string(&tsconfig_path)
            .map_err(|_| ResolveError::TsconfigNotFound(tsconfig_path.to_path_buf()))?;
        let mut tsconfig =
            TsConfig::parse(&tsconfig_path, &mut tsconfig_string).map_err(|error| {
                ResolveError::from_serde_json_error(tsconfig_path.to_path_buf(), &error)
            })?;
        callback(&mut tsconfig)?;
        let tsconfig = Arc::new(tsconfig);
        self.tsconfigs.insert(path.to_path_buf(), Arc::clone(&tsconfig));
        Ok(tsconfig)
    }
}

#[derive(Clone)]
pub struct CachedPath(Arc<CachedPathImpl>);

impl fmt::Debug for CachedPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.path.fmt(f)
    }
}

impl Hash for CachedPath {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash.hash(state);
    }
}

impl PartialEq for CachedPath {
    fn eq(&self, other: &Self) -> bool {
        self.0.path == other.0.path
    }
}
impl Eq for CachedPath {}

impl Deref for CachedPath {
    type Target = CachedPathImpl;

    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl<'a> Borrow<dyn CacheKey + 'a> for CachedPath {
    fn borrow(&self) -> &(dyn CacheKey + 'a) {
        self
    }
}

impl AsRef<CachedPathImpl> for CachedPath {
    fn as_ref(&self) -> &CachedPathImpl {
        self.0.as_ref()
    }
}

impl CacheKey for CachedPath {
    fn tuple(&self) -> (u64, &Path) {
        (self.hash, &self.path)
    }
}

pub struct CachedPathImpl {
    hash: u64,
    path: Box<Path>,
    parent: Option<CachedPath>,
    meta: OnceLock<Option<FileMetadata>>,
    symlink: OnceLock<Option<PathBuf>>,
    canonicalized: OnceLock<Option<PathBuf>>,
    node_modules: OnceLock<Option<CachedPath>>,
    package_json: OnceLock<Option<Arc<PackageJson>>>,
}

impl CachedPathImpl {
    fn new(hash: u64, path: Box<Path>, parent: Option<CachedPath>) -> Self {
        Self {
            hash,
            path,
            parent,
            meta: OnceLock::new(),
            symlink: OnceLock::new(),
            canonicalized: OnceLock::new(),
            node_modules: OnceLock::new(),
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

    fn symlink<Fs: FileSystem>(&self, fs: &Fs) -> io::Result<Option<PathBuf>> {
        self.symlink
            .get_or_try_init(|| {
                if let Ok(symlink_metadata) = fs.symlink_metadata(&self.path) {
                    if symlink_metadata.is_symlink {
                        return fs.canonicalize(self.path()).map(Some);
                    }
                }
                Ok(None)
            })
            .cloned()
    }

    pub fn realpath<Fs: FileSystem>(&self, fs: &Fs) -> io::Result<PathBuf> {
        self.canonicalized
            .get_or_try_init(|| {
                if let Some(link) = self.symlink(fs)? {
                    return Ok(Some(link));
                }
                if let Some(parent) = self.parent() {
                    let parent_path = parent.realpath(fs)?;
                    return Ok(Some(
                        parent_path.normalize_with(self.path.strip_prefix(&parent.path).unwrap()),
                    ));
                };
                Ok(None)
            })
            .cloned()
            .map(|r| r.unwrap_or_else(|| self.path.clone().to_path_buf()))
    }

    pub fn module_directory<Fs: FileSystem + Default>(
        &self,
        module_name: &str,
        cache: &Cache<Fs>,
    ) -> Option<CachedPath> {
        let cached_path = cache.value(&self.path.join(module_name));
        cached_path.is_dir(&cache.fs).then(|| cached_path)
    }

    pub fn cached_node_modules<Fs: FileSystem + Default>(
        &self,
        cache: &Cache<Fs>,
    ) -> Option<CachedPath> {
        self.node_modules.get_or_init(|| self.module_directory("node_modules", cache)).clone()
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

/// Memoized cache key, code adapted from <https://stackoverflow.com/a/50478038>.
trait CacheKey {
    fn tuple(&self) -> (u64, &Path);
}

impl Hash for dyn CacheKey + '_ {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.tuple().0.hash(state);
    }
}

impl PartialEq for dyn CacheKey + '_ {
    fn eq(&self, other: &Self) -> bool {
        self.tuple().1 == other.tuple().1
    }
}

impl Eq for dyn CacheKey + '_ {}

impl<'a> CacheKey for (u64, &'a Path) {
    fn tuple(&self) -> (u64, &Path) {
        (self.0, self.1)
    }
}

impl<'a> Borrow<dyn CacheKey + 'a> for (u64, &'a Path) {
    fn borrow(&self) -> &(dyn CacheKey + 'a) {
        self
    }
}

/// Since the cache key is memoized, use an identity hasher
/// to avoid double cache.
#[derive(Default)]
struct IdentityHasher(u64);

impl Hasher for IdentityHasher {
    fn write(&mut self, _: &[u8]) {
        unreachable!("Invalid use of IdentityHasher")
    }
    fn write_u64(&mut self, n: u64) {
        self.0 = n;
    }
    fn finish(&self) -> u64 {
        self.0
    }
}
