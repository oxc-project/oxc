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
    const EXPECTED: [(&str, RuleCategory, bool); 22] = [
        ("capitalized-calls", RuleCategory::Suspicious, false),
        ("error-boundaries", RuleCategory::Correctness, true),
        ("exhaustive-effect-dependencies", RuleCategory::Suspicious, false),
        ("globals", RuleCategory::Correctness, true),
        ("hooks", RuleCategory::Suspicious, false),
        ("immutability", RuleCategory::Correctness, true),
        ("incompatible-library", RuleCategory::Correctness, true),
        ("invariant", RuleCategory::Restriction, false),
        ("memo-dependencies", RuleCategory::Suspicious, false),
        ("no-deriving-state-in-effects", RuleCategory::Perf, false),
        ("preserve-manual-memoization", RuleCategory::Correctness, true),
        ("purity", RuleCategory::Correctness, true),
        ("refs", RuleCategory::Correctness, true),
        ("rule-suppression", RuleCategory::Restriction, false),
        ("set-state-in-effect", RuleCategory::Correctness, true),
        ("set-state-in-render", RuleCategory::Correctness, true),
        ("static-components", RuleCategory::Correctness, true),
        ("syntax", RuleCategory::Restriction, false),
        ("todo", RuleCategory::Restriction, false),
        ("unsupported-syntax", RuleCategory::Restriction, true),
        ("use-memo", RuleCategory::Correctness, true),
        ("void-use-memo", RuleCategory::Correctness, false),
    ];

    let names =
        EXPECTED.iter().map(|(name, _, _)| *name).collect::<std::collections::BTreeSet<_>>();
    assert_eq!(names.len(), EXPECTED.len(), "React Compiler rule names must be unique");

    for (name, expected_category, has_upstream_docs) in EXPECTED {
        let rule = RULES
            .iter()
            .find(|rule| rule.plugin_name() == "react" && rule.name() == name)
            .unwrap_or_else(|| panic!("React Compiler rule react/{name} must be registered"));
        assert_eq!(rule.category(), expected_category, "unexpected category for react/{name}");

        #[cfg(not(feature = "ruledocs"))]
        let _ = has_upstream_docs;

        #[cfg(feature = "ruledocs")]
        assert_eq!(
            rule.documentation().is_some_and(|docs| docs.contains(&format!(
                "https://react.dev/reference/eslint-plugin-react-hooks/lints/{name}"
            ))),
            has_upstream_docs,
            "unexpected upstream documentation link state for react/{name}"
        );
    }
}
