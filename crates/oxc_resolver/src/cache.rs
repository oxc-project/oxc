use once_cell::sync::OnceCell as OnceLock;
use std::{
    borrow::{Borrow, Cow},
    collections::VecDeque,
    convert::AsRef,
    hash::{BuildHasherDefault, Hash, Hasher},
    io,
    ops::Deref,
    path::{Component, Path, PathBuf},
    sync::Arc,
};

use dashmap::{DashMap, DashSet};
use rustc_hash::FxHasher;

use crate::{
    package_json::PackageJson, FileMetadata, FileSystem, ResolveError, ResolveOptions, TsConfig,
};

#[derive(Default)]
pub struct Cache<Fs> {
    pub(crate) fs: Fs,
    cache: DashSet<CachedPath, BuildHasherDefault<FxHasher>>,
    tsconfigs: DashMap<PathBuf, Arc<TsConfig>, BuildHasherDefault<FxHasher>>,
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
        if let Some(cache_entry) = self.cache.get(path) {
            return cache_entry.clone();
        }
        let parent = path.parent().map(|p| self.value(p));
        let data =
            CachedPath(Arc::new(CachedPathImpl::new(path.to_path_buf().into_boxed_path(), parent)));
        self.cache.insert(data.clone());
        data
    }

    pub fn tsconfig(
        &self,
        tsconfig_path: &CachedPath,
        callback: impl FnOnce(&mut TsConfig) -> Result<(), ResolveError>, // callback for modifying tsconfig with `extends`
    ) -> Result<Arc<TsConfig>, ResolveError> {
        self.tsconfigs
            .entry(tsconfig_path.path().to_path_buf())
            .or_try_insert_with(|| {
                let tsconfig_path = if tsconfig_path.is_dir(&self.fs) {
                    Cow::Owned(tsconfig_path.path().join("tsconfig.json"))
                } else {
                    Cow::Borrowed(tsconfig_path.path())
                };
                let mut tsconfig_string = self
                    .fs
                    .read_to_string(&tsconfig_path)
                    .map_err(|_| ResolveError::NotFound(tsconfig_path.to_path_buf()))?;
                let mut tsconfig =
                    TsConfig::parse(&tsconfig_path, &mut tsconfig_string).map_err(|error| {
                        ResolveError::from_serde_json_error(tsconfig_path.to_path_buf(), &error)
                    })?;
                callback(&mut tsconfig)?;
                Ok(Arc::new(tsconfig))
            })
            .map(|r| Arc::clone(r.value()))
    }

    // Code copied from parcel
    // <https://github.com/parcel-bundler/parcel/blob/cd0edbccaafeacd2203a34e34570f45e2a10f028/packages/utils/node-resolver-rs/src/path.rs#L64>
    fn canonicalize(&self, path: &Path) -> io::Result<PathBuf> {
        let mut ret = PathBuf::new();
        let mut seen_links = 0;
        let mut queue = VecDeque::new();
        queue.push_back(path.to_path_buf());
        while let Some(cur_path) = queue.pop_front() {
            let mut components = cur_path.components();
            for component in &mut components {
                match component {
                    Component::Prefix(c) => ret.push(c.as_os_str()),
                    Component::RootDir => {
                        ret.push(component.as_os_str());
                    }
                    Component::CurDir => {}
                    Component::ParentDir => {
                        ret.pop();
                    }
                    Component::Normal(c) => {
                        ret.push(c);
                        let cached_path = self.value(&ret);
                        let Some(link) = cached_path.symlink(&self.fs)? else {
                            continue;
                        };
                        seen_links += 1;
                        if seen_links > 32 {
                            return Err(io::Error::new(
                                io::ErrorKind::NotFound,
                                "Too many symlinks",
                            ));
                        }
                        if link.is_absolute() {
                            ret = PathBuf::new();
                        } else {
                            ret.pop();
                        }
                        let remaining = components.as_path();
                        if !remaining.as_os_str().is_empty() {
                            queue.push_front(remaining.to_path_buf());
                        }
                        queue.push_front(link);
                        break;
                    }
                }
            }
        }
        Ok(ret)
    }
}

#[derive(Clone)]
pub struct CachedPath(Arc<CachedPathImpl>);

impl Hash for CachedPath {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.path.hash(state);
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

impl Borrow<Path> for CachedPath {
    fn borrow(&self) -> &Path {
        &self.0.path
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
    canonicalized: OnceLock<PathBuf>,
    node_modules: OnceLock<Option<CachedPath>>,
    package_json: OnceLock<Option<Arc<PackageJson>>>,
}

impl CachedPathImpl {
    fn new(path: Box<Path>, parent: Option<CachedPath>) -> Self {
        Self {
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
                        return fs.read_link(self.path()).map(Some);
                    }
                }
                Ok(None)
            })
            .cloned()
    }

    pub fn canonicalize<Fs: FileSystem>(&self, cache: &Cache<Fs>) -> io::Result<PathBuf> {
        self.canonicalized.get_or_try_init(|| cache.canonicalize(&self.path)).cloned()
    }

    pub fn module_directory<Fs: FileSystem>(
        &self,
        module_name: &str,
        cache: &Cache<Fs>,
    ) -> Option<CachedPath> {
        let cached_path = cache.value(&self.path.join(module_name));
        cached_path.is_dir(&cache.fs).then(|| cached_path)
    }

    pub fn cached_node_modules<Fs: FileSystem>(&self, cache: &Cache<Fs>) -> Option<CachedPath> {
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
