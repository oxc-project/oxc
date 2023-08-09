//! <https://github.com/webpack/enhanced-resolve/blob/main/test/scoped-packages.test.js>

use crate::{ResolveOptions, Resolver};

#[test]
fn scoped_packages() {
    let f = super::fixture().join("scoped");

    let resolver = Resolver::new(ResolveOptions {
        alias_fields: vec!["browser".into()],
        ..ResolveOptions::default()
    });

    #[rustfmt::skip]
    let pass = [
        ("main field should work", f.clone(), "@scope/pack1", f.join("./node_modules/@scope/pack1/main.js")),
        ("browser field should work", f.clone(), "@scope/pack2", f.join("./node_modules/@scope/pack2/main.js")),
        ("folder request should work", f.clone(), "@scope/pack2/lib", f.join("./node_modules/@scope/pack2/lib/index.js"))
    ];

    for (comment, path, request, expected) in pass {
        let resolved_path = resolver.resolve(&f, request).map(|r| r.full_path());
        assert_eq!(resolved_path, Ok(expected), "{comment} {path:?} {request}");
    }
}
