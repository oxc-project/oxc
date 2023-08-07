//! <https://github.com/webpack/enhanced-resolve/blob/main/test/restrictions.test.js>

use crate::{ResolveError, ResolveOptions, Resolver, Restriction};

// TODO: regex
// * should respect RegExp restriction
// * should try to find alternative #1
// * should try to find alternative #2
// * should try to find alternative #3

#[test]
// should respect string restriction
fn restriction1() {
    let fixture = super::fixture();
    let f = fixture.join("restrictions");

    let resolver = Resolver::new(ResolveOptions {
        extensions: vec![".js".into()],
        restrictions: vec![Restriction::Path(f.clone())],
        ..ResolveOptions::default()
    });

    let resolution = resolver.resolve(&f, "pck2");
    assert_eq!(resolution, Err(ResolveError::Restriction(fixture.join("c.js"))));
}
