//! https://github.com/webpack/enhanced-resolve/blob/main/test/exportsField.test.js
//!
//! The resolution tests are at the bottom of the file.

use oxc_resolver::{Resolution, ResolveError, ResolveOptions, Resolver};

#[test]
fn exports_field() {
    let f = super::fixture().join("exports-field");
    let f2 = super::fixture().join("exports-field2");
    let f4 = super::fixture().join("exports-field-error");
    let f5 = super::fixture().join("imports-exports-wildcard");

    let resolver = Resolver::new(ResolveOptions {
        extensions: vec![".js".into()],
        fully_specified: true,
        condition_names: vec!["webpack".into()],
        ..ResolveOptions::default()
    });

    #[rustfmt::skip]
    let pass = [
        ("resolve root using exports field, not a main field", f.clone(), "exports-field", f.join("node_modules/exports-field/x.js")),
        ("resolver should respect condition names", f.clone(), "exports-field/dist/main.js", f.join("node_modules/exports-field/lib/lib2/main.js")),
        // TODO: ("resolver should respect fallback", f2.clone(), "exports-field/dist/browser.js", f2.join("node_modules/exports-field/lib/browser.js")),
        // TODO: ("resolver should respect query parameters #1", f2.clone(), "exports-field/dist/browser.js?foo", f2.join("node_modules/exports-field/lib/browser.js?foo")),
        // TODO: ("resolver should respect fragment parameters #1", f2.clone(), "exports-field/dist/browser.js#foo", f2.join("node_modules/exports-field/lib/browser.js#foo")),
        ("relative path should work, if relative path as request is used", f.clone(), "./node_modules/exports-field/lib/main.js", f.join("node_modules/exports-field/lib/main.js")),
        ("self-resolving root", f.clone(), "@exports-field/core", f.join("a.js")),
        ("should resolve with wildcard pattern #1", f5.clone(), "m/features/f.js", f5.join("node_modules/m/src/features/f.js")),
        ("should resolve with wildcard pattern #2", f5.clone(), "m/features/y/y.js", f5.join("node_modules/m/src/features/y/y.js")),
        ("should resolve with wildcard pattern #3", f5.clone(), "m/features-no-ext/y/y.js", f5.join("node_modules/m/src/features/y/y.js")),
        ("should resolve with wildcard pattern #4", f5.clone(), "m/middle/nested/f.js", f5.join("node_modules/m/src/middle/nested/f.js")),
        ("should resolve with wildcard pattern #5", f5.clone(), "m/middle-1/nested/f.js", f5.join("node_modules/m/src/middle-1/nested/f.js")),
        ("should resolve with wildcard pattern #6", f5.clone(), "m/middle-2/nested/f.js", f5.join("node_modules/m/src/middle-2/nested/f.js")),
        ("should resolve with wildcard pattern #7", f5.clone(), "m/middle-3/nested/f", f5.join("node_modules/m/src/middle-3/nested/f/nested/f.js")),
        ("should resolve with wildcard pattern #8", f5.clone(), "m/middle-4/f/nested", f5.join("node_modules/m/src/middle-4/f/f.js")),
        ("should resolve with wildcard pattern #9", f5.clone(), "m/middle-5/f$/$", f5.join("node_modules/m/src/middle-5/f$/$.js")),
    ];

    // Not needed or snapshot:
    //   * should log the correct info

    for (comment, path, request, expected) in pass {
        let resolved_path = resolver.resolve(&path, request).map(Resolution::full_path);
        assert_eq!(resolved_path, Ok(expected), "{comment} {path:?} {request}");
    }

    #[rustfmt::skip]
    let fail = [
        ("throw error if extension not provided", f2.clone(), "exports-field/dist/main", ResolveError::NotFound(f2.join("node_modules/exports-field/lib/lib2/main").into_boxed_path())),
        // TODO: ("resolver should respect query parameters #2. Direct matching", f2.clone(), "exports-field?foo", ResolveError::NotFound(f2.join("").into_boxed_path())),
        // TODO: ("resolver should respect fragment parameters #2. Direct matching", f2.clone(), "exports-field#foo", ResolveError::NotFound(f2.join("").into_boxed_path())),
        ("relative path should not work with exports field", f.clone(), "./node_modules/exports-field/dist/main.js", ResolveError::NotFound(f.join("node_modules/exports-field/dist/main.js").into_boxed_path())),
        ("backtracking should not work for request", f.clone(), "exports-field/dist/../../../a.js", ResolveError::InvalidModuleSpecifier("../../../a.js".to_string())),
        ("backtracking should not work for exports field target", f.clone(), "exports-field/dist/a.js", ResolveError::InvalidPackageTarget("./../../a.js".to_string())),
        ("not exported error", f.clone(), "exports-field/anything/else", ResolveError::PackagePathNotExported("./anything/else".to_string())),
        ("request ending with slash #1", f.clone(), "exports-field/", ResolveError::PackagePathNotExported("./".to_string())),
        // TODO: ("request ending with slash #2", f.clone(), "exports-field/dist/", ResolveError::PackagePathNotExported("".to_string())),
        // TODO: ("request ending with slash #3", f.clone(), "exports-field/lib/", ResolveError::PackagePathNotExported("".to_string())),
        ("should throw error if target is invalid", f4, "exports-field", ResolveError::InvalidPackageTarget("./a/../b/../../pack1/index.js".to_string())),
        ("throw error if exports field is invalid", f.clone(), "invalid-exports-field", ResolveError::InvalidPackageConfig(f.join("node_modules/invalid-exports-field/package.json"))),
        ("should throw error if target is 'null'", f5, "m/features/internal/file.js", ResolveError::PackagePathNotExported("./features/internal/file.js".to_string())),
    ];

    for (comment, path, request, error) in fail {
        let resolution = resolver.resolve(&path, request);
        assert_eq!(resolution, Err(error), "{comment} {path:?} {request}");
    }
}

