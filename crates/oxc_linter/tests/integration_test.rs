use oxc_linter::{FixKind, RuleCategory, RuleFixMeta, RuleFlags, RuleMeta, rules::RULES};
use oxc_macros::declare_oxc_lint_test;

struct TestRule;

mod shared_config {
    #[derive(schemars::JsonSchema)]
    pub struct Config;
}

struct TestRuleWithSharedConfig;

struct TestTsgolintRule;

struct TestTsgolintRuleWithConfig;

struct TestRecommendedRule;

struct TestRecommendedRuleWithFix;

struct TestRecommendedTsgolintRuleWithConfig;

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

declare_oxc_lint_test!(
    /// Type-aware rule without configuration
    TestTsgolintRule(tsgolint),
    typescript,
    correctness,
    version = "next",
);

declare_oxc_lint_test!(
    /// Type-aware rule with configuration
    TestTsgolintRuleWithConfig(tsgolint),
    typescript,
    correctness,
    version = "next",
    config = shared_config::Config,
);

declare_oxc_lint_test!(
    /// Recommended rule without fixes or configuration
    TestRecommendedRule,
    eslint,
    restriction,
    recommended,
    version = "next",
);

declare_oxc_lint_test!(
    /// Recommended rule with a fix kind after the recommended keyword
    TestRecommendedRuleWithFix,
    eslint,
    restriction,
    recommended,
    conditional_fix,
    version = "next",
);

declare_oxc_lint_test!(
    /// Recommended type-aware rule with configuration and a fix kind before the keyword
    TestRecommendedTsgolintRuleWithConfig(tsgolint),
    typescript,
    correctness,
    conditional_fix,
    version = "next",
    config = shared_config::Config,
    recommended,
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
    assert_eq!(TestRuleWithSharedConfig::FLAGS, RuleFlags::HAS_CONFIG);

    let mut generator =
        schemars::r#gen::SchemaGenerator::new(schemars::r#gen::SchemaSettings::default());
    assert!(TestRuleWithSharedConfig::config_schema(&mut generator).is_some());
}

#[test]
fn test_rule_flags() {
    struct DefaultFlagsRule;

    impl RuleMeta for DefaultFlagsRule {
        const NAME: &'static str = "default-flags-rule";
        const PLUGIN: &'static str = "eslint";
        const CATEGORY: RuleCategory = RuleCategory::Correctness;
        const VERSION: &'static str = "next";
    }

    assert_eq!(DefaultFlagsRule::FLAGS, RuleFlags::empty());
    assert_eq!(TestRule::FLAGS, RuleFlags::empty());
    assert_eq!(TestRuleWithSharedConfig::FLAGS, RuleFlags::HAS_CONFIG);
    assert_eq!(TestTsgolintRule::FLAGS, RuleFlags::IS_TSGOLINT_RULE);
    assert_eq!(
        TestTsgolintRuleWithConfig::FLAGS,
        RuleFlags::IS_TSGOLINT_RULE | RuleFlags::HAS_CONFIG
    );
}

#[test]
fn test_recommended_rule_flags() {
    assert_eq!(TestRecommendedRule::FLAGS, RuleFlags::IS_RECOMMENDED);
    assert_eq!(TestRecommendedRule::FIX, RuleFixMeta::None);
    assert_eq!(TestRecommendedRuleWithFix::FLAGS, RuleFlags::IS_RECOMMENDED);
    assert_eq!(TestRecommendedRuleWithFix::FIX, RuleFixMeta::Conditional(FixKind::Fix));
    assert_eq!(
        TestRecommendedTsgolintRuleWithConfig::FLAGS,
        RuleFlags::IS_TSGOLINT_RULE | RuleFlags::HAS_CONFIG | RuleFlags::IS_RECOMMENDED
    );
    assert_eq!(TestRecommendedTsgolintRuleWithConfig::FIX, RuleFixMeta::Conditional(FixKind::Fix));
}

#[test]
fn test_rule_enum_flags() {
    let cases = [
        ("eslint", "no-debugger", RuleFlags::empty()),
        ("eslint", "eqeqeq", RuleFlags::HAS_CONFIG),
        ("typescript", "no-unsafe-assignment", RuleFlags::IS_TSGOLINT_RULE),
        ("typescript", "no-floating-promises", RuleFlags::IS_TSGOLINT_RULE | RuleFlags::HAS_CONFIG),
    ];

    for (plugin, name, expected) in cases {
        let rule = RULES
            .iter()
            .find(|rule| rule.plugin_name() == plugin && rule.name() == name)
            .unwrap_or_else(|| panic!("Rule {plugin}/{name} must be registered"));
        assert_eq!(rule.flags(), expected, "unexpected flags for {plugin}/{name}");
        assert_eq!(
            rule.is_tsgolint_rule(),
            expected.contains(RuleFlags::IS_TSGOLINT_RULE),
            "unexpected tsgolint status for {plugin}/{name}"
        );
        assert_eq!(
            rule.is_recommended(),
            expected.contains(RuleFlags::IS_RECOMMENDED),
            "unexpected recommended status for {plugin}/{name}"
        );
        assert_eq!(
            rule.has_config(),
            expected.contains(RuleFlags::HAS_CONFIG),
            "unexpected configuration status for {plugin}/{name}"
        );
    }
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
