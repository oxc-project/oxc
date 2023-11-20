use std::path::{Path, PathBuf};

use napi_derive::napi;

use oxc_resolver::{ResolveOptions, Resolver};

#[napi(object)]
pub struct ResolveResult {
    pub path: Option<String>,
    pub error: Option<String>,
}

#[napi]
pub struct ResolverFactory {
    resolver: Resolver,
}

impl Default for ResolverFactory {
    fn default() -> Self {
        Self::new()
    }
}

#[napi]
impl ResolverFactory {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self { resolver: Resolver::new(ResolveOptions::default()) }
    }

    #[allow(clippy::needless_pass_by_value)]
    #[napi]
    pub fn sync(&self, path: String, request: String) -> ResolveResult {
        let path = PathBuf::from(path);
        resolve(&self.resolver, &path, &request)
    }
}

fn resolve(resolver: &Resolver, path: &Path, request: &str) -> ResolveResult {
    match resolver.resolve(path, request) {
        Ok(resolution) => ResolveResult {
            path: Some(resolution.full_path().to_string_lossy().to_string()),
            error: None,
        },
        Err(err) => ResolveResult { path: None, error: Some(err.to_string()) },
    }
}

#[allow(clippy::needless_pass_by_value)]
#[napi]
pub fn sync(path: String, request: String) -> ResolveResult {
    let path = PathBuf::from(path);
    let resolver = Resolver::new(ResolveOptions::default());
    resolve(&resolver, &path, &request)
}
