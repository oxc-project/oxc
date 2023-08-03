//! <https://github.com/webpack/enhanced-resolve/blob/main/test/extension-alias.test.js>

use std::path::PathBuf;

use crate::{Resolution, ResolveError, ResolveOptions, Resolver};

fn fixture() -> PathBuf {
    super::fixture().join("extension-alias")
}

#[test]
fn extension_alias() {
    let f = fixture();

    let resolver = Resolver::new(ResolveOptions {
        extensions: vec![".js".into()],
        main_files: vec!["index.js".into()],
        extension_alias: vec![
            (".js".into(), vec![".ts".into(), ".js".into()]),
            (".mjs".into(), vec![".mts".into()]),
        ],
        ..ResolveOptions::default()
    });

    #[rustfmt::skip]
    let pass = [
        ("should alias fully specified file", f.clone(), "./index.js", f.join("index.ts")),
        ("should alias fully specified file when there are two alternatives", f.clone(), "./dir/index.js", f.join("dir/index.ts")),
        ("should also allow the second alternative", f.clone(), "./dir2/index.js", f.join("dir2/index.js")),
        ("should support alias option without an array", f.clone(), "./dir2/index.mjs", f.join("dir2/index.mts")),
    ];

    for (comment, path, request, expected) in pass {
        let resolved_path = resolver.resolve(&path, request).map(Resolution::full_path);
        assert_eq!(resolved_path, Ok(expected), "{comment} {path:?} {request}");
    }

    #[rustfmt::skip]
    let fail = [
        ("should not allow to fallback to the original extension or add extensions", f, "./index.mjs"),
    ];

    for (comment, path, request) in fail {
        let resolution = resolver.resolve(&path, request);
        assert_eq!(resolution, Err(ResolveError::ExtensionAlias), "{comment} {path:?} {request}");
    }
}

#[test]
fn not_apply_to_extension_nor_main_files() {
    // should not apply extension alias to extensions or mainFiles field
    let options = ResolveOptions {
        extensions: vec![".js".into()],
        main_files: vec!["index.js".into()],
        extension_alias: vec![(".js".into(), vec![])],
        ..ResolveOptions::default()
    };
    let resolver = Resolver::new(options);
    let f = fixture();

    #[rustfmt::skip]
    let pass = [
        ("directory", f.clone(), "./dir2", "dir2/index.js"),
        ("file", f.clone(), "./dir2/index", "dir2/index.js"),
    ];

    for (comment, path, request, expected) in pass {
        let resolved_path = resolver.resolve(&path, request).map(Resolution::full_path);
        let expected = f.join(expected);
        assert_eq!(resolved_path, Ok(expected), "{comment} {path:?} {request}");
    }
}
