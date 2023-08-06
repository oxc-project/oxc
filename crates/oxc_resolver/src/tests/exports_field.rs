//! https://github.com/webpack/enhanced-resolve/blob/main/test/exportsField.test.js
//!
//! The huge exports field test cases are at the bottom of this file.

use crate::{
    ExportsField, PathUtil, Resolution, ResolveContext, ResolveError, ResolveOptions, Resolver,
};
use serde_json::json;
use std::path::Path;

#[test]
fn test() {
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
        // enhanced_resolve behaves differently to node.js. enhanced_resolve fallbacks when an
        // array item is unresolved, where as node.js fallbacks when an array has an
        // InvalidPackageTarget error.
        // ("resolver should respect fallback", f2.clone(), "exports-field/dist/browser.js", f2.join("node_modules/exports-field/lib/browser.js")),
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
        ("throw error if extension not provided", f2.clone(), "exports-field/dist/main", ResolveError::NotFound(f2.join("node_modules/exports-field/lib/lib2/main"))),
        // TODO: ("resolver should respect query parameters #2. Direct matching", f2.clone(), "exports-field?foo", ResolveError::NotFound(f2.join(""))),
        // TODO: ("resolver should respect fragment parameters #2. Direct matching", f2.clone(), "exports-field#foo", ResolveError::NotFound(f2.join(""))),
        ("relative path should not work with exports field", f.clone(), "./node_modules/exports-field/dist/main.js", ResolveError::NotFound(f.join("node_modules/exports-field/dist/main.js"))),
        ("backtracking should not work for request", f.clone(), "exports-field/dist/../../../a.js", ResolveError::InvalidPackageTarget("./lib/../../../a.js".to_string())),
        ("backtracking should not work for exports field target", f.clone(), "exports-field/dist/a.js", ResolveError::InvalidPackageTarget("./../../a.js".to_string())),
        ("not exported error", f.clone(), "exports-field/anything/else", ResolveError::PackagePathNotExported("./anything/else".to_string())),
        ("request ending with slash #1", f.clone(), "exports-field/", ResolveError::PackagePathNotExported("./".to_string())),
        ("request ending with slash #2", f.clone(), "exports-field/dist/", ResolveError::PackagePathNotExported("./dist/".to_string())),
        ("request ending with slash #3", f.clone(), "exports-field/lib/", ResolveError::PackagePathNotExported("./lib/".to_string())),
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

// #[test]
// field name path #1 - #5
// fn field_name() {}

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
        // enhanced-resolve has two test cases that are exactly the same here
        // https://github.com/webpack/enhanced-resolve/blob/a998c7d218b7a9ec2461fc4fddd1ad5dd7687485/test/exportsField.test.js#L2976-L3024
        ("should throw error with the `extensionAlias` option", f, "pkg/string.js", ResolveError::ExtensionAlias),
        // TODO: The error is PackagePathNotExported in enhanced_resolve
        // ("should throw error with the `extensionAlias` option", f.clone(), "pkg/string.js", ResolveError::PackagePathNotExported("node_modules/pkg/dist/string.ts".to_string())),
    ];

    for (comment, path, request, error) in fail {
        let resolution = resolver.resolve(&path, request);
        assert_eq!(resolution, Err(error), "{comment} {path:?} {request}");
    }
}

// Small script for generating the test cases from enhanced_resolve
// for (c of testCases) {
//  console.log("TestCase {")
//  console.log(`name: ${JSON.stringify(c.name)},`)
//	if (c.expect instanceof Error) {
//		console.log(`expect: None,`)
//	} else {
//		console.log(`expect: Some(vec!${JSON.stringify(c.expect)}),`)
//	}
//  console.log(`exports_field: exports_field(json!(${JSON.stringify(c.suite[0], null, 2)})),`)
//	console.log(`request: "${c.suite[1]}",`)
//  console.log(`condition_names: vec!${JSON.stringify(c.suite[2])},`)
//	console.log("},")
//}
struct TestCase {
    name: &'static str,
    expect: Option<Vec<&'static str>>,
    exports_field: ExportsField,
    request: &'static str,
    condition_names: Vec<&'static str>,
}

#[allow(clippy::needless_pass_by_value)]
fn exports_field(value: serde_json::Value) -> ExportsField {
    let s = serde_json::to_string(&value).unwrap();
    serde_json::from_str(&s).unwrap()
}

