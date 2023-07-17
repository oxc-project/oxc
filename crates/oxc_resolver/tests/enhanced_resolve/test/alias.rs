//! <https://github.com/webpack/enhanced-resolve/blob/main/test/alias.test.js>

use oxc_resolver::{ResolveOptions, ResolverGeneric};

use crate::MemoryFS;

#[test]
fn alias() {
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
    let options = ResolveOptions::default();
    let resolver = ResolverGeneric::<MemoryFS>::new_with_file_system(options, file_system);
    assert!(resolver.resolve("/a/index", ".").is_ok());
}
