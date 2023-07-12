//! # Oxc Resolver
//!
//! Tests ported from [enhanced-resolve](https://github.com/webpack/enhanced-resolve).
//!
//! Algorithm from <https://nodejs.org/api/modules.html#all-together>.

mod error;
mod path;
mod request;

use std::path::{Path, PathBuf};

pub use crate::error::ResolveError;
use crate::{path::ResolvePath, request::Request};

pub type ResolveResult = Result<PathBuf, ResolveError>;

pub struct Resolver;

impl Resolver {
    pub fn new() -> Self {
        Self
    }

    /// Resolve `request` at `path`
    ///
    /// # Errors
    ///
    /// * Will return `Err` for [ResolveError]
    pub fn resolve<P: AsRef<Path>>(&self, path: P, request: &str) -> ResolveResult {
        self.resolve_impl(path.as_ref(), request)
    }

    fn resolve_impl(&self, path: &Path, request: &str) -> ResolveResult {
        let request = Request::try_from(request).map_err(ResolveError::RequestError)?;
        let path = ResolvePath::from(path);

        match request {
            Request::Relative(_) => {
                let path = path.join(&request);
                self.load_as_file(&path).or_else(|_| self.load_as_directory(&path))
            }
            Request::Absolute(_) => {
                unreachable!()
            }
        }
    }

    #[allow(clippy::unused_self)]
    fn load_as_file(&self, path: &Path) -> ResolveResult {
        // 1. If X is a file, load X as its file extension format. STOP
        // 2. If X.js is a file, load X.js as JavaScript text. STOP
        if path.with_extension("js").exists() {
            return Ok(path.with_extension("js"));
        }
        // 3. If X.json is a file, parse X.json to a JavaScript Object. STOP
        // 4. If X.node is a file, load X.node as binary addon. STOP
        Err(ResolveError::NotFound)
    }

    fn load_as_directory(&self, path: &Path) -> ResolveResult {
        // 1. If X/package.json is a file,
        // a. Parse X/package.json, and look for "main" field.
        // b. If "main" is a falsy value, GOTO 2.
        // c. let M = X + (json main field)
        // d. LOAD_AS_FILE(M)
        // e. LOAD_INDEX(M)
        // f. LOAD_INDEX(X) DEPRECATED
        // g. THROW "not found"
        // 2. LOAD_INDEX(X)
        self.load_index(path)
    }

    #[allow(clippy::unused_self)]
    fn load_index(&self, path: &Path) -> ResolveResult {
        // 1. If X/index.js is a file, load X/index.js as JavaScript text. STOP
        if path.with_file_name("index.js").exists() {
            return Ok(path.with_file_name("index.js"));
        }
        // 2. If X/index.json is a file, parse X/index.json to a JavaScript object. STOP
        // 3. If X/index.node is a file, load X/index.node as binary addon. STOP
        Err(ResolveError::NotFound)
    }
}
