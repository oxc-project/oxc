//! https://github.com/webpack/enhanced-resolve/blob/main/test/importsField.test.js
//!
//! The huge imports field test cases are at the bottom of this file.

use serde_json::json;

use crate::{MatchObject, PathUtil, Resolution, ResolveError, ResolveOptions, Resolver};
use std::path::Path;

#[test]
fn test() {
    let f = super::fixture().join("imports-field");
    let f2 = super::fixture().join("imports-exports-wildcard/node_modules/m/");

    let resolver = Resolver::new(ResolveOptions {
        extensions: vec![".js".into()],
        main_files: vec!["index.js".into()],
        condition_names: vec!["webpack".into()],
        ..ResolveOptions::default()
    });

    #[rustfmt::skip]
    let pass = [
        ("should resolve using imports field instead of self-referencing", f.clone(), "#imports-field", f.join("b.js")),
        ("should resolve using imports field instead of self-referencing for a subpath", f.join("dir"), "#imports-field", f.join("b.js")),
        ("should resolve package #1", f.clone(), "#a/dist/main.js", f.join("node_modules/a/lib/lib2/main.js")),
        ("should resolve package #3", f.clone(), "#ccc/index.js", f.join("node_modules/c/index.js")),
        ("should resolve package #4", f.clone(), "#c", f.join("node_modules/c/index.js")),
        ("should resolve with wildcard pattern", f2.clone(), "#internal/i.js", f2.join("src/internal/i.js")),
    ];

    for (comment, path, request, expected) in pass {
        let resolved_path = resolver.resolve(&path, request).map(Resolution::full_path);
        assert_eq!(resolved_path, Ok(expected), "{comment} {path:?} {request}");
    }

    // Note added:
    // * should resolve absolute path as an imports field target
    // * should log the correct info

    #[rustfmt::skip]
    let fail = [
        ("should disallow resolve out of package scope", f.clone(), "#b", ResolveError::InvalidPackageTarget("../b.js".to_string())),
        ("should resolve package #2", f, "#a", ResolveError::PackageImportNotDefined("#a".to_string())),
    ];

    for (comment, path, request, error) in fail {
        let resolution = resolver.resolve(&path, request);
        assert_eq!(resolution, Err(error), "{comment} {path:?} {request}");
    }
}

#[test]
#[ignore = "imports field name"]
// field name path #1 - #2
fn field_name() {}

// Small script for generating the test cases from enhanced_resolve
// for (c of testCases) {
//  console.log("TestCase {")
//  console.log(`name: ${JSON.stringify(c.name)},`)
//   if (c.expect instanceof Error) {
//     console.log(`expect: None,`)
//   } else {
//     console.log(`expect: Some(vec!${JSON.stringify(c.expect)}),`)
//   }
//  console.log(`imports_field: imports_field(json!(${JSON.stringify(c.suite[0], null, 2)})),`)
//   console.log(`request: "${c.suite[1]}",`)
//  console.log(`condition_names: vec!${JSON.stringify(c.suite[2])},`)
//   console.log("},")
// }

struct TestCase {
    name: &'static str,
    expect: Option<Vec<&'static str>>,
    imports_field: MatchObject,
    request: &'static str,
    condition_names: Vec<&'static str>,
}

#[allow(clippy::needless_pass_by_value)]
fn imports_field(value: serde_json::Value) -> MatchObject {
    let s = serde_json::to_string(&value).unwrap();
    serde_json::from_str(&s).unwrap()
}

