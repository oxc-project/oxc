//! Test to ensure every rule declares version metadata.
//!
//! This test verifies that all linter rules expose `version = ...` metadata
//! through `declare_oxc_lint!`.

// NOTE: You will need to run the tests with `--features ruledocs` or
// `--all-features` for this test file to run.
#![cfg(feature = "ruledocs")]

use oxc_linter::rules::RULES;
use rustc_hash::FxHashSet;

#[test]
fn test_all_rules_have_version_metadata() {
    let mut failures = Vec::new();

    // NOTE: This should NOT ever have any values. All rules should declare
    // version metadata so the docs and release workflow can rely on it.
    let exceptions: &[&str] = &[];
    let exception_set: FxHashSet<&str> = exceptions.iter().copied().collect();

    for rule in RULES.iter() {
        let full_rule_name = format!("{}/{}", rule.plugin_name(), rule.name());

        if exception_set.contains(full_rule_name.as_str()) {
            if rule.version().is_some() {
                failures.push(format!(
                    "Rule '{full_rule_name}' is in the exceptions list but already declares version metadata.\n\
                     This rule has been fixed! Please remove it from the exceptions list."
                ));
            }
            continue;
        }

        if rule.version().is_none() {
            failures.push(format!(
                "Rule '{full_rule_name}' is missing version metadata.\n\
                 Please add `version = \"x.y.z\"` to its `declare_oxc_lint!` declaration."
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "Found {} rules missing version metadata:\n\n{}",
        failures.len(),
        failures.join("\n\n")
    );
}
