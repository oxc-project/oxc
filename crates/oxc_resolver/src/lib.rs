//! # Oxc Resolver
//!
//! Tests ported from [enhanced-resolve](https://github.com/webpack/enhanced-resolve).
//
use std::path::{Path, PathBuf};
#[derive(Debug, Eq, PartialEq)]
pub struct ResolveError;

pub type ResolveResult = Result<PathBuf, ResolveError>;

pub struct Resolver;

impl Resolver {
    pub fn new() -> Self {
        Self
    }

    /// # Errors
    pub fn resolve<P: AsRef<Path>>(&self, path: P, request: &str) -> ResolveResult {
        self.resolve_impl(path.as_ref(), request)
    }

    #[allow(clippy::unused_self)]
    fn resolve_impl(&self, _path: &Path, _request: &str) -> ResolveResult {
        unreachable!()
        // let path = path.join(request).canonicalize().unwrap();
        // Ok(path)
    }
}
