//! # Oxc Resolver
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

pub use crate::{
    cache::Cache,
    error::{JSONError, ResolveError},
    file_system::{FileMetadata, FileSystem, FileSystemOs},
    options::{AliasValue, ResolveOptions},
    resolution::Resolution,
};
use crate::{
    path::PathUtil,
    request::{Request, RequestPath},
};

pub type ResolveResult = Result<Resolution, ResolveError>;
type ResolveState = Result<Option<PathBuf>, ResolveError>;

/// Resolver with the current operating system as the file system
pub type Resolver = ResolverGeneric<FileSystemOs>;

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
        request_str: &str,
    ) -> Result<Resolution, ResolveError> {
        let path = path.as_ref();
        let request = Request::parse(request_str).map_err(ResolveError::Request)?;
        let path = if let Some(path) = self.load_alias(path, request.path.as_str())? {
            path
        } else {
            self.require(path, &request)?
        };
        Ok(Resolution {
            path,
            query: request.query.map(ToString::to_string),
            fragment: request.fragment.map(ToString::to_string),
        })
    }

    fn require(&self, path: &Path, request: &Request) -> Result<PathBuf, ResolveError> {
        // X: request
        // Y: path
        // require(X) from module at path Y
        let mut path = path;
        let request_str;

        match request.path {
            // 1. If X is a core module,
            //    a. return the core module
            //    b. STOP
            // 2. If X begins with '/'
            //    a. set Y to be the file system root
            RequestPath::Absolute(absolute_path) => {
                path = Path::new("/");
                request_str = absolute_path;
            }
            // 3. If X begins with './' or '/' or '../'
            RequestPath::Relative(relative_path) => {
                if let Some(path) = self.load_package_self(path, relative_path)? {
                    return Ok(path);
                }
                let path = path.normalize_with(relative_path);
                // a. LOAD_AS_FILE(Y + X)
                if !relative_path.ends_with('/') {
                    if let Some(path) = self.load_as_file(&path)? {
                        return Ok(path);
                    }
                }
                // b. LOAD_AS_DIRECTORY(Y + X)
                if let Some(path) = self.load_as_directory(&path)? {
                    return Ok(path);
                }
                // c. THROW "not found"
                return Err(ResolveError::NotFound(path.into_boxed_path()));
            }
            // 4. If X begins with '#'
            RequestPath::Hash(hash_path) => {
                request_str = hash_path;
            }
            //    a. LOAD_PACKAGE_IMPORTS(X, dirname(Y))
            RequestPath::Module(module_path) => {
                request_str = module_path;
            }
        }
        // 5. LOAD_PACKAGE_SELF(X, dirname(Y))
        if let Some(path) = self.load_package_self(path, request_str)? {
            return Ok(path);
        }
        // 6. LOAD_NODE_MODULES(X, dirname(Y))
        if let Some(path) = self.load_node_modules(path, request_str)? {
            return Ok(path);
        }
        if let Some(path) = self.load_as_file(&path.join(request_str))? {
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

    #[allow(clippy::unnecessary_wraps)]
    fn load_index(&self, path: &Path) -> ResolveState {
        for main_field in &self.options.main_files {
            let main_path = path.join(main_field);
            if !self.options.enforce_extension && self.cache.is_file(&main_path) {
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
        // 1. If X/package.json is a file,
        let package_json_path = path.join("package.json");
        if self.cache.is_file(&package_json_path) {
            // a. Parse X/package.json, and look for "main" field.
            let package_json = self.cache.read_package_json(&package_json_path)?;
            // b. If "main" is a falsy value, GOTO 2.
            if let Some(main_field) = &package_json.main {
                // c. let M = X + (json main field)
                let main_field_path = path.normalize_with(main_field);
                // d. LOAD_AS_FILE(M)
                if let Some(path) = self.load_as_file(&main_field_path)? {
                    return Ok(Some(path));
                }
                // e. LOAD_INDEX(M)
                if let Some(path) = self.load_index(&main_field_path)? {
                    return Ok(Some(path));
                }
                // f. LOAD_INDEX(X) DEPRECATED
                // g. THROW "not found"
                return Err(ResolveError::NotFound(main_field_path.into_boxed_path()));
            }
        }
        // 2. LOAD_INDEX(X)
        self.load_index(path)
    }

    fn load_node_modules(&self, start: &Path, request_str: &str) -> ResolveState {
        // 1. let DIRS = NODE_MODULES_PATHS(START)
        let dirs = self.node_module_paths(start);
        // 2. for each DIR in DIRS:
        for node_module_path in dirs {
            let node_module_path = node_module_path.join(request_str);
            for main_file in &self.options.main_files {
                if let Some(path) = self.load_package_self(&node_module_path, main_file)? {
                    return Ok(Some(path));
                }
            }
            // a. LOAD_PACKAGE_EXPORTS(X, DIR)
            // b. LOAD_AS_FILE(DIR/X)
            if !request_str.ends_with('/') {
                if let Some(path) = self.load_as_file(&node_module_path)? {
                    return Ok(Some(path));
                }
            }
            // c. LOAD_AS_DIRECTORY(DIR/X)
            if let Some(path) = self.load_as_directory(&node_module_path)? {
                return Ok(Some(path));
            }
        }
        Ok(None)
    }

    fn node_module_paths<'a>(&'a self, path: &'a Path) -> impl Iterator<Item = PathBuf> + 'a {
        path.ancestors()
            .flat_map(|path| self.options.modules.iter().map(|module| path.join(module)))
    }

    fn load_package_self(&self, path: &Path, request_str: &str) -> ResolveState {
        for dir in path.ancestors() {
            let package_json_path = dir.join("package.json");
            if self.cache.is_file(&package_json_path) {
                let package_json = self.cache.read_package_json(&package_json_path)?;
                if let Some(request_str) =
                    package_json.resolve_request(path, request_str, &self.options.extensions)?
                {
                    let request = Request::parse(request_str).map_err(ResolveError::Request)?;
                    // TODO: Do we need to pass query and fragment?
                    return self.require(dir, &request).map(Some);
                }
            }
        }
        Ok(None)
    }

    fn load_alias(&self, path: &Path, request_str: &str) -> ResolveState {
        for (alias, requests) in &self.options.alias {
            let exact_match = alias.strip_prefix(request_str).is_some_and(|c| c == "$");
            if request_str.starts_with(alias) || exact_match {
                for request in requests {
                    match request {
                        AliasValue::Path(new_request) => {
                            let new_request = if exact_match {
                                Cow::Borrowed(new_request)
                            } else {
                                Cow::Owned(request_str.replacen(alias, new_request, 1))
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
}