#[test]
#[allow(clippy::too_many_lines)]
fn test_cases() {
    let test_cases = [
        TestCase {
            name: "sample #1",
            expect: Some(vec!["./dist/test/file.js"]),
            imports_field: imports_field(json!({
              "#abc/": {
                "import": [
                  "./dist/",
                  "./src/"
                ],
                "webpack": "./wp/"
              },
              "#abc": "./main.js"
            })),
            request: "#abc/test/file.js",
            condition_names: vec!["import", "webpack"],
        },
        // Duplicated due to not supporting returning an array
        TestCase {
            name: "sample #1",
            expect: Some(vec!["./src/test/file.js"]),
            imports_field: imports_field(json!({
              "#abc/": {
                "import": [
                  "./src/"
                ],
                "webpack": "./wp/"
              },
              "#abc": "./main.js"
            })),
            request: "#abc/test/file.js",
            condition_names: vec!["import", "webpack"],
        },
        TestCase {
            name: "sample #2",
            expect: Some(vec!["./data/timezones/pdt.mjs"]),
            imports_field: imports_field(json!({
              "#1/timezones/": "./data/timezones/"
            })),
            request: "#1/timezones/pdt.mjs",
            condition_names: vec![],
        },
        TestCase {
            name: "sample #3",
            expect: Some(vec!["./data/timezones/timezones/pdt.mjs"]),
            imports_field: imports_field(json!({
              "#aaa/": "./data/timezones/",
              "#a/": "./data/timezones/"
            })),
            request: "#a/timezones/pdt.mjs",
            condition_names: vec![],
        },
        TestCase {
            name: "sample #4",
            expect: Some(vec![]),
            imports_field: imports_field(json!({
              "#a/lib/": {
                "browser": [
                  "./browser/"
                ]
              },
              "#a/dist/index.js": {
                "node": "./index.js"
              }
            })),
            request: "#a/dist/index.js",
            condition_names: vec!["browser"],
        },
        TestCase {
            name: "sample #5",
            expect: Some(vec!["./browser/index.js"]),
            imports_field: imports_field(json!({
              "#a/lib/": {
                "browser": [
                  "./browser/"
                ]
              },
              "#a/dist/index.js": {
                "node": "./index.js",
                "default": "./browser/index.js"
              }
            })),
            request: "#a/dist/index.js",
            condition_names: vec!["browser"],
        },
        TestCase {
            name: "sample #6",
            expect: Some(vec![]),
            imports_field: imports_field(json!({
              "#a/dist/a": "./dist/index.js"
            })),
            request: "#a/dist/aaa",
            condition_names: vec![],
        },
        TestCase {
            name: "sample #7",
            expect: Some(vec![]),
            imports_field: imports_field(json!({
              "#a/a/a/": "./dist/index.js"
            })),
            request: "#a/a/a",
            condition_names: vec![],
        },
        TestCase {
            name: "sample #8",
            expect: Some(vec![]),
            imports_field: imports_field(json!({
              "#a": "./index.js"
            })),
            request: "#a/timezones/pdt.mjs",
            condition_names: vec![],
        },
        TestCase {
            name: "sample #9",
            expect: Some(vec!["./main.js"]),
            imports_field: imports_field(json!({
              "#a/index.js": "./main.js"
            })),
            request: "#a/index.js",
            condition_names: vec![],
        },
        TestCase {
            name: "sample #10",
            expect: Some(vec!["./ok.js"]),
            imports_field: imports_field(json!({
              "#a/#foo": "./ok.js",
              "#a/module": "./ok.js",
              "#a/🎉": "./ok.js",
              "#a/%F0%9F%8E%89": "./other.js",
              "#a/bar#foo": "./ok.js",
              "#a/#zapp/": "./"
            })),
            request: "#a/#foo",
            condition_names: vec![],
        },
        TestCase {
            name: "sample #11",
            expect: Some(vec!["./ok.js"]),
            imports_field: imports_field(json!({
              "#a/#foo": "./ok.js",
              "#a/module": "./ok.js",
              "#a/🎉": "./ok.js",
              "#a/%F0%9F%8E%89": "./other.js",
              "#a/bar#foo": "./ok.js",
              "#a/#zapp/": "./"
            })),
            request: "#a/bar#foo",
            condition_names: vec![],
        },
        TestCase {
            name: "sample #12",
            expect: Some(vec!["./ok.js#abc"]),
            imports_field: imports_field(json!({
              "#a/#foo": "./ok.js",
              "#a/module": "./ok.js",
              "#a/🎉": "./ok.js",
              "#a/%F0%9F%8E%89": "./other.js",
              "#a/bar#foo": "./ok.js",
              "#a/#zapp/": "./"
            })),
            request: "#a/#zapp/ok.js#abc",
            condition_names: vec![],
        },
        TestCase {
            name: "sample #13",
            expect: Some(vec!["./ok.js?abc"]),
            imports_field: imports_field(json!({
              "#a/#foo": "./ok.js",
              "#a/module": "./ok.js",
              "#a/🎉": "./ok.js",
              "#a/%F0%9F%8E%89": "./other.js",
              "#a/bar#foo": "./ok.js",
              "#a/#zapp/": "./"
            })),
            request: "#a/#zapp/ok.js?abc",
            condition_names: vec![],
        },
        TestCase {
            name: "sample #14",
            expect: Some(vec!["./🎉.js"]),
            imports_field: imports_field(json!({
              "#a/#foo": "./ok.js",
              "#a/module": "./ok.js",
              "#a/🎉": "./ok.js",
              "#a/%F0%9F%8E%89": "./other.js",
              "#a/bar#foo": "./ok.js",
              "#a/#zapp/": "./"
            })),
            request: "#a/#zapp/🎉.js",
            condition_names: vec![],
        },
        TestCase {
            name: "sample #15",
            expect: Some(vec!["./%F0%9F%8E%89.js"]),
            imports_field: imports_field(json!({
              "#a/#foo": "./ok.js",
              "#a/module": "./ok.js",
              "#a/🎉": "./ok.js",
              "#a/%F0%9F%8E%89": "./other.js",
              "#a/bar#foo": "./ok.js",
              "#a/#zapp/": "./"
            })),
            request: "#a/#zapp/%F0%9F%8E%89.js",
            condition_names: vec![],
        },
        TestCase {
            name: "sample #16",
            expect: Some(vec!["./ok.js"]),
            imports_field: imports_field(json!({
              "#a/#foo": "./ok.js",
              "#a/module": "./ok.js",
              "#a/🎉": "./ok.js",
              "#a/%F0%9F%8E%89": "./other.js",
              "#a/bar#foo": "./ok.js",
              "#a/#zapp/": "./"
            })),
            request: "#a/🎉",
            condition_names: vec![],
        },
        TestCase {
            name: "sample #17",
            expect: Some(vec!["./other.js"]),
            imports_field: imports_field(json!({
              "#a/#foo": "./ok.js",
              "#a/module": "./ok.js",
              "#a/🎉": "./ok.js",
              "#a/%F0%9F%8E%89": "./other.js",
              "#a/bar#foo": "./ok.js",
              "#a/#zapp/": "./"
            })),
            request: "#a/%F0%9F%8E%89",
            condition_names: vec![],
        },
        TestCase {
            name: "sample #18",
            expect: Some(vec!["./ok.js"]),
            imports_field: imports_field(json!({
              "#a/#foo": "./ok.js",
              "#a/module": "./ok.js",
              "#a/🎉": "./ok.js",
              "#a/%F0%9F%8E%89": "./other.js",
              "#a/bar#foo": "./ok.js",
              "#a/#zapp/": "./"
            })),
            request: "#a/module",
            condition_names: vec![],
        },
        TestCase {
            name: "sample #19",
            expect: Some(vec![]),
            imports_field: imports_field(json!({
              "#a/#foo": "./ok.js",
              "#a/module": "./ok.js",
              "#a/🎉": "./ok.js",
              "#a/%F0%9F%8E%89": "./other.js",
              "#a/bar#foo": "./ok.js",
              "#a/#zapp/": "./"
            })),
            request: "#a/module#foo",
            condition_names: vec![],
        },
        TestCase {
            name: "sample #20",
            expect: Some(vec![]),
            imports_field: imports_field(json!({
              "#a/#foo": "./ok.js",
              "#a/module": "./ok.js",
              "#a/🎉": "./ok.js",
              "#a/%F0%9F%8E%89": "./other.js",
              "#a/bar#foo": "./ok.js",
              "#a/#zapp/": "./"
            })),
            request: "#a/module?foo",
            condition_names: vec![],
        },
        TestCase {
            name: "sample #21",
            expect: Some(vec!["./d?e?f"]),
            imports_field: imports_field(json!({
              "#a/a?b?c/": "./"
            })),
            request: "#a/a?b?c/d?e?f",
            condition_names: vec![],
        },
        TestCase {
            name: "sample #22",
            // We throw InvalidPackageTarget
            expect: None,
            // expect: Some(vec!["/user/a/index"]),
            imports_field: imports_field(json!({
              "#a/": "/user/a/"
            })),
            request: "#a/index",
            condition_names: vec![],
        },
        TestCase {
            name: "path tree edge case #1",
            expect: Some(vec!["./A/b/d.js"]),
            imports_field: imports_field(json!({
              "#a/": "./A/",
              "#a/b/c": "./c.js"
            })),
            request: "#a/b/d.js",
            condition_names: vec![],
        },
        TestCase {
            name: "path tree edge case #2",
            expect: Some(vec!["./A/c.js"]),
            imports_field: imports_field(json!({
              "#a/": "./A/",
              "#a/b": "./b.js"
            })),
            request: "#a/c.js",
            condition_names: vec![],
        },
        TestCase {
            name: "path tree edge case #3",
            expect: Some(vec!["./A/b/c/d.js"]),
            imports_field: imports_field(json!({
              "#a/": "./A/",
              "#a/b/c/d": "./c.js"
            })),
            request: "#a/b/c/d.js",
            condition_names: vec![],
        },
        TestCase {
            name: "Direct mapping #1",
            expect: Some(vec!["./dist/index.js"]),
            imports_field: imports_field(json!({
              "#a": "./dist/index.js"
            })),
            request: "#a",
            condition_names: vec![],
        },
        TestCase {
            name: "Direct mapping #2",
            expect: Some(vec![]),
            imports_field: imports_field(json!({
              "#a/": "./"
            })),
            request: "#a",
            condition_names: vec![],
        },
        TestCase {
            name: "Direct mapping #3",
            expect: Some(vec!["./dist/a.js"]),
            imports_field: imports_field(json!({
              "#a/": "./dist/",
              "#a/index.js": "./dist/a.js"
            })),
            request: "#a/index.js",
            condition_names: vec![],
        },
        TestCase {
            name: "Direct mapping #4",
            expect: Some(vec!["./index.js"]),
            imports_field: imports_field(json!({
              "#a/": {
                "browser": [
                  "./browser/"
                ]
              },
              "#a/index.js": {
                "browser": "./index.js"
              }
            })),
            request: "#a/index.js",
            condition_names: vec!["browser"],
        },
        TestCase {
            name: "Direct mapping #5",
            expect: Some(vec![]),
            imports_field: imports_field(json!({
              "#a/": {
                "browser": [
                  "./browser/"
                ]
              },
              "#a/index.js": {
                "node": "./node.js"
              }
            })),
            request: "#a/index.js",
            condition_names: vec!["browser"],
        },
        TestCase {
            name: "Direct mapping #6",
            expect: Some(vec!["./index.js"]),
            imports_field: imports_field(json!({
              "#a": {
                "browser": "./index.js",
                "node": "./src/node/index.js",
                "default": "./src/index.js"
              }
            })),
            request: "#a",
            condition_names: vec!["browser"],
        },
        TestCase {
            name: "Direct mapping #7",
            expect: None,
            imports_field: imports_field(json!({
              "#a": {
                "default": "./src/index.js",
                "browser": "./index.js",
                "node": "./src/node/index.js"
              }
            })),
            request: "#a",
            condition_names: vec!["browser"],
        },
        TestCase {
            name: "Direct mapping #8",
            expect: Some(vec!["./src/index.js"]),
            imports_field: imports_field(json!({
              "#a": {
                "browser": "./index.js",
                "node": "./src/node/index.js",
                "default": "./src/index.js"
              }
            })),
            request: "#a",
            condition_names: vec![],
        },
        TestCase {
            name: "Direct mapping #9",
            expect: Some(vec!["./index"]),
            imports_field: imports_field(json!({
              "#a": "./index"
            })),
            request: "#a",
            condition_names: vec![],
        },
        TestCase {
            name: "Direct mapping #10",
            expect: Some(vec!["./index.js"]),
            imports_field: imports_field(json!({
              "#a/index": "./index.js"
            })),
            request: "#a/index",
            condition_names: vec![],
        },
        TestCase {
            name: "Direct mapping #11",
            // We throw InvalidPackageTarget
            // expect: Some(vec!["b"]),
            expect: None,
            imports_field: imports_field(json!({
              "#a": "b"
            })),
            request: "#a",
            condition_names: vec![],
        },
        TestCase {
            name: "Direct mapping #12",
            // We throw InvalidPackageTarget
            // expect: Some(vec!["b/index"]),
            expect: None,
            imports_field: imports_field(json!({
              "#a/": "b/"
            })),
            request: "#a/index",
            condition_names: vec![],
        },
        TestCase {
            name: "Direct mapping #13",
            // We throw InvalidPackageTarget
            // expect: Some(vec!["b#anotherhashishere"]),
            expect: None,
            imports_field: imports_field(json!({
              "#a?q=a#hashishere": "b#anotherhashishere"
            })),
            request: "#a?q=a#hashishere",
            condition_names: vec![],
        },
        TestCase {
            name: "Direct and conditional mapping #1",
            expect: Some(vec![]),
            imports_field: imports_field(json!({
              "#a": [
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
            request: "#a",
            condition_names: vec![],
        },
        TestCase {
            name: "Direct and conditional mapping #2",
            expect: Some(vec!["./import.mjs"]),
            imports_field: imports_field(json!({
              "#a": [
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
            request: "#a",
            condition_names: vec!["import"],
        },
        TestCase {
            name: "Direct and conditional mapping #3",
            expect: Some(vec!["./require.js"]),
            imports_field: imports_field(json!({
              "#a": [
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
            request: "#a",
            condition_names: vec!["import", "require"],
        },
        // Duplicated due to not supporting returning an array
        TestCase {
            name: "Direct and conditional mapping #3",
            expect: Some(vec!["./import.mjs"]),
            imports_field: imports_field(json!({
              "#a": [
                {
                  "browser": "./browser.js"
                },
                {
                  "import": "./import.mjs"
                }
              ]
            })),
            request: "#a",
            condition_names: vec!["import", "require"],
        },
        TestCase {
            name: "Direct and conditional mapping #4",
            expect: Some(vec!["./require.js"]),
            imports_field: imports_field(json!({
              "#a": [
                {
                  "browser": "./browser.js"
                },
                {
                  "require": [
                    "./require.js"
                  ]
                },
                {
                  "import": [
                    "./import.mjs",
                    "#b/import.js"
                  ]
                }
              ]
            })),
            request: "#a",
            condition_names: vec!["import", "require"],
        },
        // Duplicated due to not supporting returning an array
        TestCase {
            name: "Direct and conditional mapping #4",
            expect: Some(vec!["./import.mjs"]),
            imports_field: imports_field(json!({
              "#a": [
                {
                  "browser": "./browser.js"
                },
                {
                  "import": [
                    "./import.mjs",
                    "#b/import.js"
                  ]
                }
              ]
            })),
            request: "#a",
            condition_names: vec!["import", "require"],
        },
        // Duplicated due to not supporting returning an array
        TestCase {
            name: "Direct and conditional mapping #4",
            expect: Some(vec![]),
            imports_field: imports_field(json!({
              "#a": [
                {
                  "browser": "./browser.js"
                },
                {
                  "import": [
                    "#b/import.js"
                  ]
                }
              ]
            })),
            request: "#a",
            condition_names: vec!["import", "require"],
        },
        TestCase {
            name: "mapping to a folder root #1",
            expect: Some(vec![]),
            imports_field: imports_field(json!({
              "#timezones": "./data/timezones/"
            })),
            request: "#timezones/pdt.mjs",
            condition_names: vec![],
        },
        TestCase {
            name: "mapping to a folder root #2",
            expect: None,
            imports_field: imports_field(json!({
              "#timezones/": "./data/timezones"
            })),
            request: "#timezones/pdt.mjs",
            condition_names: vec![],
        },
        TestCase {
            name: "mapping to a folder root #3",
            expect: Some(vec!["./data/timezones/pdt/index.mjs"]),
            imports_field: imports_field(json!({
              "#timezones/pdt/": "./data/timezones/pdt/"
            })),
            request: "#timezones/pdt/index.mjs",
            condition_names: vec![],
        },
        TestCase {
            name: "mapping to a folder root #4",
            expect: Some(vec!["./timezones/pdt.mjs"]),
            imports_field: imports_field(json!({
              "#a/": "./timezones/"
            })),
            request: "#a/pdt.mjs",
            condition_names: vec![],
        },
        TestCase {
            name: "mapping to a folder root #5",
            expect: Some(vec!["./timezones/pdt.mjs"]),
            imports_field: imports_field(json!({
              "#a/": "./"
            })),
            request: "#a/timezones/pdt.mjs",
            condition_names: vec![],
        },
        TestCase {
            name: "mapping to a folder root #6",
            expect: None,
            imports_field: imports_field(json!({
              "#a/": "."
            })),
            request: "#a/timezones/pdt.mjs",
            condition_names: vec![],
        },
        TestCase {
            name: "mapping to a folder root #7",
            expect: Some(vec![]),
            imports_field: imports_field(json!({
              "#a": "./"
            })),
            request: "#a/timezones/pdt.mjs",
            condition_names: vec![],
        },
        TestCase {
            name: "the longest matching path prefix is prioritized #1",
            expect: Some(vec!["./lib/index.mjs"]),
            imports_field: imports_field(json!({
              "#a/": "./",
              "#a/dist/": "./lib/"
            })),
            request: "#a/dist/index.mjs",
            condition_names: vec![],
        },
        TestCase {
            name: "the longest matching path prefix is prioritized #2",
            expect: Some(vec!["./dist/utils/index.js"]),
            imports_field: imports_field(json!({
              "#a/dist/utils/": "./dist/utils/",
              "#a/dist/": "./lib/"
            })),
            request: "#a/dist/utils/index.js",
            condition_names: vec![],
        },
        TestCase {
            name: "the longest matching path prefix is prioritized #3",
            expect: Some(vec!["./dist/utils/index.js"]),
            imports_field: imports_field(json!({
              "#a/dist/utils/index.js": "./dist/utils/index.js",
              "#a/dist/utils/": "./dist/utils/index.mjs",
              "#a/dist/": "./lib/"
            })),
            request: "#a/dist/utils/index.js",
            condition_names: vec![],
        },
        TestCase {
            name: "the longest matching path prefix is prioritized #4",
            expect: Some(vec!["./lib/index.mjs"]),
            imports_field: imports_field(json!({
              "#a/": {
                "browser": "./browser/"
              },
              "#a/dist/": "./lib/"
            })),
            request: "#a/dist/index.mjs",
            condition_names: vec!["browser"],
        },
        TestCase {
            name: "conditional mapping folder #1",
            // This behaves differently from enhanced_resolve, because `lodash/` is an an InvalidPackageConfig
            // expect: Some(vec!["lodash/index.js"]),
            expect: Some(vec!["./utils/index.js"]),
            imports_field: imports_field(json!({
              "#a/": {
                "browser": [
                  "lodash/",
                  "./utils/"
                ],
                "node": [
                  "./utils-node/"
                ]
              }
            })),
            request: "#a/index.js",
            condition_names: vec!["browser"],
        },
        // Duplicated due to not supporting returning an array
        TestCase {
            name: "conditional mapping folder #1",
            expect: Some(vec!["./utils/index.js"]),
            imports_field: imports_field(json!({
              "#a/": {
                "browser": [
                  "./utils/"
                ],
                "node": [
                  "./utils-node/"
                ]
              }
            })),
            request: "#a/index.js",
            condition_names: vec!["browser"],
        },
        TestCase {
            name: "conditional mapping folder #2",
            expect: Some(vec![]),
            imports_field: imports_field(json!({
              "#a/": {
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
            request: "#a/index.mjs",
            condition_names: vec![],
        },
        TestCase {
            name: "conditional mapping folder #3",
            expect: Some(vec!["./wpk/index.mjs"]),
            imports_field: imports_field(json!({
              "#a/": {
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
            request: "#a/index.mjs",
            condition_names: vec!["browser", "webpack"],
        },
        TestCase {
            name: "incorrect exports field #1",
            // We throw `PackageImportNotDefined`
            // expect: None,
            expect: Some(vec![]),
            imports_field: imports_field(json!({
              "/utils/": "./a/"
            })),
            request: "#a/index.mjs",
            condition_names: vec![],
        },
        TestCase {
            name: "incorrect exports field #2",
            // We throw `PackageImportNotDefined`
            // expect: None,
            expect: Some(vec![]),
            imports_field: imports_field(json!({
              "/utils/": {
                "browser": "./a/",
                "default": "./b/"
              }
            })),
            request: "#a/index.mjs",
            condition_names: vec!["browser"],
        },
        TestCase {
            name: "incorrect exports field #3",
            expect: Some(vec![]),
            imports_field: imports_field(json!({
              "#a/index": "./a/index.js"
            })),
            request: "#a/index.mjs",
            condition_names: vec![],
        },
        TestCase {
            name: "incorrect exports field #4",
            expect: Some(vec![]),
            imports_field: imports_field(json!({
              "#a/index.mjs": "./a/index.js"
            })),
            request: "#a/index",
            condition_names: vec![],
        },
        TestCase {
            name: "incorrect exports field #5",
            expect: Some(vec![]),
            imports_field: imports_field(json!({
              "#a/index": {
                "browser": "./a/index.js",
                "default": "./b/index.js"
              }
            })),
            request: "#a/index.mjs",
            condition_names: vec!["browser"],
        },
        TestCase {
            name: "incorrect exports field #6",
            expect: Some(vec![]),
            imports_field: imports_field(json!({
              "#a/index.mjs": {
                "browser": "./a/index.js",
                "default": "./b/index.js"
              }
            })),
            request: "#a/index",
            condition_names: vec!["browser"],
        },
        TestCase {
            name: "incorrect request #1",
            // We don't throw in `package_imports_exports_resolve`
            // expect: None,
            expect: Some(vec![]),
            imports_field: imports_field(json!({
              "#a/": "./a/"
            })),
            request: "/utils/index.mjs",
            condition_names: vec![],
        },
        TestCase {
            name: "incorrect request #2",
            // We don't throw in `package_imports_exports_resolve`
            // expect: None,
            expect: Some(vec![]),
            imports_field: imports_field(json!({
              "#a/": {
                "browser": "./a/",
                "default": "./b/"
              }
            })),
            request: "./utils/index.mjs",
            condition_names: vec!["browser"],
        },
        TestCase {
            name: "incorrect request #3",
            // We don't throw in `package_imports_exports_resolve`, it's thrown in `package_imports_resolve`
            // expect: None,
            expect: Some(vec![]),
            imports_field: imports_field(json!({
              "#a/": {
                "browser": "./a/",
                "default": "./b/"
              }
            })),
            request: "#",
            condition_names: vec!["browser"],
        },
        TestCase {
            name: "incorrect request #4",
            // We don't throw in `package_imports_exports_resolve`, it's thrown in `package_imports_resolve`
            // expect: None,
            expect: Some(vec![]),
            imports_field: imports_field(json!({
              "#a/": {
                "browser": "./a/",
                "default": "./b/"
              }
            })),
            request: "#/",
            condition_names: vec!["browser"],
        },
        TestCase {
            name: "incorrect request #5",
            // expect: None,
            expect: Some(vec![]),
            imports_field: imports_field(json!({
              "#a/": {
                "browser": "./a/",
                "default": "./b/"
              }
            })),
            request: "#a/",
            condition_names: vec!["browser"],
        },
        TestCase {
            name: "backtracking package base #1",
            // expect: Some(vec!["./dist/index"]),
            expect: Some(vec!["dist/index"]),
            imports_field: imports_field(json!({
              "#a/../../utils/": "./dist/"
            })),
            request: "#a/../../utils/index",
            condition_names: vec![],
        },
        TestCase {
            name: "backtracking package base #2",
            // We throw InvalidPackageTarget
            // expect: Some(vec!["./dist/../../utils/index"]),
            expect: None,
            imports_field: imports_field(json!({
              "#a/": "./dist/"
            })),
            request: "#a/../../utils/index",
            condition_names: vec![],
        },
        TestCase {
            name: "backtracking package base #3",
            // We throw InvalidPackageTarget
            // expect: Some(vec!["../src/index"]),
            expect: None,
            imports_field: imports_field(json!({
              "#a/": "../src/"
            })),
            request: "#a/index",
            condition_names: vec![],
        },
        TestCase {
            name: "backtracking package base #4",
            // We throw InvalidPackageTarget
            // expect: Some(vec!["./utils/../../../index"]),
            expect: None,
            imports_field: imports_field(json!({
              "#a/": {
                "browser": "./utils/../../../"
              }
            })),
            request: "#a/index",
            condition_names: vec!["browser"],
        },
        TestCase {
            name: "nested node_modules path #1",
            // expect: Some(vec!["moment/node_modules/lodash/dist/index.js"]),
            expect: None,
            imports_field: imports_field(json!({
              "#a/": {
                "browser": "moment/node_modules/"
              }
            })),
            request: "#a/lodash/dist/index.js",
            condition_names: vec!["browser"],
        },
        TestCase {
            name: "nested node_modules path #2",
            // We throw InvalidPackageTarget
            // expect: Some(vec!["../node_modules/lodash/dist/index.js"]),
            expect: None,
            imports_field: imports_field(json!({
              "#a/": "../node_modules/"
            })),
            request: "#a/lodash/dist/index.js",
            condition_names: vec![],
        },
        TestCase {
            name: "nested mapping #1",
            expect: Some(vec![]),
            imports_field: imports_field(json!({
              "#a/": {
                "browser": {
                  "webpack": "./",
                  "default": {
                    "node": "./node/"
                  }
                }
              }
            })),
            request: "#a/index.js",
            condition_names: vec!["browser"],
        },
        TestCase {
            name: "nested mapping #2",
            expect: Some(vec!["./index.js"]),
            imports_field: imports_field(json!({
              "#a/": {
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
            request: "#a/index.js",
            condition_names: vec!["browser", "webpack"],
        },
        // Duplicated due to not supporting returning an array
        TestCase {
            name: "nested mapping #2",
            expect: Some(vec!["./node/index.js"]),
            imports_field: imports_field(json!({
              "#a/": {
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
            request: "#a/index.js",
            condition_names: vec!["browser", "webpack"],
        },
        TestCase {
            name: "nested mapping #3",
            expect: Some(vec![]),
            imports_field: imports_field(json!({
              "#a/": {
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
            request: "#a/index.js",
            condition_names: vec!["webpack"],
        },
        TestCase {
            name: "nested mapping #4",
            // We throw NotFound
            // expect: Some(vec!["moment/node/index.js"]),
            expect: None,
            imports_field: imports_field(json!({
              "#a/": {
                "browser": {
                  "webpack": [
                    "./",
                    "./node/"
                  ],
                  "default": {
                    "node": "moment/node/"
                  }
                }
              }
            })),
            request: "#a/index.js",
            condition_names: vec!["node", "browser"],
        },
        TestCase {
            name: "nested mapping #5",
            expect: Some(vec![]),
            imports_field: imports_field(json!({
              "#a/": {
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
            request: "#a/index.js",
            condition_names: vec!["browser", "node"],
        },
        TestCase {
            name: "nested mapping #6",
            expect: Some(vec!["./index.js"]),
            imports_field: imports_field(json!({
              "#a/": {
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
            request: "#a/index.js",
            condition_names: vec!["browser", "node", "webpack"],
        },
        // Duplicated due to not supporting returning an array
        TestCase {
            name: "nested mapping #6",
            expect: Some(vec!["./node/index.js"]),
            imports_field: imports_field(json!({
              "#a/": {
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
            request: "#a/index.js",
            condition_names: vec!["browser", "node", "webpack"],
        },
        TestCase {
            name: "nested mapping #7",
            expect: Some(vec!["./y.js"]),
            imports_field: imports_field(json!({
              "#a": {
                "abc": {
                  "def": "./x.js"
                },
                "ghi": "./y.js"
              }
            })),
            request: "#a",
            condition_names: vec!["abc", "ghi"],
        },
        TestCase {
            name: "nested mapping #8",
            // We throw PackageImportNotDefined
            // expect: Some(vec![]),
            expect: None,
            imports_field: imports_field(json!({
              "#a": {
                "abc": {
                  "def": "./x.js",
                  "default": []
                },
                "ghi": "./y.js"
              }
            })),
            request: "#a",
            condition_names: vec!["abc", "ghi"],
        },
    ];

    for case in test_cases {
        let resolved = Resolver::default()
            .package_imports_exports_resolve(
                case.request,
                &case.imports_field,
                Path::new(""),
                true,
                &case.condition_names.iter().map(ToString::to_string).collect::<Vec<_>>(),
            )
            .map(|p| p.map(|p| p.to_path_buf()));
        if let Some(expect) = case.expect {
            if expect.is_empty() {
                assert!(matches!(resolved, Ok(None)), "{} {:?}", &case.name, &resolved);
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
