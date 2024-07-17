mod jest;
mod jsdoc;
mod nextjs;
mod promise;
mod react;
mod react_perf;
mod tree_shaking;
mod unicorn;

use crate::LintContext;

pub use self::{
    jest::*, jsdoc::*, nextjs::*, promise::*, react::*, react_perf::*, tree_shaking::*, unicorn::*,
};

/// Check if the Jest rule is adapted to Vitest.
/// Many Vitest rule are essentially ports of Jest plugin rules with minor modifications.
/// For these rules, we use the corresponding jest rules with some adjustments for compatibility.
pub fn is_jest_rule_adapted_to_vitest(rule_name: &str) -> bool {
    let jest_rules: &[&str] = &[
        "consistent-test-it",
        "no-disabled-tests",
        "no-focused-tests",
        "no-test-prefixes",
        "prefer-hooks-in-order",
        "valid-describe-callback",
        "valid-expect",
    ];

    jest_rules.contains(&rule_name)
}

#[derive(Clone, Copy)]
pub enum TestPluginName {
    Jest,
    Vitest,
}

impl std::fmt::Display for TestPluginName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestPluginName::Jest => write!(f, "eslint-plugin-jest"),
            TestPluginName::Vitest => write!(f, "eslint-plugin-vitest"),
        }
    }
}

pub fn get_test_plugin_name(ctx: &LintContext) -> TestPluginName {
    if is_using_vitest(ctx) {
        TestPluginName::Vitest
    } else {
        TestPluginName::Jest
    }
}

fn is_using_vitest(ctx: &LintContext) -> bool {
    // If import 'vitest' is found, we assume the user is using vitest.
    if ctx
        .semantic()
        .module_record()
        .import_entries
        .iter()
        .any(|entry| entry.module_request.name().as_str() == "vitest")
    {
        return true;
    }

    // Or, find the eslint config file
    ctx.rules().iter().any(|rule| rule.plugin_name == "vitest")
}
