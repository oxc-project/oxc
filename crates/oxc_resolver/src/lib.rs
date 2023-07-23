//! # Oxc Resolver
//!
//! Node.js Module Resolution.
//!
//! ## References:
//!
//! * Tests ported from [enhanced-resolve](https://github.com/webpack/enhanced-resolve)
//! * Algorithm adapted from [Node.js Module Resolution Algorithm](https://nodejs.org/api/modules.html#all-together) and [cjs loader](https://github.com/nodejs/node/blob/main/lib/internal/modules/cjs/loader.js)
//! * Some code adapted from [parcel-resolver](https://github.com/parcel-bundler/parcel/blob/v2/packages/utils/node-resolver-rs)

mod cache;
mod error;
mod file_system;
mod options;
mod package_json;
mod path;
mod request;
mod resolution;

use std::{
    borrow::Cow,
    ffi::OsStr,
    path::{Path, PathBuf},
};

use crate::{
    cache::Cache,
    file_system::FileSystemOs,
    package_json::PackageJson,
    path::PathUtil,
    request::{Request, RequestPath},
};
pub use crate::{
    error::{JSONError, ResolveError},
    file_system::{FileMetadata, FileSystem},
    options::{Alias, AliasValue, ResolveOptions},
    resolution::Resolution,
};

type ResolveState = Result<Option<PathBuf>, ResolveError>;

/// Resolver with the current operating system as the file system
pub type Resolver = ResolverGeneric<FileSystemOs>;

/// Generic implementation of the resolver, backed by a cached file system.
pub struct ResolverGeneric<Fs> {
    options: ResolveOptions,
    cache: Cache<Fs>,
}

impl<Fs: FileSystem> Default for ResolverGeneric<Fs> {
    fn default() -> Self {
        Self::new(ResolveOptions::default())
    }
}

impl<Fs: FileSystem> ResolverGeneric<Fs> {
    pub fn new(options: ResolveOptions) -> Self {
        Self { options: options.sanitize(), cache: Cache::default() }
    }

    pub fn new_with_file_system(options: ResolveOptions, file_system: Fs) -> Self {
        Self { cache: Cache::new(file_system), ..Self::new(options) }
    }

    /// Resolve `request` at `path`
    ///
    /// # Errors
    ///
    /// * See [ResolveError]
    pub fn resolve<P: AsRef<Path>>(
        &self,
        path: P,
        request: &str,
    ) -> Result<Resolution, ResolveError> {
        let path = path.as_ref();
        let request = Request::parse(request).map_err(ResolveError::Request)?;
        let path = if let Some(path) =
            self.load_alias(path, request.path.as_str(), &self.options.alias)?
        {
            path
        } else {
            let result = self.require(path, &request);
            if result.as_ref().is_err_and(ResolveError::is_not_found) {
                if let Some(path) =
                    self.load_alias(path, request.path.as_str(), &self.options.fallback)?
                {
                    path
                } else {
                    result?
                }
            } else {
                result?
            }
        };
        let path = self.load_symlink(&path).unwrap_or(path);
        Ok(Resolution {
            path,
            query: request.query.map(ToString::to_string),
            fragment: request.fragment.map(ToString::to_string),
        })
    }

    /// require(X) from module at path Y
    /// X: request
    /// Y: path
    fn require(&self, path: &Path, request: &Request) -> Result<PathBuf, ResolveError> {
        match request.path {
            // 1. If X is a core module,
            //    a. return the core module
            //    b. STOP
            // 2. If X begins with '/'
            //    a. set Y to be the file system root
            RequestPath::Absolute(absolute_path) => {
                if !self.options.prefer_relative && self.options.prefer_absolute {
                    if let Ok(path) = self.require_path(path, absolute_path) {
                        return Ok(path);
                    }
                }
                self.load_roots(path, absolute_path)
            }
            // 3. If X begins with './' or '/' or '../'
            RequestPath::Relative(relative_path) => self.require_relative(path, relative_path),
            // 4. If X begins with '#'
            RequestPath::Hash(hash_path) => self.require_path(path, hash_path),
            //    a. LOAD_PACKAGE_IMPORTS(X, dirname(Y))
            RequestPath::Module(module_path) => self.require_path(path, module_path),
        }
    }

    // 3. If X begins with './' or '/' or '../'
    fn require_relative(&self, path: &Path, request: &str) -> Result<PathBuf, ResolveError> {
        if let Some(path) = self.load_package_self(path, request)? {
            return Ok(path);
        }
        let path = path.normalize_with(request);
        // a. LOAD_AS_FILE(Y + X)
        if !request.ends_with('/') {
            if let Some(path) = self.load_as_file(&path)? {
                return Ok(path);
            }
        }
        // b. LOAD_AS_DIRECTORY(Y + X)
        if let Some(path) = self.load_as_directory(&path)? {
            return Ok(path);
        }
        // c. THROW "not found"
        Err(ResolveError::NotFound(path.into_boxed_path()))
    }

