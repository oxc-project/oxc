//! Tests for tsconfig project references

use crate::{ResolveError, ResolveOptions, Resolver, TsconfigOptions, TsconfigReferences};

#[test]
fn auto() {
    let f = super::fixture_root().join("tsconfig_project_references");

    let resolver = Resolver::new(ResolveOptions {
        tsconfig: Some(TsconfigOptions {
            config_file: f.join("app"),
            references: TsconfigReferences::Auto,
        }),
        ..ResolveOptions::default()
    });

    #[rustfmt::skip]
    let pass = [
        // Test normal paths alias
        (f.join("app"), "@/index.ts", f.join("app/aliased/index.ts")),
        (f.join("app"), "@/../index.ts", f.join("app/index.ts")),
        // Test project reference
        (f.join("project_a"), "@/index.ts", f.join("project_a/aliased/index.ts")),
        (f.join("project_b/src"), "@/index.ts", f.join("project_b/src/aliased/index.ts")),
        // Does not have paths alias
        (f.join("project_a"), "./index.ts", f.join("project_a/index.ts")),
        (f.join("project_c"), "./index.ts", f.join("project_c/index.ts")),
    ];

    for (path, request, expected) in pass {
        let resolved_path = resolver.resolve(&path, request).map(|f| f.full_path());
        assert_eq!(resolved_path, Ok(expected), "{request} {path:?}");
    }
}

#[test]
fn disabled() {
    let f = super::fixture_root().join("tsconfig_project_references");

    let resolver = Resolver::new(ResolveOptions {
        tsconfig: Some(TsconfigOptions {
            config_file: f.join("app"),
            references: TsconfigReferences::Disabled,
        }),
        ..ResolveOptions::default()
    });

    #[rustfmt::skip]
    let pass = [
        // Test normal paths alias
        (f.join("app"), "@/index.ts", Ok(f.join("app/aliased/index.ts"))),
        (f.join("app"), "@/../index.ts", Ok(f.join("app/index.ts"))),
        // Test project reference
        (f.join("project_a"), "@/index.ts", Err(ResolveError::NotFound(f.join("project_a")))),
        (f.join("project_b/src"), "@/index.ts", Err(ResolveError::NotFound(f.join("project_b/src")))),
        // Does not have paths alias
        (f.join("project_a"), "./index.ts", Ok(f.join("project_a/index.ts"))),
        (f.join("project_c"), "./index.ts", Ok(f.join("project_c/index.ts"))),
    ];

    for (path, request, expected) in pass {
        let resolved_path = resolver.resolve(&path, request).map(|f| f.full_path());
        assert_eq!(resolved_path, expected, "{request} {path:?}");
    }
}

#[test]
fn manual() {
    let f = super::fixture_root().join("tsconfig_project_references");

    let resolver = Resolver::new(ResolveOptions {
        tsconfig: Some(TsconfigOptions {
            config_file: f.join("app"),
            references: TsconfigReferences::Paths(vec!["../project_a/conf.json".into()]),
        }),
        ..ResolveOptions::default()
    });

    #[rustfmt::skip]
    let pass = [
        // Test normal paths alias
        (f.join("app"), "@/index.ts", Ok(f.join("app/aliased/index.ts"))),
        (f.join("app"), "@/../index.ts", Ok(f.join("app/index.ts"))),
        // Test project reference
        (f.join("project_a"), "@/index.ts", Ok(f.join("project_a/aliased/index.ts"))),
        (f.join("project_b/src"), "@/index.ts", Err(ResolveError::NotFound(f.join("project_b/src")))),
        // Does not have paths alias
        (f.join("project_a"), "./index.ts", Ok(f.join("project_a/index.ts"))),
        (f.join("project_c"), "./index.ts", Ok(f.join("project_c/index.ts"))),
    ];

    for (path, request, expected) in pass {
        let resolved_path = resolver.resolve(&path, request).map(|f| f.full_path());
        assert_eq!(resolved_path, expected, "{request} {path:?}");
    }
}
