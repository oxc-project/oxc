//! https://github.com/webpack/enhanced-resolve/blob/main/test/exportsField.test.js
//!
//! The resolution tests are at the bottom of the file.

use oxc_resolver::{Resolution, ResolveOptions, Resolver};

#[test]
// resolve root using exports field, not a main field
fn root_not_main_field() {
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

#[test]
// resolve using exports field, not a browser field #1
fn exports_not_browser_field() {
    let fixture = super::fixture().join("exports-field");

    let resolver = Resolver::new(ResolveOptions {
        alias_fields: vec!["browser".into()],
        condition_names: vec!["webpack".into()],
        extensions: vec![".js".into()],
        ..ResolveOptions::default()
    });

    let resolved_path =
        resolver.resolve(&fixture, "exports-field/dist/main.js").map(Resolution::full_path);
    assert_eq!(resolved_path, Ok(fixture.join("node_modules/exports-field/lib/lib2/main.js")));
}
