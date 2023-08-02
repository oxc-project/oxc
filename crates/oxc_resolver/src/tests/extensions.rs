//! <https://github.com/webpack/enhanced-resolve/blob/main/test/extensions.test.js>

use std::path::PathBuf;

use crate::{Resolution, ResolveError, ResolveOptions, Resolver};

fn fixture() -> PathBuf {
    super::fixture().join("extensions")
}

#[test]
fn extensions() {
    let f = fixture();

    let resolver = Resolver::new(ResolveOptions {
        extensions: vec![".ts".into(), ".js".into()],
        ..ResolveOptions::default()
    });

    #[rustfmt::skip]
    let pass = [
        ("should resolve according to order of provided extensions", "./foo", "foo.ts"),
        ("should resolve according to order of provided extensions (dir index)", "./dir", "dir/index.ts"),
        ("should resolve according to main field in module root", ".", "index.js"),
        ("should resolve single file module before directory", "module", "node_modules/module.js"),
        ("should resolve trailing slash directory before single file", "module/", "node_modules/module/index.ts"),
    ];

    for (comment, request, expected_path) in pass {
        let resolved_path = resolver.resolve(&f, request).map(Resolution::full_path);
        let expected = f.join(expected_path);
        assert_eq!(resolved_path, Ok(expected), "{comment} {request} {expected_path}");
    }

    #[rustfmt::skip]
    let fail = [
        ("not resolve to file when request has a trailing slash (relative)", "./foo.js/", f.join("foo.js"))
    ];

    for (comment, request, expected_error) in fail {
        let resolution = resolver.resolve(&f, request);
        let error = ResolveError::NotFound(expected_error.into_boxed_path());
        assert_eq!(resolution, Err(error), "{comment} {request} {resolution:?}");
    }
}

#[test]
// should default enforceExtension to true when extensions includes an empty string
fn default_enforce_extension() {
    let f = fixture();

    let resolved = Resolver::new(ResolveOptions {
        extensions: vec![".ts".into(), String::new(), ".js".into()],
        ..ResolveOptions::default()
    })
    .resolve(&f, "./foo");

    assert_eq!(resolved, Err(ResolveError::NotFound(f.join("foo").into_boxed_path())));
    // TODO: need to match missingDependencies returned from the resolve function
}

#[test]
// should respect enforceExtension when extensions includes an empty string
fn respect_enforce_extension() {
    let f = fixture();

    let resolved = Resolver::new(ResolveOptions {
        enforce_extension: Some(false),
        extensions: vec![".ts".into(), String::new(), ".js".into()],
        ..ResolveOptions::default()
    })
    .resolve(&f, "./foo");
    assert_eq!(resolved.map(Resolution::into_path_buf), Ok(f.join("foo.ts")));
    // TODO: need to match missingDependencies returned from the resolve function
}
