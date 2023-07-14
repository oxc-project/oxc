//! # Oxc Resolver
//!
//! Tests ported from [enhanced-resolve](https://github.com/webpack/enhanced-resolve).
//!
//! Algorithm from <https://nodejs.org/api/modules.html#all-together>.

mod error;
mod file_system;
mod options;
mod package_json;
mod path;
mod request;

use std::{
    fs,
    path::{Path, PathBuf},
};

pub use crate::{
    error::{JSONError, ResolveError},
    options::ResolveOptions,
};
use crate::{file_system::FileSystem, package_json::PackageJson, path::PathUtil, request::Request};

pub type ResolveResult = Result<PathBuf, ResolveError>;
type ResolveState = Result<Option<PathBuf>, ResolveError>;

pub struct Resolver {
    options: ResolveOptions,
    fs: FileSystem,
}

impl Default for Resolver {
    fn default() -> Self {
        Self::new(ResolveOptions::default())
    }
}

impl Resolver {
    pub fn new(options: ResolveOptions) -> Self {
        Self { options: options.sanitize(), fs: FileSystem::default() }
    }

    /// Resolve `request` at `path`
    ///
    /// # Errors
    ///
    /// * Will return `Err` for [ResolveError]
    pub fn resolve<P: AsRef<Path>>(&self, path: P, request: &str) -> ResolveResult {
        self.require(path.as_ref(), request)
    }

    fn require(&self, path: &Path, request: &str) -> ResolveResult {
        let request = Request::try_from(request).map_err(ResolveError::RequestError)?;
        match request {
            // 1. If X is a core module,
            //    a. return the core module
            //    b. STOP
            // 2. If X begins with '/'
            //    a. set Y to be the file system root
            // 4. If X begins with '#'
            //    a. LOAD_PACKAGE_IMPORTS(X, dirname(Y))
            Request::Absolute(_) => {
                unreachable!()
            }
            // 3. If X begins with './' or '/' or '../'
            Request::Relative(relative_path) => {
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
                Err(ResolveError::NotFound)
            }
            Request::Module(module_path) => {
                // 5. LOAD_PACKAGE_SELF(X, dirname(Y))
                // 6. LOAD_NODE_MODULES(X, dirname(Y))
                if let Some(path) = self.load_node_modules(path, module_path)? {
                    return Ok(path);
                }

                // 7. THROW "not found"
                Err(ResolveError::NotFound)
            }
        }
    }

    #[allow(clippy::unnecessary_wraps)]
    fn load_as_file(&self, path: &Path) -> ResolveState {
        // 1. If X is a file, load X as its file extension format. STOP
        if self.fs.is_file(path) {
            return Ok(Some(path.to_path_buf()));
        }
        // 2. If X.js is a file, load X.js as JavaScript text. STOP
        // 3. If X.json is a file, parse X.json to a JavaScript Object. STOP
        // 4. If X.node is a file, load X.node as binary addon. STOP
        for extension in &self.options.extensions {
            let path_with_extension = path.with_extension(extension);
            if self.fs.is_file(&path_with_extension) {
                return Ok(Some(path_with_extension));
            }
        }
        Ok(None)
    }

    #[allow(clippy::unused_self, clippy::unnecessary_wraps)]
    fn load_index(&self, path: &Path) -> ResolveState {
        // 1. If X/index.js is a file, load X/index.js as JavaScript text. STOP
        // 2. If X/index.json is a file, parse X/index.json to a JavaScript object. STOP
        // 3. If X/index.node is a file, load X/index.node as binary addon. STOP
        for extension in &self.options.extensions {
            let index_path = path.join("index").with_extension(extension);
            if self.fs.is_file(&index_path) {
                return Ok(Some(index_path));
            }
        }
        Ok(None)
    }

    fn load_as_directory(&self, path: &Path) -> ResolveState {
        // 1. If X/package.json is a file,
        let package_json_path = path.join("package.json");
        if self.fs.is_file(&package_json_path) {
            // a. Parse X/package.json, and look for "main" field.
            let package_json_string = fs::read_to_string(&package_json_path).unwrap();
            let package_json = PackageJson::try_from(package_json_string.as_str())
                .map_err(|error| ResolveError::from_serde_json_error(package_json_path, &error))?;
            // b. If "main" is a falsy value, GOTO 2.
            if let Some(main_field) = &package_json.main {
                // c. let M = X + (json main field)
                let main_field_path = path.normalize_with(main_field);
                // d. LOAD_AS_FILE(M)
                if let Some(path) = self.load_as_file(&main_field_path)? {
                    return Ok(Some(path));
                }
                // e. LOAD_INDEX(M)
                if let Some(path) = self.load_as_file(&main_field_path)? {
                    return Ok(Some(path));
                }
                // f. LOAD_INDEX(X) DEPRECATED
            }
            // g. THROW "not found"
            return Err(ResolveError::NotFound);
        }
        // 2. LOAD_INDEX(X)
        self.load_index(path)
    }

    fn load_node_modules(&self, start: &Path, module_path: &str) -> ResolveState {
        const NODE_MODULES: &str = "node_modules";
        // 1. let DIRS = NODE_MODULES_PATHS(START)
        let dirs = start
            .ancestors()
            .filter(|path| path.file_name().is_some_and(|name| name != NODE_MODULES));
        // 2. for each DIR in DIRS:
        for dir in dirs {
            // a. LOAD_PACKAGE_EXPORTS(X, DIR)
            // b. LOAD_AS_FILE(DIR/X)
            let node_module_path = dir.join(NODE_MODULES).join(module_path);
            if !module_path.ends_with('/') {
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
}
