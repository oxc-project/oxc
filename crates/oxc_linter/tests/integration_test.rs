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

struct TestRule2 {
    #[allow(dead_code)]
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
    assert_eq!(TestRule::documentation().unwrap(), "Dummy description\n# which is multiline\n");

    // Ensure structs with fields can be passed to the macro
    assert_eq!(TestRule2::documentation().unwrap(), "Dummy description2\n");

    // Auto-generated kebab-case name
    assert_eq!(TestRule::NAME, "test-rule");

    // plugin name is passed to const
    assert_eq!(TestRule::PLUGIN, "eslint");
}
