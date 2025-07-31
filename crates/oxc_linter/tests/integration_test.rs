use oxc_linter::{RuleCategory, RuleMeta};
use oxc_macros::declare_oxc_lint_test;

struct TestRule;

declare_oxc_lint_test!(
    /// Dummy description
    /// # which is multiline
    TestRule,
    eslint,
    correctness
);

#[expect(dead_code)]
struct TestRule2 {
    dummy_field: u8,
}

declare_oxc_lint_test!(
    /// Dummy description2
    TestRule2,
    eslint,
    correctness
);

#[test]
fn test_declare_oxc_lint() {
    // Simple, multiline documentation
    #[cfg(feature = "ruledocs")]
    assert_eq!(TestRule::documentation().unwrap(), "Dummy description\n# which is multiline\n");

    // Ensure structs with fields can be passed to the macro
    #[cfg(feature = "ruledocs")]
    assert_eq!(TestRule2::documentation().unwrap(), "Dummy description2\n");

    // Auto-generated kebab-case name
    assert_eq!(TestRule::NAME, "test-rule");

    // plugin name is passed to const
    assert_eq!(TestRule::PLUGIN, "eslint");
}

/// Test to ensure resolver cache clearing doesn't panic when called
#[test]
fn test_resolver_cache_clearing_integration() {
    // This test verifies that the resolver cache clearing functionality
    // compiles and can be called without panicking.
    // The actual memory leak fix is in Runtime::run_source() method.

    use oxc_resolver::{ResolveOptions, Resolver};

    // Create a resolver similar to how it's done in Runtime::get_resolver
    let resolver = Resolver::new(ResolveOptions {
        extensions: vec![".js".into(), ".ts".into(), ".jsx".into(), ".tsx".into()],
        main_fields: vec!["module".into(), "main".into()],
        condition_names: vec!["module".into(), "import".into()],
        ..ResolveOptions::default()
    });

    // Test that clear_cache can be called without panicking
    resolver.clear_cache();

    // If we reach here, the clear_cache method works correctly
    println!("Resolver cache clearing integration test passed");
}
