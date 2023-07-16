//! <https://github.com/webpack/enhanced-resolve/blob/main/test/scoped-packages.test.js>

use std::path::PathBuf;

use oxc_resolver::{ResolveError, ResolveOptions, Resolver};

fn fixture() -> PathBuf {
    super::fixture().join("scoped")
}

#[test]
fn scoped_packages() -> Result<(), ResolveError> {
    let f = fixture();

    let options =
        ResolveOptions { alias_fields: vec!["browser".into()], ..ResolveOptions::default() };

    let resolver = Resolver::new(options);

    #[rustfmt::skip]
    let pass = [
        ("main field should work", f.clone(), "@scope/pack1", f.join("./node_modules/@scope/pack1/main.js")),
        ("browser field should work", f.clone(), "@scope/pack2", f.join("./node_modules/@scope/pack2/main.js")),
        ("folder request should work", f.clone(), "@scope/pack2/lib", f.join("./node_modules/@scope/pack2/lib/index.js"))
    ];

    for (comment, path, request, expected) in pass {
        let resolution = resolver.resolve(&f, request)?;
        let resolved_path = resolution.path();
        assert_eq!(resolved_path, expected, "{comment} {path:?} {request}");
    }

    Ok(())
}
