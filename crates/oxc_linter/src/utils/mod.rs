mod jest;
mod jsdoc;
mod nextjs;
mod react;
mod react_perf;
mod tree_shaking;
mod unicorn;

use crate::LintContext;

pub use self::{
    jest::*, jsdoc::*, nextjs::*, react::*, react_perf::*, tree_shaking::*, unicorn::*,
};

/// Check if the Jest rule is adapted to Vitest.
/// Many Vitest rule are essentially ports of Jest plugin rules with minor modifications.
/// For these rules, we use the corresponding jest rules with some adjustments for compatibility.
pub fn is_jest_rule_adapted_to_vitest(rule_name: &str) -> bool {
    let jest_rules: [&str; 4] =
        ["consistent_test_it", "no-disabled-tests", "prefer-hooks-in-order", "valid-expect"];

    jest_rules.contains(&rule_name)
}

pub fn get_test_plugin_name(ctx: &LintContext) -> &'static str {
    if is_using_vitest(ctx) {
        "eslint-plugin-vitest"
    } else {
        "eslint-plugin-jest"
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