#[test]
fn test_cases() {
    let test_cases = [
        TestCase {
            name: "sample #1",
            expect: Some(vec!["./dist/test/file.js"]),
            exports_field: exports_field(json!({
                "./foo/": {
                    "import": [
                        "./dist/",
                        "./src/"
                    ],
                    "webpack": "./wp/"
                },
                ".": "./main.js"
            })),
            request: "./foo/test/file.js",
            condition_names: vec!["import", "webpack"],
        },
        // Duplicated due to not supporting returning an array
        TestCase {
            name: "sample #1",
            expect: Some(vec!["./src/test/file.js"]),
            exports_field: exports_field(json!({
                "./foo/": {
                    "import": [
                        "./src/"
                    ],
                    "webpack": "./wp/"
                },
                ".": "./main.js"
            })),
            request: "./foo/test/file.js",
            condition_names: vec!["import", "webpack"],
        },
        TestCase {
            name: "sample #1 (wildcard)",
            expect: Some(vec!["./dist/test/file.js"]),
            exports_field: exports_field(json!({
                "./foo/*": {
                    "import": [
                        "./dist/*",
                        "./src/*"
                    ],
                    "webpack": "./wp/*"
                },
                ".": "./main.js"
            })),
            request: "./foo/test/file.js",
            condition_names: vec!["import", "webpack"],
        },
        // Duplicated due to not supporting returning an array
        TestCase {
            name: "sample #1 (wildcard)",
            expect: Some(vec!["./src/test/file.js"]),
            exports_field: exports_field(json!({
                "./foo/*": {
                    "import": [
                        "./src/*"
                    ],
                    "webpack": "./wp/*"
                },
                ".": "./main.js"
            })),
            request: "./foo/test/file.js",
            condition_names: vec!["import", "webpack"],
        },
        TestCase {
            name: "sample #2",
            expect: Some(vec!["./data/timezones/pdt.mjs"]),
            exports_field: exports_field(json!({
                "./timezones/": "./data/timezones/"
            })),
            request: "./timezones/pdt.mjs",
            condition_names: vec![],
        },
        TestCase {
            name: "sample #2 (wildcard)",
            expect: Some(vec!["./data/timezones/pdt.mjs"]),
            exports_field: exports_field(json!({
                "./timezones/*": "./data/timezones/*"
            })),
            request: "./timezones/pdt.mjs",
            condition_names: vec![],
        },
        TestCase {
            name: "sample #3",
            expect: Some(vec!["./data/timezones/timezones/pdt.mjs"]),
            exports_field: exports_field(json!({
                "./": "./data/timezones/"
            })),
            request: "./timezones/pdt.mjs",
            condition_names: vec![],
        },
        TestCase {
            name: "sample #3 (wildcard)",
            expect: Some(vec!["./data/timezones/timezones/pdt.mjs"]),
            exports_field: exports_field(json!({
                "./*": "./data/timezones/*.mjs"
            })),
            request: "./timezones/pdt",
            condition_names: vec![],
        },
        TestCase {
            name: "sample #4",
            expect: Some(vec![]),
            exports_field: exports_field(json!({
                "./lib/": {
                    "browser": [
                        "./browser/"
                    ]
                },
                "./dist/index.js": {
                    "node": "./index.js"
                }
            })),
            request: "./dist/index.js",
            condition_names: vec!["browser"],
        },
        TestCase {
            name: "sample #4 (wildcard)",
            expect: Some(vec![]),
            exports_field: exports_field(json!({
                "./lib/*": {
                    "browser": [
                        "./browser/*"
                    ]
                },
                "./dist/index.js": {
                    "node": "./index.js"
                }
            })),
            request: "./dist/index.js",
            condition_names: vec!["browser"],
        },
        TestCase {
            name: "sample #5",
            expect: Some(vec!["./browser/index.js"]),
            exports_field: exports_field(json!({
                "./lib/": {
                    "browser": [
                        "./browser/"
                    ]
                },
                "./dist/index.js": {
                    "node": "./index.js",
                    "default": "./browser/index.js"
                }
            })),
            request: "./dist/index.js",
            condition_names: vec!["browser"],
        },
        TestCase {
            name: "sample #5 (wildcard)",
            expect: Some(vec!["./browser/index.js"]),
            exports_field: exports_field(json!({
                "./lib/*": {
                    "browser": [
                        "./browser/*"
                    ]
                },
                "./dist/index.js": {
                    "node": "./index.js",
                    "default": "./browser/index.js"
                }
            })),
            request: "./dist/index.js",
            condition_names: vec!["browser"],
        },
        TestCase {
            name: "sample #6",
            expect: Some(vec![]),
            exports_field: exports_field(json!({
                "./dist/a": "./dist/index.js"
            })),
            request: "./dist/aaa",
            condition_names: vec![],
        },
        TestCase {
            name: "sample #7",
            expect: Some(vec![]),
            exports_field: exports_field(json!({
                "./dist/a/a/": "./dist/index.js"
            })),
            request: "./dist/a/a",
            condition_names: vec![],
        },
        TestCase {
            name: "sample #7 (wildcard)",
            expect: Some(vec![]),
            exports_field: exports_field(json!({
                "./dist/a/a/*": "./dist/index.js"
            })),
            request: "./dist/a/a",
            condition_names: vec![],
        },
        TestCase {
            name: "sample #8",
            expect: Some(vec![]),
            exports_field: exports_field(json!({
                ".": "./index.js"
            })),
            request: "./timezones/pdt.mjs",
            condition_names: vec![],
        },
        TestCase {
            name: "sample #9",
            expect: Some(vec!["./main.js"]),
            exports_field: exports_field(json!({
                "./index.js": "./main.js"
            })),
            request: "./index.js",
            condition_names: vec![],
        },
        TestCase {
            name: "sample #10",
            expect: Some(vec!["./ok.js"]),
            exports_field: exports_field(json!({
                "./#foo": "./ok.js",
                "./module": "./ok.js",
                "./🎉": "./ok.js",
                "./%F0%9F%8E%89": "./other.js",
                "./bar#foo": "./ok.js",
                "./#zapp/": "./",
                "./#zipp*": "./zzz*"
            })),
            request: "./#foo",
            condition_names: vec![],
        },
        TestCase {
            name: "sample #11",
            expect: Some(vec!["./ok.js"]),
            exports_field: exports_field(json!({
                "./#foo": "./ok.js",
                "./module": "./ok.js",
                "./🎉": "./ok.js",
                "./%F0%9F%8E%89": "./other.js",
                "./bar#foo": "./ok.js",
                "./#zapp/": "./",
                "./#zipp*": "./zzz*"
            })),
            request: "./bar#foo",
            condition_names: vec![],
        },
        TestCase {
            name: "sample #12",
            expect: Some(vec!["./ok.js#abc"]),
            exports_field: exports_field(json!({
                "./#foo": "./ok.js",
                "./module": "./ok.js",
                "./🎉": "./ok.js",
                "./%F0%9F%8E%89": "./other.js",
                "./bar#foo": "./ok.js",
                "./#zapp/": "./",
                "./#zipp*": "./zzz*"
            })),
            request: "./#zapp/ok.js#abc",
            condition_names: vec![],
        },
        TestCase {
            name: "sample #12",
            expect: Some(vec!["./ok.js#abc"]),
            exports_field: exports_field(json!({
                "./#foo": "./ok.js",
                "./module": "./ok.js",
                "./🎉": "./ok.js",
                "./%F0%9F%8E%89": "./other.js",
                "./bar#foo": "./ok.js",
                "./#zapp/": "./",
                "./#zipp*": "./zzz*"
            })),
            request: "./#zapp/ok.js#abc",
            condition_names: vec![],
        },
        TestCase {
            name: "sample #13",
            expect: Some(vec!["./ok.js?abc"]),
            exports_field: exports_field(json!({
                "./#foo": "./ok.js",
                "./module": "./ok.js",
                "./🎉": "./ok.js",
                "./%F0%9F%8E%89": "./other.js",
                "./bar#foo": "./ok.js",
                "./#zapp/": "./",
                "./#zipp*": "./zzz*"
            })),
            request: "./#zapp/ok.js?abc",
            condition_names: vec![],
        },
        TestCase {
            name: "sample #14",
            expect: Some(vec!["./🎉.js"]),
            exports_field: exports_field(json!({
                "./#foo": "./ok.js",
                "./module": "./ok.js",
                "./🎉": "./ok.js",
                "./%F0%9F%8E%89": "./other.js",
                "./bar#foo": "./ok.js",
                "./#zapp/": "./",
                "./#zipp*": "./zzz*"
            })),
            request: "./#zapp/🎉.js",
            condition_names: vec![],
        },
        TestCase {
            name: "sample #15",
            expect: Some(vec!["./%F0%9F%8E%89.js"]),
            exports_field: exports_field(json!({
                "./#foo": "./ok.js",
                "./module": "./ok.js",
                "./🎉": "./ok.js",
                "./%F0%9F%8E%89": "./other.js",
                "./bar#foo": "./ok.js",
                "./#zapp/": "./",
                "./#zipp*": "./zzz*"
            })),
            request: "./#zapp/%F0%9F%8E%89.js",
            condition_names: vec![],
        },
        TestCase {
            name: "sample #16",
            expect: Some(vec!["./ok.js"]),
            exports_field: exports_field(json!({
                "./#foo": "./ok.js",
                "./module": "./ok.js",
                "./🎉": "./ok.js",
                "./%F0%9F%8E%89": "./other.js",
                "./bar#foo": "./ok.js",
                "./#zapp/": "./",
                "./#zipp*": "./zzz*"
            })),
            request: "./🎉",
            condition_names: vec![],
        },
        TestCase {
            name: "sample #17",
            expect: Some(vec!["./other.js"]),
            exports_field: exports_field(json!({
                "./#foo": "./ok.js",
                "./module": "./ok.js",
                "./🎉": "./ok.js",
                "./%F0%9F%8E%89": "./other.js",
                "./bar#foo": "./ok.js",
                "./#zapp/": "./",
                "./#zipp*": "./zzz*"
            })),
            request: "./%F0%9F%8E%89",
            condition_names: vec![],
        },
        TestCase {
            name: "sample #18",
            expect: Some(vec!["./ok.js"]),
            exports_field: exports_field(json!({
                "./#foo": "./ok.js",
                "./module": "./ok.js",
                "./🎉": "./ok.js",
                "./%F0%9F%8E%89": "./other.js",
                "./bar#foo": "./ok.js",
                "./#zapp/": "./",
                "./#zipp*": "./zzz*"
            })),
            request: "./module",
            condition_names: vec![],
        },
        TestCase {
            name: "sample #19",
            expect: Some(vec![]),
            exports_field: exports_field(json!({
                "./#foo": "./ok.js",
                "./module": "./ok.js",
                "./🎉": "./ok.js",
                "./%F0%9F%8E%89": "./other.js",
                "./bar#foo": "./ok.js",
                "./#zapp/": "./",
                "./#zipp*": "./zzz*"
            })),
            request: "./module#foo",
            condition_names: vec![],
        },
        TestCase {
            name: "sample #20",
            expect: Some(vec![]),
            exports_field: exports_field(json!({
                "./#foo": "./ok.js",
                "./module": "./ok.js",
                "./🎉": "./ok.js",
                "./%F0%9F%8E%89": "./other.js",
                "./bar#foo": "./ok.js",
                "./#zapp/": "./",
                "./#zipp*": "./zzz*"
            })),
            request: "./module?foo",
            condition_names: vec![],
        },
        TestCase {
            name: "sample #21",
            expect: Some(vec!["./zizizi"]),
            exports_field: exports_field(json!({
                "./#foo": "./ok.js",
                "./module": "./ok.js",
                "./🎉": "./ok.js",
                "./%F0%9F%8E%89": "./other.js",
                "./bar#foo": "./ok.js",
                "./#zapp/": "./",
                "./#zipp*": "./z*z*z*"
            })),
            request: "./#zippi",
            condition_names: vec![],
        },
        TestCase {
            name: "sample #22",
            expect: Some(vec!["./d?e?f"]),
            exports_field: exports_field(json!({
                "./a?b?c/": "./"
            })),
            request: "./a?b?c/d?e?f",
            condition_names: vec![],
        },
        TestCase {
            name: "Direct mapping #1",
            expect: Some(vec!["./dist/index.js"]),
            exports_field: exports_field(json!({
                ".": "./dist/index.js"
            })),
            request: ".",
            condition_names: vec![],
        },
        TestCase {
            name: "Direct mapping #2",
            expect: Some(vec![]),
            exports_field: exports_field(json!({
                "./": "./",
                "./*": "./*",
                "./dist/index.js": "./dist/index.js"
            })),
            request: ".",
            condition_names: vec![],
        },
        TestCase {
            name: "Direct mapping #3",
            expect: Some(vec!["./dist/a.js"]),
            exports_field: exports_field(json!({
                "./dist/": "./dist/",
                "./dist/*": "./dist/*",
                "./dist*": "./dist*",
                "./dist/index.js": "./dist/a.js"
            })),
            request: "./dist/index.js",
            condition_names: vec![],
        },
        TestCase {
            name: "Direct mapping #4",
            expect: Some(vec!["./index.js"]),
            exports_field: exports_field(json!({
                "./": {
                    "browser": [
                        "./browser/"
                    ]
                },
                "./*": {
                    "browser": [
                        "./browser/*"
                    ]
                },
                "./dist/index.js": {
                    "browser": "./index.js"
                }
            })),
            request: "./dist/index.js",
            condition_names: vec!["browser"],
        },
        TestCase {
            name: "Direct mapping #5",
            expect: Some(vec![]),
            exports_field: exports_field(json!({
                "./": {
                    "browser": [
                        "./browser/"
                    ]
                },
                "./*": {
                    "browser": [
                        "./browser/*"
                    ]
                },
                "./dist/index.js": {
                    "node": "./node.js"
                }
            })),
            request: "./dist/index.js",
            condition_names: vec!["browser"],
        },
        TestCase {
            name: "Direct mapping #6",
            expect: Some(vec!["./index.js"]),
            exports_field: exports_field(json!({
                ".": {
                    "browser": "./index.js",
                    "node": "./src/node/index.js",
                    "default": "./src/index.js"
                }
            })),
            request: ".",
            condition_names: vec!["browser"],
        },
        TestCase {
            name: "Direct mapping #7",
            expect: None,
            exports_field: exports_field(json!({
                ".": {
                    "default": "./src/index.js",
                    "browser": "./index.js",
                    "node": "./src/node/index.js"
                }
            })),
            request: ".",
            condition_names: vec!["browser"],
        },
        TestCase {
            name: "Direct mapping #8",
            expect: Some(vec!["./src/index.js"]),
            exports_field: exports_field(json!({
                ".": {
                    "browser": "./index.js",
                    "node": "./src/node/index.js",
                    "default": "./src/index.js"
                }
            })),
            request: ".",
            condition_names: vec![],
        },
        TestCase {
            name: "Direct mapping #9",
            expect: Some(vec!["./index"]),
            exports_field: exports_field(json!({
                ".": "./index"
            })),
            request: ".",
            condition_names: vec![],
        },
        TestCase {
            name: "Direct mapping #10",
            expect: Some(vec!["./index.js"]),
            exports_field: exports_field(json!({
                "./index": "./index.js"
            })),
            request: "./index",
            condition_names: vec![],
        },
        TestCase {
            name: "Direct mapping #11",
            expect: Some(vec!["./foo.js"]),
            exports_field: exports_field(json!({
                "./": "./",
                "./*": "./*",
                "./dist/index.js": "./dist/index.js"
            })),
            request: "./foo.js",
            condition_names: vec![],
        },
        TestCase {
            name: "Direct mapping #12",
            expect: Some(vec!["./foo/bar/baz.js"]),
            exports_field: exports_field(json!({
                "./": "./",
                "./*": "./*",
                "./dist/index.js": "./dist/index.js"
            })),
            request: "./foo/bar/baz.js",
            condition_names: vec![],
        },
        TestCase {
            name: "Direct mapping #13",
            expect: Some(vec!["./foo/bar/baz.js"]),
            exports_field: exports_field(json!({
                "./": "./",
                "./dist/index.js": "./dist/index.js"
            })),
            request: "./foo/bar/baz.js",
            condition_names: vec![],
        },
        TestCase {
            name: "Direct mapping #14",
            expect: Some(vec!["./foo/bar/baz.js"]),
            exports_field: exports_field(json!({
                "./*": "./*",
                "./dist/index.js": "./dist/index.js"
            })),
            request: "./foo/bar/baz.js",
            condition_names: vec![],
        },
        TestCase {
            name: "Direct and conditional mapping #1",
            expect: Some(vec![]),
            exports_field: exports_field(json!({
                ".": [{
                    "browser": "./browser.js"
                }, {
                    "require": "./require.js"
                }, {
                    "import": "./import.mjs"
                }]
            })),
            request: ".",
            condition_names: vec![],
        },
        TestCase {
            name: "Direct and conditional mapping #2",
            expect: Some(vec!["./import.mjs"]),
            exports_field: exports_field(json!({
                ".": [{
                    "browser": "./browser.js"
                }, {
                    "require": "./require.js"
                }, {
                    "import": "./import.mjs"
                }]
            })),
            request: ".",
            condition_names: vec!["import"],
        },
        TestCase {
            name: "Direct and conditional mapping #3",
            expect: Some(vec!["./require.js"]),
            exports_field: exports_field(json!({
                ".": [
                {
                    "browser": "./browser.js"
                },
                {
                    "require": "./require.js"
                },
                {
                    "import": "./import.mjs"
                }
                ]
            })),
            request: ".",
            condition_names: vec!["import", "require"],
        },
        // Duplicated due to not supporting returning an array
        TestCase {
            name: "Direct and conditional mapping #3",
            expect: Some(vec!["./import.mjs"]),
            exports_field: exports_field(json!({
                ".": [{
                    "browser": "./browser.js"
                }, {
                    "import": "./import.mjs"
                }]
            })),
            request: ".",
            condition_names: vec!["import", "require"],
        },
        TestCase {
            name: "Direct and conditional mapping #4",
            expect: Some(vec!["./require.js"]),
            exports_field: exports_field(json!({
                ".": [{
                    "browser": "./browser.js"
                }, {
                    "require": [
                        "./require.js"
                    ]
                }, {
                    "import": [
                        "./import.mjs",
                        "./import.js"
                    ]
                }]
            })),
            request: ".",
            condition_names: vec!["import", "require"],
        },
        // Duplicated due to not supporting returning an array
        TestCase {
            name: "Direct and conditional mapping #4",
            expect: Some(vec!["./import.mjs"]),
            exports_field: exports_field(json!({
                ".": [
                {
                    "browser": "./browser.js"
                },
                {
                    "import": [
                        "./import.mjs",
                        "./import.js"
                    ]
                }
                ]
            })),
            request: ".",
            condition_names: vec!["import", "require"],
        },
        // Duplicated due to not supporting returning an array
        TestCase {
            name: "Direct and conditional mapping #4",
            expect: Some(vec!["./import.js"]),
            exports_field: exports_field(json!({
                ".": [
                {
                    "browser": "./browser.js"
                },
                {
                    "import": [
                        "./import.js"
                    ]
                }
                ]
            })),
            request: ".",
            condition_names: vec!["import", "require"],
        },
        TestCase {
            name: "mapping to a folder root #1",
            expect: Some(vec![]),
            exports_field: exports_field(json!({
                "./timezones": "./data/timezones/"
            })),
            request: "./timezones/pdt.mjs",
            condition_names: vec![],
        },
        TestCase {
            name: "mapping to a folder root #2",
            expect: None,
            exports_field: exports_field(json!({
                "./timezones/": "./data/timezones"
            })),
            request: "./timezones/pdt.mjs",
            condition_names: vec![],
        },
        TestCase {
            name: "mapping to a folder root #3",
            expect: Some(vec!["./data/timezones/pdt/index.mjs"]),
            exports_field: exports_field(json!({
                "./timezones/pdt/": "./data/timezones/pdt/"
            })),
            request: "./timezones/pdt/index.mjs",
            condition_names: vec![],
        },
        TestCase {
            name: "mapping to a folder root #3 (wildcard)",
            expect: Some(vec!["./data/timezones/pdt/index.mjs"]),
            exports_field: exports_field(json!({
                "./timezones/pdt/*": "./data/timezones/pdt/*"
            })),
            request: "./timezones/pdt/index.mjs",
            condition_names: vec![],
        },
        TestCase {
            name: "mapping to a folder root #4",
            expect: Some(vec!["./timezones/pdt.mjs"]),
            exports_field: exports_field(json!({
                "./": "./timezones/"
            })),
            request: "./pdt.mjs",
            condition_names: vec![],
        },
        TestCase {
            name: "mapping to a folder root #4 (wildcard)",
            expect: Some(vec!["./timezones/pdt.mjs"]),
            exports_field: exports_field(json!({
                "./*": "./timezones/*"
            })),
            request: "./pdt.mjs",
            condition_names: vec![],
        },
        TestCase {
            name: "mapping to a folder root #5",
            expect: Some(vec!["./timezones/pdt.mjs"]),
            exports_field: exports_field(json!({
                "./": "./"
            })),
            request: "./timezones/pdt.mjs",
            condition_names: vec![],
        },
        TestCase {
            name: "mapping to a folder root #5 (wildcard)",
            expect: Some(vec!["./timezones/pdt.mjs"]),
            exports_field: exports_field(json!({
                "./*": "./*"
            })),
            request: "./timezones/pdt.mjs",
            condition_names: vec![],
        },
        TestCase {
            name: "mapping to a folder root #6",
            expect: None,
            exports_field: exports_field(json!({
                "./": "."
            })),
            request: "./timezones/pdt.mjs",
            condition_names: vec![],
        },
        TestCase {
            name: "mapping to a folder root #6 (wildcard)",
            expect: None,
            exports_field: exports_field(json!({
                "./*": "."
            })),
            request: "./timezones/pdt.mjs",
            condition_names: vec![],
        },
        TestCase {
            name: "mapping to a folder root #7",
            expect: Some(vec![]),
            exports_field: exports_field(json!({
                ".": "./"
            })),
            request: "./timezones/pdt.mjs",
            condition_names: vec![],
        },
        TestCase {
            name: "mapping to a folder root #7 (wildcard)",
            expect: Some(vec![]),
            exports_field: exports_field(json!({
                ".": "./*"
            })),
            request: "./timezones/pdt.mjs",
            condition_names: vec![],
        },
        TestCase {
            name: "the longest matching path prefix is prioritized #1",
            expect: Some(vec!["./lib/index.mjs"]),
            exports_field: exports_field(json!({
                "./": "./",
                "./dist/": "./lib/"
            })),
            request: "./dist/index.mjs",
            condition_names: vec![],
        },
        TestCase {
            name: "the longest matching path prefix is prioritized #1 (wildcard)",
            expect: Some(vec!["./lib/index.mjs"]),
            exports_field: exports_field(json!({
                "./*": "./*",
                "./dist/*": "./lib/*"
            })),
            request: "./dist/index.mjs",
            condition_names: vec![],
        },
        TestCase {
            name: "the longest matching path prefix is prioritized #2",
            expect: Some(vec!["./dist/utils/index.js"]),
            exports_field: exports_field(json!({
                "./dist/utils/": "./dist/utils/",
                "./dist/": "./lib/"
            })),
            request: "./dist/utils/index.js",
            condition_names: vec![],
        },
        TestCase {
            name: "the longest matching path prefix is prioritized #2 (wildcard)",
            expect: Some(vec!["./dist/utils/index.js"]),
            exports_field: exports_field(json!({
                "./dist/utils/*": "./dist/utils/*",
                "./dist/*": "./lib/*"
            })),
            request: "./dist/utils/index.js",
            condition_names: vec![],
        },
        TestCase {
            name: "the longest matching path prefix is prioritized #3",
            expect: Some(vec!["./dist/utils/index.js"]),
            exports_field: exports_field(json!({
                "./dist/utils/index.js": "./dist/utils/index.js",
                "./dist/utils/": "./dist/utils/index.mjs",
                "./dist/": "./lib/"
            })),
            request: "./dist/utils/index.js",
            condition_names: vec![],
        },
        TestCase {
            name: "the longest matching path prefix is prioritized #3 (wildcard)",
            expect: Some(vec!["./dist/utils/index.js"]),
            exports_field: exports_field(json!({
                "./dist/utils/index.js": "./dist/utils/index.js",
                "./dist/utils/*": "./dist/utils/index.mjs",
                "./dist/*": "./lib/*"
            })),
            request: "./dist/utils/index.js",
            condition_names: vec![],
        },
        TestCase {
            name: "the longest matching path prefix is prioritized #4",
            expect: Some(vec!["./lib/index.mjs"]),
            exports_field: exports_field(json!({
                "./": {
                    "browser": "./browser/"
                },
                "./dist/": "./lib/"
            })),
            request: "./dist/index.mjs",
            condition_names: vec!["browser"],
        },
        TestCase {
            name: "the longest matching path prefix is prioritized #4 (wildcard)",
            expect: Some(vec!["./lib/index.mjs"]),
            exports_field: exports_field(json!({
                "./*": {
                    "browser": "./browser/*"
                },
                "./dist/*": "./lib/*"
            })),
            request: "./dist/index.mjs",
            condition_names: vec!["browser"],
        },
        TestCase {
            name: "conditional mapping folder #1",
            // `lodash/` does not start with './' so fallbacks to util
            expect: Some(vec!["./utils/index.js"]),
            exports_field: exports_field(json!({
                "./utils/": {
                    "browser": [
                        "lodash/",
                        "./utils/"
                    ],
                    "node": [
                        "./utils-node/"
                    ]
                }
            })),
            request: "./utils/index.js",
            condition_names: vec!["browser"],
        },
        // Duplicated due to not supporting returning an array
        TestCase {
            name: "conditional mapping folder #1",
            expect: Some(vec!["./utils/index.js"]),
            exports_field: exports_field(json!({
                "./utils/": {
                    "browser": [
                        "./utils/"
                    ],
                    "node": [
                        "./utils-node/"
                    ]
                }
            })),
            request: "./utils/index.js",
            condition_names: vec!["browser"],
        },
        TestCase {
            name: "conditional mapping folder #1 (wildcard)",
            // `lodash/` does not start with './' so fallbacks to util
            expect: Some(vec!["./utils/index.js"]),
            exports_field: exports_field(json!({
                "./utils/*": {
                    "browser": [
                        "lodash/*",
                        "./utils/*"
                    ],
                    "node": [
                        "./utils-node/*"
                    ]
                }
            })),
            request: "./utils/index.js",
            condition_names: vec!["browser"],
        },
        // Duplicated due to not supporting returning an array
        TestCase {
            name: "conditional mapping folder #1 (wildcard)",
            expect: Some(vec!["./utils/index.js"]),
            exports_field: exports_field(json!({
                "./utils/*": {
                    "browser": [
                        "./utils/*"
                    ],
                    "node": [
                        "./utils-node/*"
                    ]
                }
            })),
            request: "./utils/index.js",
            condition_names: vec!["browser"],
        },
        TestCase {
            name: "conditional mapping folder #2",
            expect: Some(vec![]),
            exports_field: exports_field(json!({
                "./utils/": {
                    "webpack": "./wpk/",
                    "browser": [
                        "lodash/",
                        "./utils/"
                    ],
                    "node": [
                        "./node/"
                    ]
                }
            })),
            request: "./utils/index.mjs",
            condition_names: vec![],
        },
        TestCase {
            name: "conditional mapping folder #2 (wildcard)",
            expect: Some(vec![]),
            exports_field: exports_field(json!({
                "./utils/*": {
                    "webpack": "./wpk/*",
                    "browser": [
                        "lodash/*",
                        "./utils/*"
                    ],
                    "node": [
                        "./node/*"
                    ]
                }
            })),
            request: "./utils/index.mjs",
            condition_names: vec![],
        },
        TestCase {
            name: "conditional mapping folder #3",
            expect: Some(vec!["./wpk/index.mjs"]),
            exports_field: exports_field(json!({
                "./utils/": {
                    "webpack": "./wpk/",
                    "browser": [
                        "lodash/",
                        "./utils/"
                    ],
                    "node": [
                        "./utils/"
                    ]
                }
            })),
            request: "./utils/index.mjs",
            condition_names: vec!["browser", "webpack"],
        },
        TestCase {
            name: "conditional mapping folder #3 (wildcard)",
            expect: Some(vec!["./wpk/index.mjs"]),
            exports_field: exports_field(json!({
                "./utils/*": {
                    "webpack": "./wpk/*",
                    "browser": [
                        "lodash/*",
                        "./utils/*"
                    ],
                    "node": [
                        "./utils/*"
                    ]
                }
            })),
            request: "./utils/index.mjs",
            condition_names: vec!["browser", "webpack"],
        },
        TestCase {
            name: "incorrect exports field #1",
            expect: None,
            exports_field: exports_field(json!({
                "/utils/": "./a/"
            })),
            request: "./utils/index.mjs",
            condition_names: vec![],
        },
        TestCase {
            name: "incorrect exports field #2",
            expect: None,
            exports_field: exports_field(json!({
                "./utils/": "/a/"
            })),
            request: "./utils/index.mjs",
            condition_names: vec![],
        },
        TestCase {
            name: "incorrect exports field #3",
            expect: None,
            exports_field: exports_field(json!({
                "/utils/": {
                    "browser": "./a/",
                    "default": "./b/"
                }
            })),
            request: "./utils/index.mjs",
            condition_names: vec!["browser"],
        },
        TestCase {
            name: "incorrect exports field #4",
            expect: None,
            exports_field: exports_field(json!({
                "./utils/": {
                    "browser": "/a/",
                    "default": "/b/"
                }
            })),
            request: "./utils/index.mjs",
            condition_names: vec!["browser"],
        },
        TestCase {
            name: "incorrect exports field #4 (wildcard)",
            expect: None,
            exports_field: exports_field(json!({
                "./utils/*": {
                    "browser": "/a/",
                    "default": "/b/"
                }
            })),
            request: "./utils/index.mjs",
            condition_names: vec!["browser"],
        },
        TestCase {
            name: "incorrect exports field #5",
            expect: Some(vec![]),
            exports_field: exports_field(json!({
                "./utils/index": "./a/index.js"
            })),
            request: "./utils/index.mjs",
            condition_names: vec![],
        },
        TestCase {
            name: "incorrect exports field #6",
            expect: Some(vec![]),
            exports_field: exports_field(json!({
                "./utils/index.mjs": "./a/index.js"
            })),
            request: "./utils/index",
            condition_names: vec![],
        },
        TestCase {
            name: "incorrect exports field #7",
            expect: Some(vec![]),
            exports_field: exports_field(json!({
                "./utils/index": {
                    "browser": "./a/index.js",
                    "default": "./b/index.js"
                }
            })),
            request: "./utils/index.mjs",
            condition_names: vec!["browser"],
        },
        TestCase {
            name: "incorrect exports field #8",
            expect: Some(vec![]),
            exports_field: exports_field(json!({
                "./utils/index.mjs": {
                    "browser": "./a/index.js",
                    "default": "./b/index.js"
                }
            })),
            request: "./utils/index",
            condition_names: vec!["browser"],
        },
        // Requests that are not `./` does not apply to `package_exports_resolve`
        // TestCase {
        // name: "incorrect request #1",
        // expect: None,
        // exports_field: exports_field(json!({
        // "./utils/": "./a/"
        // })),
        // request: "/utils/index.mjs",
        // condition_names: vec![],
        // },
        // TestCase {
        // name: "incorrect request #2",
        // expect: None,
        // exports_field: exports_field(json!({
        // "./utils/": {
        // "browser": "./a/",
        // "default": "./b/"
        // }
        // })),
        // request: "/utils/index.mjs",
        // condition_names: vec!["browser"],
        // },
        // TestCase {
        // name: "incorrect request #3",
        // expect: None,
        // exports_field: exports_field(json!({
        // "./utils/": {
        // "browser": "./a/",
        // "default": "./b/"
        // }
        // })),
        // request: "../utils/index.mjs",
        // condition_names: vec!["browser"],
        // },
        // TestCase {
        // name: "incorrect request #4",
        // expect: None,
        // exports_field: exports_field(json!({
        // "./utils/": {
        // "browser": "./a/",
        // "default": "./b/"
        // }
        // })),
        // request: "/utils/index.mjs/",
        // condition_names: vec!["browser"],
        // },
        TestCase {
            name: "backtracking package base #1",
            expect: Some(vec!["./dist/index"]),
            exports_field: exports_field(json!({
                "./../../utils/": "./dist/"
            })),
            request: "./../../utils/index",
            condition_names: vec![],
        },
        TestCase {
            name: "backtracking package base #1 (wildcard)",
            expect: Some(vec!["./dist/index"]),
            exports_field: exports_field(json!({
                "./../../utils/*": "./dist/*"
            })),
            request: "./../../utils/index",
            condition_names: vec![],
        },
        TestCase {
            name: "backtracking package base #2",
            expect: None,
            exports_field: exports_field(json!({
                "../../utils/": "./dist/"
            })),
            request: "../../utils/index",
            condition_names: vec![],
        },
        TestCase {
            name: "backtracking package base #2 (wildcard)",
            expect: None,
            exports_field: exports_field(json!({
                "../../utils/*": "./dist/*"
            })),
            request: "../../utils/index",
            condition_names: vec![],
        },
        TestCase {
            name: "backtracking package base #3",
            expect: None,
            exports_field: exports_field(json!({
                "./utils/": "../src/"
            })),
            request: "./utils/index",
            condition_names: vec![],
        },
        TestCase {
            name: "backtracking package base #3 (wildcard)",
            expect: None,
            exports_field: exports_field(json!({
                "./utils/*": "../src/*"
            })),
            request: "./utils/index",
            condition_names: vec![],
        },
        // enhanced_resolve does not handle backtracking here
        // TestCase {
        // name: "backtracking package base #4",
        // expect: Some(vec!["./../src/index"]),
        // exports_field: exports_field(json!({
        // "./utils/": "./../src/"
        // })),
        // request: "./utils/index",
        // condition_names: vec![],
        // },
        // TestCase {
        // name: "backtracking package base #4 (wildcard)",
        // expect: Some(vec!["./../src/index"]),
        // exports_field: exports_field(json!({
        // "./utils/*": "./../src/*"
        // })),
        // request: "./utils/index",
        // condition_names: vec![],
        // },
        // TestCase {
        // name: "backtracking package base #5",
        // expect: Some(vec!["./src/../index.js"]),
        // exports_field: exports_field(json!({
        // "./utils/index": "./src/../index.js"
        // })),
        // request: "./utils/index",
        // condition_names: vec![],
        // },
        TestCase {
            name: "backtracking package base #6",
            expect: Some(vec![]),
            exports_field: exports_field(json!({
                "./utils/../utils/index": "./src/../index.js"
            })),
            request: "./utils/index",
            condition_names: vec![],
        },
        TestCase {
            name: "backtracking package base #7",
            expect: None,
            exports_field: exports_field(json!({
                "./utils/": {
                    "browser": "../this/"
                }
            })),
            request: "./utils/index",
            condition_names: vec!["browser"],
        },
        TestCase {
            name: "backtracking package base #7",
            expect: None,
            exports_field: exports_field(json!({
                "./utils/*": {
                    "browser": "../this/*"
                }
            })),
            request: "./utils/index",
            condition_names: vec!["browser"],
        },
        TestCase {
            name: "backtracking package base #8",
            // We throw "InvalidPackageTarget"
            // expect: Some(vec!["./utils/../index"]),
            expect: None,
            exports_field: exports_field(json!({
                "./utils/": {
                    "browser": "./utils/../"
                }
            })),
            request: "./utils/index",
            condition_names: vec!["browser"],
        },
        TestCase {
            name: "backtracking package base #8 (wildcard)",
            // We throw "InvalidPackageTarget"
            // expect: Some(vec!["./utils/../index"]),
            expect: None,
            exports_field: exports_field(json!({
                "./utils/*": {
                    "browser": "./utils/../*"
                }
            })),
            request: "./utils/index",
            condition_names: vec!["browser"],
        },
        TestCase {
            name: "backtracking package base #9",
            expect: Some(vec!["./dist/index"]),
            exports_field: exports_field(json!({
                "./": "./src/../../",
                "./dist/": "./dist/"
            })),
            request: "./dist/index",
            condition_names: vec!["browser"],
        },
        TestCase {
            name: "backtracking package base #9 (wildcard)",
            expect: Some(vec!["./dist/index"]),
            exports_field: exports_field(json!({
                "./*": "./src/../../*",
                "./dist/*": "./dist/*"
            })),
            request: "./dist/index",
            condition_names: vec!["browser"],
        },
        TestCase {
            name: "backtracking target folder #1",
            // We return InvalidPackageTarget
            expect: None,
            // expect: Some(vec!["./dist/timezone/../../index"]),
            exports_field: exports_field(json!({
                "./utils/": "./dist/"
            })),
            request: "./utils/timezone/../../index",
            condition_names: vec![],
        },
        TestCase {
            name: "backtracking target folder #1 (wildcard)",
            // We return InvalidPackageTarget
            expect: None,
            // expect: Some(vec!["./dist/timezone/../../index"]),
            exports_field: exports_field(json!({
                "./utils/*": "./dist/*"
            })),
            request: "./utils/timezone/../../index",
            condition_names: vec![],
        },
        TestCase {
            name: "backtracking target folder #2",
            // We return InvalidPackageTarget
            expect: None,
            // expect: Some(vec!["./dist/timezone/../index"]),
            exports_field: exports_field(json!({
                "./utils/": "./dist/"
            })),
            request: "./utils/timezone/../index",
            condition_names: vec![],
        },
        TestCase {
            name: "backtracking target folder #2 (wildcard)",
            // We return InvalidPackageTarget
            expect: None,
            // expect: Some(vec!["./dist/timezone/../index"]),
            exports_field: exports_field(json!({
                "./utils/*": "./dist/*"
            })),
            request: "./utils/timezone/../index",
            condition_names: vec![],
        },
        TestCase {
            name: "backtracking target folder #3",
            // We return InvalidPackageTarget
            expect: None,
            // expect: Some(vec!["./dist/target/../../index"]),
            exports_field: exports_field(json!({
                "./utils/": "./dist/target/"
            })),
            request: "./utils/../../index",
            condition_names: vec![],
        },
        TestCase {
            name: "backtracking target folder #3 (wildcard)",
            // We return InvalidPackageTarget
            expect: None,
            // expect: Some(vec!["./dist/target/../../index"]),
            exports_field: exports_field(json!({
                "./utils/*": "./dist/target/*"
            })),
            request: "./utils/../../index",
            condition_names: vec![],
        },
        // enhanced-resolve does not handle `node_modules` in target
        TestCase {
            name: "nested node_modules path #1",
            // We return InvalidPackageTarget
            expect: None,
            // expect: Some(vec!["./node_modules/lodash/dist/index.js"]),
            exports_field: exports_field(json!({
                "./utils/": {
                    "browser": "./node_modules/"
                }
            })),
            request: "./utils/lodash/dist/index.js",
            condition_names: vec!["browser"],
        },
        TestCase {
            name: "nested node_modules path #1 (wildcard)",
            // We return InvalidPackageTarget
            expect: None,
            // expect: Some(vec!["./node_modules/lodash/dist/index.js"]),
            exports_field: exports_field(json!({
                "./utils/*": {
                    "browser": "./node_modules/*"
                }
            })),
            request: "./utils/lodash/dist/index.js",
            condition_names: vec!["browser"],
        },
        TestCase {
            name: "nested node_modules path #2",
            // We return InvalidPackageTarget
            expect: None,
            // expect: Some(vec!["./utils/../node_modules/lodash/dist/index.js"]),
            exports_field: exports_field(json!({
                "./utils/": "./utils/../node_modules/"
            })),
            request: "./utils/lodash/dist/index.js",
            condition_names: vec![],
        },
        TestCase {
            name: "nested node_modules path #2 (wildcard)",
            // We return InvalidPackageTarget
            expect: None,
            // expect: Some(vec!["./utils/../node_modules/lodash/dist/index.js"]),
            exports_field: exports_field(json!({
                "./utils/*": "./utils/../node_modules/*"
            })),
            request: "./utils/lodash/dist/index.js",
            condition_names: vec![],
        },
        TestCase {
            name: "nested mapping #1",
            expect: Some(vec![]),
            exports_field: exports_field(json!({
                "./utils/": {
                    "browser": {
                        "webpack": "./",
                        "default": {
                            "node": "./node/"
                        }
                    }
                }
            })),
            request: "./utils/index.js",
            condition_names: vec!["browser"],
        },
        TestCase {
            name: "nested mapping #1 (wildcard)",
            expect: Some(vec![]),
            exports_field: exports_field(json!({
                "./utils/*": {
                    "browser": {
                        "webpack": "./*",
                        "default": {
                            "node": "./node/*"
                        }
                    }
                }
            })),
            request: "./utils/index.js",
            condition_names: vec!["browser"],
        },
        TestCase {
            name: "nested mapping #2",
            expect: Some(vec!["./index.js"]),
            exports_field: exports_field(json!({
                "./utils/": {
                    "browser": {
                        "webpack": [
                            "./",
                            "./node/"
                        ],
                        "default": {
                            "node": "./node/"
                        }
                    }
                }
            })),
            request: "./utils/index.js",
            condition_names: vec!["browser", "webpack"],
        },
        // Duplicated due to not supporting returning an array
        TestCase {
            name: "nested mapping #2",
            expect: Some(vec!["./node/index.js"]),
            exports_field: exports_field(json!({
                "./utils/": {
                    "browser": {
                        "webpack": [
                            "./node/"
                        ],
                        "default": {
                            "node": "./node/"
                        }
                    }
                }
            })),
            request: "./utils/index.js",
            condition_names: vec!["browser", "webpack"],
        },
        TestCase {
            name: "nested mapping #2 (wildcard)",
            expect: Some(vec!["./index.js"]),
            exports_field: exports_field(json!({
                "./utils/*": {
                    "browser": {
                        "webpack": [
                            "./*",
                            "./node/*"
                        ],
                        "default": {
                            "node": "./node/*"
                        }
                    }
                }
            })),
            request: "./utils/index.js",
            condition_names: vec!["browser", "webpack"],
        },
        // Duplicated due to not supporting returning an array
        TestCase {
            name: "nested mapping #2 (wildcard)",
            expect: Some(vec!["./node/index.js"]),
            exports_field: exports_field(json!({
                "./utils/*": {
                    "browser": {
                        "webpack": [
                            "./node/*"
                        ],
                        "default": {
                            "node": "./node/*"
                        }
                    }
                }
            })),
            request: "./utils/index.js",
            condition_names: vec!["browser", "webpack"],
        },
        TestCase {
            name: "nested mapping #3",
            expect: Some(vec![]),
            exports_field: exports_field(json!({
                "./utils/": {
                    "browser": {
                        "webpack": [
                            "./",
                            "./node/"
                        ],
                        "default": {
                            "node": "./node/"
                        }
                    }
                }
            })),
            request: "./utils/index.js",
            condition_names: vec!["webpack"],
        },
        TestCase {
            name: "nested mapping #3 (wildcard)",
            expect: Some(vec![]),
            exports_field: exports_field(json!({
                "./utils/*": {
                    "browser": {
                        "webpack": [
                            "./*",
                            "./node/*"
                        ],
                        "default": {
                            "node": "./node/*"
                        }
                    }
                }
            })),
            request: "./utils/index.js",
            condition_names: vec!["webpack"],
        },
        TestCase {
            name: "nested mapping #4",
            expect: Some(vec!["./node/index.js"]),
            exports_field: exports_field(json!({
                "./utils/": {
                    "browser": {
                        "webpack": [
                            "./",
                            "./node/"
                        ],
                        "default": {
                            "node": "./node/"
                        }
                    }
                }
            })),
            request: "./utils/index.js",
            condition_names: vec!["node", "browser"],
        },
        TestCase {
            name: "nested mapping #4 (wildcard)",
            expect: Some(vec!["./node/index.js"]),
            exports_field: exports_field(json!({
                "./utils/*": {
                    "browser": {
                        "webpack": [
                            "./*",
                            "./node/*"
                        ],
                        "default": {
                            "node": "./node/*"
                        }
                    }
                }
            })),
            request: "./utils/index.js",
            condition_names: vec!["node", "browser"],
        },
        TestCase {
            name: "nested mapping #5",
            expect: Some(vec![]),
            exports_field: exports_field(json!({
                "./utils/": {
                    "browser": {
                        "webpack": [
                            "./",
                            "./node/"
                        ],
                        "default": {
                            "node": {
                                "webpack": [
                                    "./wpck/"
                                ]
                            }
                        }
                    }
                }
            })),
            request: "./utils/index.js",
            condition_names: vec!["browser", "node"],
        },
        TestCase {
            name: "nested mapping #5 (wildcard)",
            expect: Some(vec![]),
            exports_field: exports_field(json!({
                "./utils/*": {
                    "browser": {
                        "webpack": [
                            "./*",
                            "./node/*"
                        ],
                        "default": {
                            "node": {
                                "webpack": [
                                    "./wpck/*"
                                ]
                            }
                        }
                    }
                }
            })),
            request: "./utils/index.js",
            condition_names: vec!["browser", "node"],
        },
        TestCase {
            name: "nested mapping #6",
            expect: Some(vec!["./index.js"]),
            exports_field: exports_field(json!({
                "./utils/": {
                    "browser": {
                        "webpack": [
                            "./",
                            "./node/"
                        ],
                        "default": {
                            "node": {
                                "webpack": [
                                    "./wpck/"
                                ]
                            }
                        }
                    }
                }
            })),
            request: "./utils/index.js",
            condition_names: vec!["browser", "node", "webpack"],
        },
        // Duplicated due to not supporting returning an array
        TestCase {
            name: "nested mapping #6",
            expect: Some(vec!["./node/index.js"]),
            exports_field: exports_field(json!({
                "./utils/": {
                    "browser": {
                        "webpack": [
                            "./node/"
                        ],
                        "default": {
                            "node": {
                                "webpack": [
                                    "./wpck/"
                                ]
                            }
                        }
                    }
                }
            })),
            request: "./utils/index.js",
            condition_names: vec!["browser", "node", "webpack"],
        },
        TestCase {
            name: "nested mapping #6 (wildcard)",
            expect: Some(vec!["./index.js"]),
            exports_field: exports_field(json!({
                "./utils/*": {
                    "browser": {
                        "webpack": [
                            "./*",
                            "./node/*"
                        ],
                        "default": {
                            "node": {
                                "webpack": [
                                    "./wpck/*"
                                ]
                            }
                        }
                    }
                }
            })),
            request: "./utils/index.js",
            condition_names: vec!["browser", "node", "webpack"],
        },
        // Duplicated due to not supporting returning an array
        TestCase {
            name: "nested mapping #6 (wildcard)",
            expect: Some(vec!["./node/index.js"]),
            exports_field: exports_field(json!({
                "./utils/*": {
                    "browser": {
                        "webpack": [
                            "./node/*"
                        ],
                        "default": {
                            "node": {
                                "webpack": [
                                    "./wpck/*"
                                ]
                            }
                        }
                    }
                }
            })),
            request: "./utils/index.js",
            condition_names: vec!["browser", "node", "webpack"],
        },
        TestCase {
            name: "nested mapping #7",
            expect: Some(vec!["./y.js"]),
            exports_field: exports_field(json!({
                "./a.js": {
                    "abc": {
                        "def": "./x.js"
                    },
                    "ghi": "./y.js"
                }
            })),
            request: "./a.js",
            condition_names: vec!["abc", "ghi"],
        },
        TestCase {
            name: "nested mapping #8",
            expect: Some(vec![]),
            exports_field: exports_field(json!({
                "./a.js": {
                    "abc": {
                        "def": "./x.js",
                        "default": []
                    },
                    "ghi": "./y.js"
                }
            })),
            request: "./a.js",
            condition_names: vec!["abc", "ghi"],
        },
        TestCase {
            name: "syntax sugar #1",
            expect: Some(vec!["./main.js"]),
            exports_field: exports_field(json!("./main.js")),
            request: ".",
            condition_names: vec![],
        },
        TestCase {
            name: "syntax sugar #2",
            expect: Some(vec![]),
            exports_field: exports_field(json!("./main.js")),
            request: "./lib.js",
            condition_names: vec![],
        },
        TestCase {
            name: "syntax sugar #3",
            expect: Some(vec!["./a.js"]),
            exports_field: exports_field(json!(["./a.js", "./b.js"])),
            request: ".",
            condition_names: vec![],
        },
        // Duplicated due to not supporting returning an array
        TestCase {
            name: "syntax sugar #3",
            expect: Some(vec!["./b.js"]),
            exports_field: exports_field(json!(["./b.js"])),
            request: ".",
            condition_names: vec![],
        },
        TestCase {
            name: "syntax sugar #4",
            expect: Some(vec![]),
            exports_field: exports_field(json!(["./a.js", "./b.js"])),
            request: "./lib.js",
            condition_names: vec![],
        },
        TestCase {
            name: "syntax sugar #5",
            expect: Some(vec!["./index.js"]),
            exports_field: exports_field(json!({
                "browser": {
                    "default": "./index.js"
                }
            })),
            request: ".",
            condition_names: vec!["browser"],
        },
        TestCase {
            name: "syntax sugar #6",
            expect: Some(vec![]),
            exports_field: exports_field(json!({
                "browser": {
                    "default": "./index.js"
                }
            })),
            request: "./lib.js",
            condition_names: vec!["browser"],
        },
        TestCase {
            name: "syntax sugar #7",
            expect: None,
            exports_field: exports_field(json!({
                "./node": "./node.js",
                "browser": {
                    "default": "./index.js"
                }
            })),
            request: ".",
            condition_names: vec!["browser"],
        },
        TestCase {
            name: "syntax sugar #8",
            expect: None,
            exports_field: exports_field(json!({
                "browser": {
                    "default": "./index.js"
                },
                "./node": "./node.js"
            })),
            request: ".",
            condition_names: vec!["browser"],
        },
        TestCase {
            name: "wildcard longest #1",
            expect: Some(vec!["./abc/d"]),
            exports_field: exports_field(json!({
                "./ab*": "./ab/*",
                "./abc*": "./abc/*",
                "./a*": "./a/*"
            })),
            request: "./abcd",
            condition_names: vec!["browser"],
        },
        TestCase {
            name: "wildcard longest #2",
            expect: Some(vec!["./abc/d/e"]),
            exports_field: exports_field(json!({
                "./ab*": "./ab/*",
                "./abc*": "./abc/*",
                "./a*": "./a/*"
            })),
            request: "./abcd/e",
            condition_names: vec!["browser"],
        },
        TestCase {
            name: "wildcard longest #3",
            expect: Some(vec!["./abc/d"]),
            exports_field: exports_field(json!({
                "./x/ab*": "./ab/*",
                "./x/abc*": "./abc/*",
                "./x/a*": "./a/*"
            })),
            request: "./x/abcd",
            condition_names: vec!["browser"],
        },
        TestCase {
            name: "wildcard longest #4",
            expect: Some(vec!["./abc/d/e"]),
            exports_field: exports_field(json!({
                "./x/ab*": "./ab/*",
                "./x/abc*": "./abc/*",
                "./x/a*": "./a/*"
            })),
            request: "./x/abcd/e",
            condition_names: vec!["browser"],
        },
        TestCase {
            name: "path tree edge case #1",
            expect: Some(vec!["./A/b/d.js"]),
            exports_field: exports_field(json!({
                "./a/": "./A/",
                "./a/b/c": "./c.js"
            })),
            request: "./a/b/d.js",
            condition_names: vec![],
        },
        TestCase {
            name: "path tree edge case #1 (wildcard)",
            expect: Some(vec!["./A/b/d.js"]),
            exports_field: exports_field(json!({
                "./a/*": "./A/*",
                "./a/b/c": "./c.js"
            })),
            request: "./a/b/d.js",
            condition_names: vec![],
        },
        TestCase {
            name: "path tree edge case #2",
            expect: Some(vec!["./A/c.js"]),
            exports_field: exports_field(json!({
                "./a/": "./A/",
                "./a/b": "./b.js"
            })),
            request: "./a/c.js",
            condition_names: vec![],
        },
        TestCase {
            name: "path tree edge case #2 (wildcard)",
            expect: Some(vec!["./A/c.js"]),
            exports_field: exports_field(json!({
                "./a/*": "./A/*",
                "./a/b": "./b.js"
            })),
            request: "./a/c.js",
            condition_names: vec![],
        },
        TestCase {
            name: "path tree edge case #3",
            expect: Some(vec!["./A/b/d/c.js"]),
            exports_field: exports_field(json!({
                "./a/": "./A/",
                "./a/b/c/d": "./c.js"
            })),
            request: "./a/b/d/c.js",
            condition_names: vec![],
        },
        TestCase {
            name: "path tree edge case #3 (wildcard)",
            expect: Some(vec!["./A/b/d/c.js"]),
            exports_field: exports_field(json!({
                "./a/*": "./A/*",
                "./a/b/c/d": "./c.js"
            })),
            request: "./a/b/d/c.js",
            condition_names: vec![],
        },
        TestCase {
            name: "wildcard pattern #1",
            expect: Some(vec!["./A/b.js"]),
            exports_field: exports_field(json!({
                "./a/*.js": "./A/*.js"
            })),
            request: "./a/b.js",
            condition_names: vec![],
        },
        TestCase {
            name: "wildcard pattern #2",
            expect: Some(vec!["./A/b/c.js"]),
            exports_field: exports_field(json!({
                "./a/*.js": "./A/*.js"
            })),
            request: "./a/b/c.js",
            condition_names: vec![],
        },
        TestCase {
            name: "wildcard pattern #3",
            expect: Some(vec!["./A/b/c.js"]),
            exports_field: exports_field(json!({
                "./a/*/c.js": "./A/*/c.js"
            })),
            request: "./a/b/c.js",
            condition_names: vec![],
        },
        TestCase {
            name: "wildcard pattern #4",
            expect: Some(vec!["./A/b/b.js"]),
            exports_field: exports_field(json!({
                "./a/*/c.js": "./A/*/*.js"
            })),
            request: "./a/b/c.js",
            condition_names: vec![],
        },
        TestCase {
            name: "wildcard pattern #5",
            expect: Some(vec!["./browser/index.js"]),
            exports_field: exports_field(json!({
                "./lib/*": {
                    "browser": [
                        "./browser/*"
                    ]
                },
                "./dist/*.js": {
                    "node": "./*.js",
                    "default": "./browser/*.js"
                }
            })),
            request: "./dist/index.js",
            condition_names: vec!["browser"],
        },
        TestCase {
            name: "wildcard pattern #5",
            expect: Some(vec!["./browser/index.js"]),
            exports_field: exports_field(json!({
                "./lib/*": {
                    "browser": [
                        "./browser/*"
                    ]
                },
                "./dist/*.js": {
                    "node": "./*.js",
                    "default": "./browser/*.js"
                }
            })),
            request: "./lib/index.js",
            condition_names: vec!["browser"],
        },
        TestCase {
            name: "wildcard pattern #6",
            expect: Some(vec!["./browser/foo/bar.js"]),
            exports_field: exports_field(json!({
                "./lib/*/bar.js": {
                    "browser": [
                        "./browser/*/bar.js"
                    ]
                },
                "./dist/*/bar.js": {
                    "node": "./*.js",
                    "default": "./browser/*.js"
                }
            })),
            request: "./lib/foo/bar.js",
            condition_names: vec!["browser"],
        },
        TestCase {
            name: "wildcard pattern #6",
            expect: Some(vec!["./browser/foo.js"]),
            exports_field: exports_field(json!({
                "./lib/*/bar.js": {
                    "browser": [
                        "./browser/*/bar.js"
                    ]
                },
                "./dist/*/bar.js": {
                    "node": "./*.js",
                    "default": "./browser/*.js"
                }
            })),
            request: "./dist/foo/bar.js",
            condition_names: vec!["browser"],
        },
        TestCase {
            name: "wildcard pattern #7",
            expect: Some(vec!["./browser/foo/default.js"]),
            exports_field: exports_field(json!({
                "./lib/*/bar.js": {
                    "browser": [
                        "./browser/*/bar.js"
                    ]
                },
                "./dist/*/bar.js": {
                    "node": "./*.js",
                    "default": "./browser/*/default.js"
                }
            })),
            request: "./dist/foo/bar.js",
            condition_names: vec!["default"],
        },
        TestCase {
            name: "wildcard pattern #8",
            expect: Some(vec!["./A/b/b/b.js"]),
            exports_field: exports_field(json!({
                "./a/*/c.js": "./A/*/*/*.js"
            })),
            request: "./a/b/c.js",
            condition_names: vec![],
        },
        TestCase {
            name: "wildcard pattern #9",
            expect: Some(vec!["./A/b/b/b.js"]),
            exports_field: exports_field(json!({
                "./a/*/c.js": [
                    "./A/*/*/*.js",
                    "./B/*/*/*.js"
                ]
            })),
            request: "./a/b/c.js",
            condition_names: vec![],
        },
        // Duplicated due to not supporting returning an array
        TestCase {
            name: "wildcard pattern #9",
            expect: Some(vec!["./B/b/b/b.js"]),
            exports_field: exports_field(json!({
                "./a/*/c.js": [
                    "./B/*/*/*.js"
                ]
            })),
            request: "./a/b/c.js",
            condition_names: vec![],
        },
        TestCase {
            name: "wildcard pattern #10",
            expect: Some(vec!["./A/b/b/b.js"]),
            exports_field: exports_field(json!({
                "./a/foo-*/c.js": "./A/*/*/*.js"
            })),
            request: "./a/foo-b/c.js",
            condition_names: vec![],
        },
        TestCase {
            name: "wildcard pattern #11",
            expect: Some(vec!["./A/b/b/b.js"]),
            exports_field: exports_field(json!({
                "./a/*-foo/c.js": "./A/*/*/*.js"
            })),
            request: "./a/b-foo/c.js",
            condition_names: vec![],
        },
        TestCase {
            name: "wildcard pattern #12",
            expect: Some(vec!["./A/b/b/b.js"]),
            exports_field: exports_field(json!({
                "./a/foo-*-foo/c.js": "./A/*/*/*.js"
            })),
            request: "./a/foo-b-foo/c.js",
            condition_names: vec![],
        },
        TestCase {
            name: "wildcard pattern #13",
            expect: Some(vec!["./A/b/c/d.js"]),
            exports_field: exports_field(json!({
                "./a/foo-*-foo/c.js": "./A/b/c/d.js"
            })),
            request: "./a/foo-b-foo/c.js",
            condition_names: vec![],
        },
        TestCase {
            name: "wildcard pattern #14",
            expect: Some(vec!["./A/b/c/*.js"]),
            exports_field: exports_field(json!({
                "./a/foo-foo/c.js": "./A/b/c/*.js"
            })),
            request: "./a/foo-foo/c.js",
            condition_names: vec![],
        },
    ];

    for case in test_cases {
        let resolved = Resolver::default()
            .package_exports_resolve(
                Path::new(""),
                case.request.trim_start_matches('.'),
                &case.exports_field,
                &case.condition_names.iter().map(ToString::to_string).collect::<Vec<_>>(),
                &ResolveContext::default(),
            )
            .map(|p| p.map(|p| p.to_path_buf()));
        if let Some(expect) = case.expect {
            if expect.is_empty() {
                assert!(
                    matches!(resolved, Err(ResolveError::PackagePathNotExported(_))),
                    "{} {:?}",
                    &case.name,
                    &resolved
                );
            } else {
                for expect in expect {
                    assert_eq!(resolved, Ok(Some(Path::new(expect).normalize())), "{}", &case.name);
                }
            }
        } else {
            assert!(resolved.is_err(), "{} {resolved:?}", &case.name);
        }
    }
}
