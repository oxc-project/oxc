use oxc_linter::{RuleCategory, RuleMeta};
use oxc_macros::declare_oxc_lint_test;

struct TestRule;

mod shared_config {
    #[derive(schemars::JsonSchema)]
    pub struct Config;
}

struct TestRuleWithSharedConfig;

declare_oxc_lint_test!(
    /// Dummy description
    /// # which is multiline
    TestRule,
    eslint,
    correctness,
    version = "next"
);

#[expect(dead_code)]
struct TestRule2 {
    dummy_field: u8,
}

declare_oxc_lint_test!(
    /// Dummy description2
    TestRule2,
    eslint,
    correctness,
    version = "next"
);

declare_oxc_lint_test!(
    /// Dummy description3
    TestRuleWithSharedConfig,
    eslint,
    correctness,
    version = "next",
    config = shared_config::Config,
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

    // Shared config paths can be used as config schema sources.
    let has_config = TestRuleWithSharedConfig::HAS_CONFIG;
    assert!(has_config);

    let mut generator =
        schemars::r#gen::SchemaGenerator::new(schemars::r#gen::SchemaSettings::default());
    assert!(TestRuleWithSharedConfig::config_schema(&mut generator).is_some());
}