#[test]
// resolve using exports field, not a browser field #1
fn exports_not_browser_field1() {
    let f = super::fixture().join("exports-field");

    let resolver = Resolver::new(ResolveOptions {
        alias_fields: vec!["browser".into()],
        condition_names: vec!["webpack".into()],
        extensions: vec![".js".into()],
        ..ResolveOptions::default()
    });

    let resolved_path =
        resolver.resolve(&f, "exports-field/dist/main.js").map(Resolution::full_path);
    assert_eq!(resolved_path, Ok(f.join("node_modules/exports-field/lib/lib2/main.js")));
}

#[test]
// resolve using exports field and a browser alias field #2
fn exports_not_browser_field2() {
    let f2 = super::fixture().join("exports-field2");

    let resolver = Resolver::new(ResolveOptions {
        alias_fields: vec!["browser".into()],
        extensions: vec![".js".into()],
        condition_names: vec!["node".into()],
        ..ResolveOptions::default()
    });

    let resolved_path =
        resolver.resolve(&f2, "exports-field/dist/main.js").map(Resolution::full_path);
    assert_eq!(resolved_path, Ok(f2.join("node_modules/exports-field/lib/browser.js")));
}

#[test]
#[ignore = "fullSpecified"]
// should resolve extension without fullySpecified
fn extension_without_fully_specified() {
    let f2 = super::fixture().join("exports-field2");

    let commonjs_resolver = Resolver::new(ResolveOptions {
        extensions: vec![".js".into()],
        condition_names: vec!["webpack".into()],
        ..ResolveOptions::default()
    });

    let resolved_path =
        commonjs_resolver.resolve(&f2, "exports-field/dist/main").map(Resolution::full_path);
    assert_eq!(resolved_path, Ok(f2.join("node_modules/exports-field/lib/lib2/main.js")));
}

#[test]
#[ignore = "exports field name"]
// field name path #1 - #5
fn field_name() {}

#[test]
fn extension_alias_1_2() {
    let f = super::fixture().join("exports-field-and-extension-alias");

    let resolver = Resolver::new(ResolveOptions {
        extensions: vec![".js".into()],
        extension_alias: vec![(".js".into(), vec![".ts".into(), ".js".into()])],
        fully_specified: true,
        condition_names: vec!["webpack".into(), "default".into()],
        ..ResolveOptions::default()
    });

    #[rustfmt::skip]
    let pass = [
        ("should resolve with the `extensionAlias` option", f.clone(), "@org/pkg/string.js", f.join("node_modules/@org/pkg/dist/string.js")),
        ("should resolve with the `extensionAlias` option #2", f.clone(), "pkg/string.js", f.join("node_modules/pkg/dist/string.js")),
    ];

    for (comment, path, request, expected) in pass {
        let resolved_path = resolver.resolve(&path, request).map(Resolution::full_path);
        assert_eq!(resolved_path, Ok(expected), "{comment} {path:?} {request}");
    }
}

#[test]
fn extension_alias_3() {
    let f = super::fixture().join("exports-field-and-extension-alias");

    let resolver = Resolver::new(ResolveOptions {
        extensions: vec![".js".into()],
        extension_alias: vec![(
            ".js".into(),
            vec![".foo".into(), ".baz".into(), ".baz".into(), ".ts".into(), ".js".into()],
        )],
        fully_specified: true,
        condition_names: vec!["webpack".into(), "default".into()],
        ..ResolveOptions::default()
    });

    #[rustfmt::skip]
    let pass = [
        ("should resolve with the `extensionAlias` option #3", f.clone(), "pkg/string.js", f.join("node_modules/pkg/dist/string.js")),
    ];

    for (comment, path, request, expected) in pass {
        let resolved_path = resolver.resolve(&path, request).map(Resolution::full_path);
        assert_eq!(resolved_path, Ok(expected), "{comment} {path:?} {request}");
    }
}

#[test]
#[ignore]
fn extension_alias_throw_error() {
    let f = super::fixture().join("exports-field-and-extension-alias");

    let resolver = Resolver::new(ResolveOptions {
        extensions: vec![".js".into()],
        extension_alias: vec![(".js".into(), vec![".ts".into()])],
        fully_specified: true,
        condition_names: vec!["webpack".into(), "default".into()],
        ..ResolveOptions::default()
    });

    #[rustfmt::skip]
    let fail = [
        ("should throw error with the `extensionAlias` option", f.clone(), "pkg/string.js", ResolveError::PackagePathNotExported("node_modules/pkg/dist/string.js".to_string())),
        // They are exactly the same in enhanced-resolve
        ("should throw error with the `extensionAlias` option #2", f, "pkg/string.js", ResolveError::PackagePathNotExported("node_modules/pkg/dist/string.js".to_string())),
    ];

    for (comment, path, request, error) in fail {
        let resolution = resolver.resolve(&path, request);
        assert_eq!(resolution, Err(error), "{comment} {path:?} {request}");
    }
}
