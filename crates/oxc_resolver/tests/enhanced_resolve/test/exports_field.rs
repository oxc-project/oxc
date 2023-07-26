//! https://github.com/webpack/enhanced-resolve/blob/main/test/exportsField.test.js
//!
//! The resolution tests are at the bottom of the file.

use oxc_resolver::{Resolution, ResolveOptions, Resolver};

#[test]
fn exports_field() {
    let fixture = super::fixture().join("exports-field");

    let resolver = Resolver::new(ResolveOptions {
        extensions: vec![".js".into()],
        // fullySpecified: true,
        // conditionNames: ["webpack"]
        ..ResolveOptions::default()
    });

    let resolved_path = resolver.resolve(&fixture, "exports-field").map(Resolution::full_path);
    assert_eq!(resolved_path, Ok(fixture.join("node_modules/exports-field/x.js")));
}
