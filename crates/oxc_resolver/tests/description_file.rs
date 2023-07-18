//! FYI webpack does not have any tests for description files

use std::env;

use oxc_resolver::{Resolution, ResolveError, ResolveOptions, Resolver};

#[test]
fn no_description_file() {
    let f = env::current_dir().unwrap().join("tests/enhanced_resolve");

    let resolver = Resolver::default();
    assert_eq!(
        resolver.resolve(&f, ".").map(Resolution::into_path_buf),
        Ok(f.join("lib/index.js"))
    );

    let resolver =
        Resolver::new(ResolveOptions { description_files: vec![], ..ResolveOptions::default() });
    assert_eq!(resolver.resolve(&f, "."), Err(ResolveError::NotFound(f.into_boxed_path())));
}
