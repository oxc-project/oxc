mod enhanced_resolve;
mod error_handling;

use std::path::Path;

use oxc_resolver::{ResolveResult, Resolver};

/// Test the resolve function
///
/// # Errors
/// * ResolveResult
///
/// # Panics
/// * request is empty
pub fn resolve<P: AsRef<Path>>(path: P, request: &str) -> ResolveResult {
    assert!(!request.is_empty());
    Resolver::new().resolve(path.as_ref(), request)
}
