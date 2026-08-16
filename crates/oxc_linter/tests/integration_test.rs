use oxc_linter::{RuleCategory, RuleMeta, rules::RULES};
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

#[test]
fn test_react_compiler_rule_categories() {
    const EXPECTED: [(&str, RuleCategory); 23] = [
        ("capitalized-calls", RuleCategory::Suspicious),
        ("error-boundaries", RuleCategory::Correctness),
        ("exhaustive-effect-dependencies", RuleCategory::Correctness),
        ("gating", RuleCategory::Correctness),
        ("globals", RuleCategory::Correctness),
        ("hooks", RuleCategory::Correctness),
        ("immutability", RuleCategory::Correctness),
        ("incompatible-library", RuleCategory::Suspicious),
        ("invariant", RuleCategory::Restriction),
        ("memo-dependencies", RuleCategory::Correctness),
        ("no-deriving-state-in-effects", RuleCategory::Perf),
        ("preserve-manual-memoization", RuleCategory::Correctness),
        ("purity", RuleCategory::Correctness),
        ("refs", RuleCategory::Correctness),
        ("rule-suppression", RuleCategory::Restriction),
        ("set-state-in-effect", RuleCategory::Perf),
        ("set-state-in-render", RuleCategory::Correctness),
        ("static-components", RuleCategory::Correctness),
        ("syntax", RuleCategory::Correctness),
        ("todo", RuleCategory::Restriction),
        ("unsupported-syntax", RuleCategory::Restriction),
        ("use-memo", RuleCategory::Correctness),
        ("void-use-memo", RuleCategory::Correctness),
    ];

    let names = EXPECTED.iter().map(|(name, _)| *name).collect::<std::collections::BTreeSet<_>>();
    assert_eq!(names.len(), EXPECTED.len(), "React Compiler rule names must be unique");

    for (name, expected_category) in EXPECTED {
        let rule = RULES
            .iter()
            .find(|rule| rule.plugin_name() == "react" && rule.name() == name)
            .unwrap_or_else(|| panic!("React Compiler rule react/{name} must be registered"));
        assert_eq!(rule.category(), expected_category, "unexpected category for react/{name}");

        #[cfg(feature = "ruledocs")]
        assert!(
            rule.documentation().is_some_and(|docs| docs.contains(&format!(
                "https://react.dev/reference/eslint-plugin-react-hooks/lints/{name}"
            ))),
            "missing upstream documentation link for react/{name}"
        );
    }
}
