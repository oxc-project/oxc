mod enhanced_resolve;

use std::path::Path;

use oxc_resolver::{ResolveResult, Resolver};

/// # Errors
/// # Panics
pub fn resolve<P: AsRef<Path>>(path: P, request: &str) -> ResolveResult {
    let path = path.as_ref();
    assert!(!request.is_empty());
    let resolver = Resolver::new();
    resolver.resolve(path, request)
}
