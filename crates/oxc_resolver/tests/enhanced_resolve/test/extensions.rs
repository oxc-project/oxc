//! <https://github.com/webpack/enhanced-resolve/blob/main/test/extensions.test.js>

use std::{env, path::PathBuf};

use oxc_resolver::{ResolveOptions, Resolver};

fn fixture() -> PathBuf {
    env::current_dir().unwrap().join("tests/enhanced_resolve/test/fixtures/extensions")
}

#[test]
fn extensions() {
    let fixture = fixture();

    let options = ResolveOptions {
        extensions: vec![".ts".into(), ".js".into()],
        ..ResolveOptions::default()
    };

    let resolver = Resolver::new(options);

    let pass = [
        ("should resolve according to order of provided extensions", "./foo", "foo.ts"),
        (
            "should resolve according to order of provided extensions (dir index)",
            "./dir",
            "dir/index.ts",
        ),
        ("should resolve according to main field in module root", ".", "index.js"),
        ("should resolve single file module before directory", "module", "node_modules/module.js"),
        (
            "should resolve trailing slash directory before single file",
            "module/",
            "node_modules/module/index.ts",
        ),
    ];

    for (comment, request, expected_path) in pass {
        let resolved_path = resolver.resolve(&fixture, request).map(|p| p.canonicalize().unwrap());
        let expected = fixture.join(expected_path).canonicalize().unwrap();
        assert_eq!(resolved_path, Ok(expected), "{comment} {request} {expected_path}");
    }

    let fail = [("not resolve to file when request has a trailing slash (relative)", "./foo.js/")];

    for (comment, request) in fail {
        let resolved_path = resolver.resolve(&fixture, request);
        assert!(resolved_path.is_err(), "{comment} {request} {resolved_path:?}");
    }
}

#[test]
#[ignore = "need to match missingDependencies returned from the resolve function"]
fn default_enforce_extension() {
    // should default enforceExtension to true when extensions includes an empty string
    let fixture = fixture();

    let options = ResolveOptions {
        extensions: vec![".ts".into(), String::new(), ".js".into()],
        ..ResolveOptions::default()
    };

    let resolver = Resolver::new(options);
    let _resolved = resolver.resolve(fixture, "./foo");
}

#[test]
#[ignore = "need to match missingDependencies returned from the resolve function"]
fn respect_enforce_extension() {
    // should respect enforceExtension when extensions includes an empty string
    let fixture = fixture();

    let options = ResolveOptions {
        extensions: vec![".ts".into(), String::new(), ".js".into()],
        enforce_extension: false,
    };

    let resolver = Resolver::new(options);
    let _resolved = resolver.resolve(fixture, "./foo");
}
