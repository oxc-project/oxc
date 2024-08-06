mod jest;
mod jsdoc;
mod nextjs;
mod react;
mod react_perf;
mod set;
mod tree_shaking;
mod unicorn;

pub use self::{
    jest::*, jsdoc::*, nextjs::*, react::*, react_perf::*, set::*, tree_shaking::*, unicorn::*,
};

/// Check if the Jest rule is adapted to Vitest.
/// Many Vitest rule are essentially ports of Jest plugin rules with minor modifications.
/// For these rules, we use the corresponding jest rules with some adjustments for compatibility.
pub fn is_jest_rule_adapted_to_vitest(rule_name: &str) -> bool {
    let jest_rules: &[&str] = &[
        "consistent-test-it",
        "expect-expect",
        "no-alias-methods",
        "no-conditional-expect",
        "no-commented-out-tests",
        "no-disabled-tests",
        "no-focused-tests",
        "no-identical-title",
        "no-test-prefixes",
        "prefer-hooks-in-order",
        "valid-describe-callback",
        "valid-expect",
    ];

    jest_rules.contains(&rule_name)
}
