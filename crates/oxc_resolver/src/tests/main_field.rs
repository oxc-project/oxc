//! Not part of enhanced_resolve's test suite

use crate::{ResolveOptions, Resolver};

#[test]
fn test() {
    let f = super::fixture().join("restrictions");

    let resolver = Resolver::new(ResolveOptions {
        main_fields: vec!["style".into()],
        ..ResolveOptions::default()
    });

    let resolution = resolver.resolve(&f, "pck2").map(|r| r.full_path());
    assert_eq!(resolution, Ok(f.join("node_modules/pck2/index.css")));

    let resolver = Resolver::new(ResolveOptions {
        main_fields: vec!["module".into(), "main".into()],
        ..ResolveOptions::default()
    });

    let resolution = resolver.resolve(&f, "pck2").map(|r| r.full_path());
    assert_eq!(resolution, Ok(f.join("node_modules/pck2/module.js")));
}