    fn require_path(&self, path: &Path, request: &str) -> Result<PathBuf, ResolveError> {
        let dirname = self.cache.dirname(path);
        // 5. LOAD_PACKAGE_SELF(X, dirname(Y))
        if let Some(path) = self.load_package_self(&dirname, request)? {
            return Ok(path);
        }
        // 6. LOAD_NODE_MODULES(X, dirname(Y))
        if let Some(path) = self.load_node_modules(&dirname, request)? {
            return Ok(path);
        }
        if let Some(path) = self.load_as_file(&path.join(request))? {
            return Ok(path);
        }
        // 7. THROW "not found"
        Err(ResolveError::NotFound(path.to_path_buf().into_boxed_path()))
    }

    #[allow(clippy::unnecessary_wraps)]
    fn load_as_file(&self, path: &Path) -> ResolveState {
        // enhanced-resolve feature: extension_alias
        if let Some(path) = self.load_extension_alias(path)? {
            return Ok(Some(path));
        }

        // 1. If X is a file, load X as its file extension format. STOP
        if self.cache.is_file(path) {
            return Ok(Some(path.to_path_buf()));
        }
        // 2. If X.js is a file, load X.js as JavaScript text. STOP
        // 3. If X.json is a file, parse X.json to a JavaScript Object. STOP
        // 4. If X.node is a file, load X.node as binary addon. STOP
        for extension in &self.options.extensions {
            let path_with_extension = path.with_extension(extension);
            if self.cache.is_file(&path_with_extension) {
                return Ok(Some(path_with_extension));
            }
        }
        Ok(None)
    }

    fn load_symlink(&self, path: &Path) -> Option<PathBuf> {
        if self.options.symlinks { self.cache.canonicalize(path) } else { None }
    }

    #[allow(clippy::unnecessary_wraps)]
    fn load_index(&self, path: &Path, package_json: Option<&PackageJson>) -> ResolveState {
        for main_field in &self.options.main_files {
            if let Some(package_json) = package_json {
                if let Some(path) = self.load_browser_field(path, main_field, package_json)? {
                    return Ok(Some(path));
                }
            }

            let main_path = path.join(main_field);
            if self.options.enforce_extension == Some(false) && self.cache.is_file(&main_path) {
                return Ok(Some(main_path));
            }
            // 1. If X/index.js is a file, load X/index.js as JavaScript text. STOP
            // 2. If X/index.json is a file, parse X/index.json to a JavaScript object. STOP
            // 3. If X/index.node is a file, load X/index.node as binary addon. STOP
            for extension in &self.options.extensions {
                let main_path_with_extension = main_path.with_extension(extension);
                if self.cache.is_file(&main_path_with_extension) {
                    return Ok(Some(main_path_with_extension));
                }
            }
        }
        Ok(None)
    }

    fn load_as_directory(&self, path: &Path) -> ResolveState {
        // TODO: Only package.json is supported, so warn about having other values
        // Checking for empty files is needed for omitting checks on package.json
        // 1. If X/package.json is a file,
        if !self.options.description_files.is_empty() {
            // a. Parse X/package.json, and look for "main" field.
            if let Some(package_json) = self.cache.get_package_json(path)? {
                // b. If "main" is a falsy value, GOTO 2.
                if let Some(main_field) = &package_json.main {
                    // c. let M = X + (json main field)
                    let main_field_path = path.normalize_with(main_field);
                    // d. LOAD_AS_FILE(M)
                    if let Some(path) = self.load_as_file(&main_field_path)? {
                        return Ok(Some(path));
                    }
                    // e. LOAD_INDEX(M)
                    if let Some(path) =
                        self.load_index(&main_field_path, Some(package_json.as_ref()))?
                    {
                        return Ok(Some(path));
                    }
                    // f. LOAD_INDEX(X) DEPRECATED
                    // g. THROW "not found"
                    return Err(ResolveError::NotFound(main_field_path.into_boxed_path()));
                }

                if let Some(path) = self.load_index(path, Some(package_json.as_ref()))? {
                    return Ok(Some(path));
                }
            }
        }
        // 2. LOAD_INDEX(X)
        self.load_index(path, None)
    }

    fn load_node_modules(&self, start: &Path, request: &str) -> ResolveState {
        // 1. let DIRS = NODE_MODULES_PATHS(START)
        let dirs = self.node_module_paths(start);
        // 2. for each DIR in DIRS:
        for node_module_path in dirs {
            // a. LOAD_PACKAGE_EXPORTS(X, DIR)
            if let Some(path) = self.load_package_exports(&node_module_path, request)? {
                return Ok(Some(path));
            }

            let node_module_file = node_module_path.join(request);
            // b. LOAD_AS_FILE(DIR/X)
            if !request.ends_with('/') {
                if let Some(path) = self.load_as_file(&node_module_file)? {
                    return Ok(Some(path));
                }
            }
            // c. LOAD_AS_DIRECTORY(DIR/X)
            if self.cache.is_dir(&node_module_file) {
                if let Some(path) = self.load_as_directory(&node_module_file)? {
                    return Ok(Some(path));
                }
            }
        }
        Ok(None)
    }

