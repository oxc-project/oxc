//! <https://github.com/webpack/enhanced-resolve/blob/main/test/alias.test.js>

use std::path::{Path, PathBuf};

use crate::{AliasValue, Resolution, ResolveError, ResolveOptions, Resolver, ResolverGeneric};

use super::memory_fs::MemoryFS;

#[test]
#[cfg(not(target_os = "windows"))] // MemoryFS's path separator is always `/` so the test will not pass in windows.
fn alias() {
    let f = Path::new("/");

    let file_system = MemoryFS::new(&[
        ("/a/index", ""),
        ("/a/dir/index", ""),
        ("/recursive/index", ""),
        ("/recursive/dir/index", ""),
        ("/b/index", ""),
        ("/b/dir/index", ""),
        ("/c/index", ""),
        ("/c/dir/index", ""),
        ("/d/index.js", ""),
        ("/d/dir/.empty", ""),
        ("/e/index", ""),
        ("/e/anotherDir/index", ""),
        ("/e/dir/file", ""),
    ]);

    #[rustfmt::skip]
    let options = ResolveOptions {
        alias: vec![
            ("aliasA".into(), vec![AliasValue::Path("a".into())]),
            ("b$".into(), vec![AliasValue::Path("a/index".into())]),
            ("c$".into(), vec![AliasValue::Path("/a/index".into())]),
            ("multiAlias".into(), vec![AliasValue::Path("b".into()), AliasValue::Path("c".into()), AliasValue::Path("d".into()), AliasValue::Path("e".into()), AliasValue::Path("a".into())]),
            ("recursive".into(), vec![AliasValue::Path("recursive/dir".into())]),
            ("/d/dir".into(), vec![AliasValue::Path("/c/dir".into())]),
            ("/d/index.js".into(), vec![AliasValue::Path("/c/index".into())]),
            // alias configuration should work
            ("#".into(), vec![AliasValue::Path("/c/dir".into())]),
            ("@".into(), vec![AliasValue::Path("/c/dir".into())]),
            ("ignored".into(), vec![AliasValue::Ignore]),
        ],
        modules: vec!["/".into()],
        ..ResolveOptions::default()
    };

    let resolver = ResolverGeneric::<MemoryFS>::new_with_file_system(options, file_system);

    #[rustfmt::skip]
    let pass = [
        ("should resolve a not aliased module 1", "a", "/a/index"),
        ("should resolve a not aliased module 2", "a/index", "/a/index"),
        ("should resolve a not aliased module 3", "a/dir", "/a/dir/index"),
        ("should resolve a not aliased module 4", "a/dir/index", "/a/dir/index"),
        ("should resolve an aliased module 1", "aliasA", "/a/index"),
        ("should resolve an aliased module 2", "aliasA/index", "/a/index"),
        ("should resolve an aliased module 3", "aliasA/dir", "/a/dir/index"),
        ("should resolve an aliased module 4", "aliasA/dir/index", "/a/dir/index"),
        ("should resolve '#' alias 1", "#", "/c/dir/index"),
        ("should resolve '#' alias 2", "#/index", "/c/dir/index"),
        ("should resolve '@' alias 1", "@", "/c/dir/index"),
        ("should resolve '@' alias 2", "@/index", "/c/dir/index"),
        ("should resolve a recursive aliased module 1", "recursive", "/recursive/dir/index"),
        ("should resolve a recursive aliased module 2", "recursive/index", "/recursive/dir/index"),
        // TODO recursive
        // ("should resolve a recursive aliased module 3", "recursive/dir", "/recursive/dir/index"),
        // ("should resolve a recursive aliased module 4", "recursive/dir/index", "/recursive/dir/index"),
        ("should resolve a file aliased module 1", "b", "/a/index"),
        ("should resolve a file aliased module 2", "c", "/a/index"),
        ("should resolve a file aliased module with a query 1", "b?query", "/a/index?query"),
        ("should resolve a file aliased module with a query 2", "c?query", "/a/index?query"),
        ("should resolve a path in a file aliased module 1", "b/index", "/b/index"),
        ("should resolve a path in a file aliased module 2", "b/dir", "/b/dir/index"),
        ("should resolve a path in a file aliased module 3", "b/dir/index", "/b/dir/index"),
        ("should resolve a path in a file aliased module 4", "c/index", "/c/index"),
        ("should resolve a path in a file aliased module 5", "c/dir", "/c/dir/index"),
        ("should resolve a path in a file aliased module 6", "c/dir/index", "/c/dir/index"),
        // TODO aliased file
        // ("should resolve a file aliased file 1", "d", "/c/index"),
        // ("should resolve a file aliased file 2", "d/dir/index", "/c/dir/index"),
        ("should resolve a file in multiple aliased dirs 1", "multiAlias/dir/file", "/e/dir/file"),
        ("should resolve a file in multiple aliased dirs 2", "multiAlias/anotherDir", "/e/anotherDir/index"),
    ];

    for (comment, request, expected) in pass {
        let resolved_path = resolver.resolve(f, request).map(Resolution::full_path);
        assert_eq!(resolved_path, Ok(PathBuf::from(expected)), "{comment} {request}");
    }

    #[rustfmt::skip]
    let ignore = [
        ("should resolve an ignore module", "ignored", ResolveError::Ignored(f.join("ignored").into_boxed_path()))
    ];

    for (comment, request, expected) in ignore {
        let resolution = resolver.resolve(f, request);
        assert_eq!(resolution, Err(expected), "{comment} {request}");
    }
}

#[test]
#[ignore = "TODO: absolute path"]
fn absolute_path() {
    let f = super::fixture();
    let resolver = Resolver::new(ResolveOptions {
        alias: vec![(f.join("foo").to_str().unwrap().to_string(), vec![AliasValue::Ignore])],
        modules: vec![f.clone().to_str().unwrap().to_string()],
        ..ResolveOptions::default()
    });
    let resolution = resolver.resolve(&f, "foo/index");
    assert_eq!(resolution, Err(ResolveError::Ignored(f.into_boxed_path())));
}
