mod enhanced_resolve;
mod error_handling;

use std::{env, path::Path, sync::Arc, thread};

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

#[test]
fn threaded_environment() {
    let cwd = env::current_dir().unwrap();
    let resolver = Arc::new(Resolver::new());
    for _ in 0..2 {
        _ = thread::spawn({
            let cwd = cwd.clone();
            let resolver = Arc::clone(&resolver);
            move || {
                _ = resolver.resolve(cwd, ".");
            }
        })
        .join();
    }
}