    fn node_module_paths<'a>(&'a self, path: &'a Path) -> impl Iterator<Item = PathBuf> + 'a {
        path.ancestors()
            .flat_map(|path| self.options.modules.iter().map(|module| path.join(module)))
    }

    #[allow(clippy::unnecessary_wraps, clippy::unused_self)]
    fn load_package_exports(&self, _path: &Path, _request: &str) -> ResolveState {
        // 1. Try to interpret X as a combination of NAME and SUBPATH where the name
        //    may have a @scope/ prefix and the subpath begins with a slash (`/`).
        // 2. If X does not match this pattern or DIR/NAME/package.json is not a file,
        //    return.
        // 3. Parse DIR/NAME/package.json, and look for "exports" field.
        // 4. If "exports" is null or undefined, return.
        // 5. let MATCH = PACKAGE_EXPORTS_RESOLVE(pathToFileURL(DIR/NAME), "." + SUBPATH,
        //    `package.json` "exports", ["node", "require"]) defined in the ESM resolver.
        // 6. RESOLVE_ESM_MATCH(MATCH)
        Ok(None)
    }

    /// # Panics
    ///
    /// * Parent of package.json is None
    fn load_package_self(&self, path: &Path, request: &str) -> ResolveState {
        if let Some(package_json) = self.cache.find_package_json(path)? {
            if let Some(path) = self.load_browser_field(path, request, &package_json)? {
                return Ok(Some(path));
            }
        }
        Ok(None)
    }

    fn load_browser_field(
        &self,
        path: &Path,
        request: &str,
        package_json: &PackageJson,
    ) -> ResolveState {
        if let Some(request) = package_json.resolve(path, request, &self.options.extensions)? {
            let request = Request::parse(request).map_err(ResolveError::Request)?;
            debug_assert!(package_json.path.file_name().is_some_and(|x| x == "package.json"));
            // TODO: Do we need to pass query and fragment?
            return self.require(package_json.path.parent().unwrap(), &request).map(Some);
        }
        Ok(None)
    }

    fn load_alias(&self, path: &Path, request: &str, alias: &Alias) -> ResolveState {
        for (alias, requests) in alias {
            let exact_match = alias.strip_prefix(request).is_some_and(|c| c == "$");
            if request.starts_with(alias) || exact_match {
                for r in requests {
                    match r {
                        AliasValue::Path(new_request) => {
                            let new_request = if exact_match {
                                Cow::Borrowed(new_request)
                            } else {
                                Cow::Owned(request.replacen(alias, new_request, 1))
                            };
                            let new_request =
                                Request::parse(&new_request).map_err(ResolveError::Request)?;
                            match self.require(path, &new_request) {
                                Err(ResolveError::NotFound(_)) => { /* noop */ }
                                Ok(path) => return Ok(Some(path)),
                                Err(err) => return Err(err),
                            }
                        }
                        AliasValue::Ignore => {
                            return Err(ResolveError::Ignored(path.join(alias).into_boxed_path()));
                        }
                    }
                }
                return Err(ResolveError::Alias(alias.clone()));
            }
        }
        Ok(None)
    }

    /// Given an extension alias map `{".js": [".ts", "js"]}`,
    /// load the mapping instead of the provided extension
    ///
    /// This is an enhanced-resolve feature
    ///
    /// # Errors
    ///
    /// * [ResolveError::ExtensionAlias]: When all of the aliased extensions are not found
    fn load_extension_alias(&self, path: &Path) -> ResolveState {
        let Some(path_extension) = path.extension() else { return Ok(None) };
        let Some((_, extensions)) =
            self.options.extension_alias.iter().find(|(ext, _)| OsStr::new(ext) == path_extension)
        else {
            return Ok(None);
        };
        for extension in extensions {
            let path_with_extension = path.with_extension(extension);
            if self.cache.is_file(&path_with_extension) {
                return Ok(Some(path_with_extension));
            }
        }
        Err(ResolveError::ExtensionAlias)
    }

    fn load_roots(&self, path: &Path, request: &str) -> Result<PathBuf, ResolveError> {
        debug_assert!(request.starts_with('/'));
        if self.options.roots.is_empty() {
            return self.require_path(Path::new("/"), request);
        }
        for root in &self.options.roots {
            if let Ok(path) = self.require_relative(root, request.trim_start_matches('/')) {
                return Ok(path);
            }
        }
        Err(ResolveError::NotFound(path.to_path_buf().into_boxed_path()))
    }
}
