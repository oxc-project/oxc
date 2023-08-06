//! https://github.com/webpack/enhanced-resolve/blob/main/test/fullSpecified.test.js

use std::path::PathBuf;

use crate::{AliasValue, ResolveOptions, ResolverGeneric};

use super::memory_fs::MemoryFS;

#[test]
#[cfg(not(target_os = "windows"))] // MemoryFS's path separator is always `/` so the test will not pass in windows.
fn test() {
    use crate::Resolution;

    let file_system = MemoryFS::new(&[
        ("/a/node_modules/package1/index.js", ""),
        ("/a/node_modules/package1/file.js", ""),
        ("/a/node_modules/package2/package.json", r#"{"main":"a"}"#),
        ("/a/node_modules/package2/a.js", ""),
        ("/a/node_modules/package3/package.json", r#"{"main":"dir"}"#),
        ("/a/node_modules/package3/dir/index.js", ""),
        ("/a/node_modules/package4/package.json", r#"{"browser":{"./a.js":"./b"}}"#),
        ("/a/node_modules/package4/a.js", ""),
        ("/a/node_modules/package4/b.js", ""),
        ("/a/abc.js", ""),
        ("/a/dir/index.js", ""),
        ("/a/index.js", ""),
    ]);

    let options = ResolveOptions {
        alias: vec![
            ("alias1".into(), vec![AliasValue::Path("/a/abc".into())]),
            ("alias2".into(), vec![AliasValue::Path("/a".into())]),
        ],
        alias_fields: vec!["browser".into()],
        fully_specified: true,
        ..ResolveOptions::default()
    };

    let resolver = ResolverGeneric::<MemoryFS>::new_with_file_system(options, file_system);

    let failing_resolves = [
        ("no extensions", "./abc"),
        ("no extensions (absolute)", "/a/abc"),
        ("no extensions in packages", "package1/file"),
        ("no directories", "."),
        ("no directories 2", "./"),
        ("no directories in packages", "package3/dir"),
        ("no extensions in packages 2", "package3/a"),
    ];

    for (comment, request) in failing_resolves {
        let resolution = resolver.resolve("/a", request);
        assert!(resolution.is_err(), "{comment} {request}");
    }

    let successful_resolves = [
        ("fully relative", "./abc.js", "/a/abc.js"),
        ("fully absolute", "/a/abc.js", "/a/abc.js"),
        ("fully relative in package", "package1/file.js", "/a/node_modules/package1/file.js"),
        ("extensions in mainFiles", "package1", "/a/node_modules/package1/index.js"),
        ("extensions in mainFields", "package2", "/a/node_modules/package2/a.js"),
        ("extensions in alias", "alias1", "/a/abc.js"),
        ("directories in alias", "alias2", "/a/index.js"),
        ("directories in packages", "package3", "/a/node_modules/package3/dir/index.js"),
        ("extensions in aliasFields", "package4/a.js", "/a/node_modules/package4/b.js"),
    ];

    for (comment, request, expected) in successful_resolves {
        let resolution = resolver.resolve("/a", request).map(Resolution::full_path);
        assert_eq!(resolution, Ok(PathBuf::from(expected)), "{comment} {request}");
    }
}
